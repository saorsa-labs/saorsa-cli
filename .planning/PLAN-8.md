# Phase Plan: M1P1C - Integrate Existing Tools as Tabs

**Milestone**: M1 - Foundation & Tab Infrastructure
**Phase**: 1C - Integrate Existing Tools as Tabs
**Created**: 2026-01-17
**Status**: Ready for execution

## Overview

Wrap the existing `sb` (markdown browser) and create a new disk analyzer tab to integrate with the saorsa-ui framework. The `sb` tool already uses ratatui and can be wrapped directly. The `sdisk` tool is CLI-based and needs a new ratatui interface.

## Prerequisites

- [x] saorsa-cli-core crate complete (Phase 6)
- [x] saorsa-ui crate complete (Phase 7)
- [ ] Ensure workspace builds: `cargo build --workspace`

## Architecture Analysis

### SB (Markdown Browser)
- **Status**: Full ratatui TUI app (~1900 lines in app.rs)
- **Integration**: Wrap existing `App` struct as `SbTab`
- **Complexity**: Medium - extract library, add Tab trait

### SDISK (Disk Analyzer)
- **Status**: CLI tool using println! and dialoguer (no ratatui)
- **Integration**: Create new `DiskTab` with ratatui rendering
- **Complexity**: High - needs full TUI rewrite

## New Structure

```
crates/
├── saorsa-cli-core/     # Core types (done)
├── saorsa-ui/           # UI framework (done)
├── saorsa-sb/           # ← NEW: sb as library crate
│   └── src/
│       ├── lib.rs       # Library exports
│       ├── app.rs       # Existing app logic
│       └── tab.rs       # SbTab implementation
└── saorsa-disk/         # ← NEW: Disk analyzer tab
    └── src/
        ├── lib.rs       # Library exports
        ├── analyzer.rs  # Disk analysis logic
        └── tab.rs       # DiskTab implementation
```

---

## Tasks

<task type="auto" priority="p0">
  <n>Create saorsa-sb library crate with SbTab</n>
  <files>
    Cargo.toml,
    crates/saorsa-sb/Cargo.toml,
    crates/saorsa-sb/src/lib.rs,
    crates/saorsa-sb/src/tab.rs,
    sb/src/app.rs
  </files>
  <action>
    1. Create crates/saorsa-sb/ directory structure

    2. Create crates/saorsa-sb/Cargo.toml:
       ```toml
       [package]
       name = "saorsa-sb"
       version = "0.1.0"
       edition.workspace = true
       authors.workspace = true
       license.workspace = true
       repository.workspace = true
       description = "Markdown browser tab for saorsa TUI"

       [dependencies]
       saorsa-cli-core = { path = "../saorsa-cli-core" }
       ratatui = "0.29"
       crossterm = "0.29"

       # Dependencies from sb/
       tui-textarea = "0.7"
       tui-tree-widget = "0.22"
       tui-markdown = "0.3"
       syntect = "5"
       git2 = "0.20"
       image = "0.25"
       walkdir = "2"
       humansize = "2"
       dirs = "5"
       open = "5"
       chrono = "0.4"

       [dev-dependencies]
       tempfile = "3"
       ```

    3. Copy sb/src/app.rs to crates/saorsa-sb/src/app.rs:
       - Keep all existing App struct and methods
       - Make App::new() take a root path parameter
       - Ensure ui() method is public
       - Remove any main.rs-specific dependencies

    4. Copy supporting modules from sb/src/:
       - preview.rs (markdown preview rendering)
       - git.rs (git integration)
       - video.rs (video player if exists)
       - Any other required modules

    5. Create crates/saorsa-sb/src/tab.rs with SbTab:
       ```rust
       //! SbTab - Markdown browser as a Tab

       use crate::app::App;
       use saorsa_cli_core::{Message, Tab, TabId};
       use ratatui::prelude::*;
       use crossterm::event::{KeyEvent, MouseEvent};

       /// Markdown browser tab wrapping the sb App
       pub struct SbTab {
           id: TabId,
           title: String,
           app: App,
           focused: bool,
       }

       impl SbTab {
           /// Creates a new SbTab for the given directory
           pub fn new(id: TabId, root: impl Into<std::path::PathBuf>) -> Self {
               Self {
                   id,
                   title: "Files".to_string(),
                   app: App::new(root.into()),
                   focused: false,
               }
           }

           /// Handle a key event
           pub fn handle_key(&mut self, key: KeyEvent) -> Option<Message> {
               // Route key events to App
               self.app.handle_key(key);
               None
           }

           /// Handle a mouse event
           pub fn handle_mouse(&mut self, mouse: MouseEvent) -> Option<Message> {
               // Route mouse events to App
               self.app.handle_mouse(mouse);
               None
           }
       }

       impl Tab for SbTab {
           fn id(&self) -> TabId { self.id }
           fn title(&self) -> &str { &self.title }
           fn icon(&self) -> Option<&str> { Some("📁") }
           fn can_close(&self) -> bool { true }

           fn focus(&mut self) {
               self.focused = true;
               // Resume any paused operations
           }

           fn blur(&mut self) {
               self.focused = false;
               // Pause video if playing, save any pending state
               self.app.pause_video();
           }

           fn view(&self, frame: &mut Frame, area: Rect) {
               // Delegate to App's ui function
               self.app.ui(frame, area);
           }
       }
       ```

    6. Create crates/saorsa-sb/src/lib.rs:
       ```rust
       //! saorsa-sb - Markdown browser library
       //!
       //! Provides the SbTab for integration with saorsa TUI.

       mod app;
       mod preview;
       mod git;
       // ... other modules

       mod tab;

       pub use app::App;
       pub use tab::SbTab;
       ```

    7. Update App in app.rs:
       - Add `pub fn ui(&self, frame: &mut Frame, area: Rect)` that takes area parameter
       - Add `pub fn handle_key(&mut self, key: KeyEvent)`
       - Add `pub fn handle_mouse(&mut self, mouse: MouseEvent)`
       - Add `pub fn pause_video(&mut self)` if video support exists
       - Ensure all rendering respects the provided `area` not terminal size

    8. Update workspace Cargo.toml to add "crates/saorsa-sb"

    9. Write tests:
       - SbTab creation
       - Tab trait implementation
       - Focus/blur lifecycle
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p saorsa-sb --all-features -- -D warnings
    cargo test -p saorsa-sb
    cargo doc -p saorsa-sb --no-deps
  </verify>
  <done>
    - saorsa-sb crate compiles
    - SbTab implements Tab trait correctly
    - App renders within provided area (not full terminal)
    - Focus/blur pauses video if applicable
    - Key and mouse events routed to App
    - Zero clippy warnings
  </done>
</task>

<task type="auto" priority="p0">
  <n>Create saorsa-disk crate with DiskTab</n>
  <files>
    Cargo.toml,
    crates/saorsa-disk/Cargo.toml,
    crates/saorsa-disk/src/lib.rs,
    crates/saorsa-disk/src/analyzer.rs,
    crates/saorsa-disk/src/tab.rs
  </files>
  <action>
    1. Create crates/saorsa-disk/ directory structure

    2. Create crates/saorsa-disk/Cargo.toml:
       ```toml
       [package]
       name = "saorsa-disk"
       version = "0.1.0"
       edition.workspace = true
       authors.workspace = true
       license.workspace = true
       repository.workspace = true
       description = "Disk analyzer tab for saorsa TUI"

       [dependencies]
       saorsa-cli-core = { path = "../saorsa-cli-core" }
       ratatui = "0.29"
       crossterm = "0.29"

       # Analysis dependencies (from sdisk)
       walkdir = "2"
       humansize = "2"
       sysinfo = "0.33"
       chrono = "0.4"

       [dev-dependencies]
       tempfile = "3"
       ```

    3. Create crates/saorsa-disk/src/analyzer.rs:
       ```rust
       //! Disk analysis functionality

       use std::path::{Path, PathBuf};
       use std::time::SystemTime;
       use walkdir::WalkDir;
       use humansize::{format_size, BINARY};

       /// A file entry with size and metadata
       #[derive(Debug, Clone)]
       pub struct FileEntry {
           pub path: PathBuf,
           pub size: u64,
           pub modified: Option<SystemTime>,
           pub accessed: Option<SystemTime>,
       }

       /// Disk overview for a mount point
       #[derive(Debug, Clone)]
       pub struct DiskInfo {
           pub mount_point: PathBuf,
           pub total: u64,
           pub used: u64,
           pub available: u64,
       }

       impl DiskInfo {
           pub fn usage_percent(&self) -> f64 {
               (self.used as f64 / self.total as f64) * 100.0
           }

           pub fn format_size(bytes: u64) -> String {
               format_size(bytes, BINARY)
           }
       }

       /// Analyzes disk usage
       pub struct DiskAnalyzer {
           root: PathBuf,
       }

       impl DiskAnalyzer {
           pub fn new(root: impl Into<PathBuf>) -> Self {
               Self { root: root.into() }
           }

           /// Get disk info for all mount points
           pub fn get_disk_info() -> Vec<DiskInfo> {
               use sysinfo::Disks;
               let disks = Disks::new_with_refreshed_list();
               disks.iter().map(|d| DiskInfo {
                   mount_point: d.mount_point().to_path_buf(),
                   total: d.total_space(),
                   used: d.total_space() - d.available_space(),
                   available: d.available_space(),
               }).collect()
           }

           /// Find the N largest files
           pub fn find_largest(&self, count: usize) -> Vec<FileEntry> {
               let mut entries = Vec::new();
               for entry in WalkDir::new(&self.root).into_iter().filter_map(|e| e.ok()) {
                   if entry.file_type().is_file() {
                       if let Ok(meta) = entry.metadata() {
                           entries.push(FileEntry {
                               path: entry.path().to_path_buf(),
                               size: meta.len(),
                               modified: meta.modified().ok(),
                               accessed: meta.accessed().ok(),
                           });
                       }
                   }
               }
               entries.sort_by(|a, b| b.size.cmp(&a.size));
               entries.truncate(count);
               entries
           }

           /// Find files older than N days (by access time)
           pub fn find_stale(&self, days: u64, count: usize) -> Vec<FileEntry> {
               let cutoff = SystemTime::now() - std::time::Duration::from_secs(days * 24 * 60 * 60);
               let mut entries = Vec::new();
               for entry in WalkDir::new(&self.root).into_iter().filter_map(|e| e.ok()) {
                   if entry.file_type().is_file() {
                       if let Ok(meta) = entry.metadata() {
                           if let Ok(accessed) = meta.accessed() {
                               if accessed < cutoff {
                                   entries.push(FileEntry {
                                       path: entry.path().to_path_buf(),
                                       size: meta.len(),
                                       modified: meta.modified().ok(),
                                       accessed: Some(accessed),
                                   });
                               }
                           }
                       }
                   }
               }
               entries.sort_by(|a, b| a.accessed.cmp(&b.accessed));
               entries.truncate(count);
               entries
           }
       }
       ```

    4. Create crates/saorsa-disk/src/tab.rs:
       ```rust
       //! DiskTab - Disk analyzer as a Tab

       use crate::analyzer::{DiskAnalyzer, DiskInfo, FileEntry};
       use saorsa_cli_core::{Message, Tab, TabId, Theme};
       use ratatui::prelude::*;
       use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph};
       use crossterm::event::{KeyCode, KeyEvent};
       use std::path::PathBuf;

       /// View mode for the disk tab
       #[derive(Debug, Clone, Copy, PartialEq, Eq)]
       pub enum DiskView {
           Overview,  // Disk usage bars
           Largest,   // Largest files
           Stale,     // Stale files
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

           /// Analyze largest files
           pub fn analyze_largest(&mut self, count: usize) {
               let analyzer = DiskAnalyzer::new(&self.root);
               self.largest_files = analyzer.find_largest(count);
               self.view = DiskView::Largest;
               self.list_state.select(Some(0));
           }

           /// Analyze stale files
           pub fn analyze_stale(&mut self, count: usize) {
               let analyzer = DiskAnalyzer::new(&self.root);
               self.stale_files = analyzer.find_stale(self.stale_days, count);
               self.view = DiskView::Stale;
               self.list_state.select(Some(0));
           }

           /// Handle key input
           pub fn handle_key(&mut self, key: KeyEvent) -> Option<Message> {
               match key.code {
                   KeyCode::Char('o') => {
                       self.view = DiskView::Overview;
                       self.refresh();
                   }
                   KeyCode::Char('l') => self.analyze_largest(50),
                   KeyCode::Char('s') => self.analyze_stale(50),
                   KeyCode::Char('r') => self.refresh(),
                   KeyCode::Up | KeyCode::Char('k') => {
                       let i = self.list_state.selected().unwrap_or(0);
                       self.list_state.select(Some(i.saturating_sub(1)));
                   }
                   KeyCode::Down | KeyCode::Char('j') => {
                       let i = self.list_state.selected().unwrap_or(0);
                       let len = match self.view {
                           DiskView::Largest => self.largest_files.len(),
                           DiskView::Stale => self.stale_files.len(),
                           _ => 0,
                       };
                       if len > 0 {
                           self.list_state.select(Some((i + 1).min(len - 1)));
                       }
                   }
                   _ => {}
               }
               None
           }

           fn render_overview(&self, frame: &mut Frame, area: Rect) {
               // Render disk usage gauges
               let chunks = Layout::default()
                   .direction(Direction::Vertical)
                   .constraints(self.disk_info.iter().map(|_| Constraint::Length(3)).collect::<Vec<_>>())
                   .split(area);

               for (i, info) in self.disk_info.iter().enumerate() {
                   if i >= chunks.len() { break; }
                   let gauge = Gauge::default()
                       .block(Block::default()
                           .title(format!(" {} ", info.mount_point.display()))
                           .borders(Borders::ALL))
                       .gauge_style(Style::default().fg(Color::Cyan))
                       .percent(info.usage_percent() as u16)
                       .label(format!(
                           "{} / {} ({:.1}%)",
                           DiskInfo::format_size(info.used),
                           DiskInfo::format_size(info.total),
                           info.usage_percent()
                       ));
                   frame.render_widget(gauge, chunks[i]);
               }
           }

           fn render_file_list(&self, frame: &mut Frame, area: Rect, files: &[FileEntry], title: &str) {
               let items: Vec<ListItem> = files.iter().map(|f| {
                   ListItem::new(format!(
                       "{:>10}  {}",
                       DiskInfo::format_size(f.size),
                       f.path.display()
                   ))
               }).collect();

               let list = List::new(items)
                   .block(Block::default()
                       .title(format!(" {} ", title))
                       .borders(Borders::ALL))
                   .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                   .highlight_symbol("> ");

               frame.render_stateful_widget(list, area, &mut self.list_state.clone());
           }
       }

       impl Tab for DiskTab {
           fn id(&self) -> TabId { self.id }
           fn title(&self) -> &str { "Disk" }
           fn icon(&self) -> Option<&str> { Some("💾") }
           fn can_close(&self) -> bool { true }

           fn focus(&mut self) {
               self.focused = true;
               self.refresh();
           }

           fn blur(&mut self) {
               self.focused = false;
           }

           fn view(&self, frame: &mut Frame, area: Rect) {
               // Help line at bottom
               let chunks = Layout::default()
                   .direction(Direction::Vertical)
                   .constraints([Constraint::Min(0), Constraint::Length(1)])
                   .split(area);

               match self.view {
                   DiskView::Overview => self.render_overview(frame, chunks[0]),
                   DiskView::Largest => self.render_file_list(frame, chunks[0], &self.largest_files, "Largest Files"),
                   DiskView::Stale => self.render_file_list(frame, chunks[0], &self.stale_files, "Stale Files"),
               }

               // Help line
               let help = Paragraph::new(" [o]verview  [l]argest  [s]tale  [r]efresh  [j/k] navigate")
                   .style(Style::default().fg(Color::DarkGray));
               frame.render_widget(help, chunks[1]);
           }
       }
       ```

    5. Create crates/saorsa-disk/src/lib.rs:
       ```rust
       //! saorsa-disk - Disk analyzer library
       //!
       //! Provides the DiskTab for integration with saorsa TUI.

       pub mod analyzer;
       mod tab;

       pub use analyzer::{DiskAnalyzer, DiskInfo, FileEntry};
       pub use tab::{DiskTab, DiskView};
       ```

    6. Update workspace Cargo.toml to add "crates/saorsa-disk"

    7. Write tests:
       - DiskAnalyzer functions
       - DiskTab creation and Tab trait
       - View switching
       - Key handling
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p saorsa-disk --all-features -- -D warnings
    cargo test -p saorsa-disk
    cargo doc -p saorsa-disk --no-deps
  </verify>
  <done>
    - saorsa-disk crate compiles
    - DiskTab implements Tab trait correctly
    - Analyzer finds largest and stale files
    - Disk overview shows usage gauges
    - File list supports navigation
    - Zero clippy warnings
  </done>
</task>

<task type="auto" priority="p0">
  <n>Create unified saorsa binary with both tabs</n>
  <files>
    crates/saorsa/Cargo.toml,
    crates/saorsa/src/main.rs,
    Cargo.toml
  </files>
  <action>
    1. Create crates/saorsa/ directory structure

    2. Create crates/saorsa/Cargo.toml:
       ```toml
       [package]
       name = "saorsa"
       version = "0.1.0"
       edition.workspace = true
       authors.workspace = true
       license.workspace = true
       repository.workspace = true
       description = "Unified TUI workstation"

       [[bin]]
       name = "saorsa"
       path = "src/main.rs"

       [dependencies]
       saorsa-cli-core = { path = "../saorsa-cli-core" }
       saorsa-ui = { path = "../saorsa-ui" }
       saorsa-sb = { path = "../saorsa-sb" }
       saorsa-disk = { path = "../saorsa-disk" }

       ratatui = "0.29"
       crossterm = "0.29"
       clap = { version = "4", features = ["derive"] }
       dirs = "5"
       color-eyre = "0.6"
       ```

    3. Create crates/saorsa/src/main.rs:
       ```rust
       //! Saorsa - Unified TUI Workstation
       //!
       //! A tabbed terminal interface combining file browser,
       //! disk analyzer, and more.

       use clap::Parser;
       use color_eyre::Result;
       use crossterm::{
           event::{self, Event, KeyCode, KeyModifiers},
           execute,
           terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
       };
       use ratatui::prelude::*;
       use saorsa_cli_core::Message;
       use saorsa_disk::DiskTab;
       use saorsa_sb::SbTab;
       use saorsa_ui::App;
       use std::io::{self, stdout};
       use std::path::PathBuf;

       #[derive(Parser)]
       #[command(name = "saorsa")]
       #[command(about = "Unified TUI workstation")]
       struct Cli {
           /// Starting directory
           #[arg(default_value = ".")]
           path: PathBuf,
       }

       fn main() -> Result<()> {
           color_eyre::install()?;
           let cli = Cli::parse();

           // Resolve path
           let root = cli.path.canonicalize().unwrap_or_else(|_| {
               dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
           });

           // Setup terminal
           enable_raw_mode()?;
           let mut stdout = stdout();
           execute!(stdout, EnterAlternateScreen)?;
           let backend = CrosstermBackend::new(stdout);
           let mut terminal = Terminal::new(backend)?;

           // Create app with tabs
           let mut app = App::new();

           // Add Files tab (sb)
           let files_tab = SbTab::new(1, &root);
           app.add_tab(Box::new(files_tab));

           // Add Disk tab
           let disk_tab = DiskTab::new(2, &root);
           app.add_tab(Box::new(disk_tab));

           // Set initial status
           app.set_status_left("NORMAL");
           app.set_status_center(root.display().to_string());
           app.set_status_right("? help");

           // Main loop
           let result = run_app(&mut terminal, &mut app);

           // Restore terminal
           disable_raw_mode()?;
           execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
           terminal.show_cursor()?;

           result
       }

       fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
           loop {
               // Render
               terminal.draw(|frame| app.render(frame))?;

               // Handle events
               if event::poll(std::time::Duration::from_millis(100))? {
                   match event::read()? {
                       Event::Key(key) => {
                           // Global shortcuts
                           match (key.modifiers, key.code) {
                               (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                                   app.dispatch(Message::Quit);
                               }
                               (KeyModifiers::CONTROL, KeyCode::Char('q')) => {
                                   app.dispatch(Message::Quit);
                               }
                               (_, KeyCode::Tab) => {
                                   app.dispatch(Message::NextTab);
                               }
                               (KeyModifiers::SHIFT, KeyCode::BackTab) => {
                                   app.dispatch(Message::PrevTab);
                               }
                               (KeyModifiers::ALT, KeyCode::Char(c)) if c.is_ascii_digit() => {
                                   // Alt+1-9 to switch tabs
                                   let idx = c.to_digit(10).unwrap_or(1) as u32;
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
       ```

    4. Update workspace Cargo.toml:
       - Add "crates/saorsa" to members
       - Add default-members = ["crates/saorsa"] for `cargo run`

    5. Test the unified binary:
       - `cargo run -p saorsa`
       - Verify tab switching works (Tab, Shift+Tab, Alt+1/2)
       - Verify both tabs render correctly
       - Verify Ctrl+Q quits
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p saorsa --all-features -- -D warnings
    cargo build -p saorsa --release
    cargo run -p saorsa -- --help
  </verify>
  <done>
    - saorsa binary compiles and runs
    - Both tabs display correctly
    - Tab switching works (Tab, Shift+Tab, Alt+1-9)
    - Global shortcuts work (Ctrl+C, Ctrl+Q)
    - Events routed to active tab
    - Clean exit restores terminal
    - Zero clippy warnings
  </done>
</task>

---

## Notes

### Design Decisions

1. **Separate crates per tool**: Keeps concerns isolated, allows independent testing and versioning.

2. **SbTab wraps existing App**: Minimizes changes to working code, preserves all sb features.

3. **DiskTab is new ratatui code**: sdisk was CLI-based, so we build fresh TUI from its analysis logic.

4. **Main binary in crates/saorsa**: Clear entry point, unified CLI experience.

### Dependencies Summary

```toml
# saorsa-sb inherits from sb
tui-textarea, tui-tree-widget, tui-markdown, syntect, git2, image

# saorsa-disk inherits from sdisk
walkdir, humansize, sysinfo

# saorsa main
clap, color-eyre, dirs
```

### Migration from Standalone Tools

After this phase:
- `sb` binary still works standalone (for users who want it)
- `sdisk` binary still works standalone
- `saorsa` binary provides unified experience

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Tab | Next tab |
| Shift+Tab | Previous tab |
| Alt+1-9 | Jump to tab N |
| Ctrl+C/Q | Quit |
| (in tab) | Tab-specific shortcuts |

---

## Exit Criteria

- [ ] `cargo build -p saorsa-sb` succeeds
- [ ] `cargo build -p saorsa-disk` succeeds
- [ ] `cargo build -p saorsa` succeeds
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace -- -D warnings` clean
- [ ] SbTab renders markdown browser correctly
- [ ] DiskTab shows disk usage and file lists
- [ ] Tab switching works in unified binary
- [ ] Global shortcuts work (Ctrl+Q quit, Tab switch)

## Next Phase

After completion, proceed to:
```
/gsd:plan-phase 9 "Git Tab Implementation"
```

This will add a gitui-like Git tab for staging, commits, and diffs.
