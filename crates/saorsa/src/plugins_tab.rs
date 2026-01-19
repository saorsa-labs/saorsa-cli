use crossterm::event::KeyCode;
use parking_lot::Mutex;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use saorsa_cli_core::{
    CoreResult, Message, PluginContext, PluginDescriptor, PluginHistory, PluginManager,
    PluginRunStats, Tab, TabId,
};
use std::fmt::Write;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;

pub struct PluginsTab {
    id: TabId,
    title: String,
    manager: Arc<Mutex<PluginManager>>,
    state: ListState,
    status: Option<String>,
    sender: Sender<PluginJobMessage>,
    receiver: Receiver<PluginJobMessage>,
    running: Option<String>,
    history: Arc<Mutex<PluginHistory>>,
    info_panel: Option<InfoPanel>,
}

enum PluginJobMessage {
    Finished {
        name: String,
        result: CoreResult<()>,
    },
}

#[derive(Debug, Clone)]
enum InfoPanel {
    Help(String),
    Details(String),
    Directories(String),
}

impl InfoPanel {
    fn title(&self) -> &'static str {
        match self {
            InfoPanel::Help(_) => "Plugin Help",
            InfoPanel::Details(_) => "Plugin Details",
            InfoPanel::Directories(_) => "Plugin Directories",
        }
    }

    fn content(&self) -> &str {
        match self {
            InfoPanel::Help(text) | InfoPanel::Details(text) | InfoPanel::Directories(text) => text,
        }
    }
}

impl PluginsTab {
    pub fn new(id: TabId, manager: Arc<Mutex<PluginManager>>) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        let (sender, receiver) = mpsc::channel();
        Self {
            id,
            title: "Plugins".to_string(),
            manager,
            state,
            status: None,
            sender,
            receiver,
            running: None,
            history: Arc::new(Mutex::new(PluginHistory::load())),
            info_panel: None,
        }
    }

    fn descriptors(&self) -> Vec<PluginDescriptor> {
        self.manager.lock().descriptors()
    }

    fn run_selected(&mut self) {
        if self.running.is_some() {
            self.status = Some("A plugin is already running...".into());
            return;
        }
        let idx = match self.state.selected() {
            Some(idx) => idx,
            None => return,
        };
        let plugins = self.descriptors();
        let plugin = match plugins.get(idx) {
            Some(p) => p.clone(),
            None => return,
        };
        let name = plugin.metadata.name.clone();
        self.running = Some(name.clone());
        self.status = Some(format!("Running {name}..."));

        let sender = self.sender.clone();
        let manager = self.manager.clone();
        thread::spawn(move || {
            let result = manager
                .lock()
                .execute_plugin(&name, &[], PluginContext::default());
            let _ = sender.send(PluginJobMessage::Finished { name, result });
        });
    }

    fn refresh(&mut self) {
        if self.running.is_some() {
            self.status = Some("Cannot refresh while a plugin is running".into());
            return;
        }
        match self.manager.lock().load() {
            Ok(count) => {
                self.status = Some(format!("Loaded {count} plugins"));
                if count == 0 {
                    self.state.select(None);
                } else {
                    self.state.select(Some(0));
                }
            }
            Err(e) => {
                self.status = Some(format!("Reload failed: {e}"));
            }
        }
    }

    fn selected_plugin(&self) -> Option<PluginDescriptor> {
        self.state
            .selected()
            .and_then(|idx| self.descriptors().get(idx).cloned())
    }

    fn show_help_panel(&mut self) {
        let plugin = match self.selected_plugin() {
            Some(plugin) => plugin,
            None => {
                self.status = Some("Select a plugin to view help".into());
                return;
            }
        };
        let name = plugin.metadata.name.clone();
        let help_text = {
            let manager = self.manager.lock();
            manager.help_for(&name)
        }
        .or(plugin.metadata.help.clone());

        if let Some(text) = help_text {
            self.info_panel = Some(InfoPanel::Help(text));
            self.status = Some(format!("Showing help for {}", name));
        } else {
            self.status = Some(format!("No help available for {}", name));
        }
    }

    fn show_details_panel(&mut self) {
        let plugin = match self.selected_plugin() {
            Some(plugin) => plugin,
            None => {
                self.status = Some("Select a plugin to view details".into());
                return;
            }
        };
        let mut content = String::new();
        let _ = writeln!(content, "Name: {}", plugin.metadata.name);
        let _ = writeln!(content, "Version: {}", plugin.metadata.version);
        let _ = writeln!(content, "Author: {}", plugin.metadata.author);
        let _ = writeln!(content, "Description: {}", plugin.metadata.description);
        if let Some(help) = &plugin.metadata.help {
            let _ = writeln!(content, "Help: {help}");
        }
        if let Some(manifest) = plugin.metadata.manifest_path.to_str() {
            let _ = writeln!(content, "Manifest: {manifest}");
        }
        if let Some(library) = plugin.metadata.library_path.to_str() {
            let _ = writeln!(content, "Library: {library}");
        }
        let stats_summary = {
            let history = self.history.lock();
            format_stats(history.stats_for(&plugin.metadata.name))
        };
        let _ = writeln!(content, "Stats: {stats_summary}");
        self.info_panel = Some(InfoPanel::Details(content));
        self.status = Some(format!("Showing details for {}", plugin.metadata.name));
    }

    fn show_directories_panel(&mut self) {
        let dirs = self.manager.lock().search_paths().to_owned();
        if dirs.is_empty() {
            self.status = Some("No plugin directories configured.".into());
            self.info_panel = Some(InfoPanel::Directories(
                "No plugin directories configured.".into(),
            ));
            return;
        }
        let mut content = String::new();
        for (idx, path) in dirs.iter().enumerate() {
            let _ = writeln!(content, "{}. {}", idx + 1, path.display());
        }
        self.info_panel = Some(InfoPanel::Directories(content));
        self.status = Some("Listing plugin directories".into());
    }

    fn clear_info_panel(&mut self) {
        self.info_panel = None;
    }
}

impl Tab for PluginsTab {
    fn id(&self) -> TabId {
        self.id
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn icon(&self) -> Option<&str> {
        Some("🔌")
    }

    fn can_close(&self) -> bool {
        false
    }

    fn focus(&mut self) {}

    fn blur(&mut self) {}

    fn view(&self, frame: &mut Frame, area: Rect) {
        let has_panel = self.info_panel.is_some();
        let mut constraints = vec![Constraint::Min(3)];
        if has_panel {
            constraints.push(Constraint::Length(5));
        }
        constraints.push(Constraint::Length(2));
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        let plugins = self.descriptors();
        let history = self.history.lock();
        let items: Vec<ListItem> = if plugins.is_empty() {
            vec![ListItem::new("No plugins found")]
        } else {
            plugins
                .iter()
                .map(|plugin| {
                    let stats = format_stats(history.stats_for(&plugin.metadata.name));
                    ListItem::new(format!(
                        "{} v{} — {}{}",
                        plugin.metadata.name,
                        plugin.metadata.version,
                        plugin.metadata.description,
                        stats
                    ))
                })
                .collect()
        };

        let mut state = self.state.clone();
        if plugins.is_empty() {
            state.select(None);
        }

        let list = List::new(items).block(
            Block::default()
                .title(" Plugins ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );
        frame.render_stateful_widget(list, chunks[0], &mut state);

        if let (true, Some(panel)) = (has_panel, self.info_panel.as_ref()) {
            let panel_area = chunks[1];
            self.draw_info_panel(frame, panel_area, panel);
        }

        let mut status_lines = vec![
            format!(
                "↑/↓ navigate  Enter run  r reload   Running: {}",
                self.running
                    .as_deref()
                    .unwrap_or("none (plugins run with full trust!)")
            ),
            "⚠️  Plugins have full access to your system—only load trusted code.".to_string(),
        ];
        status_lines.push("Keys: h/? help  i info  d dirs  Esc closes panel".to_string());
        if let Some(status) = &self.status {
            status_lines.push(format!("Status: {}", status));
        }

        let status = Paragraph::new(status_lines.join("\n"))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::TOP));
        let status_area = if has_panel {
            *chunks.last().unwrap()
        } else {
            chunks[1]
        };
        frame.render_widget(status, status_area);
    }

    fn draw_info_panel(&self, frame: &mut Frame, area: Rect, panel: &InfoPanel) {
        let block = Block::default()
            .title(panel.title())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        let paragraph = Paragraph::new(panel.content())
            .wrap(Wrap { trim: true })
            .block(block);
        frame.render_widget(paragraph, area);
    }

    fn handle_message(&mut self, message: &Message) -> Option<Message> {
        if let Message::Key(key) = message {
            match key.code {
                KeyCode::Up => {
                    let current = self.state.selected().unwrap_or(0);
                    let new_sel = current.saturating_sub(1);
                    self.state.select(Some(new_sel));
                }
                KeyCode::Down => {
                    let plugins_len = self.descriptors().len();
                    if plugins_len > 0 {
                        let current = self.state.selected().unwrap_or(0);
                        let max_index = plugins_len.saturating_sub(1);
                        let next = if current >= max_index {
                            max_index
                        } else {
                            current + 1
                        };
                        self.state.select(Some(next));
                    }
                }
                KeyCode::Enter => {
                    self.run_selected();
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    self.refresh();
                }
                KeyCode::Char('h') | KeyCode::Char('H') | KeyCode::Char('?') => {
                    self.show_help_panel();
                }
                KeyCode::Char('i') | KeyCode::Char('I') => {
                    self.show_details_panel();
                }
                KeyCode::Char('d') | KeyCode::Char('D') => {
                    self.show_directories_panel();
                }
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    self.clear_info_panel();
                    self.status = Some("Closed info panel".into());
                }
                KeyCode::Esc => {
                    if self.info_panel.is_some() {
                        self.clear_info_panel();
                        self.status = Some("Closed info panel".into());
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn tick(&mut self) {
        while let Ok(msg) = self.receiver.try_recv() {
            match msg {
                PluginJobMessage::Finished { name, result } => {
                    self.running = None;
                    match result {
                        Ok(_) => {
                            self.status = Some(format!("✅ {name} completed"));
                            if let Err(e) = self.history.lock().record_success(&name) {
                                eprintln!("Failed to record plugin success: {}", e);
                            }
                        }
                        Err(e) => {
                            if let Err(io_err) = self
                                .history
                                .lock()
                                .record_failure(&name, Some(e.to_string()))
                            {
                                eprintln!("Failed to record plugin failure: {}", io_err);
                            }
                            self.status = Some(format!("❌ {name} failed: {e}"));
                        }
                    }
                }
            }
        }
    }
}

fn format_stats(stats: Option<&PluginRunStats>) -> String {
    match stats {
        Some(stats) => {
            let when = stats
                .last_run
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| "never".to_string());
            format!(" (✓{} ✗{} | last {when})", stats.successes, stats.failures)
        }
        None => " (no runs yet)".to_string(),
    }
}
