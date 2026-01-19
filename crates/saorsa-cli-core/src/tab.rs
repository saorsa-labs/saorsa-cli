//! Tab trait and related types for the TUI framework.
//!
//! This module defines the [`Tab`] trait which all tab implementations
//! must satisfy to be used in the saorsa TUI framework.

use crate::event::Message;
use ratatui::prelude::*;

/// Unique identifier for a tab.
///
/// Each tab in the framework has a unique numeric identifier
/// used for lookup and management operations.
pub type TabId = u32;

/// Trait that all tabs must implement.
///
/// The `Tab` trait defines the interface for tab components in the
/// saorsa TUI framework. Tabs are the primary content containers
/// and can be focused, blurred, and rendered.
///
/// # Thread Safety
///
/// Tabs are expected to be used from the main TUI thread, so there
/// is no strict `Send + Sync` requirement.
///
/// # Example
///
/// ```ignore
/// use saorsa_cli_core::{Tab, TabId};
/// use ratatui::prelude::*;
///
/// struct MyTab {
///     id: TabId,
///     title: String,
///     focused: bool,
/// }
///
/// impl Tab for MyTab {
///     fn id(&self) -> TabId { self.id }
///     fn title(&self) -> &str { &self.title }
///     fn focus(&mut self) { self.focused = true; }
///     fn blur(&mut self) { self.focused = false; }
///     fn view(&self, frame: &mut Frame, area: Rect) {
///         // Render tab content
///     }
/// }
/// ```
pub trait Tab {
    /// Returns the unique identifier for this tab.
    ///
    /// This ID is used to reference the tab in the tab manager
    /// and should remain constant for the lifetime of the tab.
    fn id(&self) -> TabId;

    /// Returns the display title for this tab.
    ///
    /// This title is shown in the tab bar and other UI elements
    /// that reference this tab.
    fn title(&self) -> &str;

    /// Returns an optional icon character for the tab bar.
    ///
    /// If provided, this icon is displayed alongside the tab title
    /// in the tab bar. Common icons include emoji or Unicode symbols.
    ///
    /// # Returns
    ///
    /// `None` by default; override to provide a custom icon.
    fn icon(&self) -> Option<&str> {
        None
    }

    /// Returns whether this tab can be closed by the user.
    ///
    /// Some tabs (like a main dashboard) may be permanent and
    /// should not allow user-initiated closing.
    ///
    /// # Returns
    ///
    /// `true` by default; override to prevent closing.
    fn can_close(&self) -> bool {
        true
    }

    /// Called when the tab receives focus.
    ///
    /// Use this method to update internal state, start animations,
    /// or begin any operations that should only occur when the tab
    /// is the active tab.
    fn focus(&mut self);

    /// Called when the tab loses focus.
    ///
    /// Use this method to pause animations, release resources,
    /// or perform cleanup when the user switches away from this tab.
    fn blur(&mut self);

    /// Renders the tab content to the given area.
    ///
    /// This method is called during the render phase of the TUI
    /// event loop. Implementations should draw their content to
    /// the provided frame within the specified area bounds.
    ///
    /// # Arguments
    ///
    /// * `frame` - The ratatui frame to render to
    /// * `area` - The rectangular area available for this tab content
    fn view(&self, frame: &mut Frame, area: Rect);

    /// Optional handler invoked when the active tab receives a message.
    ///
    /// Tabs can inspect messages such as keyboard or mouse input and
    /// optionally return a follow-up message for the coordinator to process.
    fn handle_message(&mut self, _message: &Message) -> Option<Message> {
        None
    }

    /// Optional per-tick update invoked by the coordinator.
    fn tick(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    // A minimal test implementation of Tab
    struct TestTab {
        id: TabId,
        title: String,
        icon: Option<String>,
        closeable: bool,
        focused: bool,
    }

    impl TestTab {
        fn new(id: TabId, title: &str) -> Self {
            Self {
                id,
                title: title.to_string(),
                icon: None,
                closeable: true,
                focused: false,
            }
        }

        fn with_icon(mut self, icon: &str) -> Self {
            self.icon = Some(icon.to_string());
            self
        }

        fn uncloseable(mut self) -> Self {
            self.closeable = false;
            self
        }
    }

    impl Tab for TestTab {
        fn id(&self) -> TabId {
            self.id
        }

        fn title(&self) -> &str {
            &self.title
        }

        fn icon(&self) -> Option<&str> {
            self.icon.as_deref()
        }

        fn can_close(&self) -> bool {
            self.closeable
        }

        fn focus(&mut self) {
            self.focused = true;
        }

        fn blur(&mut self) {
            self.focused = false;
        }

        fn view(&self, _frame: &mut Frame, _area: Rect) {
            // Test implementation does nothing
        }
    }

    #[test]
    fn test_tab_id() {
        let tab = TestTab::new(42, "Test");
        assert_eq!(tab.id(), 42);
    }

    #[test]
    fn test_tab_title() {
        let tab = TestTab::new(1, "My Tab");
        assert_eq!(tab.title(), "My Tab");
    }

    #[test]
    fn test_tab_icon_default() {
        let tab = TestTab::new(1, "Test");
        assert!(tab.icon().is_none());
    }

    #[test]
    fn test_tab_icon_custom() {
        let tab = TestTab::new(1, "Test").with_icon("folder");
        assert_eq!(tab.icon(), Some("folder"));
    }

    #[test]
    fn test_tab_can_close_default() {
        let tab = TestTab::new(1, "Test");
        assert!(tab.can_close());
    }

    #[test]
    fn test_tab_can_close_false() {
        let tab = TestTab::new(1, "Test").uncloseable();
        assert!(!tab.can_close());
    }

    #[test]
    fn test_tab_focus_blur() {
        let mut tab = TestTab::new(1, "Test");
        assert!(!tab.focused);

        tab.focus();
        assert!(tab.focused);

        tab.blur();
        assert!(!tab.focused);
    }

    #[test]
    fn test_tab_trait_is_object_safe() {
        // This test verifies Tab can be used as a trait object
        fn accept_tab(_tab: &dyn Tab) {}
        let tab = TestTab::new(1, "Test");
        accept_tab(&tab);
    }
}
