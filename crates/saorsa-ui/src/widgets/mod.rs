//! UI widgets for the saorsa TUI framework
//!
//! This module provides reusable widgets built on ratatui for
//! the saorsa terminal user interface.
//!
//! ## Available Widgets
//!
//! - [`TabBar`] - Horizontal bar displaying tab titles with active tab highlighting
//! - [`StatusBar`] - Three-section status bar for mode, context, and help hints
//!
//! ## Example
//!
//! ```ignore
//! use saorsa_ui::widgets::{TabBar, StatusBar};
//! use saorsa_cli_core::Theme;
//!
//! let theme = Theme::dark();
//!
//! // Create a tab bar
//! let tab_bar = TabBar::new(&tabs, active_index, &theme);
//!
//! // Create a status bar
//! let status = StatusBar::new(&theme)
//!     .left("NORMAL")
//!     .center("main.rs")
//!     .right("?:help");
//! ```

pub mod status_bar;
pub mod tab_bar;

pub use status_bar::StatusBar;
pub use tab_bar::TabBar;
