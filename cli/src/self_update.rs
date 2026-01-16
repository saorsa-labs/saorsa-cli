//! Self-update functionality for the saorsa CLI binary.
//!
//! Provides safe binary replacement with backup and platform-specific
//! restart mechanisms (Unix exec, Windows spawn).

use crate::downloader::Downloader;
use crate::platform::Platform;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SelfUpdateError {
    #[error("Failed to get current executable path: {0}")]
    CurrentExePath(std::io::Error),
    #[error("Download failed: {0}")]
    Download(#[source] anyhow::Error),
    #[error("Failed to replace binary: {0}")]
    Replace(String),
    #[error("Restart failed: {0}")]
    Restart(String),
}

/// Result of a self-update operation.
pub struct SelfUpdateResult {
    /// Path to the updated binary
    #[allow(dead_code)] // Available for future use
    pub binary_path: PathBuf,
    /// Path to the backup of the old binary
    #[allow(dead_code)] // Available for rollback
    pub backup_path: PathBuf,
    /// Whether restart is required
    pub needs_restart: bool,
}

/// Download and install update for the CLI itself.
pub async fn perform_self_update(
    downloader: &Downloader,
    platform: &Platform,
) -> Result<SelfUpdateResult, SelfUpdateError> {
    let current_exe = std::env::current_exe().map_err(SelfUpdateError::CurrentExePath)?;

    // Download new binary to cache
    println!("Downloading update...");
    let new_binary = downloader
        .download_binary("saorsa", platform, true)
        .await
        .map_err(SelfUpdateError::Download)?;

    // Create backup path
    let backup_path = current_exe.with_extension("old");

    // Perform atomic replacement
    replace_binary(&current_exe, &new_binary, &backup_path)?;

    println!("Update installed successfully!");

    Ok(SelfUpdateResult {
        binary_path: current_exe,
        backup_path,
        needs_restart: true,
    })
}

/// Replace the current binary with the new one.
fn replace_binary(
    current: &PathBuf,
    new_binary: &PathBuf,
    backup: &PathBuf,
) -> Result<(), SelfUpdateError> {
    // Remove old backup if exists
    if backup.exists() {
        fs::remove_file(backup).ok();
    }

    // Rename current to backup
    fs::rename(current, backup)
        .map_err(|e| SelfUpdateError::Replace(format!("Failed to backup current binary: {}", e)))?;

    // Copy new binary to current location
    fs::copy(new_binary, current).map_err(|e| {
        // Try to restore from backup on failure
        if let Err(restore_err) = fs::rename(backup, current) {
            return SelfUpdateError::Replace(format!(
                "Failed to install and restore: {} / {}",
                e, restore_err
            ));
        }
        SelfUpdateError::Replace(format!("Failed to install new binary: {}", e))
    })?;

    // Set executable permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(current)
            .map_err(|e| SelfUpdateError::Replace(e.to_string()))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(current, perms).map_err(|e| SelfUpdateError::Replace(e.to_string()))?;
    }

    Ok(())
}

/// Restart the application with the new binary (Unix implementation).
#[cfg(unix)]
pub fn restart() -> Result<(), SelfUpdateError> {
    use std::os::unix::process::CommandExt;

    let current_exe = std::env::current_exe().map_err(SelfUpdateError::CurrentExePath)?;
    let args: Vec<String> = std::env::args().collect();

    // exec replaces current process entirely
    let err = std::process::Command::new(&current_exe)
        .args(&args[1..])
        .exec();

    // exec only returns on error
    Err(SelfUpdateError::Restart(format!("exec failed: {}", err)))
}

/// Restart the application with the new binary (Windows implementation).
#[cfg(windows)]
pub fn restart() -> Result<(), SelfUpdateError> {
    let current_exe = std::env::current_exe().map_err(SelfUpdateError::CurrentExePath)?;
    let args: Vec<String> = std::env::args().collect();

    // Spawn new process
    std::process::Command::new(&current_exe)
        .args(&args[1..])
        .spawn()
        .map_err(|e| SelfUpdateError::Restart(format!("spawn failed: {}", e)))?;

    // Exit current process
    std::process::exit(0);
}

/// Rollback to the backup if update failed.
#[allow(dead_code)] // Available for manual recovery
pub fn rollback(backup: &PathBuf) -> Result<(), SelfUpdateError> {
    let current_exe = std::env::current_exe().map_err(SelfUpdateError::CurrentExePath)?;

    if backup.exists() {
        // Remove failed new binary
        fs::remove_file(&current_exe).ok();
        // Restore backup
        fs::rename(backup, &current_exe)
            .map_err(|e| SelfUpdateError::Replace(format!("Rollback failed: {}", e)))?;
        println!("Rolled back to previous version.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_replace_binary_success() {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let current = dir.path().join("current");
        let new_binary = dir.path().join("new");
        let backup = dir.path().join("current.old");

        // Create test files
        File::create(&current)
            .expect("create current")
            .write_all(b"old")
            .expect("write old");
        File::create(&new_binary)
            .expect("create new")
            .write_all(b"new")
            .expect("write new");

        // Perform replacement
        replace_binary(&current, &new_binary, &backup).expect("replace should succeed");

        // Verify
        assert!(current.exists());
        assert!(backup.exists());
        assert_eq!(fs::read_to_string(&current).expect("read current"), "new");
        assert_eq!(fs::read_to_string(&backup).expect("read backup"), "old");
    }

    #[test]
    fn test_replace_binary_removes_old_backup() {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let current = dir.path().join("current");
        let new_binary = dir.path().join("new");
        let backup = dir.path().join("current.old");

        // Create test files including old backup
        File::create(&current)
            .expect("create current")
            .write_all(b"current")
            .expect("write");
        File::create(&new_binary)
            .expect("create new")
            .write_all(b"new")
            .expect("write");
        File::create(&backup)
            .expect("create backup")
            .write_all(b"very_old")
            .expect("write");

        // Perform replacement
        replace_binary(&current, &new_binary, &backup).expect("replace should succeed");

        // Verify old backup was replaced
        assert_eq!(fs::read_to_string(&backup).expect("read backup"), "current");
    }

    #[test]
    fn test_self_update_error_display() {
        let err = SelfUpdateError::Replace("test error".to_string());
        assert_eq!(err.to_string(), "Failed to replace binary: test error");

        let err = SelfUpdateError::Restart("exec failed".to_string());
        assert_eq!(err.to_string(), "Restart failed: exec failed");
    }
}
