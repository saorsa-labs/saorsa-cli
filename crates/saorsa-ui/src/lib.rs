//! # saorsa-ui
//!
//! UI framework and tab manager for the saorsa TUI.
//!
//! This crate provides the user interface components built on top of
//! [`saorsa_cli_core`], including:
//!
//! - [`TabManager`] - Manages tab collection and focus
//! - [`TabBar`] - Widget for displaying tabs
//! - [`StatusBar`] - Widget for status information
//! - [`App`] - Main application coordinator implementing [`saorsa_cli_core::AppCoordinator`]
//! - [`AppLayout`] - Layout calculation utilities
//!
//! ## Overview
//!
//! The saorsa-ui crate provides the high-level UI components for building
//! terminal applications. It builds on top of saorsa-cli-core's traits
//! and types to provide a complete TUI framework.
//!
//! ## Features
//!
//! - **Tab Management**: [`TabManager`] handles tab switching, addition,
//!   removal, and focus management.
//! - **Widgets**: Reusable UI components including [`TabBar`] and [`StatusBar`].
//! - **Application**: [`App`] implements [`saorsa_cli_core::AppCoordinator`] for full application
//!   lifecycle management.
//! - **Layout**: [`AppLayout`] and [`calculate_pane_areas`] for layout calculations.
//!
//! ## Widgets
//!
//! The [`widgets`] module provides:
//!
//! - [`TabBar`] - Horizontal bar displaying tab titles with active tab highlighting
//! - [`StatusBar`] - Three-section status bar for mode, context, and help hints
//!
//! ## Example
//!
//! ```ignore
//! use saorsa_ui::{App, TabManager, TabBar, StatusBar, AppLayout};
//! use saorsa_cli_core::{Tab, Theme, Message, AppCoordinator};
//!
//! // Create the main application
//! let mut app = App::new();
//!
//! // Add tabs
//! app.add_tab(Box::new(my_tab));
//!
//! // Main loop
//! loop {
//!     terminal.draw(|f| app.render(f))?;
//!
//!     if let Event::Key(key) = event::read()? {
//!         app.dispatch(Message::Key(key));
//!     }
//!
//!     if app.should_quit() {
//!         break;
//!     }
//! }
//! ```
//!
//! ## Layout Calculation
//!
//! ```
//! use saorsa_ui::{AppLayout, calculate_pane_areas};
//! use saorsa_cli_core::{PaneLayout, PaneNode};
//! use ratatui::prelude::Rect;
//!
//! // Calculate main layout areas
//! let area = Rect::new(0, 0, 80, 24);
//! let layout = AppLayout::new(area);
//!
//! // Content area is between tab bar and status bar
//! assert_eq!(layout.content.height, 22);
//!
//! // Calculate pane areas for split layouts
//! let pane_layout = PaneLayout::single(0);
//! let pane_areas = calculate_pane_areas(&pane_layout, layout.content);
//! assert_eq!(pane_areas.len(), 1);
//! ```

pub mod app;
pub mod renderer;
pub mod tab_manager;
pub mod widgets;

pub use app::App;
pub use renderer::{calculate_pane_areas, AppLayout};
pub use tab_manager::TabManager;
pub use widgets::{StatusBar, TabBar};
