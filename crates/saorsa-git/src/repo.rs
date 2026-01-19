//! Git repository operations
//!
//! Provides an enhanced wrapper around git2 with gitui-like operations.

use crate::error::{GitError, GitResult};
use chrono::{DateTime, TimeZone, Utc};
use git2::{DiffOptions, Repository, Status, StatusOptions};
use std::path::{Path, PathBuf};

/// File status in the repository
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileStatus {
    /// File is unchanged
    Unmodified,
    /// File is newly added
    Added,
    /// File has been modified
    Modified,
    /// File has been deleted
    Deleted,
    /// File has been renamed
    Renamed,
    /// File is untracked
    Untracked,
    /// File is ignored
    Ignored,
    /// File has merge conflicts
    Conflicted,
}

impl FileStatus {
    /// Get a single character indicator for the status
    pub fn indicator(&self) -> char {
        match self {
            FileStatus::Unmodified => ' ',
            FileStatus::Added => 'A',
            FileStatus::Modified => 'M',
            FileStatus::Deleted => 'D',
            FileStatus::Renamed => 'R',
            FileStatus::Untracked => '?',
            FileStatus::Ignored => '!',
            FileStatus::Conflicted => 'U',
        }
    }
}

impl From<Status> for FileStatus {
    fn from(status: Status) -> Self {
        if status == Status::CURRENT {
            FileStatus::Unmodified
        } else if status.intersects(Status::INDEX_NEW) {
            FileStatus::Added
        } else if status.intersects(Status::WT_NEW) {
            FileStatus::Untracked
        } else if status.intersects(Status::INDEX_MODIFIED | Status::WT_MODIFIED) {
            FileStatus::Modified
        } else if status.intersects(Status::INDEX_DELETED | Status::WT_DELETED) {
            FileStatus::Deleted
        } else if status.intersects(Status::INDEX_RENAMED | Status::WT_RENAMED) {
            FileStatus::Renamed
        } else if status.intersects(Status::IGNORED) {
            FileStatus::Ignored
        } else if status.intersects(Status::CONFLICTED) {
            FileStatus::Conflicted
        } else {
            FileStatus::Untracked
        }
    }
}

/// A file entry with its status
#[derive(Debug, Clone)]
pub struct StatusEntry {
    /// Path relative to repository root
    pub path: PathBuf,
    /// File status
    pub status: FileStatus,
    /// Whether the change is staged (in index)
    pub staged: bool,
}

/// Information about a commit
#[derive(Debug, Clone)]
pub struct CommitInfo {
    /// Commit SHA (short form)
    pub id: String,
    /// Commit message (first line)
    pub message: String,
    /// Author name
    pub author: String,
    /// Commit timestamp
    pub time: DateTime<Utc>,
}

/// A diff hunk representing a change
#[derive(Debug, Clone)]
pub struct DiffHunk {
    /// Hunk header (@@ -x,y +a,b @@)
    pub header: String,
    /// Lines in this hunk
    pub lines: Vec<DiffLine>,
}

/// A single line in a diff
#[derive(Debug, Clone)]
pub struct DiffLine {
    /// Line origin: '+' (add), '-' (remove), ' ' (context)
    pub origin: char,
    /// Line content
    pub content: String,
    /// Old line number (for context/removed lines)
    pub old_lineno: Option<u32>,
    /// New line number (for context/added lines)
    pub new_lineno: Option<u32>,
}

/// Complete diff for a file
#[derive(Debug, Clone, Default)]
pub struct Diff {
    /// Path of the file
    pub path: PathBuf,
    /// Hunks in this diff
    pub hunks: Vec<DiffHunk>,
}

/// Git repository wrapper with enhanced operations
pub struct GitRepo {
    repo: Repository,
    root: PathBuf,
}

impl std::fmt::Debug for GitRepo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GitRepo").field("root", &self.root).finish()
    }
}

impl GitRepo {
    /// Open a Git repository from the given path
    ///
    /// Discovers the repository by searching upward from the path.
    pub fn open(path: &Path) -> GitResult<Self> {
        let repo = Repository::discover(path)?;
        let root = repo
            .workdir()
            .ok_or_else(|| GitError::Git(git2::Error::from_str("bare repository")))?
            .to_path_buf();

        Ok(GitRepo { repo, root })
    }

    /// Check if a path is within a Git repository
    pub fn is_git_repo(path: &Path) -> bool {
        Repository::discover(path).is_ok()
    }

    /// Get the repository root directory
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Get the current branch name
    pub fn current_branch(&self) -> GitResult<String> {
        let head = self.repo.head()?;
        if head.is_branch() {
            Ok(head.shorthand().unwrap_or("HEAD").to_string())
        } else {
            // Detached HEAD - show short commit hash
            let commit = head.peel_to_commit()?;
            let id = commit.id();
            Ok(format!("{:.7}", id))
        }
    }

    /// Get information about the HEAD commit
    pub fn head_commit(&self) -> GitResult<CommitInfo> {
        let head = self.repo.head()?;
        let commit = head.peel_to_commit()?;
        let author = commit.author();
        let time = Utc
            .timestamp_opt(commit.time().seconds(), 0)
            .single()
            .unwrap_or_else(Utc::now);

        Ok(CommitInfo {
            id: format!("{:.7}", commit.id()),
            message: commit
                .message()
                .unwrap_or("")
                .lines()
                .next()
                .unwrap_or("")
                .to_string(),
            author: author.name().unwrap_or("Unknown").to_string(),
            time,
        })
    }

    /// Get all status entries
    pub fn status(&self) -> GitResult<Vec<StatusEntry>> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        opts.include_ignored(false);
        opts.recurse_untracked_dirs(true);

        let statuses = self.repo.statuses(Some(&mut opts))?;
        let mut entries = Vec::new();

        for entry in statuses.iter() {
            if let Some(path) = entry.path() {
                let status = entry.status();
                let path = PathBuf::from(path);

                // Check if staged (index) or unstaged (worktree)
                let has_index_changes = status.intersects(
                    Status::INDEX_NEW
                        | Status::INDEX_MODIFIED
                        | Status::INDEX_DELETED
                        | Status::INDEX_RENAMED,
                );
                let has_wt_changes = status.intersects(
                    Status::WT_NEW | Status::WT_MODIFIED | Status::WT_DELETED | Status::WT_RENAMED,
                );

                // Add staged entry if applicable
                if has_index_changes {
                    let file_status = if status.intersects(Status::INDEX_NEW) {
                        FileStatus::Added
                    } else if status.intersects(Status::INDEX_MODIFIED) {
                        FileStatus::Modified
                    } else if status.intersects(Status::INDEX_DELETED) {
                        FileStatus::Deleted
                    } else if status.intersects(Status::INDEX_RENAMED) {
                        FileStatus::Renamed
                    } else {
                        FileStatus::Modified
                    };
                    entries.push(StatusEntry {
                        path: path.clone(),
                        status: file_status,
                        staged: true,
                    });
                }

                // Add unstaged entry if applicable
                if has_wt_changes {
                    let file_status = if status.intersects(Status::WT_NEW) {
                        FileStatus::Untracked
                    } else if status.intersects(Status::WT_MODIFIED) {
                        FileStatus::Modified
                    } else if status.intersects(Status::WT_DELETED) {
                        FileStatus::Deleted
                    } else if status.intersects(Status::WT_RENAMED) {
                        FileStatus::Renamed
                    } else {
                        FileStatus::Modified
                    };
                    entries.push(StatusEntry {
                        path,
                        status: file_status,
                        staged: false,
                    });
                }
            }
        }

        Ok(entries)
    }

    /// Get only staged files
    pub fn staged_files(&self) -> GitResult<Vec<StatusEntry>> {
        Ok(self.status()?.into_iter().filter(|e| e.staged).collect())
    }

    /// Get only unstaged (modified but not staged) files
    pub fn unstaged_files(&self) -> GitResult<Vec<StatusEntry>> {
        Ok(self
            .status()?
            .into_iter()
            .filter(|e| !e.staged && e.status != FileStatus::Untracked)
            .collect())
    }

    /// Get only untracked files
    pub fn untracked_files(&self) -> GitResult<Vec<StatusEntry>> {
        Ok(self
            .status()?
            .into_iter()
            .filter(|e| !e.staged && e.status == FileStatus::Untracked)
            .collect())
    }

    /// Stage a file
    pub fn stage_file(&self, path: &Path) -> GitResult<()> {
        let mut index = self.repo.index()?;

        // Check if file exists (for add) or was deleted
        let full_path = self.root.join(path);
        if full_path.exists() {
            index.add_path(path)?;
        } else {
            index.remove_path(path)?;
        }

        index.write()?;
        Ok(())
    }

    /// Unstage a file
    pub fn unstage_file(&self, path: &Path) -> GitResult<()> {
        // Try to get HEAD - if it doesn't exist (no commits yet), just remove from index
        match self.repo.head().and_then(|h| h.peel_to_commit()) {
            Ok(head) => {
                let tree = head.tree()?;
                self.repo.reset_default(Some(&head.into_object()), [path])?;

                // If file doesn't exist in HEAD, remove from index
                if tree.get_path(path).is_err() {
                    let mut index = self.repo.index()?;
                    index.remove_path(path)?;
                    index.write()?;
                }
            }
            Err(_) => {
                // No HEAD commit yet - just remove from index
                let mut index = self.repo.index()?;
                index.remove_path(path)?;
                index.write()?;
            }
        }

        Ok(())
    }

    /// Stage all changes
    pub fn stage_all(&self) -> GitResult<()> {
        let mut index = self.repo.index()?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;
        Ok(())
    }

    /// Unstage all changes
    pub fn unstage_all(&self) -> GitResult<()> {
        let head = match self.repo.head() {
            Ok(h) => Some(h.peel_to_commit()?),
            Err(_) => None, // No commits yet
        };

        if let Some(commit) = head {
            self.repo
                .reset_default(Some(&commit.into_object()), ["*"])?;
        } else {
            // No HEAD, just clear the index
            let mut index = self.repo.index()?;
            index.clear()?;
            index.write()?;
        }

        Ok(())
    }

    /// Discard changes in a file (checkout from index or HEAD)
    pub fn discard_changes(&self, path: &Path) -> GitResult<()> {
        let mut opts = git2::build::CheckoutBuilder::new();
        opts.path(path);
        opts.force();
        self.repo.checkout_head(Some(&mut opts))?;
        Ok(())
    }

    /// Get diff for a file
    pub fn file_diff(&self, path: &Path, staged: bool) -> GitResult<Diff> {
        let mut diff_opts = DiffOptions::new();
        diff_opts.pathspec(path);
        diff_opts.context_lines(3);

        let diff = if staged {
            // Staged: diff between HEAD and index
            let tree = self.repo.head()?.peel_to_tree()?;
            self.repo
                .diff_tree_to_index(Some(&tree), None, Some(&mut diff_opts))?
        } else {
            // Unstaged: diff between index and worktree
            self.repo
                .diff_index_to_workdir(None, Some(&mut diff_opts))?
        };

        let mut result = Diff {
            path: path.to_path_buf(),
            hunks: Vec::new(),
        };

        let mut current_hunk: Option<DiffHunk> = None;

        diff.print(git2::DiffFormat::Patch, |_delta, hunk, line| {
            // Start new hunk if header provided
            if let Some(h) = hunk {
                if let Some(hunk) = current_hunk.take() {
                    result.hunks.push(hunk);
                }
                current_hunk = Some(DiffHunk {
                    header: format!(
                        "@@ -{},{} +{},{} @@",
                        h.old_start(),
                        h.old_lines(),
                        h.new_start(),
                        h.new_lines()
                    ),
                    lines: Vec::new(),
                });
            }

            // Add line to current hunk
            if let Some(ref mut hunk) = current_hunk {
                let origin = line.origin();
                if origin == '+' || origin == '-' || origin == ' ' {
                    let content = std::str::from_utf8(line.content())
                        .unwrap_or("")
                        .to_string();
                    hunk.lines.push(DiffLine {
                        origin,
                        content,
                        old_lineno: line.old_lineno(),
                        new_lineno: line.new_lineno(),
                    });
                }
            }

            true
        })?;

        // Push final hunk
        if let Some(hunk) = current_hunk {
            result.hunks.push(hunk);
        }

        Ok(result)
    }

    /// Check if there are any staged changes
    pub fn has_staged_changes(&self) -> GitResult<bool> {
        Ok(!self.staged_files()?.is_empty())
    }

    /// Check if there are any unstaged changes
    pub fn has_unstaged_changes(&self) -> GitResult<bool> {
        let entries = self.status()?;
        Ok(entries.iter().any(|e| !e.staged))
    }

    /// Get a status summary string
    pub fn status_summary(&self) -> GitResult<String> {
        let staged = self.staged_files()?.len();
        let unstaged = self.unstaged_files()?.len();
        let untracked = self.untracked_files()?.len();

        if staged == 0 && unstaged == 0 && untracked == 0 {
            return Ok("Working tree clean".to_string());
        }

        let mut parts = Vec::new();
        if staged > 0 {
            parts.push(format!("+{} staged", staged));
        }
        if unstaged > 0 {
            parts.push(format!("~{} modified", unstaged));
        }
        if untracked > 0 {
            parts.push(format!("?{} untracked", untracked));
        }

        Ok(parts.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn init_test_repo() -> (TempDir, GitRepo) {
        let temp = TempDir::new().expect("create temp dir");
        let repo = Repository::init(temp.path()).expect("init repo");

        // Configure user for commits
        let mut config = repo.config().expect("get config");
        config.set_str("user.name", "Test").expect("set name");
        config
            .set_str("user.email", "test@test.com")
            .expect("set email");

        drop(repo);

        let git_repo = GitRepo::open(temp.path()).expect("open repo");
        (temp, git_repo)
    }

    #[test]
    fn test_is_git_repo() {
        let temp = TempDir::new().unwrap();
        assert!(!GitRepo::is_git_repo(temp.path()));

        Repository::init(temp.path()).unwrap();
        assert!(GitRepo::is_git_repo(temp.path()));
    }

    #[test]
    fn test_file_status_indicator() {
        assert_eq!(FileStatus::Added.indicator(), 'A');
        assert_eq!(FileStatus::Modified.indicator(), 'M');
        assert_eq!(FileStatus::Deleted.indicator(), 'D');
        assert_eq!(FileStatus::Untracked.indicator(), '?');
    }

    #[test]
    fn test_status_empty_repo() {
        let (_temp, repo) = init_test_repo();
        let status = repo.status().expect("get status");
        assert!(status.is_empty());
    }

    #[test]
    fn test_status_untracked_file() {
        let (temp, repo) = init_test_repo();

        // Create untracked file
        fs::write(temp.path().join("test.txt"), "hello").expect("write file");

        let untracked = repo.untracked_files().expect("get untracked");
        assert_eq!(untracked.len(), 1);
        assert_eq!(untracked[0].path, PathBuf::from("test.txt"));
        assert_eq!(untracked[0].status, FileStatus::Untracked);
    }

    #[test]
    fn test_stage_file() {
        let (temp, repo) = init_test_repo();

        // Create and stage file
        fs::write(temp.path().join("test.txt"), "hello").expect("write file");
        repo.stage_file(Path::new("test.txt")).expect("stage file");

        let staged = repo.staged_files().expect("get staged");
        assert_eq!(staged.len(), 1);
        assert_eq!(staged[0].path, PathBuf::from("test.txt"));
        assert!(staged[0].staged);
    }

    #[test]
    fn test_unstage_file() {
        let (temp, repo) = init_test_repo();

        // Create, stage, then unstage
        fs::write(temp.path().join("test.txt"), "hello").expect("write file");
        repo.stage_file(Path::new("test.txt")).expect("stage");

        let staged = repo.staged_files().expect("get staged");
        assert_eq!(staged.len(), 1);

        repo.unstage_file(Path::new("test.txt")).expect("unstage");

        let staged = repo.staged_files().expect("get staged after unstage");
        assert!(staged.is_empty());
    }

    #[test]
    fn test_stage_all() {
        let (temp, repo) = init_test_repo();

        // Create multiple files
        fs::write(temp.path().join("a.txt"), "a").expect("write a");
        fs::write(temp.path().join("b.txt"), "b").expect("write b");

        repo.stage_all().expect("stage all");

        let staged = repo.staged_files().expect("get staged");
        assert_eq!(staged.len(), 2);
    }

    #[test]
    fn test_status_summary_clean() {
        let (_temp, repo) = init_test_repo();
        let summary = repo.status_summary().expect("get summary");
        assert_eq!(summary, "Working tree clean");
    }

    #[test]
    fn test_status_summary_with_changes() {
        let (temp, repo) = init_test_repo();

        fs::write(temp.path().join("test.txt"), "hello").expect("write file");

        let summary = repo.status_summary().expect("get summary");
        assert!(summary.contains("untracked"));
    }
}
