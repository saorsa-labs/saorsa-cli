# Phase Plan: M1P1 - Core Architecture Foundation

**Milestone**: M1 - Foundation & Tab Infrastructure
**Phase**: 1 - Core Architecture (Part A)
**Created**: 2026-01-17
**Completed**: 2026-01-17
**Status**: ✓ COMPLETE

## Overview

Establish the foundational `saorsa-cli-core` crate with core traits, types, and event system that all other crates will depend on. This is the architectural backbone of the unified TUI.

## Prerequisites

- [x] GSD-Hybrid initialized
- [x] Interview decisions documented
- [x] Current codebase explored
- [x] Ensure cargo builds cleanly: `cargo build --workspace`

## Architecture Decisions (from STATE.md)

- **Event System**: Message-passing with typed enums
- **State Management**: Per-tab state with global coordinator
- **Async Runtime**: Tokio
- **Navigation**: Tabs + tmux-style panes

## New Crate Structure

```
saorsa-cli/
├── Cargo.toml (workspace)
├── crates/
│   └── saorsa-cli-core/     ← NEW (this phase)
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── tab.rs       # Tab trait and TabId
│           ├── pane.rs      # Pane trait and layout
│           ├── event.rs     # Event/Message system
│           ├── theme.rs     # Theme types (stub)
│           └── error.rs     # Core error types
├── cli/                  # Existing - will integrate later
├── sb/                   # Existing - will become tab
└── sdisk/                # Existing - will become tab
```

---

## Tasks

<task type="auto" priority="p0">
  <n>Create saorsa-cli-core crate with core traits</n>
  <files>
    Cargo.toml,
    crates/saorsa-cli-core/Cargo.toml,
    crates/saorsa-cli-core/src/lib.rs,
    crates/saorsa-cli-core/src/tab.rs,
    crates/saorsa-cli-core/src/pane.rs,
    crates/saorsa-cli-core/src/error.rs
  </files>
  <action>
    1. Create crates/ directory structure
    
    2. Create crates/saorsa-cli-core/Cargo.toml:
       - name = "saorsa-cli-core"
       - version = "0.1.0"
       - Use workspace.package inheritance
       - Dependencies: thiserror, ratatui (0.29), crossterm (0.29)
       - Dev-dependencies: proptest, tokio (test-util)
    
    3. Create src/error.rs with core error types:
       - CoreError enum using thiserror
       - Variants: TabNotFound, PaneNotFound, InvalidLayout, EventError
       - Implement Display and Error traits
    
    4. Create src/tab.rs with Tab trait:
       ```rust
       pub type TabId = u32;
       
       pub trait Tab: Send + Sync {
           fn id(&self) -> TabId;
           fn title(&self) -> &str;
           fn icon(&self) -> Option<&str> { None }
           fn can_close(&self) -> bool { true }
           
           fn update(&mut self, msg: Message) -> Option<Message>;
           fn view(&self, frame: &mut Frame, area: Rect);
           fn focus(&mut self);
           fn blur(&mut self);
       }
       ```
    
    5. Create src/pane.rs with Pane types:
       ```rust
       pub type PaneId = u32;
       
       pub enum Split {
           Horizontal(u16), // percentage
           Vertical(u16),
       }
       
       pub struct PaneLayout {
           pub root: PaneNode,
       }
       
       pub enum PaneNode {
           Leaf(PaneId),
           Split { direction: Split, children: Vec<PaneNode> },
       }
       ```
    
    6. Create src/lib.rs re-exporting all modules:
       - pub mod tab;
       - pub mod pane;
       - pub mod error;
       - pub use for main types
    
    7. Update workspace Cargo.toml:
       - Add "crates/saorsa-cli-core" to members
       - Add saorsa-cli-core to workspace.dependencies
    
    8. Write unit tests for Tab trait (mock implementation)
    9. Write unit tests for PaneLayout validation
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p saorsa-cli-core --all-features -- -D warnings
    cargo test -p saorsa-cli-core
    cargo doc -p saorsa-cli-core --no-deps
  </verify>
  <done>
    - saorsa-cli-core crate compiles
    - Tab trait defined with all required methods
    - PaneLayout types support horizontal/vertical splits
    - CoreError covers all failure cases
    - All tests pass
    - Zero clippy warnings
    - Documentation compiles
  </done>
</task>

<task type="auto" priority="p0">
  <n>Implement event/message system</n>
  <files>
    crates/saorsa-cli-core/src/event.rs,
    crates/saorsa-cli-core/src/lib.rs
  </files>
  <action>
    1. Create src/event.rs with Message enum:
       ```rust
       #[derive(Debug, Clone, PartialEq)]
       pub enum Message {
           // Navigation
           SwitchTab(TabId),
           CloseTab(TabId),
           NextTab,
           PrevTab,
           
           // Pane management
           SplitPane { direction: Split, ratio: u16 },
           ClosePane(PaneId),
           FocusPane(PaneId),
           ResizePane { pane: PaneId, delta: i16 },
           
           // Global
           Quit,
           ToggleHelp,
           OpenCommandPalette,
           
           // Input forwarding
           Key(KeyEvent),
           Mouse(MouseEvent),
           Resize(u16, u16),
           
           // Custom (for tabs/plugins)
           Custom(String, serde_json::Value),
           
           // Batch operations
           Batch(Vec<Message>),
           
           // No-op
           None,
       }
       ```
    
    2. Create MessageBus for broadcasting:
       ```rust
       use tokio::sync::broadcast;
       
       pub struct MessageBus {
           sender: broadcast::Sender<Message>,
       }
       
       impl MessageBus {
           pub fn new(capacity: usize) -> Self;
           pub fn subscribe(&self) -> broadcast::Receiver<Message>;
           pub fn send(&self, msg: Message) -> Result<(), CoreError>;
       }
       ```
    
    3. Add InputEvent wrapper for crossterm events:
       ```rust
       pub enum InputEvent {
           Key(KeyEvent),
           Mouse(MouseEvent),
           Resize(u16, u16),
           Tick,
       }
       ```
    
    4. Update lib.rs to export event module
    
    5. Write tests for Message serialization/deserialization
    6. Write tests for MessageBus send/receive
    7. Write property-based tests for Message::Batch flattening
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p saorsa-cli-core --all-features -- -D warnings
    cargo test -p saorsa-cli-core
  </verify>
  <done>
    - Message enum covers all navigation/input scenarios
    - MessageBus supports multiple subscribers
    - InputEvent maps cleanly from crossterm
    - All async operations use tokio
    - Property tests validate batch operations
    - Zero clippy warnings
  </done>
</task>

<task type="auto" priority="p1">
  <n>Add theme types and App coordinator stub</n>
  <files>
    crates/saorsa-cli-core/src/theme.rs,
    crates/saorsa-cli-core/src/app.rs,
    crates/saorsa-cli-core/src/lib.rs
  </files>
  <action>
    1. Create src/theme.rs with theme schema types:
       ```rust
       use ratatui::style::{Color, Modifier, Style};
       use serde::{Deserialize, Serialize};
       
       #[derive(Debug, Clone, Serialize, Deserialize)]
       pub struct Theme {
           pub name: String,
           pub colors: ThemeColors,
           pub borders: BorderStyle,
       }
       
       #[derive(Debug, Clone, Serialize, Deserialize)]
       pub struct ThemeColors {
           pub background: Color,
           pub foreground: Color,
           pub accent: Color,
           pub selection: Color,
           pub error: Color,
           pub warning: Color,
           pub success: Color,
           pub muted: Color,
       }
       
       #[derive(Debug, Clone, Serialize, Deserialize)]
       pub enum BorderStyle {
           Rounded,
           Square,
           Double,
           None,
       }
       
       impl Default for Theme {
           fn default() -> Self { /* dark theme */ }
       }
       ```
    
    2. Create src/app.rs with AppCoordinator trait:
       ```rust
       pub trait AppCoordinator {
           fn tabs(&self) -> &[Box<dyn Tab>];
           fn active_tab(&self) -> TabId;
           fn theme(&self) -> &Theme;
           
           fn dispatch(&mut self, msg: Message);
           fn tick(&mut self);
       }
       ```
    
    3. Add built-in themes:
       - Theme::dark() - default dark theme
       - Theme::light() - light theme  
       - Theme::nord() - Nord color scheme
    
    4. Update lib.rs exports
    
    5. Write tests for Theme serialization (TOML round-trip)
    6. Write tests for default theme values
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p saorsa-cli-core --all-features -- -D warnings
    cargo test -p saorsa-cli-core
    cargo doc -p saorsa-cli-core --no-deps
  </verify>
  <done>
    - Theme struct serializes to/from TOML
    - Three built-in themes available
    - AppCoordinator trait defines app lifecycle
    - Colors use ratatui Color type
    - Zero clippy warnings
    - All documentation present
  </done>
</task>

---

## Notes

### Design Rationale

1. **Message-passing over shared state**: Chosen for clarity and testability. Each tab receives messages and optionally returns responses.

2. **TabId as u32**: Simple numeric IDs allow fast lookup. More complex scenarios can use a registry.

3. **PaneLayout as tree**: Recursive structure naturally models nested splits (like tmux).

4. **Theme with serde**: Enables loading from TOML files and future in-app theme editor.

### Dependencies Added

```toml
[dependencies]
thiserror = "2"
ratatui = "0.29"
crossterm = "0.29"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
tokio = { version = "1.42", features = ["sync"] }

[dev-dependencies]
proptest = "1.4"
tokio = { version = "1.42", features = ["test-util", "rt-multi-thread", "macros"] }
```

### Migration Path

After this phase:
1. Phase 1B: Create saorsa-ui crate with TabManager, rendering
2. Phase 2: Refactor sb to implement Tab trait
3. Phase 3: Refactor sdisk to implement Tab trait
4. Phase 4: Create unified binary

---

## Exit Criteria

- [ ] `cargo build -p saorsa-cli-core` succeeds
- [ ] `cargo test -p saorsa-cli-core` passes (100%)
- [ ] `cargo clippy -p saorsa-cli-core -- -D warnings` clean
- [ ] `cargo doc -p saorsa-cli-core --no-deps` builds without warnings
- [ ] Tab, Pane, Message, Theme types fully documented
- [ ] At least 80% code coverage on core types

## Next Phase

After completion, proceed to:
```
/gsd:plan-phase 2 "Tab Manager & UI Framework"
```
