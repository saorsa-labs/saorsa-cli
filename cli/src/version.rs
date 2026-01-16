//! Version management utilities for the auto-update system.
//!
//! Provides semantic version parsing, comparison, and version info structures.

use semver::Version;
use std::cmp::Ordering;

/// Information about a specific version of the application.
#[allow(dead_code)] // Used by update system in later phases
#[derive(Debug, Clone)]
pub struct VersionInfo {
    /// Semantic version string (e.g., "0.3.12")
    pub version: String,
    /// Parsed semantic version for comparison
    pub semver: Option<Version>,
    /// Target triple (e.g., "x86_64-apple-darwin")
    pub target: &'static str,
}

/// Get the target triple at compile time using cfg attributes.
const fn get_target() -> &'static str {
    #[cfg(all(target_arch = "x86_64", target_os = "macos"))]
    return "x86_64-apple-darwin";
    #[cfg(all(target_arch = "aarch64", target_os = "macos"))]
    return "aarch64-apple-darwin";
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    return "x86_64-unknown-linux-gnu";
    #[cfg(all(target_arch = "aarch64", target_os = "linux"))]
    return "aarch64-unknown-linux-gnu";
    #[cfg(all(target_arch = "x86_64", target_os = "windows"))]
    return "x86_64-pc-windows-msvc";
    #[cfg(not(any(
        all(target_arch = "x86_64", target_os = "macos"),
        all(target_arch = "aarch64", target_os = "macos"),
        all(target_arch = "x86_64", target_os = "linux"),
        all(target_arch = "aarch64", target_os = "linux"),
        all(target_arch = "x86_64", target_os = "windows"),
    )))]
    return "unknown";
}

#[allow(dead_code)] // Used by update system in later phases
impl VersionInfo {
    /// Create a new VersionInfo from a version string.
    pub fn new(version: &str) -> Self {
        Self {
            version: version.to_string(),
            semver: parse_version(version).ok(),
            target: get_target(),
        }
    }

    /// Get the current application version.
    pub fn current() -> Self {
        Self::new(env!("CARGO_PKG_VERSION"))
    }
}

/// Parse a version string into a semantic version.
///
/// Handles versions with or without 'v' prefix (e.g., "v0.3.12" or "0.3.12").
///
/// # Errors
/// Returns an error if the version string is not a valid semantic version.
#[allow(dead_code)] // Used by update system in later phases
pub fn parse_version(version: &str) -> Result<Version, semver::Error> {
    // Strip 'v' prefix if present (common in git tags)
    let version = version.strip_prefix('v').unwrap_or(version);
    Version::parse(version)
}

/// Compare two version strings.
///
/// Returns the ordering between the two versions.
/// Returns `None` if either version string is invalid.
#[allow(dead_code)] // Used by update system in later phases
pub fn compare_versions(a: &str, b: &str) -> Option<Ordering> {
    let va = parse_version(a).ok()?;
    let vb = parse_version(b).ok()?;
    Some(va.cmp(&vb))
}

/// Check if version `a` is newer than version `b`.
///
/// Returns `None` if either version string is invalid.
#[allow(dead_code)] // Used by update system in later phases
pub fn is_newer(a: &str, b: &str) -> Option<bool> {
    compare_versions(a, b).map(|ord| ord == Ordering::Greater)
}

/// Check if an update is available by comparing current and latest versions.
///
/// Returns `Some(true)` if latest is newer than current.
/// Returns `Some(false)` if current is up-to-date or newer.
/// Returns `None` if version comparison failed.
#[allow(dead_code)] // Used by update system in later phases
pub fn update_available(current: &str, latest: &str) -> Option<bool> {
    is_newer(latest, current)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        assert!(parse_version("0.3.12").is_ok());
        assert!(parse_version("v0.3.12").is_ok());
        assert!(parse_version("1.0.0").is_ok());
        assert!(parse_version("0.1.0-alpha.1").is_ok());
        assert!(parse_version("invalid").is_err());
    }

    #[test]
    fn test_compare_versions() {
        assert_eq!(
            compare_versions("0.3.12", "0.3.11"),
            Some(Ordering::Greater)
        );
        assert_eq!(compare_versions("0.3.12", "0.3.12"), Some(Ordering::Equal));
        assert_eq!(compare_versions("0.3.12", "0.3.13"), Some(Ordering::Less));
        assert_eq!(compare_versions("v0.3.12", "0.3.12"), Some(Ordering::Equal));
        assert_eq!(compare_versions("1.0.0", "0.9.99"), Some(Ordering::Greater));
    }

    #[test]
    fn test_is_newer() {
        assert_eq!(is_newer("0.3.13", "0.3.12"), Some(true));
        assert_eq!(is_newer("0.3.12", "0.3.12"), Some(false));
        assert_eq!(is_newer("0.3.11", "0.3.12"), Some(false));
    }

    #[test]
    fn test_update_available() {
        // Latest is newer - update available
        assert_eq!(update_available("0.3.12", "0.3.13"), Some(true));
        // Same version - no update
        assert_eq!(update_available("0.3.12", "0.3.12"), Some(false));
        // Current is newer (dev build?) - no update
        assert_eq!(update_available("0.3.13", "0.3.12"), Some(false));
    }

    #[test]
    fn test_version_info_current() {
        let info = VersionInfo::current();
        assert!(!info.version.is_empty());
        assert!(info.semver.is_some());
        assert!(!info.target.is_empty());
    }

    #[test]
    fn test_prerelease_comparison() {
        // Stable is newer than prerelease of same version
        assert_eq!(
            compare_versions("0.3.12", "0.3.12-alpha.1"),
            Some(Ordering::Greater)
        );
        // Later prerelease is newer
        assert_eq!(
            compare_versions("0.3.12-beta.1", "0.3.12-alpha.1"),
            Some(Ordering::Greater)
        );
    }
}
