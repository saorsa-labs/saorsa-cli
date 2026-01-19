//! Diff viewer widget with syntax highlighting

use crate::repo::Diff;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Widget},
};

/// Widget for displaying a diff
pub struct DiffWidget<'a> {
    diff: &'a Diff,
    scroll_offset: u16,
    focused: bool,
    show_line_numbers: bool,
}

impl<'a> DiffWidget<'a> {
    /// Create a new diff widget
    pub fn new(diff: &'a Diff) -> Self {
        DiffWidget {
            diff,
            scroll_offset: 0,
            focused: false,
            show_line_numbers: true,
        }
    }

    /// Set the scroll offset
    pub fn scroll(mut self, offset: u16) -> Self {
        self.scroll_offset = offset;
        self
    }

    /// Set whether the widget is focused
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Set whether to show line numbers
    pub fn show_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    /// Get the total number of lines in the diff
    pub fn total_lines(&self) -> usize {
        self.diff
            .hunks
            .iter()
            .map(|h| 1 + h.lines.len()) // +1 for header
            .sum()
    }
}

impl Widget for DiffWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_style = if self.focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let title = if self.diff.path.as_os_str().is_empty() {
            " Diff ".to_string()
        } else {
            format!(" {} ", self.diff.path.display())
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner = block.inner(area);
        block.render(area, buf);

        if self.diff.hunks.is_empty() {
            let msg = "No changes to display";
            let x = inner.x + (inner.width.saturating_sub(msg.len() as u16)) / 2;
            let y = inner.y + inner.height / 2;
            buf.set_string(x, y, msg, Style::default().fg(Color::DarkGray));
            return;
        }

        // Calculate line number width
        let max_lineno = self
            .diff
            .hunks
            .iter()
            .flat_map(|h| h.lines.iter())
            .filter_map(|l| l.new_lineno.or(l.old_lineno))
            .max()
            .unwrap_or(1);
        let lineno_width = if self.show_line_numbers {
            format!("{}", max_lineno).len() as u16 + 2 // +2 for spacing
        } else {
            0
        };

        let content_width = inner.width.saturating_sub(lineno_width + 1); // +1 for origin char

        let mut y = inner.y;
        let mut line_idx: u16 = 0;

        'outer: for hunk in &self.diff.hunks {
            // Skip lines before scroll offset
            if line_idx < self.scroll_offset {
                line_idx += 1;
                // Check if we need to skip hunk header
                if line_idx > self.scroll_offset {
                    // Render hunk header
                    let header_style = Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD);
                    let header = &hunk.header;
                    let x = inner.x + lineno_width;
                    buf.set_string(x, y, header, header_style);
                    y += 1;
                    if y >= inner.y + inner.height {
                        break;
                    }
                }
            } else {
                // Render hunk header
                let header_style = Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD);
                let header = &hunk.header;
                let x = inner.x + lineno_width;
                buf.set_string(x, y, header, header_style);
                y += 1;
                line_idx += 1;
                if y >= inner.y + inner.height {
                    break;
                }
            }

            // Render lines
            for line in &hunk.lines {
                if line_idx < self.scroll_offset {
                    line_idx += 1;
                    continue;
                }

                if y >= inner.y + inner.height {
                    break 'outer;
                }

                // Line number
                if self.show_line_numbers {
                    let lineno = match line.origin {
                        '+' => line.new_lineno,
                        '-' => line.old_lineno,
                        _ => line.new_lineno.or(line.old_lineno),
                    };
                    if let Some(n) = lineno {
                        let lineno_str =
                            format!("{:>width$} ", n, width = (lineno_width - 1) as usize);
                        buf.set_string(
                            inner.x,
                            y,
                            &lineno_str,
                            Style::default().fg(Color::DarkGray),
                        );
                    }
                }

                // Origin character and content
                let (fg_color, bg_color) = match line.origin {
                    '+' => (Color::Green, Some(Color::Rgb(0, 40, 0))),
                    '-' => (Color::Red, Some(Color::Rgb(40, 0, 0))),
                    _ => (Color::White, None),
                };

                let style = if let Some(bg) = bg_color {
                    Style::default().fg(fg_color).bg(bg)
                } else {
                    Style::default().fg(fg_color)
                };

                let x = inner.x + lineno_width;
                buf.set_string(x, y, line.origin.to_string(), style);

                // Content (truncated to fit)
                let content = if line.content.len() as u16 > content_width {
                    &line.content[..content_width as usize]
                } else {
                    &line.content
                };
                // Remove trailing newline for display
                let content = content.trim_end_matches('\n');
                buf.set_string(x + 1, y, content, style);

                y += 1;
                line_idx += 1;
            }
        }
    }
}

/// State for the diff widget
#[derive(Debug, Default)]
pub struct DiffWidgetState {
    /// Current scroll offset
    pub scroll: u16,
    /// Total lines in the diff
    pub total_lines: usize,
}

impl DiffWidgetState {
    /// Create a new state
    pub fn new() -> Self {
        DiffWidgetState::default()
    }

    /// Scroll down by the given amount
    pub fn scroll_down(&mut self, amount: u16, visible_height: u16) {
        let max_scroll = self.total_lines.saturating_sub(visible_height as usize) as u16;
        self.scroll = (self.scroll + amount).min(max_scroll);
    }

    /// Scroll up by the given amount
    pub fn scroll_up(&mut self, amount: u16) {
        self.scroll = self.scroll.saturating_sub(amount);
    }

    /// Scroll to top
    pub fn scroll_to_top(&mut self) {
        self.scroll = 0;
    }

    /// Scroll to bottom
    pub fn scroll_to_bottom(&mut self, visible_height: u16) {
        self.scroll = self.total_lines.saturating_sub(visible_height as usize) as u16;
    }

    /// Page down
    pub fn page_down(&mut self, visible_height: u16) {
        self.scroll_down(visible_height.saturating_sub(2), visible_height);
    }

    /// Page up
    pub fn page_up(&mut self, visible_height: u16) {
        self.scroll_up(visible_height.saturating_sub(2));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::{DiffHunk, DiffLine};
    use std::path::PathBuf;

    fn sample_diff() -> Diff {
        Diff {
            path: PathBuf::from("test.rs"),
            hunks: vec![DiffHunk {
                header: "@@ -1,3 +1,4 @@".to_string(),
                lines: vec![
                    DiffLine {
                        origin: ' ',
                        content: "fn main() {".to_string(),
                        old_lineno: Some(1),
                        new_lineno: Some(1),
                    },
                    DiffLine {
                        origin: '+',
                        content: "    println!(\"hello\");".to_string(),
                        old_lineno: None,
                        new_lineno: Some(2),
                    },
                    DiffLine {
                        origin: ' ',
                        content: "}".to_string(),
                        old_lineno: Some(2),
                        new_lineno: Some(3),
                    },
                ],
            }],
        }
    }

    #[test]
    fn test_diff_widget_total_lines() {
        let diff = sample_diff();
        let widget = DiffWidget::new(&diff);
        assert_eq!(widget.total_lines(), 4); // 1 header + 3 lines
    }

    #[test]
    fn test_diff_state_scroll_down() {
        let mut state = DiffWidgetState::new();
        state.total_lines = 100;
        state.scroll_down(10, 20);
        assert_eq!(state.scroll, 10);
    }

    #[test]
    fn test_diff_state_scroll_up() {
        let mut state = DiffWidgetState::new();
        state.scroll = 10;
        state.scroll_up(5);
        assert_eq!(state.scroll, 5);
    }

    #[test]
    fn test_diff_state_scroll_up_clamp() {
        let mut state = DiffWidgetState::new();
        state.scroll = 3;
        state.scroll_up(10);
        assert_eq!(state.scroll, 0);
    }

    #[test]
    fn test_diff_state_scroll_to_top() {
        let mut state = DiffWidgetState::new();
        state.scroll = 50;
        state.scroll_to_top();
        assert_eq!(state.scroll, 0);
    }

    #[test]
    fn test_diff_state_scroll_to_bottom() {
        let mut state = DiffWidgetState::new();
        state.total_lines = 100;
        state.scroll_to_bottom(20);
        assert_eq!(state.scroll, 80);
    }

    #[test]
    fn test_diff_widget_empty() {
        let diff = Diff::default();
        let widget = DiffWidget::new(&diff);
        assert_eq!(widget.total_lines(), 0);
    }
}
