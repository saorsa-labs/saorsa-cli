# Saorsa TUI - Project State

> Cross-session memory for the saorsa unified TUI project

## Current Position

- **Milestone**: M2 - Enhanced Tools
- **Phase**: M2P1 - Git Status & Diff View ✓ COMPLETE
- **Plan**: PLAN-9.md (executed)
- **Next**: Phase M2P2 - Search Tab (ripgrep/fd)
- **Status**: Ready for next phase

### Completed Phases
- M1P1A - Core Architecture Foundation (Phase 6) ✓
- M1P1B - Tab Manager & UI Framework (Phase 7) ✓
- M1P1C - Integrate Existing Tools as Tabs (Phase 8) ✓
- M2P1 - Git Status & Diff View (Phase 9) ✓

## Interview Decisions

| Category | Decision | Rationale |
|----------|----------|-----------|
| **Integration** | Embed as libraries | Build functionality using underlying crates (grep, git2, sysinfo). Single binary, cohesive UX |
| **Priority** | Git → Search → System → Network | Developer workflow first, monitoring tools second |
| **Plugin System** | WASM (Wasmtime) | Sandboxed, portable, hot-reloadable. Modern approach |
| **Navigation** | Tabs + tmux-style panes | Fixed main tabs, user can split panes within tabs |
| **Input** | Full mouse + keyboard | Click, resize, scroll with mouse. Full keyboard shortcuts |
| **Theming** | Full theme engine | TOML/JSON themes, multiple built-in (dark, light, nord, etc.) |
| **Config** | In-app settings UI | TUI settings panel for user-friendly configuration |
| **Testing** | TDD from start | Integration tests per tool, property-based testing |
| **Git Approach** | Enhance existing git2 | Extend sb/git.rs with gitui-like staging, commit, diff |
| **WASM Runtime** | Wasmtime | Mature, well-documented, WASI support |
| **Search UX** | Unified search bar | Single input with mode toggle (files/content/git) |
| **MVP Scope** | Tabbed shell + Git + existing | Tab interface with sb, sdisk, plus new Git tab |
| **Codebase Structure** | Monorepo feature crates | saorsa-core, saorsa-git, saorsa-search, etc. |
| **Async Runtime** | Tokio | Consistent with existing codebase |

## Existing Codebase Foundation

### What Exists (leverage these)
- cli/ - Interactive menu, plugin system (libloading), auto-update, config
- sb/ - Markdown browser/editor, dual-pane, git integration, file tree, syntax highlighting
- sdisk/ - Disk analyzer, interactive cleanup, progress bars
- Ratatui 0.29 + Crossterm 0.29 already in use
- tokio async runtime
- git2 integration in sb

### What Needs Building
- Tab infrastructure with pane splitting
- Enhanced Git tab (gitui-like)
- Search tab (ripgrep/fd integration)
- System monitor tab (bottom-like)
- Network monitor tab (bandwhich-like)
- WASM plugin runtime (Wasmtime)
- Theme engine
- Settings UI
- Unified search bar

## Session Log

### 2026-01-17 - Project Initialization
- Completed deep research on Rust TUI ecosystem
- Identified best-in-class tools: Ratatui, gitui, bottom, bandwhich, rat-widget
- Completed interview process (14 decisions captured)
- Created planning structure
- Created PLAN-6.md for M1P1A Core Architecture Foundation
- **Executed Phase 6**: saorsa-cli-core crate created
  - 7 source files, ~57KB total
  - 69 unit tests + 26 doc tests (95 total)
  - Core types: Tab, TabId, PaneLayout, Message, MessageBus, Theme
  - Three built-in themes: dark, light, nord
- **Executed Phase 7**: saorsa-ui crate created
  - TabManager for tab lifecycle management
  - TabBar and StatusBar widgets with theme support
  - App struct implementing AppCoordinator trait
  - Layout calculation and pane area rendering
  - 108 tests passing, zero clippy warnings
- **Executed Phase 8**: Integrated existing tools as tabs
  - Created saorsa-sb crate wrapping sb file browser
  - Created saorsa-disk crate wrapping sdisk analyzer
  - Created unified saorsa binary with tabbed interface
  - Tab switching with Tab/Shift+Tab, quit with Ctrl+Q
- **Executed Phase 9**: Git Tab with Status & Diff View
  - Created saorsa-git crate with git2 integration
  - GitRepo wrapper with status, staging, diff operations
  - StatusWidget showing staged/unstaged/untracked files
  - DiffWidget with syntax highlighting and scrolling
  - GitTab with section navigation (j/k), staging (s), unstaging (u)
  - 29 tests for saorsa-git, 260+ total tests passing
  - Zero clippy warnings, all code formatted

## Blockers

None currently.

## Handoff Context

**For next session - Phase M2P2: Search Tab (ripgrep/fd integration)**

All Phase 9 components are complete. The unified binary now has:
- Files tab (saorsa-sb)
- Disk tab (saorsa-disk)
- Git tab (saorsa-git)

Next phase should add search functionality:
1. Create saorsa-search crate
2. Integrate ripgrep (grep-regex, grep-searcher crates)
3. Integrate fd (ignore crate for fast file finding)
4. Build unified search bar with mode toggle (files/content/git)
5. Results navigation with preview

Key patterns established:
- Tab trait with parking_lot::Mutex for interior mutability
- Widgets use direct buffer rendering (buf.set_string)
- Pre-compute values before method calls to avoid borrow issues
- Integration tests with tempfile for git operations

Key types:
```rust
use saorsa_cli_core::{Tab, TabId, Message, Theme};
use saorsa_ui::{App, TabManager};
use saorsa_git::{GitTab, GitRepo};
```
