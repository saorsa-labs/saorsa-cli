# Phase Plan: M1P1B - Tab Manager & UI Framework

**Milestone**: M1 - Foundation & Tab Infrastructure
**Phase**: 1B - Tab Manager & UI Framework
**Created**: 2026-01-17
**Completed**: 2026-01-17
**Status**: ✓ COMPLETE

## Overview

Build the `saorsa-ui` crate with TabManager, rendering widgets, and the main App struct that implements AppCoordinator. This creates the actual tabbed interface infrastructure.

## Prerequisites

- [x] saorsa-cli-core crate complete (Phase 6)
- [x] Core types available: Tab, TabId, PaneLayout, Message, MessageBus, Theme
- [ ] Ensure workspace builds: `cargo build --workspace`

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  TabBar  [Files] [Git] [Search] [System] [+]               │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│                     Active Tab Content                      │
│                    (rendered by Tab::view)                  │
│                                                             │
│  ┌─────────────────────┬───────────────────────────────┐   │
│  │    Left Pane        │        Right Pane             │   │
│  │    (if split)       │        (if split)             │   │
│  └─────────────────────┴───────────────────────────────┘   │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│  StatusBar  [mode] [branch] [file]           [time] [help] │
└─────────────────────────────────────────────────────────────┘
```

## New Crate Structure

```
crates/saorsa-ui/
├── Cargo.toml
└── src/
    ├── lib.rs           # Crate root, exports
    ├── tab_manager.rs   # TabManager struct
    ├── widgets/
    │   ├── mod.rs
    │   ├── tab_bar.rs   # Tab bar widget
    │   └── status_bar.rs # Status bar widget
    ├── app.rs           # App struct implementing AppCoordinator
    └── renderer.rs      # Layout and rendering helpers
```

---

## Tasks

<task type="auto" priority="p0">
  <n>Create saorsa-ui crate with TabManager</n>
  <files>
    Cargo.toml,
    crates/saorsa-ui/Cargo.toml,
    crates/saorsa-ui/src/lib.rs,
    crates/saorsa-ui/src/tab_manager.rs
  </files>
  <action>
    1. Create crates/saorsa-ui/ directory
    
    2. Create crates/saorsa-ui/Cargo.toml:
       ```toml
       [package]
       name = "saorsa-ui"
       version = "0.1.0"
       edition.workspace = true
       authors.workspace = true
       license.workspace = true
       repository.workspace = true
       description = "UI framework and tab manager for saorsa TUI"

       [dependencies]
       saorsa-cli-core = { path = "../saorsa-cli-core" }
       ratatui = "0.29"
       crossterm = "0.29"
       thiserror = "2"
       tracing = "0.1"

       [dev-dependencies]
       proptest = "1.4"
       ```
    
    3. Create src/tab_manager.rs with TabManager:
       ```rust
       //! Tab management for the TUI framework
       
       use saorsa_cli_core::{CoreError, CoreResult, Message, Tab, TabId};
       use std::collections::HashMap;
       
       /// Manages a collection of tabs
       pub struct TabManager {
           tabs: Vec<Box<dyn Tab>>,
           active_index: usize,
           tab_indices: HashMap<TabId, usize>,
           next_id: TabId,
       }
       
       impl TabManager {
           /// Creates a new empty tab manager
           pub fn new() -> Self { ... }
           
           /// Adds a tab and returns its ID
           pub fn add_tab(&mut self, tab: Box<dyn Tab>) -> TabId { ... }
           
           /// Removes a tab by ID
           pub fn remove_tab(&mut self, id: TabId) -> CoreResult<()> { ... }
           
           /// Gets the active tab
           pub fn active_tab(&self) -> Option<&dyn Tab> { ... }
           
           /// Gets the active tab mutably
           pub fn active_tab_mut(&mut self) -> Option<&mut Box<dyn Tab>> { ... }
           
           /// Gets the active tab ID
           pub fn active_id(&self) -> Option<TabId> { ... }
           
           /// Switches to a specific tab
           pub fn switch_to(&mut self, id: TabId) -> CoreResult<()> { ... }
           
           /// Switches to next tab
           pub fn next_tab(&mut self) { ... }
           
           /// Switches to previous tab  
           pub fn prev_tab(&mut self) { ... }
           
           /// Returns all tabs
           pub fn tabs(&self) -> &[Box<dyn Tab>] { ... }
           
           /// Returns tab count
           pub fn len(&self) -> usize { ... }
           
           /// Returns true if empty
           pub fn is_empty(&self) -> bool { ... }
           
           /// Handles a message, returns response if any
           pub fn handle_message(&mut self, msg: &Message) -> Option<Message> { ... }
       }
       
       impl Default for TabManager {
           fn default() -> Self { Self::new() }
       }
       ```
       
       Key behaviors:
       - When switching tabs, call blur() on old tab and focus() on new tab
       - Handle Message::SwitchTab, NextTab, PrevTab, CloseTab
       - Tab indices update when tabs are removed
       - Can't close last tab if can_close() returns false
    
    4. Create src/lib.rs with module exports
    
    5. Update workspace Cargo.toml to add "crates/saorsa-ui"
    
    6. Write comprehensive tests for TabManager:
       - Add/remove tabs
       - Tab switching (by ID, next, prev)
       - Message handling
       - Edge cases (empty manager, single tab)
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p saorsa-ui --all-features -- -D warnings
    cargo test -p saorsa-ui
  </verify>
  <done>
    - saorsa-ui crate compiles
    - TabManager manages tab lifecycle correctly
    - Tab switching calls focus/blur appropriately
    - All message types handled
    - Tests cover all public methods
    - Zero clippy warnings
  </done>
</task>

<task type="auto" priority="p0">
  <n>Create TabBar and StatusBar widgets</n>
  <files>
    crates/saorsa-ui/src/widgets/mod.rs,
    crates/saorsa-ui/src/widgets/tab_bar.rs,
    crates/saorsa-ui/src/widgets/status_bar.rs,
    crates/saorsa-ui/src/lib.rs
  </files>
  <action>
    1. Create src/widgets/mod.rs:
       ```rust
       //! UI widgets for the saorsa TUI framework
       
       pub mod tab_bar;
       pub mod status_bar;
       
       pub use tab_bar::TabBar;
       pub use status_bar::StatusBar;
       ```
    
    2. Create src/widgets/tab_bar.rs:
       ```rust
       //! Tab bar widget for displaying and selecting tabs
       
       use ratatui::prelude::*;
       use ratatui::widgets::{Block, Borders, Tabs, Widget};
       use saorsa_cli_core::{Tab, TabId, Theme};
       
       /// Tab bar widget that displays tab titles
       pub struct TabBar<'a> {
           tabs: &'a [Box<dyn Tab>],
           active_index: usize,
           theme: &'a Theme,
       }
       
       impl<'a> TabBar<'a> {
           /// Creates a new tab bar
           pub fn new(tabs: &'a [Box<dyn Tab>], active_index: usize, theme: &'a Theme) -> Self { ... }
       }
       
       impl Widget for TabBar<'_> {
           fn render(self, area: Rect, buf: &mut Buffer) {
               // Build tab titles with icons
               let titles: Vec<Line> = self.tabs.iter().map(|t| {
                   let icon = t.icon().unwrap_or("");
                   let title = t.title();
                   if icon.is_empty() {
                       Line::from(format!(" {} ", title))
                   } else {
                       Line::from(format!(" {} {} ", icon, title))
                   }
               }).collect();
               
               // Style based on theme
               let tabs_widget = Tabs::new(titles)
                   .select(self.active_index)
                   .style(Style::default().fg(theme_to_ratatui_color(self.theme.colors.muted)))
                   .highlight_style(Style::default()
                       .fg(theme_to_ratatui_color(self.theme.colors.accent))
                       .add_modifier(Modifier::BOLD))
                   .divider(" │ ");
               
               tabs_widget.render(area, buf);
           }
       }
       ```
    
    3. Create src/widgets/status_bar.rs:
       ```rust
       //! Status bar widget for displaying application state
       
       use ratatui::prelude::*;
       use ratatui::widgets::{Paragraph, Widget};
       use saorsa_cli_core::Theme;
       
       /// Status bar displaying mode, info, and help hints
       pub struct StatusBar<'a> {
           left: &'a str,    // Mode or context
           center: &'a str,  // File/branch info
           right: &'a str,   // Help hint
           theme: &'a Theme,
       }
       
       impl<'a> StatusBar<'a> {
           /// Creates a new status bar
           pub fn new(theme: &'a Theme) -> Self { ... }
           
           /// Sets the left section (mode)
           pub fn left(mut self, text: &'a str) -> Self { ... }
           
           /// Sets the center section (info)
           pub fn center(mut self, text: &'a str) -> Self { ... }
           
           /// Sets the right section (help)
           pub fn right(mut self, text: &'a str) -> Self { ... }
       }
       
       impl Widget for StatusBar<'_> {
           fn render(self, area: Rect, buf: &mut Buffer) {
               // Render three sections with appropriate alignment
               // Left: left-aligned
               // Center: centered  
               // Right: right-aligned
               
               // Use theme colors for styling
           }
       }
       ```
    
    4. Update src/lib.rs to export widgets module
    
    5. Write tests for widget rendering (use buffer assertions)
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p saorsa-ui --all-features -- -D warnings
    cargo test -p saorsa-ui
  </verify>
  <done>
    - TabBar renders tab titles with active highlight
    - TabBar uses theme colors
    - StatusBar renders three sections
    - Widgets implement ratatui Widget trait
    - Tests verify rendering output
    - Zero clippy warnings
  </done>
</task>

<task type="auto" priority="p0">
  <n>Create App struct implementing AppCoordinator</n>
  <files>
    crates/saorsa-ui/src/app.rs,
    crates/saorsa-ui/src/renderer.rs,
    crates/saorsa-ui/src/lib.rs
  </files>
  <action>
    1. Create src/renderer.rs with layout helpers:
       ```rust
       //! Rendering and layout utilities
       
       use ratatui::prelude::*;
       use saorsa_cli_core::{PaneLayout, PaneNode, Split};
       
       /// Calculates areas for a pane layout within the given area
       pub fn calculate_pane_areas(layout: &PaneLayout, area: Rect) -> Vec<(u32, Rect)> {
           // Recursively split area based on PaneNode structure
           // Returns Vec of (PaneId, Rect) pairs
       }
       
       fn calculate_node_areas(node: &PaneNode, area: Rect, result: &mut Vec<(u32, Rect)>) {
           match node {
               PaneNode::Leaf(id) => result.push((*id, area)),
               PaneNode::Split { direction, children } => {
                   let areas = split_area(area, direction, children.len());
                   for (child, child_area) in children.iter().zip(areas) {
                       calculate_node_areas(child, child_area, result);
                   }
               }
           }
       }
       
       fn split_area(area: Rect, direction: &Split, count: usize) -> Vec<Rect> {
           // Split horizontally or vertically based on ratio
       }
       
       /// Main layout areas for the app
       pub struct AppLayout {
           pub tab_bar: Rect,
           pub content: Rect,
           pub status_bar: Rect,
       }
       
       impl AppLayout {
           /// Calculate layout areas from terminal size
           pub fn new(area: Rect) -> Self {
               // Tab bar: 1 line at top
               // Status bar: 1 line at bottom
               // Content: everything else
           }
       }
       ```
    
    2. Create src/app.rs with App struct:
       ```rust
       //! Main application struct
       
       use crate::tab_manager::TabManager;
       use crate::renderer::AppLayout;
       use crate::widgets::{TabBar, StatusBar};
       use saorsa_cli_core::{AppCoordinator, Message, MessageBus, Tab, TabId, Theme};
       use ratatui::prelude::*;
       
       /// Main application state
       pub struct App {
           tab_manager: TabManager,
           theme: Theme,
           message_bus: MessageBus,
           should_quit: bool,
           status_left: String,
           status_center: String,
           status_right: String,
       }
       
       impl App {
           /// Creates a new app with default theme
           pub fn new() -> Self { ... }
           
           /// Creates app with custom theme
           pub fn with_theme(theme: Theme) -> Self { ... }
           
           /// Adds a tab to the application
           pub fn add_tab(&mut self, tab: Box<dyn Tab>) -> TabId { ... }
           
           /// Gets the message bus for subscribing
           pub fn message_bus(&self) -> &MessageBus { ... }
           
           /// Sets the status bar left section
           pub fn set_status_left(&mut self, text: impl Into<String>) { ... }
           
           /// Sets the status bar center section
           pub fn set_status_center(&mut self, text: impl Into<String>) { ... }
           
           /// Sets the status bar right section
           pub fn set_status_right(&mut self, text: impl Into<String>) { ... }
           
           /// Renders the entire application
           pub fn render(&self, frame: &mut Frame) {
               let layout = AppLayout::new(frame.area());
               
               // Render tab bar
               let tab_bar = TabBar::new(
                   self.tab_manager.tabs(),
                   self.tab_manager.active_index(),
                   &self.theme,
               );
               frame.render_widget(tab_bar, layout.tab_bar);
               
               // Render active tab content
               if let Some(tab) = self.tab_manager.active_tab() {
                   tab.view(frame, layout.content);
               }
               
               // Render status bar
               let status = StatusBar::new(&self.theme)
                   .left(&self.status_left)
                   .center(&self.status_center)
                   .right(&self.status_right);
               frame.render_widget(status, layout.status_bar);
           }
       }
       
       impl AppCoordinator for App {
           fn tabs(&self) -> &[Box<dyn Tab>] {
               self.tab_manager.tabs()
           }
           
           fn active_tab(&self) -> TabId {
               self.tab_manager.active_id().unwrap_or(0)
           }
           
           fn theme(&self) -> &Theme {
               &self.theme
           }
           
           fn dispatch(&mut self, msg: Message) {
               match &msg {
                   Message::Quit => self.should_quit = true,
                   Message::NextTab => self.tab_manager.next_tab(),
                   Message::PrevTab => self.tab_manager.prev_tab(),
                   Message::SwitchTab(id) => { let _ = self.tab_manager.switch_to(*id); }
                   Message::CloseTab(id) => { let _ = self.tab_manager.remove_tab(*id); }
                   _ => {
                       // Forward to active tab
                       if let Some(tab) = self.tab_manager.active_tab_mut() {
                           // Tab would need update method - defer to later
                       }
                   }
               }
               // Broadcast to subscribers
               let _ = self.message_bus.send(msg);
           }
           
           fn tick(&mut self) {
               // Called periodically for updates
           }
           
           fn should_quit(&self) -> bool {
               self.should_quit
           }
       }
       
       impl Default for App {
           fn default() -> Self { Self::new() }
       }
       ```
    
    3. Update src/lib.rs to export App and renderer
    
    4. Write tests for:
       - App creation and tab management
       - Message dispatch (Quit, tab switching)
       - Layout calculation
       - Pane area calculation
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p saorsa-ui --all-features -- -D warnings
    cargo test -p saorsa-ui
    cargo doc -p saorsa-ui --no-deps
  </verify>
  <done>
    - App implements AppCoordinator trait
    - App renders tab bar, content, status bar
    - Message dispatch handles navigation and quit
    - Layout calculation works for any terminal size
    - Pane areas calculated correctly from PaneLayout
    - Tests cover all major functionality
    - Zero clippy warnings
    - Documentation compiles
  </done>
</task>

---

## Notes

### Design Decisions

1. **TabManager separate from App**: Keeps tab logic testable and reusable.

2. **Widgets use references**: TabBar and StatusBar borrow data to avoid copies.

3. **App owns MessageBus**: Central point for event distribution.

4. **Renderer module**: Separates layout math from widget rendering.

### Dependencies

```toml
[dependencies]
saorsa-cli-core = { path = "../saorsa-cli-core" }
ratatui = "0.29"
crossterm = "0.29"
thiserror = "2"
tracing = "0.1"
```

### Integration with Existing Code

After this phase, the cli/ crate can be updated to use App:

```rust
use saorsa_ui::App;

let mut app = App::new();
app.add_tab(Box::new(FilesTab::new()));  // sb
app.add_tab(Box::new(DiskTab::new()));   // sdisk

// Main loop
loop {
    terminal.draw(|f| app.render(f))?;
    
    if let Event::Key(key) = event::read()? {
        app.dispatch(Message::Key(key));
    }
    
    if app.should_quit() {
        break;
    }
}
```

---

## Exit Criteria

- [x] `cargo build -p saorsa-ui` succeeds
- [x] `cargo test -p saorsa-ui` passes (100%) - 108 tests
- [x] `cargo clippy -p saorsa-ui -- -D warnings` clean
- [x] `cargo doc -p saorsa-ui --no-deps` builds without warnings
- [x] TabManager handles all tab operations correctly
- [x] TabBar and StatusBar render with theme colors
- [x] App implements full AppCoordinator interface
- [x] Layout calculation handles edge cases

## Next Phase

After completion, proceed to:
```
/gsd:plan-phase 8 "Integrate Existing Tools as Tabs"
```

This will wrap sb and sdisk as tabs within the new framework.
