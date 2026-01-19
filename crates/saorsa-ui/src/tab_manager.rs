//! Tab management for the TUI framework
//!
//! The [`TabManager`] struct manages a collection of tabs, handling
//! tab switching, addition, removal, and focus management.
//!
//! # Example
//!
//! ```ignore
//! use saorsa_ui::TabManager;
//! use saorsa_cli_core::Tab;
//!
//! let mut manager = TabManager::new();
//! let tab_id = manager.add_tab(Box::new(my_tab));
//! manager.switch_to(tab_id)?;
//! ```

use saorsa_cli_core::{CoreError, CoreResult, Message, Tab, TabId};
use std::collections::HashMap;

/// Manages a collection of tabs in the TUI.
///
/// TabManager handles:
/// - Adding and removing tabs
/// - Switching between tabs (by ID, next, previous)
/// - Focus management (calling focus/blur on tabs)
/// - Message routing for tab-related operations
///
/// # Thread Safety
///
/// TabManager itself is not thread-safe. If you need to share it across
/// threads, wrap it in appropriate synchronization primitives.
///
/// # Example
///
/// ```ignore
/// use saorsa_ui::TabManager;
///
/// let mut manager = TabManager::new();
/// assert!(manager.is_empty());
///
/// // Add tabs
/// let id = manager.add_tab(Box::new(my_tab));
/// assert_eq!(manager.len(), 1);
///
/// // Navigate between tabs
/// manager.next_tab();
/// manager.prev_tab();
/// ```
pub struct TabManager {
    /// Collection of tabs, stored in order
    tabs: Vec<Box<dyn Tab>>,
    /// Index of the currently active tab
    active_index: usize,
    /// Mapping from TabId to index for O(1) lookup
    tab_indices: HashMap<TabId, usize>,
}

impl TabManager {
    /// Creates a new empty tab manager.
    ///
    /// # Example
    ///
    /// ```
    /// use saorsa_ui::TabManager;
    ///
    /// let manager = TabManager::new();
    /// assert!(manager.is_empty());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        TabManager {
            tabs: Vec::new(),
            active_index: 0,
            tab_indices: HashMap::new(),
        }
    }

    /// Adds a tab and returns its ID.
    ///
    /// The new tab is added at the end. If this is the first tab,
    /// it automatically becomes active and receives focus.
    ///
    /// # Arguments
    ///
    /// * `tab` - The tab to add, boxed as a trait object
    ///
    /// # Returns
    ///
    /// The unique identifier of the added tab
    ///
    /// # Example
    ///
    /// ```ignore
    /// let id = manager.add_tab(Box::new(my_tab));
    /// assert_eq!(manager.len(), 1);
    /// ```
    pub fn add_tab(&mut self, mut tab: Box<dyn Tab>) -> TabId {
        let id = tab.id();
        let index = self.tabs.len();

        // If first tab, give it focus
        if self.tabs.is_empty() {
            tab.focus();
        }

        self.tab_indices.insert(id, index);
        self.tabs.push(tab);
        id
    }

    /// Removes a tab by ID.
    ///
    /// If the removed tab was active, focus shifts to an adjacent tab.
    /// After removal, all tab indices are updated to maintain consistency.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the tab to remove
    ///
    /// # Errors
    ///
    /// Returns `CoreError::TabNotFound` if the tab doesn't exist.
    /// Returns `CoreError::InvalidLayout` if trying to remove the last tab
    /// when it cannot be closed.
    ///
    /// # Example
    ///
    /// ```ignore
    /// manager.remove_tab(tab_id)?;
    /// ```
    pub fn remove_tab(&mut self, id: TabId) -> CoreResult<()> {
        let index = self
            .tab_indices
            .get(&id)
            .copied()
            .ok_or(CoreError::TabNotFound(id))?;

        // Check if tab can be closed when it's the last one
        if !self.tabs[index].can_close() && self.tabs.len() == 1 {
            return Err(CoreError::InvalidLayout(
                "cannot close the last tab".to_string(),
            ));
        }

        // Blur if this was the active tab
        if index == self.active_index {
            self.tabs[index].blur();
        }

        // Remove the tab
        self.tabs.remove(index);
        self.tab_indices.remove(&id);

        // Update indices for tabs after the removed one
        for (_, idx) in self.tab_indices.iter_mut() {
            if *idx > index {
                *idx -= 1;
            }
        }

        // Adjust active index if needed
        if !self.tabs.is_empty() {
            if self.active_index >= self.tabs.len() {
                self.active_index = self.tabs.len() - 1;
            }
            // Focus the new active tab
            self.tabs[self.active_index].focus();
        }

        Ok(())
    }

    /// Gets a reference to the active tab.
    ///
    /// # Returns
    ///
    /// `Some(&dyn Tab)` if there is an active tab, `None` if the manager is empty.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Some(tab) = manager.active_tab() {
    ///     println!("Active tab: {}", tab.title());
    /// }
    /// ```
    #[must_use]
    pub fn active_tab(&self) -> Option<&dyn Tab> {
        self.tabs.get(self.active_index).map(|t| t.as_ref())
    }

    /// Gets a mutable reference to the active tab.
    ///
    /// # Returns
    ///
    /// `Some(&mut Box<dyn Tab>)` if there is an active tab, `None` if the manager is empty.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Some(tab) = manager.active_tab_mut() {
    ///     // Modify the active tab
    /// }
    /// ```
    pub fn active_tab_mut(&mut self) -> Option<&mut Box<dyn Tab>> {
        self.tabs.get_mut(self.active_index)
    }

    /// Gets the active tab's ID.
    ///
    /// # Returns
    ///
    /// `Some(TabId)` if there is an active tab, `None` if the manager is empty.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Some(id) = manager.active_id() {
    ///     println!("Active tab ID: {}", id);
    /// }
    /// ```
    #[must_use]
    pub fn active_id(&self) -> Option<TabId> {
        self.tabs.get(self.active_index).map(|t| t.id())
    }

    /// Gets the active tab index.
    ///
    /// # Returns
    ///
    /// The zero-based index of the currently active tab.
    /// Returns 0 if the manager is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use saorsa_ui::TabManager;
    ///
    /// let manager = TabManager::new();
    /// assert_eq!(manager.active_index(), 0);
    /// ```
    #[must_use]
    pub fn active_index(&self) -> usize {
        self.active_index
    }

    /// Switches to a specific tab by ID.
    ///
    /// The currently active tab receives a blur event, and the
    /// newly activated tab receives a focus event.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the tab to switch to
    ///
    /// # Errors
    ///
    /// Returns `CoreError::TabNotFound` if the tab doesn't exist.
    ///
    /// # Example
    ///
    /// ```ignore
    /// manager.switch_to(tab_id)?;
    /// ```
    pub fn switch_to(&mut self, id: TabId) -> CoreResult<()> {
        let new_index = self
            .tab_indices
            .get(&id)
            .copied()
            .ok_or(CoreError::TabNotFound(id))?;

        if new_index != self.active_index && !self.tabs.is_empty() {
            // Blur old tab
            self.tabs[self.active_index].blur();
            // Update index
            self.active_index = new_index;
            // Focus new tab
            self.tabs[self.active_index].focus();
        }

        Ok(())
    }

    /// Switches to the next tab (wraps around).
    ///
    /// If there is only one tab or no tabs, this is a no-op.
    /// The active tab receives a blur event, and the next tab
    /// receives a focus event.
    ///
    /// # Example
    ///
    /// ```ignore
    /// manager.next_tab();
    /// ```
    pub fn next_tab(&mut self) {
        if self.tabs.len() <= 1 {
            return;
        }

        self.tabs[self.active_index].blur();
        self.active_index = (self.active_index + 1) % self.tabs.len();
        self.tabs[self.active_index].focus();
    }

    /// Switches to the previous tab (wraps around).
    ///
    /// If there is only one tab or no tabs, this is a no-op.
    /// The active tab receives a blur event, and the previous tab
    /// receives a focus event.
    ///
    /// # Example
    ///
    /// ```ignore
    /// manager.prev_tab();
    /// ```
    pub fn prev_tab(&mut self) {
        if self.tabs.len() <= 1 {
            return;
        }

        self.tabs[self.active_index].blur();
        self.active_index = if self.active_index == 0 {
            self.tabs.len() - 1
        } else {
            self.active_index - 1
        };
        self.tabs[self.active_index].focus();
    }

    /// Returns a slice of all tabs.
    ///
    /// # Returns
    ///
    /// A slice containing references to all tabs in order.
    ///
    /// # Example
    ///
    /// ```ignore
    /// for tab in manager.tabs() {
    ///     println!("Tab: {}", tab.title());
    /// }
    /// ```
    #[must_use]
    pub fn tabs(&self) -> &[Box<dyn Tab>] {
        &self.tabs
    }

    /// Returns the number of tabs.
    ///
    /// # Example
    ///
    /// ```
    /// use saorsa_ui::TabManager;
    ///
    /// let manager = TabManager::new();
    /// assert_eq!(manager.len(), 0);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.tabs.len()
    }

    /// Returns true if there are no tabs.
    ///
    /// # Example
    ///
    /// ```
    /// use saorsa_ui::TabManager;
    ///
    /// let manager = TabManager::new();
    /// assert!(manager.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    /// Gets a tab by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the tab to retrieve
    ///
    /// # Returns
    ///
    /// `Some(&dyn Tab)` if the tab exists, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Some(tab) = manager.get(tab_id) {
    ///     println!("Found tab: {}", tab.title());
    /// }
    /// ```
    #[must_use]
    pub fn get(&self, id: TabId) -> Option<&dyn Tab> {
        self.tab_indices
            .get(&id)
            .and_then(|&idx| self.tabs.get(idx))
            .map(|t| t.as_ref())
    }

    /// Gets a mutable tab by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the tab to retrieve
    ///
    /// # Returns
    ///
    /// `Some(&mut Box<dyn Tab>)` if the tab exists, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Some(tab) = manager.get_mut(tab_id) {
    ///     // Modify the tab
    /// }
    /// ```
    pub fn get_mut(&mut self, id: TabId) -> Option<&mut Box<dyn Tab>> {
        if let Some(&idx) = self.tab_indices.get(&id) {
            self.tabs.get_mut(idx)
        } else {
            None
        }
    }

    /// Handles a tab-related message.
    ///
    /// Processes messages like `SwitchTab`, `CloseTab`, `NextTab`, and `PrevTab`.
    /// Other messages are returned unchanged for further processing.
    ///
    /// # Arguments
    ///
    /// * `msg` - The message to handle
    ///
    /// # Returns
    ///
    /// `Some(Message)` if the message should be propagated (not handled),
    /// `None` if the message was consumed.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Some(unhandled) = manager.handle_message(&Message::NextTab) {
    ///     // Message was not tab-related, handle elsewhere
    /// }
    /// ```
    pub fn handle_message(&mut self, msg: &Message) -> Option<Message> {
        match msg {
            Message::SwitchTab(id) => {
                let _ = self.switch_to(*id);
                None
            }
            Message::CloseTab(id) => {
                let _ = self.remove_tab(*id);
                None
            }
            Message::NextTab => {
                self.next_tab();
                None
            }
            Message::PrevTab => {
                self.prev_tab();
                None
            }
            _ => Some(msg.clone()),
        }
    }
}

impl Default for TabManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::prelude::*;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use std::sync::Arc;

    /// Mock tab implementation for testing
    struct MockTab {
        id: TabId,
        title: String,
        can_close: bool,
        focused: Arc<AtomicBool>,
        focus_count: Arc<AtomicU32>,
        blur_count: Arc<AtomicU32>,
    }

    impl MockTab {
        fn new(id: TabId, title: &str) -> Self {
            MockTab {
                id,
                title: title.to_string(),
                can_close: true,
                focused: Arc::new(AtomicBool::new(false)),
                focus_count: Arc::new(AtomicU32::new(0)),
                blur_count: Arc::new(AtomicU32::new(0)),
            }
        }

        fn with_can_close(mut self, can_close: bool) -> Self {
            self.can_close = can_close;
            self
        }
    }

    impl Tab for MockTab {
        fn id(&self) -> TabId {
            self.id
        }

        fn title(&self) -> &str {
            &self.title
        }

        fn can_close(&self) -> bool {
            self.can_close
        }

        fn focus(&mut self) {
            self.focused.store(true, Ordering::SeqCst);
            self.focus_count.fetch_add(1, Ordering::SeqCst);
        }

        fn blur(&mut self) {
            self.focused.store(false, Ordering::SeqCst);
            self.blur_count.fetch_add(1, Ordering::SeqCst);
        }

        fn view(&self, _frame: &mut Frame, _area: Rect) {
            // Test implementation does nothing
        }
    }

    // ==================== Basic Construction Tests ====================

    #[test]
    fn test_new_creates_empty_manager() {
        let manager = TabManager::new();
        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
        assert_eq!(manager.active_index(), 0);
    }

    #[test]
    fn test_default_creates_empty_manager() {
        let manager = TabManager::default();
        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
    }

    // ==================== Add Tab Tests ====================

    #[test]
    fn test_add_tab_increases_count() {
        let mut manager = TabManager::new();
        let tab = MockTab::new(1, "Tab 1");
        manager.add_tab(Box::new(tab));

        assert_eq!(manager.len(), 1);
        assert!(!manager.is_empty());
    }

    #[test]
    fn test_add_tab_returns_id() {
        let mut manager = TabManager::new();
        let tab = MockTab::new(42, "Tab 42");
        let id = manager.add_tab(Box::new(tab));

        assert_eq!(id, 42);
    }

    #[test]
    fn test_first_tab_gets_focus() {
        let mut manager = TabManager::new();
        let tab = MockTab::new(1, "Tab 1");
        let focused = Arc::clone(&tab.focused);
        let focus_count = Arc::clone(&tab.focus_count);

        manager.add_tab(Box::new(tab));

        assert!(focused.load(Ordering::SeqCst));
        assert_eq!(focus_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_subsequent_tabs_dont_get_focus() {
        let mut manager = TabManager::new();

        let tab1 = MockTab::new(1, "Tab 1");
        manager.add_tab(Box::new(tab1));

        let tab2 = MockTab::new(2, "Tab 2");
        let focused2 = Arc::clone(&tab2.focused);
        manager.add_tab(Box::new(tab2));

        assert!(!focused2.load(Ordering::SeqCst));
    }

    #[test]
    fn test_add_multiple_tabs() {
        let mut manager = TabManager::new();

        for i in 1..=5 {
            let tab = MockTab::new(i, &format!("Tab {}", i));
            manager.add_tab(Box::new(tab));
        }

        assert_eq!(manager.len(), 5);
    }

    // ==================== Remove Tab Tests ====================

    #[test]
    fn test_remove_tab_decreases_count() {
        let mut manager = TabManager::new();

        let tab1 = MockTab::new(1, "Tab 1");
        let tab2 = MockTab::new(2, "Tab 2");
        manager.add_tab(Box::new(tab1));
        manager.add_tab(Box::new(tab2));

        manager.remove_tab(1).expect("should remove tab");
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn test_remove_tab_not_found() {
        let mut manager = TabManager::new();
        let tab = MockTab::new(1, "Tab 1");
        manager.add_tab(Box::new(tab));

        let result = manager.remove_tab(999);
        assert!(result.is_err());
        assert!(matches!(result, Err(CoreError::TabNotFound(999))));
    }

    #[test]
    fn test_remove_tab_updates_indices() {
        let mut manager = TabManager::new();

        let tab1 = MockTab::new(1, "Tab 1");
        let tab2 = MockTab::new(2, "Tab 2");
        let tab3 = MockTab::new(3, "Tab 3");

        manager.add_tab(Box::new(tab1));
        manager.add_tab(Box::new(tab2));
        manager.add_tab(Box::new(tab3));

        // Remove the middle tab
        manager.remove_tab(2).expect("should remove tab");

        // Tab 3 should now be at index 1
        assert!(manager.get(3).is_some());
        assert_eq!(manager.len(), 2);
    }

    #[test]
    fn test_remove_active_tab_shifts_focus() {
        let mut manager = TabManager::new();

        let tab1 = MockTab::new(1, "Tab 1");
        let tab2 = MockTab::new(2, "Tab 2");
        let tab2_focused = Arc::clone(&tab2.focused);

        manager.add_tab(Box::new(tab1));
        manager.add_tab(Box::new(tab2));

        // Remove the active (first) tab
        manager.remove_tab(1).expect("should remove tab");

        // Tab 2 should now be focused
        assert!(tab2_focused.load(Ordering::SeqCst));
        assert_eq!(manager.active_id(), Some(2));
    }

    #[test]
    fn test_remove_last_tab_when_uncloseable() {
        let mut manager = TabManager::new();

        let tab = MockTab::new(1, "Tab 1").with_can_close(false);
        manager.add_tab(Box::new(tab));

        let result = manager.remove_tab(1);
        assert!(result.is_err());
        assert!(matches!(result, Err(CoreError::InvalidLayout(_))));
    }

    #[test]
    fn test_remove_last_tab_when_closeable() {
        let mut manager = TabManager::new();

        let tab = MockTab::new(1, "Tab 1").with_can_close(true);
        manager.add_tab(Box::new(tab));

        let result = manager.remove_tab(1);
        assert!(result.is_ok());
        assert!(manager.is_empty());
    }

    // ==================== Active Tab Tests ====================

    #[test]
    fn test_active_tab_empty_manager() {
        let manager = TabManager::new();
        assert!(manager.active_tab().is_none());
        assert!(manager.active_id().is_none());
    }

    #[test]
    fn test_active_tab_returns_first_added() {
        let mut manager = TabManager::new();

        let tab1 = MockTab::new(1, "Tab 1");
        let tab2 = MockTab::new(2, "Tab 2");

        manager.add_tab(Box::new(tab1));
        manager.add_tab(Box::new(tab2));

        assert_eq!(manager.active_tab().map(|t| t.id()), Some(1));
        assert_eq!(manager.active_id(), Some(1));
    }

    #[test]
    fn test_active_tab_mut() {
        let mut manager = TabManager::new();
        let tab = MockTab::new(1, "Tab 1");
        manager.add_tab(Box::new(tab));

        let active = manager.active_tab_mut();
        assert!(active.is_some());
    }

    // ==================== Switch To Tests ====================

    #[test]
    fn test_switch_to_valid_tab() {
        let mut manager = TabManager::new();

        let tab1 = MockTab::new(1, "Tab 1");
        let tab1_blur_count = Arc::clone(&tab1.blur_count);

        let tab2 = MockTab::new(2, "Tab 2");
        let tab2_focus_count = Arc::clone(&tab2.focus_count);

        manager.add_tab(Box::new(tab1));
        manager.add_tab(Box::new(tab2));

        manager.switch_to(2).expect("should switch");

        assert_eq!(manager.active_id(), Some(2));
        assert_eq!(tab1_blur_count.load(Ordering::SeqCst), 1);
        // Tab 2 was not focused initially, so focus count should be 1 after switch
        assert_eq!(tab2_focus_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_switch_to_invalid_tab() {
        let mut manager = TabManager::new();
        let tab = MockTab::new(1, "Tab 1");
        manager.add_tab(Box::new(tab));

        let result = manager.switch_to(999);
        assert!(result.is_err());
        assert!(matches!(result, Err(CoreError::TabNotFound(999))));
    }

    #[test]
    fn test_switch_to_same_tab_no_op() {
        let mut manager = TabManager::new();

        let tab = MockTab::new(1, "Tab 1");
        let blur_count = Arc::clone(&tab.blur_count);
        let focus_count = Arc::clone(&tab.focus_count);

        manager.add_tab(Box::new(tab));

        // Initial focus was called once
        assert_eq!(focus_count.load(Ordering::SeqCst), 1);

        // Switch to same tab should not blur/focus again
        manager.switch_to(1).expect("should switch");
        assert_eq!(blur_count.load(Ordering::SeqCst), 0);
        assert_eq!(focus_count.load(Ordering::SeqCst), 1);
    }

    // ==================== Next/Prev Tab Tests ====================

    #[test]
    fn test_next_tab_cycles() {
        let mut manager = TabManager::new();

        let tab1 = MockTab::new(1, "Tab 1");
        let tab2 = MockTab::new(2, "Tab 2");
        let tab3 = MockTab::new(3, "Tab 3");

        manager.add_tab(Box::new(tab1));
        manager.add_tab(Box::new(tab2));
        manager.add_tab(Box::new(tab3));

        assert_eq!(manager.active_id(), Some(1));

        manager.next_tab();
        assert_eq!(manager.active_id(), Some(2));

        manager.next_tab();
        assert_eq!(manager.active_id(), Some(3));

        manager.next_tab();
        assert_eq!(manager.active_id(), Some(1)); // Wraps around
    }

    #[test]
    fn test_prev_tab_cycles() {
        let mut manager = TabManager::new();

        let tab1 = MockTab::new(1, "Tab 1");
        let tab2 = MockTab::new(2, "Tab 2");
        let tab3 = MockTab::new(3, "Tab 3");

        manager.add_tab(Box::new(tab1));
        manager.add_tab(Box::new(tab2));
        manager.add_tab(Box::new(tab3));

        assert_eq!(manager.active_id(), Some(1));

        manager.prev_tab();
        assert_eq!(manager.active_id(), Some(3)); // Wraps to end

        manager.prev_tab();
        assert_eq!(manager.active_id(), Some(2));

        manager.prev_tab();
        assert_eq!(manager.active_id(), Some(1));
    }

    #[test]
    fn test_next_tab_single_tab_no_op() {
        let mut manager = TabManager::new();
        let tab = MockTab::new(1, "Tab 1");
        let blur_count = Arc::clone(&tab.blur_count);

        manager.add_tab(Box::new(tab));

        manager.next_tab();

        // Should not have blurred since there's only one tab
        assert_eq!(blur_count.load(Ordering::SeqCst), 0);
        assert_eq!(manager.active_id(), Some(1));
    }

    #[test]
    fn test_prev_tab_single_tab_no_op() {
        let mut manager = TabManager::new();
        let tab = MockTab::new(1, "Tab 1");
        let blur_count = Arc::clone(&tab.blur_count);

        manager.add_tab(Box::new(tab));

        manager.prev_tab();

        // Should not have blurred since there's only one tab
        assert_eq!(blur_count.load(Ordering::SeqCst), 0);
        assert_eq!(manager.active_id(), Some(1));
    }

    #[test]
    fn test_next_tab_empty_no_op() {
        let mut manager = TabManager::new();
        manager.next_tab(); // Should not panic
        assert!(manager.is_empty());
    }

    #[test]
    fn test_prev_tab_empty_no_op() {
        let mut manager = TabManager::new();
        manager.prev_tab(); // Should not panic
        assert!(manager.is_empty());
    }

    // ==================== Get Tab Tests ====================

    #[test]
    fn test_get_existing_tab() {
        let mut manager = TabManager::new();
        let tab = MockTab::new(42, "The Answer");
        manager.add_tab(Box::new(tab));

        let retrieved = manager.get(42);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.map(|t| t.title()), Some("The Answer"));
    }

    #[test]
    fn test_get_nonexistent_tab() {
        let manager = TabManager::new();
        assert!(manager.get(1).is_none());
    }

    #[test]
    fn test_get_mut_existing_tab() {
        let mut manager = TabManager::new();
        let tab = MockTab::new(42, "Tab");
        manager.add_tab(Box::new(tab));

        let retrieved = manager.get_mut(42);
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_get_mut_nonexistent_tab() {
        let mut manager = TabManager::new();
        assert!(manager.get_mut(1).is_none());
    }

    // ==================== Tabs Slice Tests ====================

    #[test]
    fn test_tabs_returns_all() {
        let mut manager = TabManager::new();

        for i in 1..=3 {
            let tab = MockTab::new(i, &format!("Tab {}", i));
            manager.add_tab(Box::new(tab));
        }

        let tabs = manager.tabs();
        assert_eq!(tabs.len(), 3);
    }

    #[test]
    fn test_tabs_empty() {
        let manager = TabManager::new();
        let tabs = manager.tabs();
        assert!(tabs.is_empty());
    }

    // ==================== Message Handling Tests ====================

    #[test]
    fn test_handle_message_switch_tab() {
        let mut manager = TabManager::new();

        let tab1 = MockTab::new(1, "Tab 1");
        let tab2 = MockTab::new(2, "Tab 2");
        manager.add_tab(Box::new(tab1));
        manager.add_tab(Box::new(tab2));

        let result = manager.handle_message(&Message::SwitchTab(2));
        assert!(result.is_none()); // Message was consumed
        assert_eq!(manager.active_id(), Some(2));
    }

    #[test]
    fn test_handle_message_close_tab() {
        let mut manager = TabManager::new();

        let tab1 = MockTab::new(1, "Tab 1");
        let tab2 = MockTab::new(2, "Tab 2");
        manager.add_tab(Box::new(tab1));
        manager.add_tab(Box::new(tab2));

        let result = manager.handle_message(&Message::CloseTab(1));
        assert!(result.is_none()); // Message was consumed
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn test_handle_message_next_tab() {
        let mut manager = TabManager::new();

        let tab1 = MockTab::new(1, "Tab 1");
        let tab2 = MockTab::new(2, "Tab 2");
        manager.add_tab(Box::new(tab1));
        manager.add_tab(Box::new(tab2));

        let result = manager.handle_message(&Message::NextTab);
        assert!(result.is_none()); // Message was consumed
        assert_eq!(manager.active_id(), Some(2));
    }

    #[test]
    fn test_handle_message_prev_tab() {
        let mut manager = TabManager::new();

        let tab1 = MockTab::new(1, "Tab 1");
        let tab2 = MockTab::new(2, "Tab 2");
        manager.add_tab(Box::new(tab1));
        manager.add_tab(Box::new(tab2));

        // Switch to tab 2 first
        manager.switch_to(2).expect("should switch");

        let result = manager.handle_message(&Message::PrevTab);
        assert!(result.is_none()); // Message was consumed
        assert_eq!(manager.active_id(), Some(1));
    }

    #[test]
    fn test_handle_message_unhandled() {
        let mut manager = TabManager::new();
        let tab = MockTab::new(1, "Tab 1");
        manager.add_tab(Box::new(tab));

        let result = manager.handle_message(&Message::Quit);
        assert!(result.is_some()); // Message was not consumed
        assert!(matches!(result, Some(Message::Quit)));
    }

    #[test]
    fn test_handle_message_toggle_help() {
        let mut manager = TabManager::new();

        let result = manager.handle_message(&Message::ToggleHelp);
        assert!(result.is_some());
        assert!(matches!(result, Some(Message::ToggleHelp)));
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_remove_all_tabs_sequentially() {
        let mut manager = TabManager::new();

        for i in 1..=3 {
            let tab = MockTab::new(i, &format!("Tab {}", i));
            manager.add_tab(Box::new(tab));
        }

        manager.remove_tab(1).expect("remove 1");
        manager.remove_tab(2).expect("remove 2");
        manager.remove_tab(3).expect("remove 3");

        assert!(manager.is_empty());
    }

    #[test]
    fn test_active_index_after_removing_last() {
        let mut manager = TabManager::new();

        let tab1 = MockTab::new(1, "Tab 1");
        let tab2 = MockTab::new(2, "Tab 2");
        let tab3 = MockTab::new(3, "Tab 3");

        manager.add_tab(Box::new(tab1));
        manager.add_tab(Box::new(tab2));
        manager.add_tab(Box::new(tab3));

        // Switch to last tab
        manager.switch_to(3).expect("switch to 3");
        assert_eq!(manager.active_index(), 2);

        // Remove last tab
        manager.remove_tab(3).expect("remove 3");

        // Active index should be adjusted
        assert!(manager.active_index() < manager.len());
        assert_eq!(manager.active_index(), 1);
    }

    #[test]
    fn test_focus_blur_counts() {
        let mut manager = TabManager::new();

        let tab1 = MockTab::new(1, "Tab 1");
        let tab1_focus = Arc::clone(&tab1.focus_count);
        let tab1_blur = Arc::clone(&tab1.blur_count);

        let tab2 = MockTab::new(2, "Tab 2");
        let tab2_focus = Arc::clone(&tab2.focus_count);
        let tab2_blur = Arc::clone(&tab2.blur_count);

        manager.add_tab(Box::new(tab1));
        manager.add_tab(Box::new(tab2));

        // Tab 1 focused once on add
        assert_eq!(tab1_focus.load(Ordering::SeqCst), 1);
        assert_eq!(tab1_blur.load(Ordering::SeqCst), 0);

        // Tab 2 not focused yet
        assert_eq!(tab2_focus.load(Ordering::SeqCst), 0);
        assert_eq!(tab2_blur.load(Ordering::SeqCst), 0);

        // Switch to tab 2
        manager.next_tab();
        assert_eq!(tab1_focus.load(Ordering::SeqCst), 1);
        assert_eq!(tab1_blur.load(Ordering::SeqCst), 1);
        assert_eq!(tab2_focus.load(Ordering::SeqCst), 1);
        assert_eq!(tab2_blur.load(Ordering::SeqCst), 0);

        // Switch back to tab 1
        manager.prev_tab();
        assert_eq!(tab1_focus.load(Ordering::SeqCst), 2);
        assert_eq!(tab1_blur.load(Ordering::SeqCst), 1);
        assert_eq!(tab2_focus.load(Ordering::SeqCst), 1);
        assert_eq!(tab2_blur.load(Ordering::SeqCst), 1);
    }
}
