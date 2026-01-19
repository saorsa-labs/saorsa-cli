# PLAN-9: M2P1 - Git Status & Diff View

**STATUS: ✓ COMPLETE** (2026-01-17)

## Objective

Create a gitui-like Git tab with status view, diff viewer, and basic staging operations.

## Context

Phase 8 completed M1 with the unified TUI framework. The `sb/src/git.rs` module provides git2 foundation:
- `GitRepository` struct with status, diff, file operations
- `FileStatus` enum for change detection
- Basic diff generation

This phase creates a dedicated Git tab building on that foundation.

## Tasks

### Task 1: Create saorsa-git Crate Structure

Create the crate skeleton with proper dependencies.

**Files to create:**
- `crates/saorsa-git/Cargo.toml`
- `crates/saorsa-git/src/lib.rs`
- `crates/saorsa-git/src/error.rs`
- `crates/saorsa-git/src/repo.rs` (enhanced GitRepository)

**Dependencies:**
- saorsa-cli-core
- git2
- ratatui
- thiserror

### Task 2: Enhanced Git Repository Wrapper

Extend git2 with gitui-like operations.

**File: `crates/saorsa-git/src/repo.rs`**

```rust
pub struct GitRepo {
    repo: Repository,
    root: PathBuf,
}

impl GitRepo {
    // Existing
    pub fn open(path: &Path) -> Result<Self>;
    pub fn status(&self) -> Result<Vec<StatusEntry>>;
    pub fn file_diff(&self, path: &Path) -> Result<Diff>;

    // New for Phase 9
    pub fn staged_files(&self) -> Result<Vec<StatusEntry>>;
    pub fn unstaged_files(&self) -> Result<Vec<StatusEntry>>;
    pub fn untracked_files(&self) -> Result<Vec<StatusEntry>>;
    pub fn stage_file(&self, path: &Path) -> Result<()>;
    pub fn unstage_file(&self, path: &Path) -> Result<()>;
    pub fn stage_all(&self) -> Result<()>;
    pub fn unstage_all(&self) -> Result<()>;
    pub fn discard_changes(&self, path: &Path) -> Result<()>;
    pub fn current_branch(&self) -> Result<String>;
    pub fn head_commit(&self) -> Result<CommitInfo>;
}

pub struct StatusEntry {
    pub path: PathBuf,
    pub status: FileStatus,
    pub staged: bool,
}

pub struct CommitInfo {
    pub id: String,
    pub message: String,
    pub author: String,
    pub time: DateTime<Utc>,
}

pub struct Diff {
    pub hunks: Vec<DiffHunk>,
}

pub struct DiffHunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
}

pub struct DiffLine {
    pub origin: char,  // '+', '-', ' '
    pub content: String,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
}
```

### Task 3: Git Status Widget

Status view showing staged, unstaged, and untracked files.

**File: `crates/saorsa-git/src/widgets/status.rs`**

```rust
pub struct StatusWidget<'a> {
    repo: &'a GitRepo,
    selected_section: Section,  // Staged, Unstaged, Untracked
    selected_index: usize,
}

enum Section {
    Staged,
    Unstaged,
    Untracked,
}

impl Widget for StatusWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Three collapsible sections
        // - Staged Changes (green)
        // - Changes not staged (yellow)
        // - Untracked files (gray)
        // Keyboard: j/k navigate, Enter to stage/unstage, s to stage all
    }
}
```

### Task 4: Diff Viewer Widget

Show diff with syntax highlighting for selected file.

**File: `crates/saorsa-git/src/widgets/diff.rs`**

```rust
pub struct DiffWidget<'a> {
    diff: &'a Diff,
    scroll_offset: u16,
    show_line_numbers: bool,
}

impl Widget for DiffWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Hunks with headers (@@ -x,y +a,b @@)
        // Added lines in green (+)
        // Removed lines in red (-)
        // Context lines in default color
        // Line numbers on left
    }
}
```

### Task 5: GitTab Implementation

Main Git tab combining status and diff views.

**File: `crates/saorsa-git/src/tab.rs`**

```rust
pub struct GitTab {
    id: TabId,
    repo: Option<GitRepo>,
    status_widget: StatusWidget,
    diff_widget: DiffWidget,
    focus: GitFocus,  // Status or Diff
    selected_file: Option<PathBuf>,
}

enum GitFocus {
    Status,
    Diff,
}

impl Tab for GitTab {
    fn id(&self) -> TabId { self.id }
    fn title(&self) -> &str { "Git" }
    fn icon(&self) -> Option<&str> { Some("󰊢") }  // Nerd font git icon

    fn view(&self, frame: &mut Frame, area: Rect) {
        // Split: 40% status, 60% diff
        // Or full-width for each when focused
    }

    fn handle_message(&mut self, msg: Message) -> Message {
        // Key handlers:
        // - Tab/Shift+Tab: switch focus
        // - j/k: navigate files
        // - Enter: toggle staged
        // - s: stage all
        // - u: unstage all
        // - d: discard changes (with confirmation)
        // - r: refresh
    }
}
```

### Task 6: Add Git Tab to Unified Binary

Update `crates/saorsa/src/main.rs` to include Git tab.

```rust
use saorsa_git::GitTab;

// In main():
let git_tab = GitTab::new(3, &root);
app.add_tab(Box::new(git_tab));
```

### Task 7: Tests

**Unit tests:**
- StatusEntry creation and filtering
- Diff parsing
- Widget rendering

**Integration tests:**
- Git operations on temporary repo
- Stage/unstage workflows

## File Structure

```
crates/saorsa-git/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── error.rs
    ├── repo.rs          # Enhanced GitRepo
    ├── tab.rs           # GitTab implementation
    └── widgets/
        ├── mod.rs
        ├── status.rs    # Status list widget
        └── diff.rs      # Diff viewer widget
```

## Dependencies

```toml
[dependencies]
saorsa-cli-core = { path = "../saorsa-cli-core" }
git2 = "0.20"
ratatui = "0.29"
thiserror = "2"
chrono = { version = "0.4", features = ["serde"] }
```

## Exit Criteria

- [ ] `cargo build -p saorsa-git` succeeds
- [ ] `cargo test -p saorsa-git` passes
- [ ] `cargo clippy -p saorsa-git -- -D warnings` clean
- [ ] Git tab shows staged/unstaged/untracked sections
- [ ] File selection updates diff view
- [ ] Stage/unstage operations work (Enter key)
- [ ] j/k navigation works in status view
- [ ] Tab switching includes Git tab
- [ ] Graceful handling when not in git repo

## Keyboard Shortcuts (Git Tab)

| Key | Action |
|-----|--------|
| `j`/`↓` | Move down in list |
| `k`/`↑` | Move up in list |
| `Enter` | Toggle stage/unstage |
| `s` | Stage all |
| `u` | Unstage all |
| `d` | Discard changes (confirm) |
| `r` | Refresh status |
| `Tab` | Switch focus (status ↔ diff) |
| `J`/`K` | Scroll diff up/down |
| `gg` | Go to top |
| `G` | Go to bottom |

## Notes

- Phase 10 will add commit dialog and history
- Phase 11 will add branch operations
- Keep operations non-destructive by default
- Always confirm destructive actions (discard)
