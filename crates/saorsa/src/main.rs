//! Saorsa - Unified TUI Workstation
//!
//! A tabbed terminal interface combining file browser, disk analyzer, and more.

mod plugins_tab;

use clap::Parser;
use color_eyre::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use parking_lot::Mutex;
use plugins_tab::PluginsTab;
use ratatui::prelude::*;
use saorsa_cli_core::{AppCoordinator, Message, PluginManager};
use saorsa_disk::DiskTab;
use saorsa_git::GitTab;
use saorsa_sb::SbTab;
use saorsa_ui::App;
use std::io::stdout;
use std::path::PathBuf;
use std::sync::Arc;

/// Unified TUI workstation combining file browser, disk analyzer, and more
#[derive(Parser)]
#[command(name = "saorsa")]
#[command(version, about, long_about = None)]
struct Cli {
    /// Starting directory
    #[arg(default_value = ".")]
    path: PathBuf,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    // Resolve path to absolute
    let root = cli
        .path
        .canonicalize()
        .unwrap_or_else(|_| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")));

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app with tabs
    let mut app = App::new();

    // Add Files tab (sb)
    match SbTab::new(1, &root) {
        Ok(files_tab) => {
            app.add_tab(Box::new(files_tab));
        }
        Err(e) => {
            // Clean up terminal before printing error
            disable_raw_mode()?;
            execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
            terminal.show_cursor()?;
            return Err(color_eyre::eyre::eyre!(
                "Failed to initialize file browser: {}",
                e
            ));
        }
    }

    // Add Disk tab
    let disk_tab = DiskTab::new(2, &root);
    app.add_tab(Box::new(disk_tab));

    // Add Git tab
    let git_tab = GitTab::new(3, &root);
    app.add_tab(Box::new(git_tab));

    // Plugins tab shares manager with CLI
    let plugin_manager = Arc::new(Mutex::new(PluginManager::default()));
    if let Err(e) = plugin_manager.lock().load() {
        eprintln!("Failed to load plugins: {e}");
    }
    let plugins_tab = PluginsTab::new(4, plugin_manager.clone());
    app.add_tab(Box::new(plugins_tab));

    // Set initial status
    app.set_status_left("NORMAL");
    app.set_status_center(root.display().to_string());
    app.set_status_right("Tab:switch  Ctrl+Q:quit");

    // Main loop
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app<B>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()>
where
    B: Backend,
    <B as Backend>::Error: std::error::Error + Send + Sync + 'static,
{
    loop {
        // Render
        terminal.draw(|frame| app.render(frame))?;

        // Handle events
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    // Global shortcuts first
                    match (key.modifiers, key.code) {
                        (KeyModifiers::CONTROL, KeyCode::Char('c' | 'q')) => {
                            app.dispatch(Message::Quit);
                        }
                        (KeyModifiers::NONE, KeyCode::Tab) => {
                            app.dispatch(Message::NextTab);
                        }
                        (KeyModifiers::SHIFT, KeyCode::BackTab) => {
                            app.dispatch(Message::PrevTab);
                        }
                        (KeyModifiers::ALT, KeyCode::Char(c)) if c.is_ascii_digit() => {
                            // Alt+1-9 to switch tabs
                            let idx = c.to_digit(10).unwrap_or(1);
                            app.dispatch(Message::SwitchTab(idx));
                        }
                        _ => {
                            // Forward to active tab
                            app.dispatch(Message::Key(key));
                        }
                    }
                }
                Event::Mouse(mouse) => {
                    app.dispatch(Message::Mouse(mouse));
                }
                Event::Resize(w, h) => {
                    app.dispatch(Message::Resize(w, h));
                }
                _ => {}
            }
        }

        // Check quit
        if app.should_quit() {
            break;
        }

        // Tick for animations/updates
        app.tick();
    }

    Ok(())
}
