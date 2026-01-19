//! # saorsa-cli-core
//!
//! Core traits and types for the saorsa TUI framework.
//!
//! This crate provides the foundational abstractions used by all
//! saorsa components for building terminal user interfaces.
//!
//! ## Overview
//!
//! The saorsa TUI framework is designed around a tab-based interface
//! where each tab can contain one or more panes arranged in a
//! flexible split layout. Themes provide customizable colors and borders.
//!
//! ## Core Abstractions
//!
//! - [`Tab`] - Trait for tab implementations
//! - [`AppCoordinator`] - Trait for the main application loop
//! - [`PaneLayout`] - Layout management for split panes
//! - [`Message`] - Event/message types for framework communication
//! - [`MessageBus`] - Publish-subscribe message distribution
//! - [`Theme`] - Theming system with colors and border styles
//! - [`CoreError`] - Error types for framework operations
//!
//! ## Example
//!
//! ```ignore
//! use saorsa_cli_core::{Tab, TabId, PaneLayout, PaneNode, Message, MessageBus, Theme};
//! use ratatui::prelude::*;
//!
//! // Define a custom tab
//! struct MyTab {
//!     id: TabId,
//!     layout: PaneLayout,
//! }
//!
//! impl Tab for MyTab {
//!     fn id(&self) -> TabId { self.id }
//!     fn title(&self) -> &str { "My Tab" }
//!     fn focus(&mut self) { /* handle focus */ }
//!     fn blur(&mut self) { /* handle blur */ }
//!     fn view(&self, frame: &mut Frame, area: Rect) {
//!         // Render based on self.layout
//!     }
//! }
//!
//! // Create a split layout
//! let layout = PaneLayout {
//!     root: PaneNode::vsplit(30, vec![
//!         PaneNode::leaf(0),  // sidebar
//!         PaneNode::leaf(1),  // main content
//!     ]),
//! };
//!
//! // Set up message bus for event handling
//! let bus = MessageBus::new(100);
//! let rx = bus.subscribe();
//!
//! // Load a theme
//! let theme = Theme::dark();
//! println!("Using theme: {}", theme.name);
//! ```

pub mod app;
pub mod error;
pub mod event;
pub mod pane;
pub mod plugin;
pub mod plugin_history;
pub mod tab;
pub mod theme;

pub use app::AppCoordinator;
pub use error::{CoreError, CoreResult};
pub use event::{InputEvent, Message, MessageBus};
pub use pane::{PaneId, PaneLayout, PaneNode, Split};
pub use plugin::{
    Plugin, PluginContext, PluginDescriptor, PluginManager, PluginManifest, PluginMetadata,
    PluginSecurityPolicy,
};
pub use plugin_history::{PluginHistory, PluginRunStats};
pub use tab::{Tab, TabId};
pub use theme::{BorderStyle, Theme, ThemeColors};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_exports() {
        // Verify all public types are accessible
        let _: TabId = 0;
        let _: PaneId = 0;
        let _ = PaneLayout::default();
        let _ = PaneNode::leaf(0);
        let _ = Split::Horizontal(50);
        let _ = CoreError::TabNotFound(0);
    }

    #[test]
    fn test_event_exports() {
        // Verify event types are accessible
        let _ = Message::Quit;
        let _ = Message::None;
        let _ = MessageBus::new(100);
        let _ = InputEvent::Tick;
    }

    #[test]
    fn test_theme_exports() {
        // Verify theme types are accessible
        let theme = Theme::dark();
        assert_eq!(theme.name, "Dark");
        let _ = BorderStyle::Rounded;
        let _ = theme.colors.background;
    }

    #[test]
    fn test_core_result_usage() {
        fn example_function() -> CoreResult<u32> {
            Ok(42)
        }

        fn failing_function() -> CoreResult<u32> {
            Err(CoreError::TabNotFound(1))
        }

        assert_eq!(example_function().ok(), Some(42));
        assert!(failing_function().is_err());
    }
}
