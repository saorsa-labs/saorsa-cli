//! Main application struct
//!
//! The [`App`] struct implements [`AppCoordinator`] and serves as the
//! central coordinator for the TUI application.
//!
//! # Overview
//!
//! The App struct provides:
//!
//! - **Tab Management**: Adding, removing, and navigating between tabs
//! - **Theme Support**: Customizable theming for the entire application
//! - **Message Bus**: Publish-subscribe messaging for component communication
//! - **Status Bar**: Configurable status information display
//!
//! # Example
//!
//! ```ignore
//! use saorsa_ui::App;
//! use saorsa_cli_core::Message;
//!
//! let mut app = App::new();
//! app.add_tab(Box::new(my_tab));
//!
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

use crate::renderer::AppLayout;
use crate::tab_manager::TabManager;
use crate::widgets::{StatusBar, TabBar};
use ratatui::prelude::*;
use saorsa_cli_core::{AppCoordinator, CoreResult, Message, MessageBus, Tab, TabId, Theme};

/// Main application state
///
/// App coordinates all TUI components including tabs, themes, and messaging.
/// It implements the [`AppCoordinator`] trait for integration with the framework.
///
/// # Thread Safety
///
/// App itself is not thread-safe. If you need to share it across threads,
/// wrap it in appropriate synchronization primitives.
///
/// # Example
///
/// ```ignore
/// let mut app = App::new();
/// app.add_tab(Box::new(my_tab));
///
/// loop {
///     terminal.draw(|f| app.render(f))?;
///
///     if let Event::Key(key) = event::read()? {
///         app.dispatch(Message::Key(key));
///     }
///
///     if app.should_quit() {
///         break;
///     }
/// }
/// ```
pub struct App {
    /// Manages the collection of tabs
    tab_manager: TabManager,
    /// Current theme for styling
    theme: Theme,
    /// Message bus for component communication
    message_bus: MessageBus,
    /// Flag indicating the application should quit
    should_quit: bool,
    /// Left section of status bar (typically mode)
    status_left: String,
    /// Center section of status bar (typically file info)
    status_center: String,
    /// Right section of status bar (typically help hints)
    status_right: String,
}

impl App {
    /// Creates a new app with the default dark theme
    ///
    /// # Example
    ///
    /// ```
    /// use saorsa_ui::App;
    /// use saorsa_cli_core::AppCoordinator;
    ///
    /// let app = App::new();
    /// assert!(!app.should_quit());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        App {
            tab_manager: TabManager::new(),
            theme: Theme::dark(),
            message_bus: MessageBus::new(256),
            should_quit: false,
            status_left: String::new(),
            status_center: String::new(),
            status_right: "?:help  q:quit".to_string(),
        }
    }

    /// Creates a new app with a custom theme
    ///
    /// # Arguments
    ///
    /// * `theme` - The theme to use for styling
    ///
    /// # Example
    ///
    /// ```
    /// use saorsa_ui::App;
    /// use saorsa_cli_core::{Theme, AppCoordinator};
    ///
    /// let app = App::with_theme(Theme::light());
    /// assert_eq!(app.theme().name, "Light");
    /// ```
    #[must_use]
    pub fn with_theme(theme: Theme) -> Self {
        App {
            tab_manager: TabManager::new(),
            theme,
            message_bus: MessageBus::new(256),
            should_quit: false,
            status_left: String::new(),
            status_center: String::new(),
            status_right: "?:help  q:quit".to_string(),
        }
    }

    /// Adds a tab to the application
    ///
    /// Returns the tab's ID for later reference.
    ///
    /// # Arguments
    ///
    /// * `tab` - The tab to add
    ///
    /// # Returns
    ///
    /// The TabId of the newly added tab.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut app = App::new();
    /// let id = app.add_tab(Box::new(my_tab));
    /// assert_eq!(app.tabs().len(), 1);
    /// ```
    pub fn add_tab(&mut self, tab: Box<dyn Tab>) -> TabId {
        self.tab_manager.add_tab(tab)
    }

    /// Removes a tab by ID
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the tab to remove
    ///
    /// # Errors
    ///
    /// Returns an error if the tab doesn't exist.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut app = App::new();
    /// let id = app.add_tab(Box::new(my_tab));
    /// app.remove_tab(id)?;
    /// assert!(app.tabs().is_empty());
    /// ```
    pub fn remove_tab(&mut self, id: TabId) -> CoreResult<()> {
        self.tab_manager.remove_tab(id)
    }

    /// Gets a reference to the message bus
    ///
    /// The message bus can be used to subscribe to messages or send
    /// messages to all subscribers.
    ///
    /// # Example
    ///
    /// ```
    /// use saorsa_ui::App;
    /// use saorsa_cli_core::AppCoordinator;
    ///
    /// let app = App::new();
    /// let _rx = app.message_bus().subscribe();
    /// ```
    #[must_use]
    pub fn message_bus(&self) -> &MessageBus {
        &self.message_bus
    }

    /// Sets the status bar left section (typically mode)
    ///
    /// # Arguments
    ///
    /// * `text` - The text to display in the left section
    ///
    /// # Example
    ///
    /// ```
    /// use saorsa_ui::App;
    ///
    /// let mut app = App::new();
    /// app.set_status_left("NORMAL");
    /// ```
    pub fn set_status_left(&mut self, text: impl Into<String>) {
        self.status_left = text.into();
    }

    /// Sets the status bar center section (typically file info)
    ///
    /// # Arguments
    ///
    /// * `text` - The text to display in the center section
    ///
    /// # Example
    ///
    /// ```
    /// use saorsa_ui::App;
    ///
    /// let mut app = App::new();
    /// app.set_status_center("main.rs [+]");
    /// ```
    pub fn set_status_center(&mut self, text: impl Into<String>) {
        self.status_center = text.into();
    }

    /// Sets the status bar right section (typically help)
    ///
    /// # Arguments
    ///
    /// * `text` - The text to display in the right section
    ///
    /// # Example
    ///
    /// ```
    /// use saorsa_ui::App;
    ///
    /// let mut app = App::new();
    /// app.set_status_right("Ln 42, Col 8");
    /// ```
    pub fn set_status_right(&mut self, text: impl Into<String>) {
        self.status_right = text.into();
    }

    /// Sets the theme
    ///
    /// # Arguments
    ///
    /// * `theme` - The new theme to use
    ///
    /// # Example
    ///
    /// ```
    /// use saorsa_ui::App;
    /// use saorsa_cli_core::{Theme, AppCoordinator};
    ///
    /// let mut app = App::new();
    /// app.set_theme(Theme::nord());
    /// assert_eq!(app.theme().name, "Nord");
    /// ```
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    /// Gets the tab manager
    ///
    /// # Example
    ///
    /// ```
    /// use saorsa_ui::App;
    /// use saorsa_cli_core::AppCoordinator;
    ///
    /// let app = App::new();
    /// assert!(app.tab_manager().is_empty());
    /// ```
    #[must_use]
    pub fn tab_manager(&self) -> &TabManager {
        &self.tab_manager
    }

    /// Gets the tab manager mutably
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut app = App::new();
    /// app.tab_manager_mut().add_tab(Box::new(my_tab));
    /// ```
    pub fn tab_manager_mut(&mut self) -> &mut TabManager {
        &mut self.tab_manager
    }

    /// Renders the entire application
    ///
    /// This draws the tab bar, active tab content, and status bar.
    ///
    /// # Arguments
    ///
    /// * `frame` - The ratatui frame to render to
    ///
    /// # Example
    ///
    /// ```ignore
    /// terminal.draw(|f| app.render(f))?;
    /// ```
    pub fn render(&self, frame: &mut Frame) {
        let layout = AppLayout::new(frame.area());

        // Render tab bar
        let tab_bar = TabBar::new(
            self.tab_manager.tabs(),
            self.tab_manager.active_index(),
            &self.theme,
        );
        frame.render_widget(tab_bar, layout.tab_bar);

        // Render active tab content
        if let Some(tab) = self.tab_manager.active_tab() {
            tab.view(frame, layout.content);
        }

        // Render status bar
        let status = StatusBar::new(&self.theme)
            .left(&self.status_left)
            .center(&self.status_center)
            .right(&self.status_right);
        frame.render_widget(status, layout.status_bar);
    }

    /// Returns whether the app has any tabs
    ///
    /// # Example
    ///
    /// ```
    /// use saorsa_ui::App;
    /// use saorsa_cli_core::AppCoordinator;
    ///
    /// let app = App::new();
    /// assert!(app.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tab_manager.is_empty()
    }

    /// Returns the number of tabs
    ///
    /// # Example
    ///
    /// ```
    /// use saorsa_ui::App;
    /// use saorsa_cli_core::AppCoordinator;
    ///
    /// let app = App::new();
    /// assert_eq!(app.tab_count(), 0);
    /// ```
    #[must_use]
    pub fn tab_count(&self) -> usize {
        self.tab_manager.len()
    }
}

impl AppCoordinator for App {
    fn tabs(&self) -> &[Box<dyn Tab>] {
        self.tab_manager.tabs()
    }

    fn active_tab(&self) -> TabId {
        self.tab_manager.active_id().unwrap_or(0)
    }

    fn theme(&self) -> &Theme {
        &self.theme
    }

    fn dispatch(&mut self, msg: Message) {
        let mut handled = false;
        match &msg {
            Message::Quit => {
                self.should_quit = true;
                handled = true;
            }
            Message::NextTab => {
                self.tab_manager.next_tab();
                handled = true;
            }
            Message::PrevTab => {
                self.tab_manager.prev_tab();
                handled = true;
            }
            Message::SwitchTab(id) => {
                let _ = self.tab_manager.switch_to(*id);
                handled = true;
            }
            Message::CloseTab(id) => {
                let _ = self.tab_manager.remove_tab(*id);
                handled = true;
            }
            Message::ToggleHelp => {
                // Toggle help hint in status bar
                if self.status_right.contains("help") {
                    self.status_right = "Press ? for help".to_string();
                } else {
                    self.status_right = "?:help  q:quit".to_string();
                }
                handled = true;
            }
            Message::Batch(messages) => {
                // Process batch messages recursively
                for m in messages.clone() {
                    self.dispatch(m);
                }
                return; // Skip broadcasting the batch itself
            }
            Message::None => {
                return; // No-op, skip broadcasting
            }
            _ => {}
        }

        if !handled {
            if let Some(tab) = self.tab_manager.active_tab_mut() {
                if let Some(response) = tab.handle_message(&msg) {
                    self.dispatch(response);
                }
            }
        }

        // Broadcast message to all subscribers
        let _ = self.message_bus.send(msg);
    }

    fn tick(&mut self) {
        if let Some(tab) = self.tab_manager.active_tab_mut() {
            tab.tick();
        }
    }

    fn should_quit(&self) -> bool {
        self.should_quit
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saorsa_cli_core::Tab;

    /// Test implementation of the Tab trait
    struct MockTab {
        id: TabId,
        title: String,
    }

    impl MockTab {
        fn new(id: TabId, title: &str) -> Self {
            MockTab {
                id,
                title: title.to_string(),
            }
        }
    }

    impl Tab for MockTab {
        fn id(&self) -> TabId {
            self.id
        }

        fn title(&self) -> &str {
            &self.title
        }

        fn focus(&mut self) {}

        fn blur(&mut self) {}

        fn view(&self, _frame: &mut Frame, _area: Rect) {}
    }

    #[test]
    fn test_app_new() {
        let app = App::new();
        assert!(!app.should_quit());
        assert_eq!(app.tabs().len(), 0);
        assert!(app.is_empty());
    }

    #[test]
    fn test_app_default() {
        let app = App::default();
        assert!(!app.should_quit());
        assert!(app.is_empty());
    }

    #[test]
    fn test_app_with_theme() {
        let app = App::with_theme(Theme::light());
        assert_eq!(app.theme().name, "Light");
    }

    #[test]
    fn test_app_add_tab() {
        let mut app = App::new();
        let id = app.add_tab(Box::new(MockTab::new(1, "Test")));
        assert_eq!(id, 1);
        assert_eq!(app.tabs().len(), 1);
        assert!(!app.is_empty());
        assert_eq!(app.tab_count(), 1);
    }

    #[test]
    fn test_app_remove_tab() {
        let mut app = App::new();
        let id = app.add_tab(Box::new(MockTab::new(1, "Test")));
        assert!(app.remove_tab(id).is_ok());
        assert!(app.is_empty());
    }

    #[test]
    fn test_app_remove_nonexistent_tab() {
        let mut app = App::new();
        assert!(app.remove_tab(999).is_err());
    }

    #[test]
    fn test_app_dispatch_quit() {
        let mut app = App::new();
        assert!(!app.should_quit());
        app.dispatch(Message::Quit);
        assert!(app.should_quit());
    }

    #[test]
    fn test_app_dispatch_tab_navigation() {
        let mut app = App::new();
        app.add_tab(Box::new(MockTab::new(1, "Tab1")));
        app.add_tab(Box::new(MockTab::new(2, "Tab2")));

        assert_eq!(app.active_tab(), 1);
        app.dispatch(Message::NextTab);
        assert_eq!(app.active_tab(), 2);
        app.dispatch(Message::PrevTab);
        assert_eq!(app.active_tab(), 1);
    }

    #[test]
    fn test_app_dispatch_switch_tab() {
        let mut app = App::new();
        app.add_tab(Box::new(MockTab::new(1, "Tab1")));
        app.add_tab(Box::new(MockTab::new(2, "Tab2")));

        app.dispatch(Message::SwitchTab(2));
        assert_eq!(app.active_tab(), 2);
    }

    #[test]
    fn test_app_dispatch_close_tab() {
        let mut app = App::new();
        app.add_tab(Box::new(MockTab::new(1, "Tab1")));
        app.add_tab(Box::new(MockTab::new(2, "Tab2")));

        app.dispatch(Message::CloseTab(1));
        assert_eq!(app.tab_count(), 1);
        assert_eq!(app.active_tab(), 2);
    }

    #[test]
    fn test_app_dispatch_toggle_help() {
        let mut app = App::new();
        let initial = app.status_right.clone();

        app.dispatch(Message::ToggleHelp);
        assert_ne!(app.status_right, initial);

        app.dispatch(Message::ToggleHelp);
        // Should toggle back
    }

    #[test]
    fn test_app_dispatch_none() {
        let mut app = App::new();
        app.dispatch(Message::None);
        // Should not crash or change state
        assert!(!app.should_quit());
    }

    #[test]
    fn test_app_dispatch_batch() {
        let mut app = App::new();
        app.add_tab(Box::new(MockTab::new(1, "Tab1")));
        app.add_tab(Box::new(MockTab::new(2, "Tab2")));

        app.dispatch(Message::Batch(vec![Message::NextTab, Message::ToggleHelp]));

        assert_eq!(app.active_tab(), 2);
    }

    #[test]
    fn test_app_status_bar() {
        let mut app = App::new();
        app.set_status_left("MODE");
        app.set_status_center("file.rs");
        app.set_status_right("help");

        // Just verify no panics - actual rendering tested in widget tests
    }

    #[test]
    fn test_app_set_theme() {
        let mut app = App::new();
        app.set_theme(Theme::nord());
        assert_eq!(app.theme().name, "Nord");
    }

    #[test]
    fn test_app_message_bus() {
        let app = App::new();
        let _rx = app.message_bus().subscribe();
        // Verify we can subscribe
    }

    #[test]
    fn test_app_tab_manager() {
        let mut app = App::new();
        assert!(app.tab_manager().is_empty());

        let id = app
            .tab_manager_mut()
            .add_tab(Box::new(MockTab::new(1, "Test")));
        assert_eq!(id, 1);
        assert!(!app.tab_manager().is_empty());
    }

    #[test]
    fn test_app_tick() {
        let mut app = App::new();
        app.tick(); // Should not panic
    }

    #[test]
    fn test_app_multiple_tabs() {
        let mut app = App::new();

        for i in 1..=5 {
            app.add_tab(Box::new(MockTab::new(i, &format!("Tab{}", i))));
        }

        assert_eq!(app.tab_count(), 5);
        assert_eq!(app.active_tab(), 1);

        // Navigate to last tab
        for _ in 0..4 {
            app.dispatch(Message::NextTab);
        }
        assert_eq!(app.active_tab(), 5);

        // Wrap around
        app.dispatch(Message::NextTab);
        assert_eq!(app.active_tab(), 1);
    }

    #[test]
    fn test_app_empty_active_tab() {
        let app = App::new();
        // Should return 0 when no tabs
        assert_eq!(app.active_tab(), 0);
    }
}
