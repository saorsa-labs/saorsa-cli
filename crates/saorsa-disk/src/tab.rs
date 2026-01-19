//! DiskTab - Disk analyzer as a Tab
//!
//! Provides a TUI interface for analyzing disk usage.

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph};
use saorsa_cli_core::{Message, Tab, TabId};
use std::path::PathBuf;

use crate::analyzer::{DiskAnalyzer, DiskInfo, FileEntry};

/// View mode for the disk tab
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiskView {
    /// Disk usage overview with gauges
    #[default]
    Overview,
    /// List of largest files
    Largest,
    /// List of stale (old) files
    Stale,
}

/// Disk analyzer tab
pub struct DiskTab {
    id: TabId,
    root: PathBuf,
    view: DiskView,
    disk_info: Vec<DiskInfo>,
    largest_files: Vec<FileEntry>,
    stale_files: Vec<FileEntry>,
    list_state: ListState,
    stale_days: u64,
    focused: bool,
}

impl DiskTab {
    /// Create a new disk analyzer tab
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for this tab
    /// * `root` - Root directory to analyze
    #[must_use]
    pub fn new(id: TabId, root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let disk_info = DiskAnalyzer::get_disk_info();

        Self {
            id,
            root,
            view: DiskView::Overview,
            disk_info,
            largest_files: Vec::new(),
            stale_files: Vec::new(),
            list_state: ListState::default(),
            stale_days: 30,
            focused: false,
        }
    }

    /// Refresh disk information
    pub fn refresh(&mut self) {
        self.disk_info = DiskAnalyzer::get_disk_info();
    }

    /// Set the number of days for stale file detection
    pub fn set_stale_days(&mut self, days: u64) {
        self.stale_days = days;
    }

    /// Analyze and display largest files
    pub fn analyze_largest(&mut self, count: usize) {
        let analyzer = DiskAnalyzer::new(&self.root);
        self.largest_files = analyzer.find_largest(count);
        self.view = DiskView::Largest;
        if !self.largest_files.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    /// Analyze and display stale files
    pub fn analyze_stale(&mut self, count: usize) {
        let analyzer = DiskAnalyzer::new(&self.root);
        self.stale_files = analyzer.find_stale(self.stale_days, count);
        self.view = DiskView::Stale;
        if !self.stale_files.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    /// Handle a key event
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<Message> {
        match key.code {
            KeyCode::Char('o') | KeyCode::Char('O') => {
                self.view = DiskView::Overview;
                self.refresh();
            }
            KeyCode::Char('l') | KeyCode::Char('L') => {
                self.analyze_largest(50);
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.analyze_stale(50);
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.refresh();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_selection_up();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_selection_down();
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.select_first();
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.select_last();
            }
            _ => {}
        }
        None
    }

    fn move_selection_up(&mut self) {
        let i = self.list_state.selected().unwrap_or(0);
        self.list_state.select(Some(i.saturating_sub(1)));
    }

    fn move_selection_down(&mut self) {
        let len = self.current_list_len();
        if len == 0 {
            return;
        }
        let i = self.list_state.selected().unwrap_or(0);
        self.list_state.select(Some((i + 1).min(len - 1)));
    }

    fn select_first(&mut self) {
        if self.current_list_len() > 0 {
            self.list_state.select(Some(0));
        }
    }

    fn select_last(&mut self) {
        let len = self.current_list_len();
        if len > 0 {
            self.list_state.select(Some(len - 1));
        }
    }

    fn current_list_len(&self) -> usize {
        match self.view {
            DiskView::Overview => 0,
            DiskView::Largest => self.largest_files.len(),
            DiskView::Stale => self.stale_files.len(),
        }
    }

    /// Render the overview with disk usage gauges
    fn render_overview(&self, frame: &mut Frame, area: Rect) {
        if self.disk_info.is_empty() {
            let msg = Paragraph::new("No disk information available")
                .block(
                    Block::default()
                        .title(" Disk Overview ")
                        .borders(Borders::ALL),
                )
                .alignment(Alignment::Center);
            frame.render_widget(msg, area);
            return;
        }

        let block = Block::default()
            .title(" Disk Overview ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Calculate constraints for disk gauges
        let gauge_height = 3u16;
        let num_disks = self
            .disk_info
            .len()
            .min(inner.height as usize / gauge_height as usize);

        if num_disks == 0 {
            return;
        }

        let constraints: Vec<Constraint> = (0..num_disks)
            .map(|_| Constraint::Length(gauge_height))
            .collect();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner);

        for (i, info) in self.disk_info.iter().enumerate().take(num_disks) {
            let percent = info.usage_percent();
            let color = if percent >= 90.0 {
                Color::Red
            } else if percent >= 70.0 {
                Color::Yellow
            } else {
                Color::Green
            };

            let gauge = Gauge::default()
                .block(
                    Block::default()
                        .title(format!(" {} ", info.display_name()))
                        .borders(Borders::ALL),
                )
                .gauge_style(Style::default().fg(color))
                .percent(percent as u16)
                .label(format!(
                    "{} / {} ({:.1}%)",
                    DiskInfo::format_bytes(info.used),
                    DiskInfo::format_bytes(info.total),
                    percent
                ));

            frame.render_widget(gauge, chunks[i]);
        }
    }

    /// Render a file list (largest or stale)
    fn render_file_list(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        files: &[FileEntry],
        title: &str,
    ) {
        let items: Vec<ListItem> = files
            .iter()
            .map(|f| {
                let size_str = f.format_size();
                let path_str = f.path.display().to_string();
                // Truncate path if too long
                let max_path_len = area.width.saturating_sub(15) as usize;
                let display_path = if path_str.len() > max_path_len {
                    format!(
                        "...{}",
                        &path_str[path_str.len().saturating_sub(max_path_len - 3)..]
                    )
                } else {
                    path_str
                };
                ListItem::new(format!("{:>10}  {}", size_str, display_path))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!(" {} ({} files) ", title, files.len()))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::REVERSED)
                    .fg(Color::Yellow),
            )
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }
}

impl Tab for DiskTab {
    fn id(&self) -> TabId {
        self.id
    }

    fn title(&self) -> &str {
        "Disk"
    }

    fn icon(&self) -> Option<&str> {
        Some("\u{1F4BE}") // Floppy disk emoji
    }

    fn can_close(&self) -> bool {
        true
    }

    fn focus(&mut self) {
        self.focused = true;
        self.refresh();
    }

    fn blur(&mut self) {
        self.focused = false;
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        // Layout: main content + help line
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(area);

        // We need mutable access for list rendering, so clone self for the mutable part
        let mut this = DiskTab {
            id: self.id,
            root: self.root.clone(),
            view: self.view,
            disk_info: self.disk_info.clone(),
            largest_files: self.largest_files.clone(),
            stale_files: self.stale_files.clone(),
            list_state: self.list_state.clone(),
            stale_days: self.stale_days,
            focused: self.focused,
        };

        match this.view {
            DiskView::Overview => this.render_overview(frame, chunks[0]),
            DiskView::Largest => {
                let files = this.largest_files.clone();
                this.render_file_list(frame, chunks[0], &files, "Largest Files");
            }
            DiskView::Stale => {
                let files = this.stale_files.clone();
                this.render_file_list(frame, chunks[0], &files, "Stale Files");
            }
        }

        // Help line
        let help_text =
            " [o]verview  [l]argest  [s]tale  [r]efresh  [j/k] navigate  [g/G] first/last";
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(help, chunks[1]);
    }

    fn handle_message(&mut self, message: &Message) -> Option<Message> {
        if let Message::Key(key) = message {
            return self.handle_key(*key);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_disk_tab_creation() {
        let dir = tempdir().expect("create temp dir");
        let tab = DiskTab::new(1, dir.path());

        assert_eq!(tab.id(), 1);
        assert_eq!(tab.title(), "Disk");
        assert!(tab.can_close());
        assert_eq!(tab.view, DiskView::Overview);
    }

    #[test]
    fn test_disk_tab_icon() {
        let dir = tempdir().expect("create temp dir");
        let tab = DiskTab::new(1, dir.path());

        assert!(tab.icon().is_some());
    }

    #[test]
    fn test_disk_tab_focus_blur() {
        let dir = tempdir().expect("create temp dir");
        let mut tab = DiskTab::new(1, dir.path());

        tab.focus();
        assert!(tab.focused);

        tab.blur();
        assert!(!tab.focused);
    }

    #[test]
    fn test_view_switching() {
        let dir = tempdir().expect("create temp dir");
        let mut tab = DiskTab::new(1, dir.path());

        assert_eq!(tab.view, DiskView::Overview);

        tab.analyze_largest(10);
        assert_eq!(tab.view, DiskView::Largest);

        tab.analyze_stale(10);
        assert_eq!(tab.view, DiskView::Stale);
    }

    #[test]
    fn test_stale_days_config() {
        let dir = tempdir().expect("create temp dir");
        let mut tab = DiskTab::new(1, dir.path());

        assert_eq!(tab.stale_days, 30);
        tab.set_stale_days(60);
        assert_eq!(tab.stale_days, 60);
    }
}
