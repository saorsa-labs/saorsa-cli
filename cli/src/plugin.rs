use anyhow::{Context, Result};
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn version(&self) -> &str;
    fn author(&self) -> &str;
    fn help(&self) -> &str;
    fn execute(&self, args: &[String]) -> Result<()>;
}

/// Metadata about a loaded plugin.
/// Some fields are reserved for future plugin management features.
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    #[allow(dead_code)]
    pub author: String,
    #[allow(dead_code)]
    pub path: PathBuf,
}

/// Extended plugin information including help text.
/// Reserved for future plugin details UI.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub metadata: PluginMetadata,
    pub help: String,
}

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
    libs: Vec<Library>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            libs: Vec::new(),
        }
    }

    pub fn load_plugin(&mut self, path: &Path) -> Result<()> {
        unsafe {
            let lib =
                Library::new(path).with_context(|| format!("Failed to load plugin: {:?}", path))?;

            let plugin_init: Symbol<unsafe extern "C" fn() -> *mut dyn Plugin> = lib
                .get(b"_plugin_init")
                .with_context(|| format!("Failed to find _plugin_init in {:?}", path))?;

            let plugin = Box::from_raw(plugin_init());
            let name = plugin.name().to_string();
            self.plugins.insert(name, plugin);
            self.libs.push(lib);
        }
        Ok(())
    }

    pub fn load_plugins_from_dir(&mut self, dir: &Path) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "so" || ext == "dylib" || ext == "dll" {
                        self.load_plugin(&path)?;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn get_plugins(&self) -> Vec<PluginMetadata> {
        self.plugins
            .values()
            .map(|p| PluginMetadata {
                name: p.name().to_string(),
                description: p.description().to_string(),
                version: p.version().to_string(),
                author: p.author().to_string(),
                path: PathBuf::new(), // This is not ideal, but we don't have the path here
            })
            .collect()
    }

    pub fn execute_plugin(&self, name: &str, args: &[String]) -> Result<()> {
        if let Some(plugin) = self.plugins.get(name) {
            plugin.execute(args)
        } else {
            anyhow::bail!("Plugin not found: {}", name)
        }
    }

    #[allow(dead_code)]
    pub fn remove_plugin(&mut self, name: &str) -> Result<()> {
        if self.plugins.remove(name).is_some() {
            // How to remove the lib?
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    pub fn get_plugin_names(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    pub fn get_plugin_info(&self, name: &str) -> Option<PluginInfo> {
        self.plugins.get(name).map(|p| {
            let metadata = PluginMetadata {
                name: p.name().to_string(),
                description: p.description().to_string(),
                version: p.version().to_string(),
                author: p.author().to_string(),
                path: PathBuf::new(),
            };
            PluginInfo {
                metadata,
                help: p.help().to_string(),
            }
        })
    }
}

#[macro_export]
macro_rules! declare_plugin {
    ($plugin_type:ty, $constructor:expr) => {
        #[no_mangle]
        pub extern "C" fn _plugin_init() -> *mut dyn $crate::plugin::Plugin {
            let constructor: fn() -> $plugin_type = $constructor;
            let object = constructor();
            let boxed: Box<dyn $crate::plugin::Plugin> = Box::new(object);
            Box::into_raw(boxed)
        }
    };
}

/// Example plugin demonstrating the plugin API.
/// This is reference code for plugin developers.
#[allow(dead_code)]
pub struct ExamplePlugin {
    name: String,
    description: String,
    version: String,
    author: String,
}

impl Default for ExamplePlugin {
    fn default() -> Self {
        Self {
            name: "example".to_string(),
            description: "An example plugin demonstrating the plugin system".to_string(),
            version: "1.0.0".to_string(),
            author: "Saorsa".to_string(),
        }
    }
}

impl Plugin for ExamplePlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn author(&self) -> &str {
        &self.author
    }

    fn help(&self) -> &str {
        "This is an example plugin."
    }

    fn execute(&self, _args: &[String]) -> Result<()> {
        println!("Hello from the example plugin!");
        Ok(())
    }
}

pub fn init_plugin_system() -> Result<PluginManager> {
    let mut manager = PluginManager::new();

    // Load plugins from a known directory
    if let Some(home_dir) = dirs::home_dir() {
        let plugin_dir = home_dir.join(".saorsa-cli/plugins");
        manager.load_plugins_from_dir(&plugin_dir)?;
    }

    Ok(manager)
}
