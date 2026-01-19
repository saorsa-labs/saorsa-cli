//! Git-related widgets for the TUI
//!
//! This module provides widgets for displaying git status and diffs.

mod diff;
mod status;

pub use diff::{DiffWidget, DiffWidgetState};
pub use status::{Section, StatusWidget, StatusWidgetState};
