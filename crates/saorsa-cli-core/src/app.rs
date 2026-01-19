//! Application coordinator trait and types
//!
//! The `AppCoordinator` trait defines the interface for the main application
//! loop that coordinates tabs, handles input, and manages the overall state.

use crate::event::Message;
use crate::tab::{Tab, TabId};
use crate::theme::Theme;

/// Trait for the main application coordinator
///
/// This trait defines the interface for managing the application lifecycle,
/// including tab management, theme handling, and message dispatching.
///
/// # Thread Safety
///
/// Implementations should be careful about thread safety when dealing with
/// tabs and message dispatching, as the TUI may run across multiple threads.
///
/// # Example
///
/// ```ignore
/// use saorsa_cli_core::{AppCoordinator, Tab, TabId, Message, Theme};
///
/// struct MyApp {
///     tabs: Vec<Box<dyn Tab>>,
///     active: TabId,
///     theme: Theme,
///     quit: bool,
/// }
///
/// impl AppCoordinator for MyApp {
///     fn tabs(&self) -> &[Box<dyn Tab>] { &self.tabs }
///     fn active_tab(&self) -> TabId { self.active }
///     fn theme(&self) -> &Theme { &self.theme }
///     fn dispatch(&mut self, msg: Message) {
///         match msg {
///             Message::Quit => self.quit = true,
///             _ => {}
///         }
///     }
///     fn tick(&mut self) { /* periodic updates */ }
///     fn should_quit(&self) -> bool { self.quit }
/// }
/// ```
pub trait AppCoordinator {
    /// Returns a slice of all tabs
    ///
    /// The order of tabs in the slice corresponds to their display order
    /// in the tab bar.
    fn tabs(&self) -> &[Box<dyn Tab>];

    /// Returns the currently active tab ID
    ///
    /// The active tab is the one currently receiving input focus
    /// and being displayed in the main content area.
    fn active_tab(&self) -> TabId;

    /// Returns the current theme
    ///
    /// The theme is used by all UI components for consistent styling.
    fn theme(&self) -> &Theme;

    /// Dispatch a message to be processed
    ///
    /// Messages can trigger state changes, navigate between tabs,
    /// or trigger custom actions. Implementations should handle
    /// all relevant message types from the `Message` enum.
    fn dispatch(&mut self, msg: Message);

    /// Called on each tick of the main loop
    ///
    /// This method is called periodically (typically every 100-250ms)
    /// and can be used for animations, polling, or other periodic updates.
    fn tick(&mut self);

    /// Returns true if the application should quit
    ///
    /// The main loop checks this after processing each batch of events.
    /// When this returns `true`, the application will begin shutdown.
    fn should_quit(&self) -> bool;
}
