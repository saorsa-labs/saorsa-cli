# Saorsa TUI - Project Roadmap

## Vision

Transform saorsa-cli into a comprehensive, extensible TUI workstation integrating:
- Git client (gitui-like)
- Code/file search (ripgrep/fd)
- Markdown editor (existing sb)
- Disk analyzer (existing sdisk)
- System monitor (htop/bottom-like)
- Network monitor (bandwhich-like)
- Plugin system (WASM-based)

All in a modern tabbed interface with mouse support, theming, and tmux-style pane splitting.

---

## M1: Foundation & Tab Infrastructure

Goal: Establish core architecture and tabbed interface with existing tools

### Phase 1: Core Architecture Restructure
Status: pending

- [ ] Create workspace restructure plan
  - saorsa-core (shared types, traits, event system)
  - saorsa-ui (tab manager, pane system, widgets)
  - saorsa-git (git operations)
  - saorsa-search (ripgrep/fd integration)
  - saorsa-monitor (system/network)
  - saorsa-plugins (WASM runtime)
  - saorsa (main binary)
- [ ] Define core traits (Tab, Pane, Widget, Plugin)
- [ ] Set up shared event/message system
- [ ] Migrate existing cli/ functionality to new structure

### Phase 2: Tab & Pane System
Status: pending

- [ ] Implement TabManager with fixed tabs
- [ ] Add rat-widget integration for advanced widgets
- [ ] Build pane splitting (horizontal/vertical)
- [ ] Implement mouse support for tab switching, pane resizing
- [ ] Add keyboard shortcuts (Ctrl+1-9 for tabs, Ctrl+backslash for splits)
- [ ] Create tab bar widget with icons/badges

### Phase 3: Existing Tool Integration
Status: pending

- [ ] Integrate sb as Files tab
- [ ] Integrate sdisk as Disk tab
- [ ] Create placeholder tabs for future features
- [ ] Implement unified status bar
- [ ] Add command palette (Ctrl+P / :command)

Milestone Completion Criteria:
- Single binary launches tabbed interface
- sb and sdisk accessible as tabs
- Pane splitting works within tabs
- Mouse and keyboard navigation functional

---

## M2: Git Integration (gitui-like)

Goal: Full Git client in the Git tab

### Phase 1: Git Status & Diff View
Status: pending

- [ ] Enhance git2 integration from sb/git.rs
- [ ] Implement status view (staged, unstaged, untracked)
- [ ] Add diff viewer with syntax highlighting
- [ ] Show file tree with git status indicators
- [ ] Implement hunk-level staging/unstaging

### Phase 2: Commit & History
Status: pending

- [ ] Commit dialog with message editor
- [ ] Commit history log view
- [ ] Commit detail view (files changed, diff)
- [ ] Branch visualization
- [ ] Stash management

### Phase 3: Advanced Git Operations
Status: pending

- [ ] Branch create/switch/delete/rename
- [ ] Merge and rebase UI
- [ ] Remote management (fetch, pull, push)
- [ ] Conflict resolution view
- [ ] Blame view integration

Milestone Completion Criteria:
- Can stage/unstage files and hunks
- Can commit with message
- Can view history and diffs
- Basic branch operations work

---

## M3: Search Integration (ripgrep/fd)

Goal: Powerful search with unified search bar

### Phase 1: Unified Search Bar
Status: pending

- [ ] Implement Ctrl+P search overlay
- [ ] Add mode toggle (files/content/git)
- [ ] Integrate fd for file finding (ignore patterns)
- [ ] Add fuzzy matching for file names

### Phase 2: Content Search (ripgrep)
Status: pending

- [ ] Integrate grep crate for content search
- [ ] Results list with file:line preview
- [ ] Search within results (filter)
- [ ] Replace functionality
- [ ] Search history

### Phase 3: Advanced Search Features
Status: pending

- [ ] Regex search mode
- [ ] Search scopes (directory, project, workspace)
- [ ] Saved searches
- [ ] Search and replace across files

Milestone Completion Criteria:
- Unified search bar works for files and content
- Results are navigable and jump to location
- Respects gitignore

---

## M4: Theme Engine & Settings UI

Goal: Full customization through themes and in-app settings

### Phase 1: Theme Engine
Status: pending

- [ ] Define theme schema (TOML)
- [ ] Implement theme loader
- [ ] Create built-in themes (dark, light, nord, dracula, gruvbox)
- [ ] Hot-reload theme changes
- [ ] Per-component style overrides

### Phase 2: Settings UI
Status: pending

- [ ] Create settings tab/modal
- [ ] Keybinding configuration
- [ ] Theme selection
- [ ] Tool-specific settings
- [ ] Import/export configuration

Milestone Completion Criteria:
- Users can switch themes
- Settings configurable via TUI
- Custom themes loadable from files

---

## M5: System Monitor (bottom-like)

Goal: htop-like system monitoring tab

### Phase 1: Core Metrics
Status: pending

- [ ] CPU usage graph (per-core)
- [ ] Memory usage graph
- [ ] Process list with sorting
- [ ] Process search and filter

### Phase 2: Extended Monitoring
Status: pending

- [ ] Disk I/O graphs
- [ ] Network usage graphs
- [ ] Temperature sensors (where available)
- [ ] Battery status (laptops)

### Phase 3: Process Management
Status: pending

- [ ] Kill process
- [ ] Process tree view
- [ ] Process details panel

Milestone Completion Criteria:
- Real-time CPU/memory/process display
- Can kill processes
- Graphs update smoothly

---

## M6: Network Monitor (bandwhich-like)

Goal: Network traffic analysis tab

### Phase 1: Connection View
Status: pending

- [ ] Active connections list
- [ ] Per-process network usage
- [ ] Protocol identification

### Phase 2: Traffic Analysis
Status: pending

- [ ] Bandwidth graphs
- [ ] Remote host resolution
- [ ] Port/service identification

Milestone Completion Criteria:
- Shows active connections with bandwidth
- Per-process breakdown
- Works on macOS and Linux

---

## M7: WASM Plugin System

Goal: Extensible plugin architecture using WebAssembly

### Phase 1: Plugin Runtime
Status: pending

- [ ] Integrate Wasmtime
- [ ] Define plugin API (wit/interface types)
- [ ] Plugin discovery and loading
- [ ] Plugin lifecycle management

### Phase 2: Plugin SDK
Status: pending

- [ ] Create saorsa-plugin-sdk crate
- [ ] Example plugins (custom tab, widget, command)
- [ ] Documentation and templates
- [ ] Plugin marketplace concept

### Phase 3: Migration
Status: pending

- [ ] Convert libloading plugins to WASM
- [ ] Maintain backward compatibility shim

Milestone Completion Criteria:
- WASM plugins can add custom tabs
- Plugin hot-reload works
- SDK documented with examples

---

## Future Milestones (Post-M7)

- M8: Remote/SSH mode
- M9: Collaborative features
- M10: AI assistant integration
- M11: Custom dashboard widgets
- M12: Project templates and scaffolding

---

## Timeline Note

This roadmap focuses on what needs to be built, not when. Each milestone is self-contained and can be prioritized based on needs. M1 is foundational and must complete first.
