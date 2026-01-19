use crate::event::MessageBus;
use crate::{CoreError, CoreResult};
use hex;
use libloading::{Library, Symbol};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Default filename expected inside a plugin directory.
pub const MANIFEST_NAME: &str = "saorsa-plugin.toml";

/// Metadata describing a plugin for UI display and selection.
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub help: Option<String>,
    pub manifest_path: PathBuf,
    pub library_path: PathBuf,
}

/// Runtime descriptor for an instantiated plugin.
#[derive(Debug, Clone)]
pub struct PluginDescriptor {
    pub metadata: PluginMetadata,
}

/// Manifest structure stored on disk next to plugin binaries.
#[derive(Debug, Clone, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub library: PathBuf,
    #[serde(default)]
    pub help: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub entry_symbol: Option<String>,
    #[serde(default)]
    pub sha256: Option<String>,
}

/// Context passed to plugins during execution.
#[derive(Debug, Clone, Default)]
pub struct PluginContext<'a> {
    pub message_bus: Option<&'a MessageBus>,
}

impl<'a> PluginContext<'a> {
    #[must_use]
    pub fn new(message_bus: Option<&'a MessageBus>) -> Self {
        Self { message_bus }
    }
}

/// Trait implemented by dynamic Saorsa plugins.
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn version(&self) -> &str;
    fn author(&self) -> &str;
    fn help(&self) -> &str;
    fn execute(&self, args: &[String], ctx: PluginContext<'_>) -> CoreResult<()>;
}

#[allow(improper_ctypes_definitions)]
type PluginInit = unsafe extern "C" fn() -> *mut dyn Plugin;

struct LoadedPlugin {
    descriptor: PluginDescriptor,
    instance: Box<dyn Plugin>,
    _library: Library,
}

/// Manages discovery and execution of Saorsa plugins.
pub struct PluginManager {
    search_paths: Vec<PathBuf>,
    plugins: HashMap<String, LoadedPlugin>,
    security_policy: PluginSecurityPolicy,
}

impl PluginManager {
    /// Create a manager with default search paths.
    pub fn new() -> Self {
        Self {
            search_paths: default_paths(),
            plugins: HashMap::new(),
            security_policy: PluginSecurityPolicy::default(),
        }
    }

    /// Create a manager with explicit search paths.
    pub fn with_paths(paths: Vec<PathBuf>) -> Self {
        let mut manager = Self::new();
        manager.search_paths = paths;
        manager
    }

    /// Create a manager with an explicit security policy.
    pub fn with_policy(policy: PluginSecurityPolicy) -> Self {
        Self {
            search_paths: default_paths(),
            plugins: HashMap::new(),
            security_policy: policy,
        }
    }

    /// Override the current security policy.
    pub fn set_security_policy(&mut self, policy: PluginSecurityPolicy) {
        self.security_policy = policy;
    }

    /// Returns immutable view of configured search paths.
    pub fn search_paths(&self) -> &[PathBuf] {
        &self.search_paths
    }

    /// Adds an additional search path.
    pub fn add_search_path(&mut self, path: impl Into<PathBuf>) {
        self.search_paths.push(path.into());
    }

    /// Remove all currently loaded plugins.
    pub fn clear(&mut self) {
        self.plugins.clear();
    }

    /// Discover and load plugins from the configured search paths.
    pub fn load(&mut self) -> CoreResult<usize> {
        self.plugins.clear();
        let mut loaded = 0usize;

        for path in self.search_paths.clone() {
            if path.exists() {
                loaded += self.load_from_path(&path)?;
            }
        }

        Ok(loaded)
    }

    /// Returns metadata for all loaded plugins.
    pub fn descriptors(&self) -> Vec<PluginDescriptor> {
        self.plugins
            .values()
            .map(|p| p.descriptor.clone())
            .collect()
    }

    /// Execute plugin by name.
    pub fn execute_plugin(
        &self,
        name: &str,
        args: &[String],
        ctx: PluginContext<'_>,
    ) -> CoreResult<()> {
        let plugin = self
            .plugins
            .get(name)
            .ok_or_else(|| crate::CoreError::PluginNotFound(name.to_string()))?;
        plugin.instance.execute(args, ctx)
    }

    /// Returns detailed help text for a plugin if available.
    pub fn help_for(&self, name: &str) -> Option<String> {
        self.plugins
            .get(name)
            .map(|p| p.instance.help().to_string())
    }

    fn load_from_path(&mut self, root: &Path) -> CoreResult<usize> {
        let mut count = 0usize;
        if root.is_dir() {
            for entry in fs::read_dir(root)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    let manifest_path = path.join(MANIFEST_NAME);
                    if manifest_path.is_file() {
                        self.load_manifest(&manifest_path)?;
                        count += 1;
                    }
                } else if is_manifest(&path) {
                    self.load_manifest(&path)?;
                    count += 1;
                }
            }
        }
        Ok(count)
    }

    fn load_manifest(&mut self, manifest_path: &Path) -> CoreResult<()> {
        let data = fs::read_to_string(manifest_path)?;
        let manifest: PluginManifest =
            toml::from_str(&data).map_err(|err| crate::CoreError::PluginManifest {
                path: manifest_path.to_path_buf(),
                source: err,
            })?;

        let base_dir = manifest_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        let library_path = if manifest.library.is_absolute() {
            manifest.library.clone()
        } else {
            base_dir.join(&manifest.library)
        };

        if !library_path.exists() {
            return Err(crate::CoreError::PluginLibraryMissing { path: library_path });
        }

        self.verify_integrity(&manifest, manifest_path, &library_path)?;

        let symbol_name = manifest
            .entry_symbol
            .as_deref()
            .unwrap_or("_plugin_init")
            .as_bytes();

        unsafe {
            let library =
                Library::new(&library_path).map_err(|err| crate::CoreError::PluginLoadFailed {
                    path: library_path.clone(),
                    source: err,
                })?;
            let constructor: Symbol<PluginInit> =
                library
                    .get(symbol_name)
                    .map_err(|err| crate::CoreError::PluginLoadFailed {
                        path: library_path.clone(),
                        source: err,
                    })?;
            let boxed = Box::from_raw(constructor());
            let metadata = PluginMetadata {
                name: manifest.name.clone(),
                version: manifest.version.clone(),
                description: manifest.description.clone(),
                author: manifest.author.clone(),
                help: manifest.help.clone(),
                manifest_path: manifest_path.to_path_buf(),
                library_path: library_path.clone(),
            };
            let descriptor = PluginDescriptor {
                metadata: metadata.clone(),
            };

            if self.plugins.contains_key(&metadata.name) {
                return Err(crate::CoreError::PluginDuplicate(metadata.name));
            }

            self.plugins.insert(
                metadata.name.clone(),
                LoadedPlugin {
                    descriptor,
                    instance: boxed,
                    _library: library,
                },
            );
        }

        Ok(())
    }

    fn verify_integrity(
        &self,
        manifest: &PluginManifest,
        manifest_path: &Path,
        library_path: &Path,
    ) -> CoreResult<()> {
        if !self.security_policy.require_hash {
            return Ok(());
        }

        let expected_raw =
            manifest
                .sha256
                .as_ref()
                .ok_or_else(|| CoreError::PluginHashMissing {
                    path: manifest_path.to_path_buf(),
                })?;

        let expected = decode_hash(expected_raw, manifest_path)?;
        let actual = compute_sha256(library_path)?;

        if expected != actual {
            return Err(CoreError::PluginHashMismatch {
                path: library_path.to_path_buf(),
                expected: hex::encode(expected),
                actual: hex::encode(actual),
            });
        }

        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

fn default_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".saorsa/plugins"));
    }
    if let Some(data) = dirs::data_dir() {
        paths.push(data.join("saorsa/plugins"));
    }
    if cfg!(unix) {
        paths.push(PathBuf::from("/usr/local/share/saorsa/plugins"));
    }
    if let Ok(cwd) = std::env::current_dir() {
        paths.push(cwd.join("plugins"));
    }
    paths
}

fn is_manifest(path: &Path) -> bool {
    matches!(path.file_name().and_then(OsStr::to_str), Some(name) if name == MANIFEST_NAME)
}

/// Plugin security policy governing integrity requirements.
#[derive(Debug, Clone)]
pub struct PluginSecurityPolicy {
    /// Require every plugin manifest to ship a sha256 checksum.
    pub require_hash: bool,
}

impl PluginSecurityPolicy {
    /// Strict policy that requires hashes.
    #[must_use]
    pub fn strict() -> Self {
        Self { require_hash: true }
    }

    /// Permissive policy that allows unsigned plugins (not recommended).
    #[must_use]
    pub fn permissive() -> Self {
        Self {
            require_hash: false,
        }
    }
}

impl Default for PluginSecurityPolicy {
    fn default() -> Self {
        Self::strict()
    }
}

/// Macro helper for plugin authors.
#[macro_export]
macro_rules! declare_plugin {
    ($plugin_type:ty, $constructor:expr) => {
        #[no_mangle]
        pub extern "C" fn _plugin_init() -> *mut dyn $crate::Plugin {
            let constructor: fn() -> $plugin_type = $constructor;
            let object = constructor();
            let boxed: Box<dyn $crate::Plugin> = Box::new(object);
            Box::into_raw(boxed)
        }
    };
}

fn compute_sha256(path: &Path) -> CoreResult<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let digest = hasher.finalize();
    Ok(digest.to_vec())
}

fn decode_hash(raw: &str, manifest_path: &Path) -> CoreResult<Vec<u8>> {
    let sanitized: String = raw
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();
    hex::decode(&sanitized).map_err(|source| CoreError::PluginHashInvalid {
        path: manifest_path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_temp_file(contents: &[u8]) -> PathBuf {
        let mut tmp = NamedTempFile::new().expect("temp file");
        tmp.write_all(contents).expect("write temp");
        tmp.into_temp_path().to_path_buf()
    }

    #[test]
    fn compute_sha256_matches_known_value() {
        let path = write_temp_file(b"saorsa");
        let digest = compute_sha256(&path).expect("hash");
        assert_eq!(
            hex::encode(digest),
            "ebe4c8eea0d9c924166636608506cdc9e780d81a0cbe6cb94ad3cf59c18348eb"
        );
    }

    #[test]
    fn decode_hash_rejects_invalid_hex() {
        let err = decode_hash("zzzz", Path::new("manifest")).unwrap_err();
        assert!(matches!(err, CoreError::PluginHashInvalid { .. }));
    }

    #[test]
    fn verify_integrity_detects_mismatch() {
        let mut manager = PluginManager::new();
        let manifest_path = Path::new("manifest");
        let library_path = write_temp_file(b"abc");
        let manifest = PluginManifest {
            name: "demo".into(),
            version: "0.1.0".into(),
            description: "test".into(),
            author: "tester".into(),
            library: library_path.clone(),
            help: None,
            homepage: None,
            entry_symbol: None,
            sha256: Some("aaaaaaaa".into()),
        };
        let err = manager
            .verify_integrity(&manifest, manifest_path, &library_path)
            .unwrap_err();
        assert!(matches!(err, CoreError::PluginHashMismatch { .. }));
    }
}
