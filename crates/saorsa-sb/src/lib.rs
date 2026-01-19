//! saorsa-sb - Markdown browser library
//!
//! Provides the `SbTab` for integration with the saorsa TUI framework.
//!
//! # Overview
//!
//! This crate wraps the `sb` markdown browser as a Tab that can be used
//! in the saorsa unified TUI. It provides:
//!
//! - File tree navigation
//! - Markdown preview with syntax highlighting
//! - Inline editing capabilities
//! - Git integration
//!
//! # Example
//!
//! ```ignore
//! use saorsa_sb::SbTab;
//!
//! let tab = SbTab::new(1, "/path/to/browse")?;
//! ```

mod tab;

pub use tab::SbTab;

// Re-export useful types from sb
pub use sb::{App, Config, Focus, GitRepository};
