//! Error types for the saorsa-git crate

use thiserror::Error;

/// Git-specific errors
#[derive(Error, Debug)]
pub enum GitError {
    /// Not inside a Git repository
    #[error("not a git repository")]
    NotARepository,

    /// Git operation failed
    #[error("git error: {0}")]
    Git(#[from] git2::Error),

    /// IO operation failed
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Path is not within the repository
    #[error("path not in repository: {0}")]
    PathNotInRepo(String),

    /// No HEAD commit found
    #[error("no HEAD commit")]
    NoHead,
}

/// Result type alias for Git operations
pub type GitResult<T> = Result<T, GitError>;
