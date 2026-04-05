use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use console::style;
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use humansize::{format_size, BINARY};
use indicatif::{ProgressBar, ProgressStyle};
use sysinfo::Disks;
use walkdir::WalkDir;

mod error;
use error::SdiskError;

/// sdisk: Analyze disk usage and suggest cleanups
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Root path to analyze (defaults to current directory)
    #[arg(global = true, short, long)]
    path: Option<PathBuf>,

    /// Minimum days since last access to consider stale
    #[arg(global = true, long, default_value_t = 90)]
    stale_days: u64,

    /// Run non-interactively (no selection UI)
    #[arg(global = true, long)]
    non_interactive: bool,
    /// Assume yes for confirmations (non-interactive)
    #[arg(global = true, long)]
    yes: bool,
    /// Dry run: show what would be removed
    #[arg(global = true, long)]
    dry_run: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show disk overview (total/free/used per mount)
    Info,
    /// Analyze directories by size (top N)
    Top {
        /// Number of entries to show
        #[arg(short, long, default_value_t = 20)]
        count: usize,
        /// Optional paths to analyze (defaults to CWD if none and no --path)
        #[arg(value_name = "PATH")]
        paths: Vec<PathBuf>,
    },
    /// List stale files/dirs older than --stale-days
    Stale {
        /// Show at most N items
        #[arg(short, long, default_value_t = 100)]
        limit: usize,
        /// Optional paths to analyze (defaults to CWD if none and no --path)
        #[arg(value_name = "PATH")]
        paths: Vec<PathBuf>,
    },
    /// Remove stale items after confirmation
    Clean {
        /// Show at most N candidates
        #[arg(short, long, default_value_t = 100)]
        limit: usize,
        /// Optional paths to analyze (defaults to CWD if none and no --path)
        #[arg(value_name = "PATH")]
        paths: Vec<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command.unwrap_or(Commands::Info) {
        Commands::Info => cmd_info(),
        Commands::Top { count, paths } => {
            let roots = collect_roots(cli.path, paths)?;
            cmd_top(roots, count, !cli.non_interactive, cli.yes, cli.dry_run)
        }
        Commands::Stale { limit, paths } | Commands::Clean { limit, paths } => {
            let roots = collect_roots(cli.path, paths)?;
            cmd_stale(
                roots,
                cli.stale_days,
                limit,
                !cli.non_interactive,
                !cli.yes,
                cli.dry_run,
            )
        }
    }
}

fn cmd_info() -> Result<()> {
    let disks = Disks::new_with_refreshed_list();
    println!("{}", style("Disk overview").bold());
    for disk in disks.list() {
        let name = disk.name().to_string_lossy();
        let total = format_size(disk.total_space(), BINARY);
        let avail = format_size(disk.available_space(), BINARY);
        let used = format_size(disk.total_space() - disk.available_space(), BINARY);
        println!(
            "- {} total: {}, used: {}, free: {}",
            name, total, used, avail
        );
    }
    Ok(())
}

fn collect_roots(opt_root: Option<PathBuf>, extra: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Some(r) = opt_root {
        roots.push(canonicalize_root(&r)?);
    }
    for p in extra {
        let canonical = canonicalize_root(&p)?;
        if !roots.contains(&canonical) {
            roots.push(canonical);
        }
    }
    if roots.is_empty() {
        roots.push(canonicalize_root(&std::env::current_dir()?)?);
    }
    Ok(roots)
}

#[derive(Debug, Clone)]
struct DeleteTarget {
    canonical_path: PathBuf,
    is_dir: bool,
}

#[derive(Debug, Clone)]
struct DeletionGuard {
    allowed_roots: Vec<PathBuf>,
}

impl DeletionGuard {
    fn new(roots: &[PathBuf]) -> Result<Self> {
        let mut allowed_roots = Vec::new();
        for root in roots {
            let canonical = canonicalize_root(root)?;
            if !allowed_roots.contains(&canonical) {
                allowed_roots.push(canonical);
            }
        }
        Ok(Self { allowed_roots })
    }

    fn describe_prompt(&self, item_count: usize, dir_count: usize) -> String {
        if dir_count > 0 {
            format!(
                "Delete {item_count} selected items including {dir_count} director{} recursively?",
                if dir_count == 1 { "y" } else { "ies" }
            )
        } else {
            format!("Delete {item_count} selected file(s)?")
        }
    }

    fn is_allowed_candidate(&self, path: &Path) -> bool {
        self.validate(path).is_ok()
    }

    fn delete(&self, path: &Path) -> Result<()> {
        let target = self.validate(path)?;
        if target.is_dir {
            std::fs::remove_dir_all(&target.canonical_path).with_context(|| {
                format!("removing directory {}", target.canonical_path.display())
            })?;
        } else {
            std::fs::remove_file(&target.canonical_path)
                .with_context(|| format!("removing file {}", target.canonical_path.display()))?;
        }
        Ok(())
    }

    fn validate(&self, path: &Path) -> Result<DeleteTarget> {
        let metadata = std::fs::symlink_metadata(path)
            .with_context(|| format!("inspecting delete target {}", path.display()))?;
        if metadata.file_type().is_symlink() {
            return Err(anyhow!(
                "refusing to delete symlink target {}",
                path.display()
            ));
        }

        let canonical_path = path
            .canonicalize()
            .with_context(|| format!("resolving delete target {}", path.display()))?;

        if self.allowed_roots.contains(&canonical_path) {
            return Err(anyhow!(
                "refusing to delete scan root {}",
                canonical_path.display()
            ));
        }

        if !self
            .allowed_roots
            .iter()
            .any(|root| canonical_path.starts_with(root))
        {
            return Err(anyhow!(
                "refusing to delete path outside scan roots: {}",
                canonical_path.display()
            ));
        }

        Ok(DeleteTarget {
            canonical_path,
            is_dir: metadata.is_dir(),
        })
    }
}

fn canonicalize_root(path: &Path) -> Result<PathBuf> {
    path.canonicalize()
        .with_context(|| format!("resolving scan root {}", path.display()))
}

fn cmd_top(
    roots: Vec<PathBuf>,
    count: usize,
    interactive: bool,
    yes: bool,
    dry_run: bool,
) -> Result<()> {
    let delete_guard = DeletionGuard::new(&roots)?;
    for root in &roots {
        println!("{} {}", style("Scanning").bold(), root.display());
    }
    let pb = spinner().context("Failed to create progress bar")?;
    pb.set_message("Scanning directories...");
    let mut entries: Vec<(PathBuf, u64)> = Vec::new();
    for root in &roots {
        for entry in WalkDir::new(root)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Ok(meta) = path.metadata() {
                    entries.push((path.to_path_buf(), meta.len()));
                }
            }
        }
    }
    pb.finish_and_clear();
    entries.sort_by_key(|(_, size)| std::cmp::Reverse(*size));
    let entries: Vec<(PathBuf, u64)> = entries.into_iter().take(count).collect();
    for (i, (path, size)) in entries.iter().enumerate() {
        println!(
            "{:>3}. {} — {}",
            i + 1,
            format_size(*size, BINARY),
            path.display()
        );
    }
    if interactive && !entries.is_empty() {
        let items: Vec<String> = entries
            .iter()
            .map(|(p, s)| format!("{} — {}", format_size(*s, BINARY), p.display()))
            .collect();
        let theme = ColorfulTheme::default();
        let selection = MultiSelect::with_theme(&theme)
            .with_prompt("Select files to delete (space to toggle, enter to confirm)")
            .items(&items)
            .interact()?;
        if selection.is_empty() {
            return Ok(());
        }
        if dry_run {
            println!("Would remove:");
            for idx in selection {
                println!("- {}", entries[idx].0.display());
            }
            return Ok(());
        }
        if !yes && !confirm(&delete_guard.describe_prompt(selection.len(), 0))? {
            println!("Aborted.");
            return Ok(());
        }
        for idx in selection {
            let path = &entries[idx].0;
            delete_guard.delete(path)?;
            println!("Removed {}", path.display());
        }
    }
    Ok(())
}

fn cmd_stale(
    roots: Vec<PathBuf>,
    days: u64,
    limit: usize,
    interactive: bool,
    prompt: bool,
    dry_run: bool,
) -> Result<()> {
    let delete_guard = DeletionGuard::new(&roots)?;
    use std::time::{Duration, SystemTime};

    let cutoff = SystemTime::now() - Duration::from_secs(days * 24 * 60 * 60);
    for root in &roots {
        println!(
            "{} {} (older than {} days)",
            style("Finding stale items in").bold(),
            root.display(),
            days
        );
    }
    let pb = spinner().context("Failed to create progress bar")?;
    pb.set_message("Finding stale files...");
    let mut items: Vec<(PathBuf, u64, SystemTime)> = Vec::new();
    for root in &roots {
        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path().to_path_buf();
            if let Ok(meta) = path.symlink_metadata() {
                // Prefer last access; fall back to modified
                let time = meta
                    .accessed()
                    .ok()
                    .or_else(|| meta.modified().ok())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                if time <= cutoff && delete_guard.is_allowed_candidate(&path) {
                    let size = if meta.is_file() {
                        meta.len()
                    } else {
                        dir_size(&path).unwrap_or(0)
                    };
                    items.push((path, size, time));
                }
            }
        }
    }
    pb.finish_and_clear();
    // Largest first
    items.sort_by_key(|(_, size, _)| std::cmp::Reverse(*size));
    let items = items.into_iter().take(limit).collect::<Vec<_>>();
    for (i, (path, size, time)) in items.iter().enumerate() {
        let age_days = SystemTime::now()
            .duration_since(*time)
            .unwrap_or_default()
            .as_secs()
            / 86400;
        println!(
            "{:>3}. {} — {} — {} days old",
            i + 1,
            format_size(*size, BINARY),
            path.display(),
            age_days
        );
    }

    if dry_run || items.is_empty() {
        return Ok(());
    }

    if interactive {
        let labels: Vec<String> = items
            .iter()
            .map(|(p, s, t)| {
                let age_days = std::time::SystemTime::now()
                    .duration_since(*t)
                    .unwrap_or_default()
                    .as_secs()
                    / 86400;
                format!(
                    "{} — {} — {} days",
                    format_size(*s, BINARY),
                    p.display(),
                    age_days
                )
            })
            .collect();
        let theme = ColorfulTheme::default();
        let selection = MultiSelect::with_theme(&theme)
            .with_prompt("Select items to delete (space to toggle, enter to confirm)")
            .items(&labels)
            .interact()?;
        if selection.is_empty() {
            return Ok(());
        }
        if dry_run {
            println!("Would remove:");
            for idx in selection {
                println!("- {}", items[idx].0.display());
            }
            return Ok(());
        }
        let dir_count = selection
            .iter()
            .filter(|&&idx| items[idx].0.is_dir())
            .count();
        if !confirm(&delete_guard.describe_prompt(selection.len(), dir_count))? {
            println!("Aborted.");
            return Ok(());
        }
        for idx in selection {
            let path = &items[idx].0;
            delete_guard.delete(path)?;
            println!("Removed {}", path.display());
        }
        return Ok(());
    }

    let dir_count = items.iter().filter(|(path, _, _)| path.is_dir()).count();
    if prompt && !confirm(&delete_guard.describe_prompt(items.len(), dir_count))? {
        println!("Aborted.");
        return Ok(());
    }

    for (path, _, _) in items {
        delete_guard.delete(&path)?;
        println!("Removed {}", path.display());
    }

    Ok(())
}

fn spinner() -> Result<ProgressBar> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner} {msg}").map_err(|e| {
            SdiskError::progress_bar(format!("Failed to create progress bar style: {}", e))
        })?,
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    Ok(pb)
}

fn dir_size(path: &PathBuf) -> Result<u64> {
    let mut size: u64 = 0;
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();
        if p.is_file() {
            if let Ok(meta) = p.metadata() {
                size = size.saturating_add(meta.len());
            }
        }
    }
    Ok(size)
}

fn confirm(prompt: &str) -> Result<bool> {
    use std::io::{self, Write};
    print!("{} [y/N] ", prompt);
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).context("reading input")?;
    let trimmed = input.trim().to_lowercase();
    Ok(trimmed == "y" || trimmed == "yes")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn deletion_guard_rejects_scan_root() {
        let root = tempdir().expect("tempdir");
        let guard = DeletionGuard::new(&[root.path().to_path_buf()]).expect("guard");

        let err = guard
            .validate(root.path())
            .expect_err("scan root must not be deletable");
        assert!(err.to_string().contains("scan root"));
    }

    #[test]
    fn deletion_guard_allows_nested_file() {
        let root = tempdir().expect("tempdir");
        let file = root.path().join("cache.log");
        std::fs::write(&file, "cache").expect("write file");
        let guard = DeletionGuard::new(&[root.path().to_path_buf()]).expect("guard");

        let target = guard.validate(&file).expect("nested file should validate");
        assert!(!target.is_dir);
        assert_eq!(
            target.canonical_path,
            file.canonicalize().expect("canonical path")
        );
    }

    #[cfg(unix)]
    #[test]
    fn deletion_guard_rejects_symlinks() {
        use std::os::unix::fs::symlink;

        let root = tempdir().expect("tempdir");
        let target = root.path().join("target.txt");
        let link = root.path().join("target.link");
        std::fs::write(&target, "target").expect("write target");
        symlink(&target, &link).expect("symlink");

        let guard = DeletionGuard::new(&[root.path().to_path_buf()]).expect("guard");
        let err = guard.validate(&link).expect_err("symlink must be rejected");
        assert!(err.to_string().contains("symlink"));
    }
}
