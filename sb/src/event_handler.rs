//! Event handling module for better organization of input events
//!
//! This module provides a clear, organized way to handle different types of
//! input events and application modes, making the main event loop cleaner
//! and more maintainable.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;
use tui_textarea::{Input as TextInput, Key as TextKey};

fn to_textarea_input(key_event: KeyEvent) -> TextInput {
    let key = match key_event.code {
        KeyCode::Char(c) => TextKey::Char(c),
        KeyCode::Backspace => TextKey::Backspace,
        KeyCode::Enter => TextKey::Enter,
        KeyCode::Left => TextKey::Left,
        KeyCode::Right => TextKey::Right,
        KeyCode::Up => TextKey::Up,
        KeyCode::Down => TextKey::Down,
        KeyCode::Tab => TextKey::Tab,
        KeyCode::Delete => TextKey::Delete,
        KeyCode::Home => TextKey::Home,
        KeyCode::End => TextKey::End,
        KeyCode::PageUp => TextKey::PageUp,
        KeyCode::PageDown => TextKey::PageDown,
        KeyCode::Esc => TextKey::Esc,
        KeyCode::F(n) => TextKey::F(n),
        _ => TextKey::Null,
    };

    TextInput {
        key,
        ctrl: key_event.modifiers.contains(KeyModifiers::CONTROL),
        alt: key_event.modifiers.contains(KeyModifiers::ALT),
        shift: key_event.modifiers.contains(KeyModifiers::SHIFT),
    }
}

use crate::app::{App, Focus, OpMode};

/// Main event handler dispatcher
///
/// Routes key events to the appropriate handler based on the current application state.
/// This provides a clean separation of concerns for different input modes.
///
/// # Arguments
///
/// * `app` - Mutable reference to the application state
/// * `key_event` - The key event to process
///
/// # Returns
///
/// * `Some(())` - Event was handled, continue the main loop
/// * `None` - Event requests application exit
pub fn handle_key_event(app: &mut App, key_event: KeyEvent) -> Option<()> {
    match app.current_mode() {
        AppMode::FileCreation => handle_file_creation(app, key_event),
        AppMode::FilePicker => handle_file_picker(app, key_event),
        AppMode::MoveDestination => handle_move_destination(app, key_event),
        AppMode::GitStatus => handle_git_status(app, key_event),
        AppMode::Operation => handle_operation(app, key_event),
        AppMode::LineEdit => handle_line_edit(app, key_event),
        AppMode::EditorCommand => handle_editor_command(app, key_event),
        AppMode::RawEditor => handle_raw_editor(app, key_event),
        AppMode::Help => handle_help(app, key_event),
        AppMode::DeleteConfirmation => handle_delete_confirmation(app, key_event),
        AppMode::Normal => handle_normal_mode(app, key_event),
    }
}

/// Application modes for cleaner event routing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    /// Creating a new file
    FileCreation,
    /// File picker overlay is active
    FilePicker,
    /// Move destination picker is active  
    MoveDestination,
    /// Git status display is active
    GitStatus,
    /// File operation mode (copy/move/etc.)
    Operation,
    /// Inline line editing mode
    LineEdit,
    /// Editor command mode (:w, :q, etc.)
    EditorCommand,
    /// Raw editor mode
    RawEditor,
    /// Help screen is displayed
    Help,
    /// Delete confirmation dialog
    DeleteConfirmation,
    /// Normal browsing/navigation mode
    Normal,
}

impl App {
    /// Determine the current application mode for event routing
    pub fn current_mode(&self) -> AppMode {
        if self.creating_file {
            AppMode::FileCreation
        } else if self.picking_file {
            AppMode::FilePicker
        } else if self.showing_move_dest {
            AppMode::MoveDestination
        } else if self.showing_git_status {
            AppMode::GitStatus
        } else if !matches!(self.op_mode, OpMode::None) {
            AppMode::Operation
        } else if self.editing_line {
            AppMode::LineEdit
        } else if self.editor_cmd_mode {
            AppMode::EditorCommand
        } else if self.show_raw_editor {
            AppMode::RawEditor
        } else if self.show_help {
            AppMode::Help
        } else if self.confirming_delete {
            AppMode::DeleteConfirmation
        } else {
            AppMode::Normal
        }
    }
}

/// Handle file creation mode events
fn handle_file_creation(app: &mut App, key_event: KeyEvent) -> Option<()> {
    match (key_event.code, key_event.modifiers) {
        (KeyCode::Enter, _) => {
            let _ = app.confirm_create_file();
        }
        (KeyCode::Esc, _) => {
            app.cancel_create_file();
        }
        _ => {
            let _ = app.filename_input.input(to_textarea_input(key_event));
        }
    }
    Some(())
}

/// Handle file picker mode events
fn handle_file_picker(app: &mut App, key_event: KeyEvent) -> Option<()> {
    match (key_event.code, key_event.modifiers) {
        (KeyCode::Esc, _) => app.picker_cancel(),
        (KeyCode::Enter, _) => {
            let _ = app.picker_activate();
        }
        (KeyCode::Up, modifiers) if modifiers.contains(KeyModifiers::SHIFT) => {
            app.picker_up_with_selection()
        }
        (KeyCode::Down, modifiers) if modifiers.contains(KeyModifiers::SHIFT) => {
            app.picker_down_with_selection()
        }
        (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
            app.picker_clear_selection();
            app.picker_up();
        }
        (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
            app.picker_clear_selection();
            app.picker_down();
        }
        (KeyCode::Char('d') | KeyCode::Char('D'), _) => {
            let _ = app.picker_delete_with_git_check();
        }
        (KeyCode::Char('m') | KeyCode::Char('M'), _) => {
            let _ = app.picker_start_move();
        }
        (KeyCode::Char('p') | KeyCode::Char('P'), _) => {
            let _ = app.picker_parent_dir();
        }
        (KeyCode::Char('g') | KeyCode::Char('G'), _) => {
            app.picker_show_git_status();
        }
        (KeyCode::Char('o') | KeyCode::Char('O'), _) => {
            if !app.picker_selection.is_empty() {
                // Bulk open mode
                let selected_files: Vec<PathBuf> = app
                    .picker_selection
                    .iter()
                    .filter_map(|&i| app.picker_items.get(i).cloned())
                    .filter(|p| p.is_file())
                    .collect();

                if !selected_files.is_empty() {
                    let count = selected_files.len();
                    for path in &selected_files {
                        let _ = app.open_in_editor(path);
                    }
                    app.status = format!("Opened {} files in editor", count);
                } else {
                    app.status = "No files selected to open".to_string();
                }
            } else if app.picker_index < app.picker_items.len() {
                // Single file open (original behavior)
                let path = app.picker_items[app.picker_index].clone();
                let _ = app.open_in_editor(&path);
            }
        }
        (KeyCode::Char(' '), _) => {
            // Space to toggle selection of current item
            app.picker_toggle_selection();
        }
        (KeyCode::Char('a'), modifiers) if modifiers.contains(KeyModifiers::CONTROL) => {
            // Ctrl+A to select all
            app.picker_selection.clear();
            for i in 0..app.picker_items.len() {
                app.picker_selection.insert(i);
            }
            app.picker_selection_anchor = None;
        }
        (KeyCode::Char(c), _) => {
            app.status = format!("File picker: Unknown key '{}'", c);
        }
        _ => {}
    }
    Some(())
}

/// Handle move destination picker mode events
fn handle_move_destination(app: &mut App, key_event: KeyEvent) -> Option<()> {
    match (key_event.code, key_event.modifiers) {
        (KeyCode::Esc, _) => app.cancel_move(),
        (KeyCode::Enter, _) => {
            let _ = app.confirm_move();
        }
        (KeyCode::Up, _) | (KeyCode::Char('k'), _) => app.move_dest_up(),
        (KeyCode::Down, _) | (KeyCode::Char('j'), _) => app.move_dest_down(),
        (KeyCode::Right, _) | (KeyCode::Char('l'), _) => {
            let _ = app.move_dest_enter();
        }
        _ => {}
    }
    Some(())
}

/// Handle Git status display mode events
fn handle_git_status(app: &mut App, key_event: KeyEvent) -> Option<()> {
    match (key_event.code, key_event.modifiers) {
        (KeyCode::Esc, _) | (KeyCode::Enter, _) | (KeyCode::Char('g'), _) => {
            app.close_git_status();
        }
        _ => {}
    }
    Some(())
}

/// Handle file operation mode events
fn handle_operation(app: &mut App, key_event: KeyEvent) -> Option<()> {
    match (key_event.code, key_event.modifiers) {
        (KeyCode::Esc, _) => app.cancel_op(),
        (KeyCode::Enter, _) => {
            let _ = app.confirm_op();
        }
        _ => {
            let _ = app.op_input.input(to_textarea_input(key_event));
        }
    }
    Some(())
}

/// Handle inline line editing mode events
fn handle_line_edit(app: &mut App, key_event: KeyEvent) -> Option<()> {
    match (key_event.code, key_event.modifiers) {
        (KeyCode::Enter, _) => {
            app.confirm_line_edit();
        }
        (KeyCode::Esc, _) => app.cancel_line_edit(),
        _ => {
            let _ = app.line_input.input(to_textarea_input(key_event));
        }
    }
    Some(())
}

/// Handle editor command mode events
fn handle_editor_command(app: &mut App, key_event: KeyEvent) -> Option<()> {
    match (key_event.code, key_event.modifiers) {
        (KeyCode::Esc, _) => app.cancel_editor_cmd(),
        (KeyCode::Enter, _) => {
            let _ = app.confirm_editor_cmd();
        }
        _ => {
            let _ = app.editor_cmd_input.input(to_textarea_input(key_event));
        }
    }
    Some(())
}

/// Handle raw editor mode events
fn handle_raw_editor(app: &mut App, key_event: KeyEvent) -> Option<()> {
    match (key_event.code, key_event.modifiers) {
        (KeyCode::Esc, _) => {
            app.show_raw_editor = false;
            app.prefer_raw_editor = false;
            app.focus = Focus::Preview;
        }
        (KeyCode::Tab, _) => {
            // Exit raw editor but handle Tab like in normal mode
            app.show_raw_editor = false;
            app.prefer_raw_editor = true;
            return handle_normal_mode(app, key_event);
        }
        _ => {
            if !app.editor_cmd_mode {
                app.editor.handle_key_event(key_event);
            }
        }
    }
    Some(())
}

/// Handle help screen mode events
fn handle_help(app: &mut App, key_event: KeyEvent) -> Option<()> {
    match (key_event.code, key_event.modifiers) {
        (KeyCode::Esc, _) | (KeyCode::Char('?'), _) | (KeyCode::Char('h'), _) => {
            app.toggle_help();
        }
        _ => {}
    }
    Some(())
}

/// Handle delete confirmation mode events
fn handle_delete_confirmation(app: &mut App, key_event: KeyEvent) -> Option<()> {
    match key_event.code {
        KeyCode::Enter | KeyCode::Char('d') => {
            let _ = app.confirm_delete_with_git();
        }
        KeyCode::Esc => app.cancel_delete(),
        _ => {}
    }
    Some(())
}

/// Handle normal browsing/navigation mode events
fn handle_normal_mode(app: &mut App, key_event: KeyEvent) -> Option<()> {
    match (key_event.code, key_event.modifiers) {
        // Application exit commands
        (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
            return None; // Signal exit
        }
        (KeyCode::Esc, _) => {
            if matches!(app.focus, Focus::Preview) {
                return None; // Signal exit
            }
        }

        // Focus and navigation commands
        (KeyCode::Tab, mods) => handle_tab_navigation(app, mods),
        (KeyCode::BackTab, _) => handle_back_tab_navigation(app),

        // Application commands
        (KeyCode::Char('?'), _) => app.toggle_help(),
        (KeyCode::Char('i'), mods) if mods.contains(KeyModifiers::CONTROL) => {
            let _ = app.begin_file_picker();
        }
        (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
            let _ = app.save();
        }
        (KeyCode::Char('o'), _) if matches!(app.focus, Focus::Left | Focus::Editor) => {
            // Check if we have tree selections for bulk open
            if !app.tree_selection.is_empty() && matches!(app.focus, Focus::Left) {
                let _ = app.tree_open_selected();
            } else {
                let _ = app.open_externally();
            }
        }

        // Video controls (avoid overriding tree selection bindings)
        (KeyCode::Char(' '), _) if !matches!(app.focus, Focus::Left) => {
            app.toggle_pause_video();
        }
        (KeyCode::Char('s'), _) if !matches!(app.focus, Focus::Left) => {
            app.stop_video();
        }
        (KeyCode::Char('v'), KeyModifiers::CONTROL) => {
            app.autoplay_video = !app.autoplay_video;
            app.status = format!(
                "Autoplay video {}",
                if app.autoplay_video { "on" } else { "off" }
            );
        }

        // Pane controls
        (KeyCode::Char('b'), KeyModifiers::CONTROL) | (KeyCode::F(9), _) => {
            app.toggle_left_pane();
        }
        (KeyCode::Char(','), modifiers) | (KeyCode::Char('-'), modifiers)
            if modifiers.contains(KeyModifiers::CONTROL) =>
        {
            app.decrease_left_pane_width();
        }
        (KeyCode::Char('.'), modifiers) | (KeyCode::Char('='), modifiers)
            if modifiers.contains(KeyModifiers::CONTROL) =>
        {
            app.increase_left_pane_width();
        }

        // Function keys (Midnight Commander style)
        (KeyCode::F(2), _) => {
            let _ = app.begin_file_picker();
        }
        (KeyCode::F(4), _) => {
            app.focus = Focus::Editor;
            if app.prefer_raw_editor {
                app.show_raw_editor = true;
            }
        }
        (KeyCode::F(5), _) => {
            app.begin_copy();
        }
        (KeyCode::F(6), _) => {
            app.begin_move();
        }
        (KeyCode::F(7), _) => {
            app.begin_mkdir();
        }
        (KeyCode::F(8), _) => {
            app.begin_delete();
        }
        (KeyCode::F(10), _) => {
            return None;
        }

        // Context-specific commands
        (KeyCode::Char('n'), _) if matches!(app.focus, Focus::Left) => {
            app.begin_create_file();
        }
        (KeyCode::Char('d'), _) if matches!(app.focus, Focus::Left) => {
            app.begin_delete();
        }
        (KeyCode::Char(':'), _) if matches!(app.focus, Focus::Editor) => {
            app.begin_editor_cmd();
        }

        // Delegate to focus-specific handlers
        _ => handle_focus_specific(app, key_event),
    }

    Some(())
}

/// Handle Tab navigation between panes
fn handle_tab_navigation(app: &mut App, modifiers: KeyModifiers) {
    if modifiers.contains(KeyModifiers::CONTROL) {
        // Ctrl+I for file picker
        let _ = app.begin_file_picker();
    } else if app.show_left_pane {
        // Tab between left pane and right pane
        app.focus = match app.focus {
            Focus::Left => {
                // When moving from left to right, go to Preview by default
                // Only go to Editor if raw editor is already active
                if app.show_raw_editor && app.opened.is_some() {
                    Focus::Editor
                } else {
                    Focus::Preview
                }
            }
            Focus::Preview => Focus::Left,
            Focus::Editor => {
                // When leaving editor, turn off raw editor mode
                app.show_raw_editor = false;
                app.prefer_raw_editor = false;
                Focus::Left
            }
        };
    }
}

/// Handle BackTab (Shift+Tab) navigation between panes
fn handle_back_tab_navigation(app: &mut App) {
    if app.show_left_pane {
        // Same logic as Tab for 2-pane navigation
        app.focus = match app.focus {
            Focus::Left => {
                // When moving from left to right, go to Preview by default
                if app.show_raw_editor && app.opened.is_some() {
                    Focus::Editor
                } else {
                    Focus::Preview
                }
            }
            Focus::Preview => Focus::Left,
            Focus::Editor => {
                // When leaving editor, turn off raw editor mode
                app.show_raw_editor = false;
                app.prefer_raw_editor = false;
                Focus::Left
            }
        };
    }
}

/// Handle focus-specific key events
fn handle_focus_specific(app: &mut App, key_event: KeyEvent) {
    match app.focus {
        Focus::Left => handle_left_pane_keys(app, key_event),
        Focus::Editor => handle_editor_keys(app, key_event),
        Focus::Preview => handle_preview_keys(app, key_event),
    }
}

/// Handle key events when left pane has focus
fn handle_left_pane_keys(app: &mut App, key_event: KeyEvent) {
    match (key_event.code, key_event.modifiers) {
        (KeyCode::Up, modifiers) if modifiers.contains(KeyModifiers::SHIFT) => {
            app.tree_up_with_selection();
        }
        (KeyCode::Down, modifiers) if modifiers.contains(KeyModifiers::SHIFT) => {
            app.tree_down_with_selection();
        }
        (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
            let _ = app.left_state.key_up();
        }
        (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
            let _ = app.left_state.key_down();
        }
        (KeyCode::Left, _) => {
            // Try to collapse tree node
            let _ = app.left_state.key_left();
        }
        (KeyCode::Right, _) => {
            // Try to expand tree node or move to preview if it's a file
            let is_file = app
                .left_state
                .selected()
                .first()
                .and_then(|s| std::path::Path::new(s).to_str())
                .map(|s| std::path::Path::new(s).is_file())
                .unwrap_or(false);

            if is_file {
                // If it's a file, open it and switch to preview pane
                let _ = app.open_selected();
                app.focus = Focus::Preview;
            } else {
                // If it's a directory, try to expand it
                let _ = app.left_state.key_right();
            }
        }
        (KeyCode::Enter, _) => {
            let _ = app.activate_on_tree();
        }
        (KeyCode::Char('r'), _) => {
            let _ = app.refresh_tree();
        }
        (KeyCode::Char('h'), _) => app.toggle_left_pane(),
        (KeyCode::Char('s') | KeyCode::Char('S'), _) => {
            app.tree_accumulate_selection();
        }
        (KeyCode::Esc, _) if !app.tree_selection.is_empty() => {
            // Clear all selections when Esc pressed in left pane with selections
            app.tree_clear_selection();
        }
        (KeyCode::Char('n'), _) => app.begin_create_file(),
        (KeyCode::Char('d'), _) => app.begin_delete(),
        (KeyCode::Char('a'), modifiers) if modifiers.contains(KeyModifiers::CONTROL) => {
            app.tree_select_all();
        }
        (KeyCode::F(5), _) => app.begin_copy(),
        (KeyCode::F(6), _) => app.begin_move(),
        (KeyCode::F(7), _) => app.begin_create_file(),
        (KeyCode::F(8), _) => app.begin_delete(),
        _ => {}
    }
}

/// Handle key events when editor has focus
fn handle_editor_keys(app: &mut App, key_event: KeyEvent) {
    match (key_event.code, key_event.modifiers) {
        (KeyCode::Char('r'), KeyModifiers::CONTROL) => {
            // Toggle to raw editor mode
            app.show_raw_editor = true;
            app.prefer_raw_editor = true;
        }
        _ => {
            // Default editor input handling
            app.editor.handle_key_event(key_event);
        }
    }
}

/// Handle key events when preview pane has focus
fn handle_preview_keys(app: &mut App, key_event: KeyEvent) {
    match (key_event.code, key_event.modifiers) {
        (KeyCode::Up, _) | (KeyCode::Char('k'), _) => app.move_cursor_up(),
        (KeyCode::Down, _) | (KeyCode::Char('j'), _) => app.move_cursor_down(),
        (KeyCode::Char('i'), _) => app.begin_line_edit(),
        (KeyCode::Char('e'), _) => {
            app.focus = Focus::Editor;
            app.show_raw_editor = true;
            app.prefer_raw_editor = true;
            app.editor.set_cursor(app.preview_cursor, app.preview_col);
        }
        (KeyCode::Char('h'), _) => app.toggle_left_pane(),
        (KeyCode::Char('r'), KeyModifiers::CONTROL) => {
            // Switch to raw editor mode
            app.focus = Focus::Editor;
            app.show_raw_editor = true;
            app.prefer_raw_editor = true;
        }
        (KeyCode::PageUp, _) => {
            for _ in 0..10 {
                app.move_cursor_up();
            }
        }
        (KeyCode::PageDown, _) => {
            for _ in 0..10 {
                app.move_cursor_down();
            }
        }
        (KeyCode::Left, _) => app.move_col_left(),
        (KeyCode::Right, _) => app.move_col_right(),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_app_mode_detection() {
        let mut app = App::new(PathBuf::from(".")).unwrap();

        // Test normal mode
        assert_eq!(app.current_mode(), AppMode::Normal);

        // Test file creation mode
        app.creating_file = true;
        assert_eq!(app.current_mode(), AppMode::FileCreation);
        app.creating_file = false;

        // Test file picker mode
        app.picking_file = true;
        assert_eq!(app.current_mode(), AppMode::FilePicker);
        app.picking_file = false;

        // Test help mode
        app.show_help = true;
        assert_eq!(app.current_mode(), AppMode::Help);
        app.show_help = false;
    }

    #[test]
    fn test_key_event_routing() {
        let mut app = App::new(PathBuf::from(".")).unwrap();

        // Test normal mode key handling
        let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let result = handle_key_event(&mut app, key_event);
        assert!(result.is_none()); // Should signal exit

        // Test help toggle
        let key_event = KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE);
        assert_eq!(app.show_help, false);
        handle_key_event(&mut app, key_event);
        assert_eq!(app.show_help, true);
    }

    #[test]
    fn test_preview_e_enters_raw_editor() {
        let mut app = App::new(PathBuf::from(".")).unwrap();
        app.focus = Focus::Preview;
        let key_event = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
        handle_key_event(&mut app, key_event);
        assert!(app.show_raw_editor);
        assert!(matches!(app.focus, Focus::Editor));
    }

    #[test]
    fn test_raw_editor_tab_falls_back_to_normal() {
        let mut app = App::new(PathBuf::from(".")).unwrap();
        app.focus = Focus::Editor;
        app.show_raw_editor = true;
        let key_event = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        let result = handle_key_event(&mut app, key_event);
        assert!(result.is_some());
        assert!(!app.show_raw_editor);
    }
}
