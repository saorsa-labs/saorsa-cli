//! Disk analysis functionality
//!
//! Provides utilities for analyzing disk usage, finding large files,
//! and identifying stale files.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use humansize::{format_size, BINARY};
use sysinfo::Disks;
use walkdir::WalkDir;

/// A file entry with size and metadata
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// Path to the file
    pub path: PathBuf,
    /// Size in bytes
    pub size: u64,
    /// Last modified time
    pub modified: Option<SystemTime>,
    /// Last accessed time
    pub accessed: Option<SystemTime>,
}

impl FileEntry {
    /// Format the file size as a human-readable string
    #[must_use]
    pub fn format_size(&self) -> String {
        format_size(self.size, BINARY)
    }

    /// Get the filename (last component of path)
    #[must_use]
    pub fn filename(&self) -> &str {
        self.path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("(unknown)")
    }
}

/// Disk information for a mount point
#[derive(Debug, Clone)]
pub struct DiskInfo {
    /// Mount point path
    pub mount_point: PathBuf,
    /// Filesystem name
    pub name: String,
    /// Total space in bytes
    pub total: u64,
    /// Used space in bytes
    pub used: u64,
    /// Available space in bytes
    pub available: u64,
}

impl DiskInfo {
    /// Get usage percentage (0.0 to 100.0)
    #[must_use]
    pub fn usage_percent(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (self.used as f64 / self.total as f64) * 100.0
    }

    /// Format bytes as human-readable string
    #[must_use]
    pub fn format_bytes(bytes: u64) -> String {
        format_size(bytes, BINARY)
    }

    /// Get a short display name for the disk
    #[must_use]
    pub fn display_name(&self) -> String {
        if self.name.is_empty() {
            self.mount_point.display().to_string()
        } else {
            format!("{} ({})", self.name, self.mount_point.display())
        }
    }
}

/// Disk analyzer for scanning and analyzing files
pub struct DiskAnalyzer {
    root: PathBuf,
}

impl DiskAnalyzer {
    /// Create a new disk analyzer for the given root path
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Get the root path being analyzed
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Get disk information for all mounted disks
    #[must_use]
    pub fn get_disk_info() -> Vec<DiskInfo> {
        let disks = Disks::new_with_refreshed_list();
        disks
            .iter()
            .map(|d| DiskInfo {
                mount_point: d.mount_point().to_path_buf(),
                name: d.name().to_string_lossy().to_string(),
                total: d.total_space(),
                used: d.total_space().saturating_sub(d.available_space()),
                available: d.available_space(),
            })
            .collect()
    }

    /// Find the N largest files in the root directory
    ///
    /// # Arguments
    ///
    /// * `count` - Maximum number of files to return
    ///
    /// # Returns
    ///
    /// A vector of `FileEntry` sorted by size (largest first)
    #[must_use]
    pub fn find_largest(&self, count: usize) -> Vec<FileEntry> {
        let mut entries = Vec::new();

        for entry in WalkDir::new(&self.root).into_iter().filter_map(Result::ok) {
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
    ///
    /// # Arguments
    ///
    /// * `days` - Minimum age in days
    /// * `count` - Maximum number of files to return
    ///
    /// # Returns
    ///
    /// A vector of `FileEntry` sorted by access time (oldest first)
    #[must_use]
    pub fn find_stale(&self, days: u64, count: usize) -> Vec<FileEntry> {
        let cutoff = SystemTime::now()
            .checked_sub(std::time::Duration::from_secs(days * 24 * 60 * 60))
            .unwrap_or(SystemTime::UNIX_EPOCH);

        let mut entries = Vec::new();

        for entry in WalkDir::new(&self.root).into_iter().filter_map(Result::ok) {
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

    /// Count total files and calculate total size
    ///
    /// # Returns
    ///
    /// A tuple of (file_count, total_size_bytes)
    #[must_use]
    pub fn count_files(&self) -> (usize, u64) {
        let mut count = 0usize;
        let mut total_size = 0u64;

        for entry in WalkDir::new(&self.root).into_iter().filter_map(Result::ok) {
            if entry.file_type().is_file() {
                if let Ok(meta) = entry.metadata() {
                    count += 1;
                    total_size += meta.len();
                }
            }
        }

        (count, total_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_disk_info() {
        let info = DiskAnalyzer::get_disk_info();
        // Should have at least one disk on any system
        assert!(!info.is_empty());

        // Check first disk has valid data
        let first = &info[0];
        assert!(first.total > 0);
        assert!(first.usage_percent() >= 0.0);
        assert!(first.usage_percent() <= 100.0);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(DiskInfo::format_bytes(0), "0 B");
        assert_eq!(DiskInfo::format_bytes(1024), "1 KiB");
        assert_eq!(DiskInfo::format_bytes(1024 * 1024), "1 MiB");
    }

    #[test]
    fn test_find_largest() {
        let dir = tempdir().expect("create temp dir");

        // Create some test files
        let mut f1 = File::create(dir.path().join("small.txt")).expect("create small");
        f1.write_all(b"small").expect("write small");

        let mut f2 = File::create(dir.path().join("large.txt")).expect("create large");
        f2.write_all(&vec![0u8; 1000]).expect("write large");

        let analyzer = DiskAnalyzer::new(dir.path());
        let largest = analyzer.find_largest(10);

        assert_eq!(largest.len(), 2);
        assert_eq!(largest[0].filename(), "large.txt");
        assert_eq!(largest[1].filename(), "small.txt");
    }

    #[test]
    fn test_count_files() {
        let dir = tempdir().expect("create temp dir");

        // Create test files
        File::create(dir.path().join("a.txt")).expect("create a");
        File::create(dir.path().join("b.txt")).expect("create b");
        fs::create_dir(dir.path().join("subdir")).expect("create subdir");
        File::create(dir.path().join("subdir/c.txt")).expect("create c");

        let analyzer = DiskAnalyzer::new(dir.path());
        let (count, _size) = analyzer.count_files();

        assert_eq!(count, 3);
    }

    #[test]
    fn test_file_entry_format() {
        let entry = FileEntry {
            path: PathBuf::from("/test/file.txt"),
            size: 1024 * 1024, // 1 MiB
            modified: None,
            accessed: None,
        };

        assert_eq!(entry.format_size(), "1 MiB");
        assert_eq!(entry.filename(), "file.txt");
    }
}
