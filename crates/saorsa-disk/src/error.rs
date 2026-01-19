//! Error types for the disk analyzer

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during disk analysis
#[derive(Debug, Error)]
pub enum DiskError {
    /// Failed to read directory
    #[error("failed to read directory: {path}")]
    ReadDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Failed to get file metadata
    #[error("failed to get metadata for: {path}")]
    Metadata {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Path does not exist
    #[error("path does not exist: {0}")]
    PathNotFound(PathBuf),

    /// Permission denied
    #[error("permission denied: {0}")]
    PermissionDenied(PathBuf),

    /// Analysis was cancelled
    #[error("analysis cancelled")]
    Cancelled,
}

/// Result type for disk operations
pub type DiskResult<T> = Result<T, DiskError>;
