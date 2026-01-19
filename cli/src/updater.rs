//! Background update checker for the auto-update system.
//!
//! Provides non-blocking update checking that respects cache TTL
//! and integrates with the config system for persistence.

use crate::config::Config;
use crate::downloader::Downloader;
use crate::version;
use std::sync::{Arc, RwLock};

/// Result of an update check.
#[allow(dead_code)] // Used in Task 2 integration
#[derive(Debug, Clone)]
pub struct UpdateCheckResult {
    /// Whether an update is available
    pub update_available: bool,
    /// The latest version string (e.g., "0.3.13")
    pub latest_version: Option<String>,
    /// The current version string
    pub current_version: String,
    /// Human-readable message
    pub message: Option<String>,
}

/// Background update checker.
///
/// Spawns a non-blocking task to check for updates on startup,
/// respecting the configured cache TTL to avoid API rate limiting.
#[allow(dead_code)] // Used in Task 2 integration
pub struct UpdateChecker {
    config: Arc<RwLock<Config>>,
    downloader: Arc<Downloader>,
}

#[allow(dead_code)] // Used in Task 2 integration
impl UpdateChecker {
    /// Create a new update checker.
    pub fn new(config: Arc<RwLock<Config>>, downloader: Arc<Downloader>) -> Self {
        Self { config, downloader }
    }

    /// Check for updates (respects cache TTL).
    ///
    /// Returns cached result if within TTL, otherwise performs a fresh check.
    /// Returns `None` if update checking is disabled or check fails.
    pub fn check(&self) -> Option<UpdateCheckResult> {
        // Check if we should even run (respects --no-update-check and cache)
        {
            let config = self.config.read().ok()?;
            if !config.should_check_for_updates() {
                // Return cached result if available
                if let Some(latest) = config.get_latest_version() {
                    let current = env!("CARGO_PKG_VERSION");
                    let update_available =
                        version::update_available(current, latest).unwrap_or(false);
                    return Some(UpdateCheckResult {
                        update_available,
                        latest_version: Some(latest.clone()),
                        current_version: current.to_string(),
                        message: if update_available {
                            Some(format!("Update available: {} -> {}", current, latest))
                        } else {
                            None
                        },
                    });
                }
                return None;
            }
        }

        // Perform the actual check
        self.perform_check()
    }

    /// Actually perform the update check (bypasses cache).
    fn perform_check(&self) -> Option<UpdateCheckResult> {
        let current = env!("CARGO_PKG_VERSION");

        match self.downloader.get_latest_release() {
            Ok(release) => {
                let latest = release.tag_name.trim_start_matches('v');
                let update_available = version::update_available(current, latest).unwrap_or(false);

                // Update cache
                if let Ok(mut config) = self.config.write() {
                    config.record_update_check(Some(latest.to_string()));
                    // Save config (ignore errors for background task)
                    if let Err(e) = config.save() {
                        tracing::debug!("Failed to save config after update check: {}", e);
                    }
                }

                Some(UpdateCheckResult {
                    update_available,
                    latest_version: Some(latest.to_string()),
                    current_version: current.to_string(),
                    message: if update_available {
                        Some(format!("Update available: {} -> {}", current, latest))
                    } else {
                        None
                    },
                })
            }
            Err(e) => {
                tracing::debug!("Update check failed: {}", e);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_check_result_with_update() {
        let result = UpdateCheckResult {
            update_available: true,
            latest_version: Some("0.4.0".to_string()),
            current_version: "0.3.12".to_string(),
            message: Some("Update available: 0.3.12 -> 0.4.0".to_string()),
        };

        assert!(result.update_available);
        assert_eq!(result.latest_version, Some("0.4.0".to_string()));
        assert_eq!(result.current_version, "0.3.12");
        assert!(result.message.is_some());
        assert!(result.message.unwrap().contains("0.4.0"));
    }

    #[test]
    fn test_update_check_result_no_update() {
        let result = UpdateCheckResult {
            update_available: false,
            latest_version: Some("0.3.12".to_string()),
            current_version: "0.3.12".to_string(),
            message: None,
        };

        assert!(!result.update_available);
        assert_eq!(result.latest_version, Some("0.3.12".to_string()));
        assert!(result.message.is_none());
    }

    #[test]
    fn test_update_check_result_clone() {
        let result = UpdateCheckResult {
            update_available: true,
            latest_version: Some("1.0.0".to_string()),
            current_version: "0.9.0".to_string(),
            message: Some("Update available".to_string()),
        };

        let cloned = result.clone();
        assert_eq!(cloned.update_available, result.update_available);
        assert_eq!(cloned.latest_version, result.latest_version);
        assert_eq!(cloned.current_version, result.current_version);
        assert_eq!(cloned.message, result.message);
    }
}
