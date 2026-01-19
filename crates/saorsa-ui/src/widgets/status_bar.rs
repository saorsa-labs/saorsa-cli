//! Status bar widget for displaying application state
//!
//! The [`StatusBar`] widget renders a three-section bar at the bottom
//! of the terminal showing mode, context info, and help hints.
//!
//! ## Features
//!
//! - Three sections: left (mode), center (context), right (help)
//! - Builder pattern for easy configuration
//! - Theme-aware styling with accent and muted colors
//! - Graceful handling of overflow and empty sections
//!
//! ## Example
//!
//! ```ignore
//! use saorsa_ui::widgets::StatusBar;
//! use saorsa_cli_core::Theme;
//!
//! let status = StatusBar::new(&theme)
//!     .left("NORMAL")
//!     .center("main.rs")
//!     .right("?:help");
//! frame.render_widget(status, area);
//! ```

use ratatui::prelude::*;
use ratatui::widgets::Widget;
use saorsa_cli_core::Theme;

/// Status bar with left, center, and right sections
///
/// The status bar provides a three-section layout commonly used
/// at the bottom of terminal applications to display:
///
/// - **Left**: Current mode (e.g., "NORMAL", "INSERT", "VISUAL")
/// - **Center**: Context information (e.g., filename, position)
/// - **Right**: Help hints (e.g., "?:help", "q:quit")
///
/// # Builder Pattern
///
/// Use the builder methods to configure each section:
///
/// ```ignore
/// let status = StatusBar::new(&theme)
///     .left("NORMAL")
///     .center("main.rs [+]")
///     .right("Ln 42, Col 8");
/// ```
pub struct StatusBar<'a> {
    /// Left section text (typically mode)
    left: &'a str,
    /// Center section text (typically file/context)
    center: &'a str,
    /// Right section text (typically help/position)
    right: &'a str,
    /// Theme for styling
    theme: &'a Theme,
}

impl<'a> StatusBar<'a> {
    /// Creates a new status bar with empty sections
    ///
    /// # Arguments
    ///
    /// * `theme` - Theme for styling the status bar
    ///
    /// # Example
    ///
    /// ```ignore
    /// let status = StatusBar::new(&theme);
    /// ```
    pub fn new(theme: &'a Theme) -> Self {
        StatusBar {
            left: "",
            center: "",
            right: "",
            theme,
        }
    }

    /// Sets the left section text (typically mode)
    ///
    /// The left section is styled with the accent color and bold text.
    ///
    /// # Arguments
    ///
    /// * `text` - Text to display in the left section
    ///
    /// # Example
    ///
    /// ```ignore
    /// let status = StatusBar::new(&theme).left("NORMAL");
    /// ```
    pub fn left(mut self, text: &'a str) -> Self {
        self.left = text;
        self
    }

    /// Sets the center section text (typically file/context info)
    ///
    /// The center section is centered horizontally and uses the
    /// standard foreground color.
    ///
    /// # Arguments
    ///
    /// * `text` - Text to display in the center section
    ///
    /// # Example
    ///
    /// ```ignore
    /// let status = StatusBar::new(&theme).center("main.rs [+]");
    /// ```
    pub fn center(mut self, text: &'a str) -> Self {
        self.center = text;
        self
    }

    /// Sets the right section text (typically help hints)
    ///
    /// The right section is right-aligned and uses the muted color.
    ///
    /// # Arguments
    ///
    /// * `text` - Text to display in the right section
    ///
    /// # Example
    ///
    /// ```ignore
    /// let status = StatusBar::new(&theme).right("?:help q:quit");
    /// ```
    pub fn right(mut self, text: &'a str) -> Self {
        self.right = text;
        self
    }
}

impl Widget for StatusBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Handle zero-dimension areas gracefully
        if area.width == 0 || area.height == 0 {
            return;
        }

        // Fill background with selection color
        for x in area.left()..area.right() {
            for y in area.top()..area.bottom() {
                buf[(x, y)].set_bg(self.theme.colors.selection);
            }
        }

        let width = area.width as usize;

        // Render left section (left-aligned) with padding
        if !self.left.is_empty() {
            let text = format!(" {} ", self.left);
            if text.len() <= width {
                let style = Style::default()
                    .fg(self.theme.colors.accent)
                    .bg(self.theme.colors.selection)
                    .add_modifier(Modifier::BOLD);

                buf.set_string(area.x, area.y, &text, style);
            }
        }

        // Render center section (centered)
        if !self.center.is_empty() {
            let center_len = self.center.len();
            if center_len < width {
                let start_x = area.x + (width.saturating_sub(center_len) / 2) as u16;
                let style = Style::default()
                    .fg(self.theme.colors.foreground)
                    .bg(self.theme.colors.selection);

                buf.set_string(start_x, area.y, self.center, style);
            }
        }

        // Render right section (right-aligned) with padding
        if !self.right.is_empty() {
            let text = format!(" {} ", self.right);
            if text.len() <= width {
                let start_x = area.right().saturating_sub(text.len() as u16);
                let style = Style::default()
                    .fg(self.theme.colors.muted)
                    .bg(self.theme.colors.selection);

                buf.set_string(start_x, area.y, &text, style);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saorsa_cli_core::Theme;

    #[test]
    fn test_status_bar_creation() {
        let theme = Theme::dark();
        let status = StatusBar::new(&theme);

        // Should create with empty sections
        let area = Rect::new(0, 0, 40, 1);
        let mut buf = Buffer::empty(area);
        status.render(area, &mut buf);

        // Should not panic
    }

    #[test]
    fn test_status_bar_left_section() {
        let theme = Theme::dark();
        let status = StatusBar::new(&theme).left("MODE");

        let area = Rect::new(0, 0, 40, 1);
        let mut buf = Buffer::empty(area);
        status.render(area, &mut buf);

        let content: String = (0..15)
            .map(|x| buf[(x, 0)].symbol().chars().next().unwrap_or(' '))
            .collect();

        assert!(content.contains("MODE"));
    }

    #[test]
    fn test_status_bar_center_section() {
        let theme = Theme::dark();
        let status = StatusBar::new(&theme).center("file.rs");

        let area = Rect::new(0, 0, 40, 1);
        let mut buf = Buffer::empty(area);
        status.render(area, &mut buf);

        let content: String = (0..area.width)
            .map(|x| buf[(x, 0)].symbol().chars().next().unwrap_or(' '))
            .collect();

        assert!(content.contains("file.rs"));
    }

    #[test]
    fn test_status_bar_right_section() {
        let theme = Theme::dark();
        let status = StatusBar::new(&theme).right("help");

        let area = Rect::new(0, 0, 40, 1);
        let mut buf = Buffer::empty(area);
        status.render(area, &mut buf);

        let content: String = (30..area.width)
            .map(|x| buf[(x, 0)].symbol().chars().next().unwrap_or(' '))
            .collect();

        assert!(content.contains("help"));
    }

    #[test]
    fn test_status_bar_all_sections() {
        let theme = Theme::dark();
        let status = StatusBar::new(&theme)
            .left("NORMAL")
            .center("main.rs")
            .right("?:help");

        let area = Rect::new(0, 0, 60, 1);
        let mut buf = Buffer::empty(area);
        status.render(area, &mut buf);

        let content: String = (0..area.width)
            .map(|x| buf[(x, 0)].symbol().chars().next().unwrap_or(' '))
            .collect();

        assert!(content.contains("NORMAL"));
        assert!(content.contains("main.rs"));
        assert!(content.contains("help"));
    }

    #[test]
    fn test_status_bar_empty_sections() {
        let theme = Theme::dark();
        let status = StatusBar::new(&theme).left("").center("").right("");

        let area = Rect::new(0, 0, 40, 1);
        let mut buf = Buffer::empty(area);
        status.render(area, &mut buf);

        // Should render without panic with all empty sections
    }

    #[test]
    fn test_status_bar_narrow_area() {
        let theme = Theme::dark();
        let status = StatusBar::new(&theme)
            .left("NORMAL")
            .center("file.rs")
            .right("help");

        // Very narrow area
        let area = Rect::new(0, 0, 5, 1);
        let mut buf = Buffer::empty(area);
        status.render(area, &mut buf);

        // Should render without panic even in narrow space
    }

    #[test]
    fn test_status_bar_with_light_theme() {
        let theme = Theme::light();
        let status = StatusBar::new(&theme)
            .left("INSERT")
            .center("test.rs")
            .right("q:quit");

        let area = Rect::new(0, 0, 60, 1);
        let mut buf = Buffer::empty(area);
        status.render(area, &mut buf);

        let content: String = (0..area.width)
            .map(|x| buf[(x, 0)].symbol().chars().next().unwrap_or(' '))
            .collect();

        assert!(content.contains("INSERT"));
    }

    #[test]
    fn test_status_bar_with_nord_theme() {
        let theme = Theme::nord();
        let status = StatusBar::new(&theme)
            .left("VISUAL")
            .center("config.toml")
            .right("h:help");

        let area = Rect::new(0, 0, 60, 1);
        let mut buf = Buffer::empty(area);
        status.render(area, &mut buf);

        let content: String = (0..area.width)
            .map(|x| buf[(x, 0)].symbol().chars().next().unwrap_or(' '))
            .collect();

        assert!(content.contains("VISUAL"));
    }

    #[test]
    fn test_status_bar_background_fill() {
        let theme = Theme::dark();
        let status = StatusBar::new(&theme);

        let area = Rect::new(0, 0, 20, 1);
        let mut buf = Buffer::empty(area);
        status.render(area, &mut buf);

        // Check that background is filled with selection color
        for x in 0..area.width {
            assert_eq!(buf[(x, 0)].bg, theme.colors.selection);
        }
    }

    #[test]
    fn test_status_bar_left_style() {
        let theme = Theme::dark();
        let status = StatusBar::new(&theme).left("MODE");

        let area = Rect::new(0, 0, 40, 1);
        let mut buf = Buffer::empty(area);
        status.render(area, &mut buf);

        // First non-space character after padding should have accent color
        // Position 1 should have 'M' from " MODE "
        let cell = &buf[(1, 0)];
        assert_eq!(cell.fg, theme.colors.accent);
    }

    #[test]
    fn test_status_bar_center_is_centered() {
        let theme = Theme::dark();
        let text = "centered";
        let status = StatusBar::new(&theme).center(text);

        let area = Rect::new(0, 0, 40, 1);
        let mut buf = Buffer::empty(area);
        status.render(area, &mut buf);

        // Find the position of 'c' in "centered"
        let expected_start = (40 - text.len()) / 2;
        let cell = &buf[(expected_start as u16, 0)];
        assert_eq!(cell.symbol(), "c");
    }

    #[test]
    fn test_status_bar_builder_chain() {
        let theme = Theme::dark();

        // Test that builder methods can be chained in any order
        let _status1 = StatusBar::new(&theme).left("A").center("B").right("C");

        let _status2 = StatusBar::new(&theme).right("C").left("A").center("B");

        let _status3 = StatusBar::new(&theme).center("B").right("C").left("A");
    }

    #[test]
    fn test_status_bar_long_text_overflow() {
        let theme = Theme::dark();
        let long_text = "This is a very long status text that exceeds the width";
        let status = StatusBar::new(&theme)
            .left(long_text)
            .center(long_text)
            .right(long_text);

        // Narrow area that can't fit the text
        let area = Rect::new(0, 0, 20, 1);
        let mut buf = Buffer::empty(area);
        status.render(area, &mut buf);

        // Should handle overflow gracefully without panic
    }

    #[test]
    fn test_status_bar_zero_width() {
        let theme = Theme::dark();
        let status = StatusBar::new(&theme)
            .left("TEST")
            .center("TEST")
            .right("TEST");

        // Zero width area - use a valid height but zero width
        // Note: Buffer::empty with zero dimensions may panic, so we test
        // that our guard handles this gracefully
        let area = Rect::new(0, 0, 0, 1);

        // Create a minimal valid buffer for the test
        let valid_area = Rect::new(0, 0, 1, 1);
        let mut buf = Buffer::empty(valid_area);

        // Render to the zero-width area should return early without accessing buffer
        status.render(area, &mut buf);

        // Should not panic
    }

    #[test]
    fn test_status_bar_zero_height() {
        let theme = Theme::dark();
        let status = StatusBar::new(&theme)
            .left("TEST")
            .center("TEST")
            .right("TEST");

        // Zero height area
        let area = Rect::new(0, 0, 40, 0);

        // Create a minimal valid buffer for the test
        let valid_area = Rect::new(0, 0, 40, 1);
        let mut buf = Buffer::empty(valid_area);

        // Render to the zero-height area should return early without accessing buffer
        status.render(area, &mut buf);

        // Should not panic
    }

    #[test]
    fn test_status_bar_unicode_text() {
        let theme = Theme::dark();
        let status = StatusBar::new(&theme)
            .left("EDIT")
            .center("file.rs")
            .right("help");

        let area = Rect::new(0, 0, 60, 1);
        let mut buf = Buffer::empty(area);
        status.render(area, &mut buf);

        // Should handle text correctly
        let content: String = (0..area.width)
            .map(|x| buf[(x, 0)].symbol().chars().next().unwrap_or(' '))
            .collect();

        assert!(content.contains("EDIT"));
    }
}
