//! Git status widget showing staged, unstaged, and untracked files

use crate::repo::{FileStatus, StatusEntry};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, StatefulWidget},
};

/// Section in the status view
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Section {
    /// Staged changes (ready for commit)
    #[default]
    Staged,
    /// Changes not staged for commit
    Unstaged,
    /// Untracked files
    Untracked,
}

impl Section {
    /// Get the display title for this section
    pub fn title(&self) -> &'static str {
        match self {
            Section::Staged => "Staged Changes",
            Section::Unstaged => "Changes not staged",
            Section::Untracked => "Untracked files",
        }
    }

    /// Get the color for this section
    pub fn color(&self) -> Color {
        match self {
            Section::Staged => Color::Green,
            Section::Unstaged => Color::Yellow,
            Section::Untracked => Color::Gray,
        }
    }

    /// Cycle to next section
    pub fn next(&self) -> Section {
        match self {
            Section::Staged => Section::Unstaged,
            Section::Unstaged => Section::Untracked,
            Section::Untracked => Section::Staged,
        }
    }

    /// Cycle to previous section
    pub fn prev(&self) -> Section {
        match self {
            Section::Staged => Section::Untracked,
            Section::Unstaged => Section::Staged,
            Section::Untracked => Section::Unstaged,
        }
    }
}

/// Widget for displaying git status
pub struct StatusWidget<'a> {
    staged: &'a [StatusEntry],
    unstaged: &'a [StatusEntry],
    untracked: &'a [StatusEntry],
    focused: bool,
    branch: &'a str,
}

impl<'a> StatusWidget<'a> {
    /// Create a new status widget
    pub fn new(
        staged: &'a [StatusEntry],
        unstaged: &'a [StatusEntry],
        untracked: &'a [StatusEntry],
        branch: &'a str,
    ) -> Self {
        StatusWidget {
            staged,
            unstaged,
            untracked,
            focused: false,
            branch,
        }
    }

    /// Set whether the widget is focused
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

/// State for the status widget
#[derive(Debug, Default, Clone)]
pub struct StatusWidgetState {
    /// Currently selected section
    pub section: Section,
    /// Selected index within the current section
    pub selected: usize,
}

impl StatusWidgetState {
    /// Create a new state
    pub fn new() -> Self {
        StatusWidgetState::default()
    }

    /// Move selection down
    pub fn move_down(&mut self, staged_len: usize, unstaged_len: usize, untracked_len: usize) {
        let current_len = match self.section {
            Section::Staged => staged_len,
            Section::Unstaged => unstaged_len,
            Section::Untracked => untracked_len,
        };

        if current_len == 0 {
            self.next_section_with_items(staged_len, unstaged_len, untracked_len);
            return;
        }

        if self.selected + 1 < current_len {
            self.selected += 1;
        } else {
            // Move to next section
            self.next_section_with_items(staged_len, unstaged_len, untracked_len);
        }
    }

    /// Move selection up
    pub fn move_up(&mut self, staged_len: usize, unstaged_len: usize, untracked_len: usize) {
        if self.selected > 0 {
            self.selected -= 1;
        } else {
            // Move to previous section
            self.prev_section_with_items(staged_len, unstaged_len, untracked_len);
        }
    }

    fn next_section_with_items(
        &mut self,
        staged_len: usize,
        unstaged_len: usize,
        untracked_len: usize,
    ) {
        let start = self.section;
        loop {
            self.section = self.section.next();
            self.selected = 0;

            let len = match self.section {
                Section::Staged => staged_len,
                Section::Unstaged => unstaged_len,
                Section::Untracked => untracked_len,
            };

            if len > 0 || self.section == start {
                break;
            }
        }
    }

    fn prev_section_with_items(
        &mut self,
        staged_len: usize,
        unstaged_len: usize,
        untracked_len: usize,
    ) {
        let start = self.section;
        loop {
            self.section = self.section.prev();

            let len = match self.section {
                Section::Staged => staged_len,
                Section::Unstaged => unstaged_len,
                Section::Untracked => untracked_len,
            };

            if len > 0 {
                self.selected = len.saturating_sub(1);
                break;
            }

            if self.section == start {
                self.selected = 0;
                break;
            }
        }
    }

    /// Ensure selection is valid
    pub fn clamp(&mut self, staged_len: usize, unstaged_len: usize, untracked_len: usize) {
        let len = match self.section {
            Section::Staged => staged_len,
            Section::Unstaged => unstaged_len,
            Section::Untracked => untracked_len,
        };

        if len == 0 {
            self.next_section_with_items(staged_len, unstaged_len, untracked_len);
        } else {
            self.selected = self.selected.min(len.saturating_sub(1));
        }
    }
}

impl StatefulWidget for StatusWidget<'_> {
    type State = StatusWidgetState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let border_style = if self.focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let block = Block::default()
            .title(format!(" {} ", self.branch))
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner = block.inner(area);
        ratatui::widgets::Widget::render(block, area, buf);

        if inner.height < 3 {
            return;
        }

        // Calculate heights for each section
        let total_items = self.staged.len() + self.unstaged.len() + self.untracked.len();
        if total_items == 0 {
            // Show "Working tree clean" message
            let msg = "Working tree clean";
            let x = inner.x + (inner.width.saturating_sub(msg.len() as u16)) / 2;
            let y = inner.y + inner.height / 2;
            buf.set_string(x, y, msg, Style::default().fg(Color::Green));
            return;
        }

        let mut y = inner.y;

        // Render staged section
        if !self.staged.is_empty() {
            let header_style = Style::default()
                .fg(Section::Staged.color())
                .add_modifier(Modifier::BOLD);
            buf.set_string(
                inner.x,
                y,
                format!("── {} ({}) ──", Section::Staged.title(), self.staged.len()),
                header_style,
            );
            y += 1;

            for (i, entry) in self.staged.iter().enumerate() {
                if y >= inner.y + inner.height {
                    break;
                }
                let selected = state.section == Section::Staged && state.selected == i;
                render_entry(
                    buf,
                    inner.x,
                    y,
                    inner.width,
                    entry,
                    selected,
                    Section::Staged.color(),
                );
                y += 1;
            }
            y += 1; // Spacing
        }

        // Render unstaged section
        if !self.unstaged.is_empty() && y < inner.y + inner.height {
            let header_style = Style::default()
                .fg(Section::Unstaged.color())
                .add_modifier(Modifier::BOLD);
            buf.set_string(
                inner.x,
                y,
                format!(
                    "── {} ({}) ──",
                    Section::Unstaged.title(),
                    self.unstaged.len()
                ),
                header_style,
            );
            y += 1;

            for (i, entry) in self.unstaged.iter().enumerate() {
                if y >= inner.y + inner.height {
                    break;
                }
                let selected = state.section == Section::Unstaged && state.selected == i;
                render_entry(
                    buf,
                    inner.x,
                    y,
                    inner.width,
                    entry,
                    selected,
                    Section::Unstaged.color(),
                );
                y += 1;
            }
            y += 1; // Spacing
        }

        // Render untracked section
        if !self.untracked.is_empty() && y < inner.y + inner.height {
            let header_style = Style::default()
                .fg(Section::Untracked.color())
                .add_modifier(Modifier::BOLD);
            buf.set_string(
                inner.x,
                y,
                format!(
                    "── {} ({}) ──",
                    Section::Untracked.title(),
                    self.untracked.len()
                ),
                header_style,
            );
            y += 1;

            for (i, entry) in self.untracked.iter().enumerate() {
                if y >= inner.y + inner.height {
                    break;
                }
                let selected = state.section == Section::Untracked && state.selected == i;
                render_entry(
                    buf,
                    inner.x,
                    y,
                    inner.width,
                    entry,
                    selected,
                    Section::Untracked.color(),
                );
                y += 1;
            }
        }
    }
}

fn render_entry(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    width: u16,
    entry: &StatusEntry,
    selected: bool,
    section_color: Color,
) {
    let status_char = entry.status.indicator();
    let color = match entry.status {
        FileStatus::Added => Color::Green,
        FileStatus::Modified => Color::Yellow,
        FileStatus::Deleted => Color::Red,
        FileStatus::Renamed => Color::Cyan,
        FileStatus::Untracked => Color::Gray,
        FileStatus::Conflicted => Color::Magenta,
        _ => Color::White,
    };

    let style = if selected {
        Style::default()
            .fg(Color::Black)
            .bg(section_color)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(color)
    };

    let path_str = entry.path.display().to_string();
    let text = format!(" {} {}", status_char, path_str);
    let text = if text.len() as u16 > width {
        format!("{}...", &text[..width.saturating_sub(3) as usize])
    } else {
        text
    };

    buf.set_string(x, y, &text, style);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_section_cycle() {
        let section = Section::Staged;
        assert_eq!(section.next(), Section::Unstaged);
        assert_eq!(section.next().next(), Section::Untracked);
        assert_eq!(section.next().next().next(), Section::Staged);
    }

    #[test]
    fn test_section_prev_cycle() {
        let section = Section::Staged;
        assert_eq!(section.prev(), Section::Untracked);
        assert_eq!(section.prev().prev(), Section::Unstaged);
        assert_eq!(section.prev().prev().prev(), Section::Staged);
    }

    #[test]
    fn test_status_widget_state_new() {
        let state = StatusWidgetState::new();
        assert_eq!(state.section, Section::Staged);
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn test_move_down_within_section() {
        let mut state = StatusWidgetState::new();
        state.move_down(3, 2, 1); // 3 staged items
        assert_eq!(state.section, Section::Staged);
        assert_eq!(state.selected, 1);
    }

    #[test]
    fn test_move_down_to_next_section() {
        let mut state = StatusWidgetState::new();
        state.selected = 2; // Last item in staged
        state.move_down(3, 2, 1);
        assert_eq!(state.section, Section::Unstaged);
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn test_move_up_to_prev_section() {
        let mut state = StatusWidgetState::new();
        state.section = Section::Unstaged;
        state.selected = 0;
        state.move_up(3, 2, 1);
        assert_eq!(state.section, Section::Staged);
        assert_eq!(state.selected, 2); // Last item in staged
    }

    #[test]
    fn test_clamp_selection() {
        let mut state = StatusWidgetState::new();
        state.selected = 10; // Beyond any list
        state.clamp(2, 3, 1);
        assert_eq!(state.selected, 1); // Clamped to max index
    }

    #[test]
    fn test_skip_empty_sections() {
        let mut state = StatusWidgetState::new();
        state.move_down(0, 2, 1); // No staged items
        assert_eq!(state.section, Section::Unstaged);
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn test_file_status_indicator() {
        let entry = StatusEntry {
            path: PathBuf::from("test.rs"),
            status: FileStatus::Added,
            staged: true,
        };
        assert_eq!(entry.status.indicator(), 'A');
    }
}
