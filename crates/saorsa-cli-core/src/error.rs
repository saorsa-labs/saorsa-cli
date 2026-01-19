//! Core error types for saorsa-cli-core
//!
//! This module provides error types used throughout the saorsa TUI framework.

use hex::FromHexError;
use libloading::Error as LibraryError;
use std::io;
use std::path::PathBuf;
use thiserror::Error;
use toml::de::Error as ManifestError;

/// Core errors that can occur in the saorsa TUI framework.
///
/// These errors represent failures in the core framework operations
/// such as tab management, pane layout, and event handling.
#[derive(Debug, Error)]
pub enum CoreError {
    /// Tab with the specified ID was not found.
    ///
    /// This occurs when attempting to access, focus, or close a tab
    /// that does not exist in the tab manager.
    #[error("tab not found: {0}")]
    TabNotFound(u32),

    /// Pane with the specified ID was not found.
    ///
    /// This occurs when attempting to access or manipulate a pane
    /// that does not exist in the current layout.
    #[error("pane not found: {0}")]
    PaneNotFound(u32),

    /// Invalid layout configuration.
    ///
    /// This occurs when attempting to create a layout with invalid
    /// parameters, such as invalid split ratios or circular references.
    #[error("invalid layout: {0}")]
    InvalidLayout(String),

    /// Event system error.
    ///
    /// This occurs when the event handling system encounters an error,
    /// such as failed channel communication or invalid event types.
    #[error("event error: {0}")]
    EventError(String),

    /// Underlying IO error bubbled up from filesystem operations.
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    /// Attempted to interact with a plugin that does not exist.
    #[error("plugin not found: {0}")]
    PluginNotFound(String),

    /// Plugin manifest could not be parsed.
    #[error("invalid plugin manifest at {path:?}: {source}")]
    PluginManifest {
        path: PathBuf,
        source: ManifestError,
    },

    /// Referenced plugin library file is missing.
    #[error("plugin library missing: {path:?}")]
    PluginLibraryMissing { path: PathBuf },

    /// Dynamic library failed to load or initialize.
    #[error("failed to load plugin at {path:?}: {source}")]
    PluginLoadFailed { path: PathBuf, source: LibraryError },

    /// Duplicate plugin names detected during discovery.
    #[error("duplicate plugin name: {0}")]
    PluginDuplicate(String),

    /// Plugin manifest missing required sha256 hash.
    #[error("plugin manifest missing sha256 checksum at {path:?}")]
    PluginHashMissing { path: PathBuf },

    /// Plugin manifest sha256 value is invalid hex.
    #[error("plugin manifest has invalid sha256 at {path:?}: {source}")]
    PluginHashInvalid { path: PathBuf, source: FromHexError },

    /// Plugin checksum did not match expected value.
    #[error("plugin checksum mismatch at {path:?}: expected {expected}, got {actual}")]
    PluginHashMismatch {
        path: PathBuf,
        expected: String,
        actual: String,
    },
}

/// Result type alias using [`CoreError`].
pub type CoreResult<T> = Result<T, CoreError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_not_found_error_display() {
        let err = CoreError::TabNotFound(42);
        assert_eq!(err.to_string(), "tab not found: 42");
    }

    #[test]
    fn test_pane_not_found_error_display() {
        let err = CoreError::PaneNotFound(7);
        assert_eq!(err.to_string(), "pane not found: 7");
    }

    #[test]
    fn test_invalid_layout_error_display() {
        let err = CoreError::InvalidLayout("ratio must be between 0 and 100".to_string());
        assert_eq!(
            err.to_string(),
            "invalid layout: ratio must be between 0 and 100"
        );
    }

    #[test]
    fn test_event_error_display() {
        let err = CoreError::EventError("channel closed".to_string());
        assert_eq!(err.to_string(), "event error: channel closed");
    }

    #[test]
    fn test_plugin_not_found_display() {
        let err = CoreError::PluginNotFound("example".into());
        assert_eq!(err.to_string(), "plugin not found: example");
    }

    #[test]
    fn test_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<CoreError>();
    }
}
