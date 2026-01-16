use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum MenuChoice {
    RunSB,
    RunSDisk,
    UpdateBinaries,
    UpdateCLI,
    Settings,
    Plugins,
    Exit,
}

pub struct Menu {
    state: ListState,
    items: Vec<(String, MenuChoice)>,
    sb_path: Option<PathBuf>,
    sdisk_path: Option<PathBuf>,
    update_available: Option<String>,
}

impl Menu {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        let mut menu = Self {
            state,
            items: Vec::new(),
            sb_path: None,
            sdisk_path: None,
            update_available: None,
        };
        menu.rebuild_items();
        menu
    }

    /// Set update status and rebuild menu items.
    pub fn set_update_status(&mut self, latest_version: Option<String>) {
        self.update_available = latest_version;
        self.rebuild_items();
    }

    /// Rebuild menu items based on current state.
    fn rebuild_items(&mut self) {
        self.items = vec![
            ("📚 Run Saorsa Browser (sb)".to_string(), MenuChoice::RunSB),
            (
                "💾 Run Saorsa Disk (sdisk)".to_string(),
                MenuChoice::RunSDisk,
            ),
            ("🔄 Update Binaries".to_string(), MenuChoice::UpdateBinaries),
        ];

        // Add update option if available
        if let Some(version) = &self.update_available {
            self.items.push((
                format!("⬆️  Update CLI to v{}", version),
                MenuChoice::UpdateCLI,
            ));
        }

        self.items.extend([
            ("⚙️  Settings".to_string(), MenuChoice::Settings),
            ("🔌 Plugins".to_string(), MenuChoice::Plugins),
            ("🚪 Exit".to_string(), MenuChoice::Exit),
        ]);
    }

    pub fn set_binary_paths(&mut self, sb_path: Option<PathBuf>, sdisk_path: Option<PathBuf>) {
        self.sb_path = sb_path;
        self.sdisk_path = sdisk_path;
    }

    pub async fn run(&mut self) -> Result<MenuChoice> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_loop(&mut terminal).await;

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        result
    }

    async fn run_loop<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<MenuChoice> {
        loop {
            terminal.draw(|f| self.draw(f))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => self.previous(),
                        KeyCode::Down | KeyCode::Char('j') => self.next(),
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            if let Some(selected) = self.state.selected() {
                                return Ok(self.items[selected].1.clone());
                            }
                        }
                        KeyCode::Char('q') | KeyCode::Esc => {
                            return Ok(MenuChoice::Exit);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn draw(&mut self, f: &mut Frame) {
        // Adjust header height to accommodate update notification
        let header_height = if self.update_available.is_some() {
            4
        } else {
            3
        };
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(header_height),
                Constraint::Min(10),
                Constraint::Length(4),
            ])
            .split(f.area());

        self.draw_header(f, chunks[0]);
        self.draw_menu(f, chunks[1]);
        self.draw_footer(f, chunks[2]);
    }

    fn draw_header(&self, f: &mut Frame, area: Rect) {
        let mut lines = vec![
            Line::from(vec![Span::styled(
                "Saorsa CLI",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::raw("Interactive menu for Saorsa tools")]),
        ];

        // Add update notification if available
        if let Some(version) = &self.update_available {
            lines.push(Line::from(vec![Span::styled(
                format!("Update available: v{}", version),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]));
        }

        let header = Paragraph::new(lines)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::BOTTOM));

        f.render_widget(header, area);
    }

    fn draw_menu(&mut self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, (label, choice))| {
                let mut style = Style::default();
                let mut suffix = String::new();

                // Add status indicators
                match choice {
                    MenuChoice::RunSB => {
                        if self.sb_path.is_none() {
                            style = style.fg(Color::DarkGray);
                            suffix = " (not installed)".to_string();
                        } else {
                            style = style.fg(Color::Green);
                        }
                    }
                    MenuChoice::RunSDisk => {
                        if self.sdisk_path.is_none() {
                            style = style.fg(Color::DarkGray);
                            suffix = " (not installed)".to_string();
                        } else {
                            style = style.fg(Color::Green);
                        }
                    }
                    MenuChoice::UpdateCLI => {
                        style = style.fg(Color::Yellow).add_modifier(Modifier::BOLD);
                    }
                    _ => {}
                }

                // Highlight selected item
                if Some(i) == self.state.selected() {
                    style = style.add_modifier(Modifier::REVERSED);
                }

                ListItem::new(format!("{}{}", label, suffix)).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(" Menu ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(Style::default())
            .highlight_symbol("> ");

        f.render_stateful_widget(list, area, &mut self.state);
    }

    fn draw_footer(&self, f: &mut Frame, area: Rect) {
        let footer = Paragraph::new(vec![Line::from(vec![
            Span::styled("Navigation: ", Style::default().fg(Color::DarkGray)),
            Span::styled("↑↓/jk", Style::default().fg(Color::Cyan)),
            Span::styled(" | Select: ", Style::default().fg(Color::DarkGray)),
            Span::styled("Enter/Space", Style::default().fg(Color::Cyan)),
            Span::styled(" | Quit: ", Style::default().fg(Color::DarkGray)),
            Span::styled("q/Esc", Style::default().fg(Color::Cyan)),
        ])])
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));

        f.render_widget(footer, area);
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}
