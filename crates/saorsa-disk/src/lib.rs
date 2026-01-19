//! saorsa-disk - Disk analyzer library
//!
//! Provides disk usage analysis functionality for the saorsa TUI framework.

pub mod analyzer;
pub mod error;
mod tab;

pub use analyzer::{DiskAnalyzer, DiskInfo, FileEntry};
pub use error::{DiskError, DiskResult};
pub use tab::{DiskTab, DiskView};
