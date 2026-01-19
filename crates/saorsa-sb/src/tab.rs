//! SbTab - Markdown browser as a Tab
//!
//! Wraps the sb markdown browser App for integration with the saorsa TUI framework.

use anyhow::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use parking_lot::Mutex;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use saorsa_cli_core::{Message, Tab, TabId};
use sb::{handle_key_event, App};
use std::path::PathBuf;

/// Markdown browser tab wrapping the sb App
///
/// Uses `Mutex<App>` to provide interior mutability while satisfying
/// the `Send + Sync` requirements of the `Tab` trait.
pub struct SbTab {
    id: TabId,
    title: String,
    app: Mutex<App>,
    focused: bool,
}

impl SbTab {
    /// Creates a new SbTab for the given directory
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for this tab
    /// * `root` - Root directory to browse
    ///
    /// # Errors
    ///
    /// Returns an error if the App fails to initialize (e.g., invalid path).
    pub fn new(id: TabId, root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        let app = App::new(root)?;
        Ok(Self {
            id,
            title: "Files".to_string(),
            app: Mutex::new(app),
            focused: false,
        })
    }

    /// Creates a new SbTab with a custom title
    pub fn with_title(
        id: TabId,
        root: impl Into<PathBuf>,
        title: impl Into<String>,
    ) -> Result<Self> {
        let mut tab = Self::new(id, root)?;
        tab.title = title.into();
        Ok(tab)
    }

    /// Handle a key event
    ///
    /// Routes key events to the internal sb App's event handler.
    pub fn handle_key(&self, key: KeyEvent) -> Option<Message> {
        let mut app = self.app.lock();
        // Use sb's event handler - returns None if app wants to quit
        if handle_key_event(&mut app, key).is_none() {
            return Some(Message::Quit);
        }
        None
    }

    /// Handle a mouse event
    ///
    /// Routes mouse events to the internal sb App.
    pub fn handle_mouse(&self, _mouse: MouseEvent) -> Option<Message> {
        // Mouse handling would go here if sb supports it
        // For now, pass through without action
        None
    }

    /// Get the current root directory
    pub fn root(&self) -> PathBuf {
        self.app.lock().root.clone()
    }

    /// Get the currently opened file, if any
    pub fn opened_file(&self) -> Option<PathBuf> {
        self.app.lock().opened.clone()
    }
}

impl Tab for SbTab {
    fn id(&self) -> TabId {
        self.id
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn icon(&self) -> Option<&str> {
        Some("\u{1F4C1}") // Folder emoji
    }

    fn can_close(&self) -> bool {
        true
    }

    fn focus(&mut self) {
        self.focused = true;
        // Could resume any paused operations here
    }

    fn blur(&mut self) {
        self.focused = false;
        // Pause video if playing
        let mut app = self.app.lock();
        app.pause_video();
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        // Lock the app for rendering
        let mut app = self.app.lock();

        // Calculate layout
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),    // Main content
                Constraint::Length(1), // Status bar
            ])
            .split(area);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(if app.show_left_pane {
                let left_width = app.calculate_left_pane_width(main_chunks[0].width);
                [Constraint::Length(left_width), Constraint::Min(40)]
            } else {
                [Constraint::Length(0), Constraint::Min(40)]
            })
            .split(main_chunks[0]);

        // Render left pane (file tree) if visible
        if app.show_left_pane {
            let left_border = if matches!(app.focus, sb::Focus::Left) {
                Color::Cyan
            } else {
                Color::Blue
            };
            let left_block = Block::default()
                .title("Files")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(left_border));

            // Clone tree items to avoid borrow conflict with tree state
            let tree_items = app.left_tree.clone();

            // Render tree widget
            if let Ok(tree) = tui_tree_widget::Tree::new(&tree_items) {
                let tree = tree
                    .block(left_block)
                    .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
                frame.render_stateful_widget(tree, chunks[0], &mut app.left_state);
            } else {
                // Fallback if tree fails
                frame.render_widget(left_block, chunks[0]);
            }
        }

        // Render right pane (preview or editor)
        let right_border = if matches!(app.focus, sb::Focus::Preview) {
            Color::Cyan
        } else {
            Color::Blue
        };

        if app.show_raw_editor {
            let block = Block::default()
                .title("Edit (raw)")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green));
            frame.render_widget(block.clone(), chunks[1]);
            let inner = block.inner(chunks[1]);
            let view = app.editor.view();
            frame.render_widget(view, inner);
        } else {
            // Render preview
            let text = app.editor.text();
            if let Some(path) = app.opened.as_ref() {
                if let Ok(preview) = sb::preview::Preview::from_markdown(path, &text) {
                    sb::preview::render_preview(frame, chunks[1], &preview);
                } else {
                    let block = Block::default()
                        .title("Preview")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(right_border));
                    let content = Paragraph::new("(preview error)").block(block);
                    frame.render_widget(content, chunks[1]);
                }
            } else {
                let block = Block::default()
                    .title("Preview")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(right_border));
                let content = Paragraph::new("Select a file to preview").block(block);
                frame.render_widget(content, chunks[1]);
            }
        }

        // Render status bar
        let file_label = app
            .opened
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("(no file)");

        let status_text = format!(" {} | {} | ? help", file_label, app.status);
        let status = Paragraph::new(status_text)
            .style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        frame.render_widget(status, main_chunks[1]);

        // Render overlays if active
        if app.show_help {
            render_help_overlay(frame, area);
        }
    }

    fn handle_message(&mut self, message: &Message) -> Option<Message> {
        match message {
            Message::Key(key) => self.handle_key(*key),
            Message::Mouse(mouse) => self.handle_mouse(*mouse),
            _ => None,
        }
    }
}

/// Render a centered help overlay
fn render_help_overlay(frame: &mut Frame, area: Rect) {
    use ratatui::widgets::{Clear, Wrap};

    let help = [
        "sb - Markdown TUI",
        "",
        "Navigation:",
        "  Tab/Shift+Tab  Switch focus",
        "  Up/Down, j/k   Navigate tree",
        "  Enter          Open file",
        "  Left/Right     Collapse/expand",
        "",
        "Editing:",
        "  e              Edit mode",
        "  Ctrl+S         Save",
        "  Esc            Exit edit mode",
        "",
        "Files:",
        "  n              New file",
        "  d              Delete",
        "  F2             File picker",
        "",
        "Other:",
        "  ?              Toggle help",
        "  q              Close tab",
    ]
    .join("\n");

    let w = area.width.min(50);
    let h = area.height.min(20);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let popup = Rect {
        x,
        y,
        width: w,
        height: h,
    };

    frame.render_widget(Clear, popup);
    let paragraph = Paragraph::new(help)
        .block(Block::default().title(" Help ").borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, popup);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_sb_tab_creation() {
        let dir = tempdir().expect("create temp dir");
        let tab = SbTab::new(1, dir.path()).expect("create tab");

        assert_eq!(tab.id(), 1);
        assert_eq!(tab.title(), "Files");
        assert!(tab.can_close());
    }

    #[test]
    fn test_sb_tab_with_title() {
        let dir = tempdir().expect("create temp dir");
        let tab = SbTab::with_title(1, dir.path(), "My Files").expect("create tab");

        assert_eq!(tab.title(), "My Files");
    }

    #[test]
    fn test_sb_tab_icon() {
        let dir = tempdir().expect("create temp dir");
        let tab = SbTab::new(1, dir.path()).expect("create tab");

        assert!(tab.icon().is_some());
    }

    #[test]
    fn test_sb_tab_focus_blur() {
        let dir = tempdir().expect("create temp dir");
        let mut tab = SbTab::new(1, dir.path()).expect("create tab");

        tab.focus();
        assert!(tab.focused);

        tab.blur();
        assert!(!tab.focused);
    }
}
