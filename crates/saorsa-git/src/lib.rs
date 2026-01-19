//! saorsa-git - Git integration for the saorsa TUI framework
//!
//! This crate provides a gitui-like Git tab for the saorsa unified TUI,
//! including status views, diff viewing, and staging operations.
//!
//! # Features
//!
//! - Status view with staged/unstaged/untracked sections
//! - Diff viewer with syntax highlighting
//! - Stage/unstage individual files or all changes
//! - Discard changes with confirmation
//!
//! # Example
//!
//! ```no_run
//! use saorsa_git::{GitTab, GitRepo};
//! use std::path::Path;
//!
//! // Create a Git tab for the current directory
//! let tab = GitTab::new(1, Path::new("."));
//! ```

pub mod error;
pub mod repo;
mod tab;
pub mod widgets;

pub use error::{GitError, GitResult};
pub use repo::{CommitInfo, Diff, DiffHunk, DiffLine, FileStatus, GitRepo, StatusEntry};
pub use tab::GitTab;
