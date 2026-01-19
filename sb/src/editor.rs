use crossterm::event::{Event as CrosstermEvent, KeyEvent};
use edtui::{
    actions::{Execute, InsertChar, LineBreak},
    events::EditorEventHandler,
    EditorState, EditorTheme, EditorView, LineNumbers, Lines,
};
use ratatui::style::{Color, Style};

/// Wrapper around the edtui editor so the rest of the app can stay agnostic of the
/// underlying widget implementation.
pub struct MainEditor {
    state: EditorState,
    handler: EditorEventHandler,
    line_numbers: LineNumbers,
    wrap: bool,
}

impl MainEditor {
    #[must_use]
    pub fn new() -> Self {
        let mut state = EditorState::new(Lines::from(""));
        // edtui expects at least one line to exist
        if state.lines.is_empty() {
            state.lines.push(Vec::new());
        }

        Self {
            state,
            handler: EditorEventHandler::vim_mode(),
            line_numbers: LineNumbers::Relative,
            wrap: false,
        }
    }

    fn lines_to_string(lines: &Lines) -> String {
        let mut buf = String::new();
        let total_rows = lines.len();
        for (idx, row) in lines.iter_row().enumerate() {
            for ch in row {
                buf.push(*ch);
            }
            if idx + 1 != total_rows {
                buf.push('\n');
            }
        }
        buf
    }

    fn string_to_lines<S: AsRef<str>>(text: S) -> Lines {
        Lines::from(text.as_ref())
    }

    #[must_use]
    pub fn text(&self) -> String {
        Self::lines_to_string(&self.state.lines)
    }

    #[must_use]
    pub fn lines_vec(&self) -> Vec<String> {
        self.text().split('\n').map(|s| s.to_string()).collect()
    }

    pub fn set_text<S: AsRef<str>>(&mut self, text: S) {
        self.state = EditorState::new(Self::string_to_lines(text));
        if self.state.lines.is_empty() {
            self.state.lines.push(Vec::new());
        }
    }

    pub fn set_lines_vec(&mut self, lines: Vec<String>) {
        let joined = if lines.is_empty() {
            String::new()
        } else {
            lines.join("\n")
        };
        self.set_text(joined);
    }

    #[must_use]
    pub fn line_count(&self) -> usize {
        self.state.lines.len()
    }

    #[must_use]
    pub fn line_at(&self, idx: usize) -> Option<String> {
        self.lines_vec().get(idx).cloned()
    }

    pub fn insert_text(&mut self, text: &str) {
        for ch in text.chars() {
            InsertChar(ch).execute(&mut self.state);
        }
    }

    #[allow(dead_code)]
    pub fn insert_newline(&mut self) {
        LineBreak(1).execute(&mut self.state);
    }

    pub fn cursor(&self) -> (usize, usize) {
        (self.state.cursor.row, self.state.cursor.col)
    }

    pub fn set_cursor(&mut self, row: usize, col: usize) {
        self.state.cursor.row = row.min(self.line_count().saturating_sub(1));
        self.state.cursor.col = col;
    }

    pub fn insert_str(&mut self, text: &str) {
        self.insert_text(text);
    }

    pub fn handle_key_event(&mut self, event: KeyEvent) {
        self.handler
            .on_event(CrosstermEvent::Key(event), &mut self.state);
    }

    #[must_use]
    pub fn view<'a>(&'a mut self) -> EditorView<'a, 'static> {
        let theme = EditorTheme::default()
            .line_numbers_style(Style::default().fg(Color::DarkGray))
            .base(Style::default().fg(Color::White))
            .cursor_style(Style::default().bg(Color::White).fg(Color::Black));
        EditorView::new(&mut self.state)
            .theme(theme)
            .line_numbers(self.line_numbers)
            .wrap(self.wrap)
    }

    #[allow(dead_code)]
    pub fn set_wrap(&mut self, wrap: bool) {
        self.wrap = wrap;
    }
}

impl Default for MainEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for MainEditor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MainEditor")
            .field("line_count", &self.line_count())
            .field("cursor", &self.cursor())
            .field("wrap", &self.wrap)
            .finish()
    }
}
