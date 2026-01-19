//! Git tab implementation

use crate::repo::{Diff, GitRepo, StatusEntry};
use crate::widgets::{DiffWidgetState, Section, StatusWidgetState};
use crossterm::event::{KeyCode, KeyModifiers};
use parking_lot::Mutex;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders};
use ratatui::Frame;
use saorsa_cli_core::{Message, Tab, TabId};
use std::path::Path;

/// Focus state within the Git tab
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum GitFocus {
    /// Status list is focused
    #[default]
    Status,
    /// Diff view is focused
    Diff,
}

/// Internal mutable state for the Git tab
struct GitTabState {
    repo: Option<GitRepo>,
    staged: Vec<StatusEntry>,
    unstaged: Vec<StatusEntry>,
    untracked: Vec<StatusEntry>,
    current_diff: Diff,
    branch: String,
    focus: GitFocus,
    status_state: StatusWidgetState,
    diff_state: DiffWidgetState,
    error_message: Option<String>,
    last_area_height: u16,
}

impl GitTabState {
    fn new(path: &Path) -> Self {
        let mut state = GitTabState {
            repo: None,
            staged: Vec::new(),
            unstaged: Vec::new(),
            untracked: Vec::new(),
            current_diff: Diff::default(),
            branch: String::from("(no repo)"),
            focus: GitFocus::Status,
            status_state: StatusWidgetState::new(),
            diff_state: DiffWidgetState::new(),
            error_message: None,
            last_area_height: 20,
        };

        // Try to open repository
        match GitRepo::open(path) {
            Ok(repo) => {
                state.repo = Some(repo);
                state.refresh();
            }
            Err(e) => {
                state.error_message = Some(format!("Not a git repository: {}", e));
            }
        }

        state
    }

    fn refresh(&mut self) {
        if let Some(ref repo) = self.repo {
            self.branch = repo
                .current_branch()
                .unwrap_or_else(|_| "(detached)".into());
            self.staged = repo.staged_files().unwrap_or_default();
            self.unstaged = repo.unstaged_files().unwrap_or_default();
            self.untracked = repo.untracked_files().unwrap_or_default();

            self.status_state
                .clamp(self.staged.len(), self.unstaged.len(), self.untracked.len());

            self.update_diff();
        }
    }

    fn update_diff(&mut self) {
        let entry = self.selected_entry();
        if let (Some(ref repo), Some(entry)) = (&self.repo, entry) {
            let staged = entry.staged;
            match repo.file_diff(&entry.path, staged) {
                Ok(diff) => {
                    self.current_diff = diff;
                    self.diff_state.total_lines = self
                        .current_diff
                        .hunks
                        .iter()
                        .map(|h| 1 + h.lines.len())
                        .sum();
                    self.diff_state.scroll = 0;
                }
                Err(_) => {
                    self.current_diff = Diff::default();
                }
            }
        } else {
            self.current_diff = Diff::default();
        }
    }

    fn selected_entry(&self) -> Option<StatusEntry> {
        match self.status_state.section {
            Section::Staged => self.staged.get(self.status_state.selected).cloned(),
            Section::Unstaged => self.unstaged.get(self.status_state.selected).cloned(),
            Section::Untracked => self.untracked.get(self.status_state.selected).cloned(),
        }
    }

    fn toggle_stage(&mut self) {
        if let Some(ref repo) = self.repo {
            if let Some(entry) = self.selected_entry() {
                let result = if entry.staged {
                    repo.unstage_file(&entry.path)
                } else {
                    repo.stage_file(&entry.path)
                };

                if let Err(e) = result {
                    self.error_message = Some(format!("Failed: {}", e));
                } else {
                    self.refresh();
                }
            }
        }
    }

    fn stage_all(&mut self) {
        if let Some(ref repo) = self.repo {
            if let Err(e) = repo.stage_all() {
                self.error_message = Some(format!("Failed to stage all: {}", e));
            } else {
                self.refresh();
            }
        }
    }

    fn unstage_all(&mut self) {
        if let Some(ref repo) = self.repo {
            if let Err(e) = repo.unstage_all() {
                self.error_message = Some(format!("Failed to unstage all: {}", e));
            } else {
                self.refresh();
            }
        }
    }
}

/// Git tab providing status view and diff viewer
pub struct GitTab {
    id: TabId,
    state: Mutex<GitTabState>,
}

impl GitTab {
    /// Create a new Git tab
    pub fn new(id: TabId, path: &Path) -> Self {
        GitTab {
            id,
            state: Mutex::new(GitTabState::new(path)),
        }
    }

    /// Handle a key event
    pub fn handle_key(&self, code: KeyCode, modifiers: KeyModifiers) {
        let mut state = self.state.lock();

        match state.focus {
            GitFocus::Status => {
                // Pre-compute lengths to avoid borrow issues
                let staged_len = state.staged.len();
                let unstaged_len = state.unstaged.len();
                let untracked_len = state.untracked.len();

                match (modifiers, code) {
                    // Navigation
                    (KeyModifiers::NONE, KeyCode::Char('j') | KeyCode::Down) => {
                        state
                            .status_state
                            .move_down(staged_len, unstaged_len, untracked_len);
                        state.update_diff();
                    }
                    (KeyModifiers::NONE, KeyCode::Char('k') | KeyCode::Up) => {
                        state
                            .status_state
                            .move_up(staged_len, unstaged_len, untracked_len);
                        state.update_diff();
                    }

                    // Focus switching
                    (KeyModifiers::NONE, KeyCode::Char('l')) => {
                        state.focus = GitFocus::Diff;
                    }

                    // Stage/unstage
                    (KeyModifiers::NONE, KeyCode::Enter)
                    | (KeyModifiers::NONE, KeyCode::Char(' ')) => {
                        state.toggle_stage();
                    }

                    // Stage/unstage all
                    (KeyModifiers::NONE, KeyCode::Char('s')) => {
                        state.stage_all();
                    }
                    (KeyModifiers::NONE, KeyCode::Char('u')) => {
                        state.unstage_all();
                    }

                    // Refresh
                    (KeyModifiers::NONE, KeyCode::Char('r')) => {
                        state.refresh();
                    }

                    // Go to top
                    (KeyModifiers::NONE, KeyCode::Char('g')) => {
                        state.status_state.section = Section::Staged;
                        state.status_state.selected = 0;
                        state
                            .status_state
                            .clamp(staged_len, unstaged_len, untracked_len);
                        state.update_diff();
                    }

                    _ => {}
                }
            }
            GitFocus::Diff => {
                let area_height = state.last_area_height;

                match (modifiers, code) {
                    // Focus switching
                    (KeyModifiers::NONE, KeyCode::Char('h')) => {
                        state.focus = GitFocus::Status;
                    }

                    // Scrolling
                    (KeyModifiers::NONE, KeyCode::Char('j') | KeyCode::Down) => {
                        state.diff_state.scroll_down(1, area_height);
                    }
                    (KeyModifiers::NONE, KeyCode::Char('k') | KeyCode::Up) => {
                        state.diff_state.scroll_up(1);
                    }
                    (KeyModifiers::NONE, KeyCode::PageDown) => {
                        state.diff_state.page_down(area_height);
                    }
                    (KeyModifiers::NONE, KeyCode::PageUp) => {
                        state.diff_state.page_up(area_height);
                    }
                    (KeyModifiers::NONE, KeyCode::Char('g')) => {
                        state.diff_state.scroll_to_top();
                    }

                    _ => {}
                }
            }
        }
    }
}

impl Tab for GitTab {
    fn id(&self) -> TabId {
        self.id
    }

    fn title(&self) -> &str {
        "Git"
    }

    fn icon(&self) -> Option<&str> {
        Some("\u{f1d3}") // Git icon (nerd font)
    }

    fn can_close(&self) -> bool {
        false // Git tab is a core tab
    }

    fn focus(&mut self) {
        self.state.lock().refresh();
    }

    fn blur(&mut self) {
        // Nothing to do
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        let state = self.state.lock();

        // Split into status (40%) and diff (60%)
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area);

        // Render status panel
        render_status_panel(frame, chunks[0], &state);

        // Render diff panel
        render_diff_panel(frame, chunks[1], &state);
    }

    fn handle_message(&mut self, message: &Message) -> Option<Message> {
        if let Message::Key(key) = message {
            self.handle_key(key.code, key.modifiers);
        }
        None
    }
}

fn render_status_panel(frame: &mut Frame, area: Rect, state: &GitTabState) {
    let border_style = if state.focus == GitFocus::Status {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(format!(" {} ", state.branch))
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 3 {
        return;
    }

    let buf = frame.buffer_mut();
    let total_items = state.staged.len() + state.unstaged.len() + state.untracked.len();

    if total_items == 0 {
        let msg = "Working tree clean";
        let x = inner.x + (inner.width.saturating_sub(msg.len() as u16)) / 2;
        let y = inner.y + inner.height / 2;
        buf.set_string(x, y, msg, Style::default().fg(Color::Green));
        return;
    }

    let mut y = inner.y;

    // Render staged section
    if !state.staged.is_empty() {
        let header_style = Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD);
        buf.set_string(
            inner.x,
            y,
            format!("── Staged ({}) ──", state.staged.len()),
            header_style,
        );
        y += 1;

        for (i, entry) in state.staged.iter().enumerate() {
            if y >= inner.y + inner.height {
                break;
            }
            let selected =
                state.status_state.section == Section::Staged && state.status_state.selected == i;
            render_status_entry(buf, inner.x, y, inner.width, entry, selected, Color::Green);
            y += 1;
        }
        y += 1;
    }

    // Render unstaged section
    if !state.unstaged.is_empty() && y < inner.y + inner.height {
        let header_style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);
        buf.set_string(
            inner.x,
            y,
            format!("── Modified ({}) ──", state.unstaged.len()),
            header_style,
        );
        y += 1;

        for (i, entry) in state.unstaged.iter().enumerate() {
            if y >= inner.y + inner.height {
                break;
            }
            let selected =
                state.status_state.section == Section::Unstaged && state.status_state.selected == i;
            render_status_entry(buf, inner.x, y, inner.width, entry, selected, Color::Yellow);
            y += 1;
        }
        y += 1;
    }

    // Render untracked section
    if !state.untracked.is_empty() && y < inner.y + inner.height {
        let header_style = Style::default()
            .fg(Color::Gray)
            .add_modifier(Modifier::BOLD);
        buf.set_string(
            inner.x,
            y,
            format!("── Untracked ({}) ──", state.untracked.len()),
            header_style,
        );
        y += 1;

        for (i, entry) in state.untracked.iter().enumerate() {
            if y >= inner.y + inner.height {
                break;
            }
            let selected = state.status_state.section == Section::Untracked
                && state.status_state.selected == i;
            render_status_entry(buf, inner.x, y, inner.width, entry, selected, Color::Gray);
            y += 1;
        }
    }
}

fn render_status_entry(
    buf: &mut ratatui::buffer::Buffer,
    x: u16,
    y: u16,
    width: u16,
    entry: &StatusEntry,
    selected: bool,
    section_color: Color,
) {
    let status_char = entry.status.indicator();
    let path_str = entry.path.display().to_string();

    let style = if selected {
        Style::default()
            .fg(Color::Black)
            .bg(section_color)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(match entry.status {
            crate::repo::FileStatus::Added => Color::Green,
            crate::repo::FileStatus::Modified => Color::Yellow,
            crate::repo::FileStatus::Deleted => Color::Red,
            crate::repo::FileStatus::Untracked => Color::Gray,
            _ => Color::White,
        })
    };

    let text = format!(" {} {}", status_char, path_str);
    let text = if text.len() as u16 > width {
        format!("{}...", &text[..width.saturating_sub(3) as usize])
    } else {
        text
    };

    buf.set_string(x, y, &text, style);
}

fn render_diff_panel(frame: &mut Frame, area: Rect, state: &GitTabState) {
    let border_style = if state.focus == GitFocus::Diff {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title = if state.current_diff.path.as_os_str().is_empty() {
        " Diff ".to_string()
    } else {
        format!(" {} ", state.current_diff.path.display())
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if state.current_diff.hunks.is_empty() {
        let msg = "No changes to display";
        let buf = frame.buffer_mut();
        let x = inner.x + (inner.width.saturating_sub(msg.len() as u16)) / 2;
        let y = inner.y + inner.height / 2;
        buf.set_string(x, y, msg, Style::default().fg(Color::DarkGray));
        return;
    }

    let buf = frame.buffer_mut();
    let mut y = inner.y;
    let mut line_idx: u16 = 0;

    'outer: for hunk in &state.current_diff.hunks {
        if line_idx < state.diff_state.scroll {
            line_idx += 1 + hunk.lines.len() as u16;
            continue;
        }

        // Render hunk header
        if line_idx >= state.diff_state.scroll {
            let header_style = Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD);
            buf.set_string(inner.x, y, &hunk.header, header_style);
            y += 1;
            if y >= inner.y + inner.height {
                break;
            }
        }
        line_idx += 1;

        // Render lines
        for line in &hunk.lines {
            if line_idx < state.diff_state.scroll {
                line_idx += 1;
                continue;
            }

            if y >= inner.y + inner.height {
                break 'outer;
            }

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

            let content = line.content.trim_end_matches('\n');
            let text = format!("{}{}", line.origin, content);
            let text = if text.len() as u16 > inner.width {
                format!("{}...", &text[..inner.width.saturating_sub(3) as usize])
            } else {
                text
            };

            buf.set_string(inner.x, y, &text, style);
            y += 1;
            line_idx += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::Repository;
    use std::fs;
    use tempfile::TempDir;

    fn init_test_repo() -> TempDir {
        let temp = TempDir::new().expect("create temp dir");
        let repo = Repository::init(temp.path()).expect("init repo");

        let mut config = repo.config().expect("get config");
        config.set_str("user.name", "Test").expect("set name");
        config
            .set_str("user.email", "test@test.com")
            .expect("set email");

        temp
    }

    #[test]
    fn test_git_tab_new_not_repo() {
        let temp = TempDir::new().unwrap();
        let tab = GitTab::new(1, temp.path());
        let state = tab.state.lock();
        assert!(state.repo.is_none());
        assert!(state.error_message.is_some());
    }

    #[test]
    fn test_git_tab_new_valid_repo() {
        let temp = init_test_repo();
        let tab = GitTab::new(1, temp.path());
        let state = tab.state.lock();
        assert!(state.repo.is_some());
        assert!(state.error_message.is_none());
    }

    #[test]
    fn test_git_tab_refresh() {
        let temp = init_test_repo();
        let tab = GitTab::new(1, temp.path());

        // Create a file
        fs::write(temp.path().join("test.txt"), "hello").expect("write file");

        {
            let mut state = tab.state.lock();
            state.refresh();
            assert_eq!(state.untracked.len(), 1);
        }
    }

    #[test]
    fn test_git_tab_properties() {
        let temp = init_test_repo();
        let tab = GitTab::new(1, temp.path());

        assert_eq!(tab.id(), 1);
        assert_eq!(tab.title(), "Git");
        assert!(!tab.can_close());
        assert!(tab.icon().is_some());
    }
}
