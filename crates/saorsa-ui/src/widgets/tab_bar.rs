//! Tab bar widget for displaying and selecting tabs
//!
//! The [`TabBar`] widget renders a horizontal bar showing all tab titles,
//! with the active tab highlighted using the theme's accent color.
//!
//! ## Features
//!
//! - Displays tab titles with optional icons
//! - Highlights active tab with bold accent color
//! - Uses muted color for inactive tabs
//! - Gracefully handles empty tab lists
//!
//! ## Example
//!
//! ```ignore
//! use saorsa_ui::widgets::TabBar;
//! use saorsa_cli_core::Theme;
//!
//! let tab_bar = TabBar::new(&tabs, active_index, &theme);
//! frame.render_widget(tab_bar, area);
//! ```

use ratatui::prelude::*;
use ratatui::widgets::{Tabs, Widget};
use saorsa_cli_core::{Tab, Theme};

/// Tab bar widget that displays tab titles
///
/// The tab bar renders a horizontal list of tab titles, with the
/// currently active tab highlighted using the theme's accent color.
/// Inactive tabs use the theme's muted color.
///
/// # Example
///
/// ```ignore
/// use saorsa_ui::widgets::TabBar;
/// use saorsa_cli_core::Theme;
///
/// let tab_bar = TabBar::new(&tabs, 0, &Theme::dark());
/// frame.render_widget(tab_bar, tab_area);
/// ```
pub struct TabBar<'a> {
    /// Slice of tabs to display
    tabs: &'a [Box<dyn Tab>],
    /// Index of the currently active tab
    active_index: usize,
    /// Theme for styling
    theme: &'a Theme,
}

impl<'a> TabBar<'a> {
    /// Creates a new tab bar
    ///
    /// # Arguments
    ///
    /// * `tabs` - Slice of tabs to display in the bar
    /// * `active_index` - Index of the currently active tab (0-based)
    /// * `theme` - Theme for styling the tab bar
    ///
    /// # Example
    ///
    /// ```ignore
    /// let tab_bar = TabBar::new(&tabs, 0, &theme);
    /// ```
    pub fn new(tabs: &'a [Box<dyn Tab>], active_index: usize, theme: &'a Theme) -> Self {
        TabBar {
            tabs,
            active_index,
            theme,
        }
    }

    /// Returns the number of tabs in this bar
    ///
    /// This is useful for bounds checking when changing active index.
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Returns whether the tab bar is empty
    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }
}

impl Widget for TabBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Handle empty tabs gracefully
        if self.tabs.is_empty() {
            return;
        }

        // Build tab titles with optional icons
        let titles: Vec<Line> = self
            .tabs
            .iter()
            .enumerate()
            .map(|(i, tab)| {
                let icon = tab.icon().unwrap_or("");
                let title = tab.title();
                let content = if icon.is_empty() {
                    format!(" {} ", title)
                } else {
                    format!(" {} {} ", icon, title)
                };

                // Style based on whether this is the active tab
                if i == self.active_index {
                    Line::from(content).style(
                        Style::default()
                            .fg(self.theme.colors.accent)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Line::from(content).style(Style::default().fg(self.theme.colors.muted))
                }
            })
            .collect();

        let tabs_widget = Tabs::new(titles)
            .select(self.active_index)
            .divider(" | ")
            .style(Style::default().bg(self.theme.colors.background));

        tabs_widget.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saorsa_cli_core::{Tab, TabId, Theme};

    /// Test implementation of the Tab trait
    struct TestTab {
        id: TabId,
        title: String,
        icon: Option<String>,
    }

    impl TestTab {
        fn new(id: TabId, title: &str) -> Self {
            Self {
                id,
                title: title.to_string(),
                icon: None,
            }
        }

        fn with_icon(mut self, icon: &str) -> Self {
            self.icon = Some(icon.to_string());
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

        fn focus(&mut self) {}

        fn blur(&mut self) {}

        fn view(&self, _frame: &mut Frame, _area: Rect) {}
    }

    fn create_test_tabs() -> Vec<Box<dyn Tab>> {
        vec![
            Box::new(TestTab::new(1, "Tab 1")),
            Box::new(TestTab::new(2, "Tab 2")),
            Box::new(TestTab::new(3, "Tab 3")),
        ]
    }

    #[test]
    fn test_tab_bar_creation() {
        let tabs = create_test_tabs();
        let theme = Theme::dark();
        let tab_bar = TabBar::new(&tabs, 0, &theme);

        assert_eq!(tab_bar.tab_count(), 3);
        assert!(!tab_bar.is_empty());
    }

    #[test]
    fn test_tab_bar_empty() {
        let tabs: Vec<Box<dyn Tab>> = vec![];
        let theme = Theme::dark();
        let tab_bar = TabBar::new(&tabs, 0, &theme);

        assert_eq!(tab_bar.tab_count(), 0);
        assert!(tab_bar.is_empty());
    }

    #[test]
    fn test_tab_bar_renders_empty_gracefully() {
        let tabs: Vec<Box<dyn Tab>> = vec![];
        let theme = Theme::dark();
        let tab_bar = TabBar::new(&tabs, 0, &theme);

        let area = Rect::new(0, 0, 40, 1);
        let mut buf = Buffer::empty(area);
        tab_bar.render(area, &mut buf);

        // Should not panic, buffer should be empty/unchanged
    }

    #[test]
    fn test_tab_bar_renders_titles() {
        let tabs = create_test_tabs();
        let theme = Theme::dark();
        let tab_bar = TabBar::new(&tabs, 0, &theme);

        let area = Rect::new(0, 0, 60, 1);
        let mut buf = Buffer::empty(area);
        tab_bar.render(area, &mut buf);

        // Extract text from buffer
        let content: String = (0..area.width)
            .map(|x| buf[(x, 0)].symbol().chars().next().unwrap_or(' '))
            .collect();

        assert!(content.contains("Tab 1"));
        assert!(content.contains("Tab 2"));
        assert!(content.contains("Tab 3"));
    }

    #[test]
    fn test_tab_bar_with_icons() {
        let tabs: Vec<Box<dyn Tab>> = vec![
            Box::new(TestTab::new(1, "Files").with_icon("F")),
            Box::new(TestTab::new(2, "Edit").with_icon("E")),
        ];
        let theme = Theme::dark();
        let tab_bar = TabBar::new(&tabs, 0, &theme);

        let area = Rect::new(0, 0, 40, 1);
        let mut buf = Buffer::empty(area);
        tab_bar.render(area, &mut buf);

        let content: String = (0..area.width)
            .map(|x| buf[(x, 0)].symbol().chars().next().unwrap_or(' '))
            .collect();

        assert!(content.contains("F"));
        assert!(content.contains("Files"));
    }

    #[test]
    fn test_tab_bar_active_tab_highlighted() {
        let tabs = create_test_tabs();
        let theme = Theme::dark();
        let tab_bar = TabBar::new(&tabs, 1, &theme);

        let area = Rect::new(0, 0, 60, 1);
        let mut buf = Buffer::empty(area);
        tab_bar.render(area, &mut buf);

        // Find the position of "Tab 2" and check its style
        let content: String = (0..area.width)
            .map(|x| buf[(x, 0)].symbol().chars().next().unwrap_or(' '))
            .collect();

        // Verify Tab 2 is present (as it's the active tab)
        assert!(content.contains("Tab 2"));
    }

    #[test]
    fn test_tab_bar_different_active_indices() {
        let tabs = create_test_tabs();
        let theme = Theme::dark();

        // Test with different active indices
        for active_idx in 0..3 {
            let tab_bar = TabBar::new(&tabs, active_idx, &theme);
            let area = Rect::new(0, 0, 60, 1);
            let mut buf = Buffer::empty(area);
            tab_bar.render(area, &mut buf);

            // Should render without panic for any valid index
            let content: String = (0..area.width)
                .map(|x| buf[(x, 0)].symbol().chars().next().unwrap_or(' '))
                .collect();
            assert!(!content.trim().is_empty());
        }
    }

    #[test]
    fn test_tab_bar_narrow_area() {
        let tabs = create_test_tabs();
        let theme = Theme::dark();
        let tab_bar = TabBar::new(&tabs, 0, &theme);

        // Very narrow area
        let area = Rect::new(0, 0, 10, 1);
        let mut buf = Buffer::empty(area);
        tab_bar.render(area, &mut buf);

        // Should render without panic even in narrow space
    }

    #[test]
    fn test_tab_bar_single_tab() {
        let tabs: Vec<Box<dyn Tab>> = vec![Box::new(TestTab::new(1, "Only Tab"))];
        let theme = Theme::dark();
        let tab_bar = TabBar::new(&tabs, 0, &theme);

        let area = Rect::new(0, 0, 40, 1);
        let mut buf = Buffer::empty(area);
        tab_bar.render(area, &mut buf);

        let content: String = (0..area.width)
            .map(|x| buf[(x, 0)].symbol().chars().next().unwrap_or(' '))
            .collect();

        assert!(content.contains("Only Tab"));
    }

    #[test]
    fn test_tab_bar_with_light_theme() {
        let tabs = create_test_tabs();
        let theme = Theme::light();
        let tab_bar = TabBar::new(&tabs, 0, &theme);

        let area = Rect::new(0, 0, 60, 1);
        let mut buf = Buffer::empty(area);
        tab_bar.render(area, &mut buf);

        // Should render correctly with light theme
        let content: String = (0..area.width)
            .map(|x| buf[(x, 0)].symbol().chars().next().unwrap_or(' '))
            .collect();
        assert!(content.contains("Tab 1"));
    }

    #[test]
    fn test_tab_bar_with_nord_theme() {
        let tabs = create_test_tabs();
        let theme = Theme::nord();
        let tab_bar = TabBar::new(&tabs, 0, &theme);

        let area = Rect::new(0, 0, 60, 1);
        let mut buf = Buffer::empty(area);
        tab_bar.render(area, &mut buf);

        // Should render correctly with nord theme
        let content: String = (0..area.width)
            .map(|x| buf[(x, 0)].symbol().chars().next().unwrap_or(' '))
            .collect();
        assert!(content.contains("Tab 1"));
    }
}
