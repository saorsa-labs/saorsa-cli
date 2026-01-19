# Saorsa TUI - Issues Backlog

## Priority Levels

- P0: Blockers - Must fix immediately
- P1: Next phase - Address in upcoming work
- P2: This milestone - Tackle within current milestone
- P3: Future - Defer to later milestones

---

## P0: Blockers

None currently

---

## P1: Next Phase

### Architecture Decisions Needed

- [ ] ARCH-001: Define inter-crate communication pattern
  - Options: Message passing, shared state with Arc Mutex, actor model
  - Decision needed before M1P1 starts

- [ ] ARCH-002: Event system design
  - Should events bubble up from child to parent?
  - How to handle cross-tab communication?
  - Reference: rat-salsa event queue pattern

- [ ] ARCH-003: State management strategy
  - Global app state vs per-tab state
  - How to persist state across restarts?
  - Session restore capability?

### Technical Research

- [ ] RESEARCH-001: Evaluate rat-widget integration
  - Test tabbed, split widgets
  - Assess compatibility with existing code
  - Document migration path

- [ ] RESEARCH-002: Wasmtime plugin API design
  - Study wit-bindgen for interface definitions
  - Define minimal plugin capabilities
  - Security sandboxing requirements

---

## P2: This Milestone (M1)

### Known Technical Debt

- [ ] DEBT-001: Windows build disabled
  - Vendored OpenSSL issues with git2
  - Need to resolve before M1 complete
  - Workaround: native TLS on Windows

- [ ] DEBT-002: sb/sdisk are separate binaries
  - Need to refactor as library crates
  - Main functionality should be embeddable
  - CLI wrappers can remain for standalone use

### Testing Gaps

- [ ] TEST-001: No TUI integration tests
  - Need test harness for terminal rendering
  - Consider ratatui-test or similar

- [ ] TEST-002: Plugin system untested
  - Current libloading plugins need test coverage
  - Before migrating to WASM

---

## P3: Future

### Feature Requests

- [ ] FEAT-001: SSH/remote mode
  - Run saorsa locally, operate on remote files
  - Consider tramp-mode-like approach

- [ ] FEAT-002: Multiplexer integration
  - tmux/screen awareness
  - Nested terminal handling

- [ ] FEAT-003: Image preview in terminal
  - Sixel/iTerm2 protocol support
  - ratatui-image already available

- [ ] FEAT-004: Jupyter notebook support
  - View/edit .ipynb files
  - Execute cells if kernel available

- [ ] FEAT-005: AI assistant panel
  - LLM integration for code assistance
  - Context-aware suggestions

### Performance Considerations

- [ ] PERF-001: Large repository handling
  - Git operations on Linux kernel-sized repos
  - Lazy loading strategies

- [ ] PERF-002: Search performance
  - Benchmark against native rg/fd
  - Memory usage during large searches

### Platform-Specific

- [ ] PLAT-001: macOS menu bar integration
  - Native notifications
  - Dock icon badges

- [ ] PLAT-002: Linux desktop integration
  - .desktop file
  - freedesktop notifications

- [ ] PLAT-003: Windows terminal considerations
  - Windows Terminal vs cmd.exe
  - ConPTY handling

---

## Resolved Issues

Move completed issues here with resolution notes

---

## Issue Template

CATEGORY-XXX: Title

Priority: P0/P1/P2/P3
Milestone: MX
Status: Open/In Progress/Resolved

Description:
Brief description of the issue.

Context:
Why this matters, what triggered it.

Proposed Solution:
How we might address it.

Resolution (when closed):
What was done to resolve it.
