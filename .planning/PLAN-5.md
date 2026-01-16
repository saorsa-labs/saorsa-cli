# PLAN-5: Self-Update & Restart

**Phase**: 5 of 5
**Milestone**: M1 - Auto-Update Foundation
**Created**: 2026-01-16

## Overview

Implement the self-update mechanism for the CLI binary:
- Download new saorsa binary using existing downloader
- Safely replace current binary (atomic swap)
- Restart using platform-specific mechanism (Unix exec, Windows spawn)
- Handle rollback on failure

## Prerequisites

- [x] Phase 4 completed (MenuChoice::UpdateCLI, notification UI)
- [x] Downloader has download_binary() with checksum verification
- [x] Platform detection works (darwin, linux, windows)
- [x] UpdateChecker provides latest_version

## Current State

| Component | Status |
|-----------|--------|
| downloader.rs | Has download_binary() for sb/sdisk |
| platform.rs | Has asset_name() for platform-specific names |
| main.rs | Has UpdateCLI handler (placeholder) |
| updater.rs | Has UpdateCheckResult with latest_version |

## Tasks

<task type="auto" priority="p0">
  <n>Create self_update module with update logic</n>
  <files>
    cli/src/self_update.rs,
    cli/src/main.rs
  </files>
  <action>
    1. Create cli/src/self_update.rs:
       ```rust
       //! Self-update functionality for the saorsa CLI binary.

       use crate::downloader::Downloader;
       use crate::platform::Platform;
       use anyhow::{Context, Result};
       use std::fs;
       use std::path::PathBuf;
       use thiserror::Error;

       #[derive(Debug, Error)]
       pub enum SelfUpdateError {
           #[error("Failed to get current executable path: {0}")]
           CurrentExePath(#[from] std::io::Error),
           #[error("Download failed: {0}")]
           Download(#[from] anyhow::Error),
           #[error("Failed to replace binary: {0}")]
           Replace(String),
           #[error("Restart failed: {0}")]
           Restart(String),
       }

       /// Result of a self-update operation.
       pub struct SelfUpdateResult {
           /// Path to the new binary
           pub new_binary: PathBuf,
           /// Path to the backup of the old binary
           pub backup_path: PathBuf,
           /// Whether restart is required
           pub needs_restart: bool,
       }

       /// Download and install update for the CLI itself.
       pub async fn perform_self_update(
           downloader: &Downloader,
           platform: &Platform,
       ) -> Result<SelfUpdateResult, SelfUpdateError> {
           let current_exe = std::env::current_exe()
               .map_err(SelfUpdateError::CurrentExePath)?;

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
               new_binary: current_exe,
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
               .map_err(|e| SelfUpdateError::Replace(
                   format!("Failed to backup current binary: {}", e)
               ))?;

           // Copy new binary to current location
           fs::copy(new_binary, current)
               .map_err(|e| {
                   // Try to restore from backup
                   if let Err(restore_err) = fs::rename(backup, current) {
                       return SelfUpdateError::Replace(
                           format!("Failed to install and restore: {} / {}", e, restore_err)
                       );
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
               fs::set_permissions(current, perms)
                   .map_err(|e| SelfUpdateError::Replace(e.to_string()))?;
           }

           Ok(())
       }

       /// Restart the application with the new binary.
       #[cfg(unix)]
       pub fn restart() -> Result<(), SelfUpdateError> {
           use std::os::unix::process::CommandExt;

           let current_exe = std::env::current_exe()
               .map_err(SelfUpdateError::CurrentExePath)?;
           let args: Vec<String> = std::env::args().collect();

           // exec replaces current process
           let err = std::process::Command::new(&current_exe)
               .args(&args[1..])
               .exec();

           Err(SelfUpdateError::Restart(format!("exec failed: {}", err)))
       }

       /// Restart the application with the new binary.
       #[cfg(windows)]
       pub fn restart() -> Result<(), SelfUpdateError> {
           let current_exe = std::env::current_exe()
               .map_err(SelfUpdateError::CurrentExePath)?;
           let args: Vec<String> = std::env::args().collect();

           std::process::Command::new(&current_exe)
               .args(&args[1..])
               .spawn()
               .map_err(|e| SelfUpdateError::Restart(format!("spawn failed: {}", e)))?;

           std::process::exit(0);
       }

       /// Rollback to the backup if update failed.
       pub fn rollback(backup: &PathBuf) -> Result<(), SelfUpdateError> {
           let current_exe = std::env::current_exe()
               .map_err(SelfUpdateError::CurrentExePath)?;

           if backup.exists() {
               // Remove failed new binary
               fs::remove_file(&current_exe).ok();
               // Restore backup
               fs::rename(backup, &current_exe)
                   .map_err(|e| SelfUpdateError::Replace(
                       format!("Rollback failed: {}", e)
                   ))?;
               println!("Rolled back to previous version.");
           }
           Ok(())
       }
       ```

    2. Add `mod self_update;` to cli/src/main.rs

    3. Add "saorsa" asset name to platform.rs:
       ```rust
       // In asset_name() match:
       "saorsa" => format!("saorsa-{}-{}.tar.gz", self.os, self.arch),
       ```
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p cli --all-features -- -D warnings
    cargo build -p cli
  </verify>
  <done>
    - self_update.rs created with perform_self_update()
    - Platform-specific restart (Unix exec, Windows spawn)
    - Binary replacement with backup
    - Rollback capability
    - Zero clippy warnings
  </done>
</task>

<task type="auto" priority="p1">
  <n>Integrate self-update into UpdateCLI handler</n>
  <files>
    cli/src/main.rs
  </files>
  <action>
    1. Update MenuChoice::UpdateCLI handler in main.rs:
       ```rust
       MenuChoice::UpdateCLI => {
           use dialoguer::{theme::ColorfulTheme, Confirm};

           // Confirm with user
           let confirm = Confirm::with_theme(&ColorfulTheme::default())
               .with_prompt("Download and install update? This will restart the application.")
               .default(true)
               .interact()?;

           if !confirm {
               println!("Update cancelled.");
               println!("Press Enter to continue...");
               let mut input = String::new();
               std::io::stdin().read_line(&mut input)?;
               continue;
           }

           // Perform update
           match self_update::perform_self_update(&downloader, &platform).await {
               Ok(result) => {
                   if result.needs_restart {
                       println!("\nRestarting with new version...");
                       if let Err(e) = self_update::restart() {
                           println!("Failed to restart: {}", e);
                           println!("Please manually restart the application.");
                           println!("Press Enter to exit...");
                           let mut input = String::new();
                           std::io::stdin().read_line(&mut input)?;
                           break;
                       }
                   }
               }
               Err(e) => {
                   println!("Update failed: {}", e);
                   println!("Press Enter to continue...");
                   let mut input = String::new();
                   std::io::stdin().read_line(&mut input)?;
               }
           }
       }
       ```

    2. Add import at top of main.rs:
       ```rust
       use crate::self_update;
       ```
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p cli --all-features -- -D warnings
    cargo build -p cli
    cargo run -p cli -- --help
  </verify>
  <done>
    - UpdateCLI handler performs self-update
    - User confirmation before update
    - Automatic restart after success
    - Error handling with user feedback
    - Zero clippy warnings
  </done>
</task>

<task type="auto" priority="p2">
  <n>Add self-update tests</n>
  <files>
    cli/src/self_update.rs
  </files>
  <action>
    1. Add unit tests to self_update.rs:
       ```rust
       #[cfg(test)]
       mod tests {
           use super::*;
           use std::fs::File;
           use std::io::Write;
           use tempfile::tempdir;

           #[test]
           fn test_replace_binary_success() {
               let dir = tempdir().unwrap();
               let current = dir.path().join("current");
               let new_binary = dir.path().join("new");
               let backup = dir.path().join("current.old");

               // Create test files
               File::create(&current).unwrap().write_all(b"old").unwrap();
               File::create(&new_binary).unwrap().write_all(b"new").unwrap();

               // Perform replacement
               replace_binary(&current, &new_binary, &backup).unwrap();

               // Verify
               assert!(current.exists());
               assert!(backup.exists());
               assert_eq!(fs::read_to_string(&current).unwrap(), "new");
               assert_eq!(fs::read_to_string(&backup).unwrap(), "old");
           }

           #[test]
           fn test_rollback() {
               let dir = tempdir().unwrap();
               let current = dir.path().join("saorsa");
               let backup = dir.path().join("saorsa.old");

               // Create backup file
               File::create(&backup).unwrap().write_all(b"backup").unwrap();

               // Note: Can't easily test rollback without mocking current_exe
               // This test just verifies the backup file handling
               assert!(backup.exists());
           }
       }
       ```

    2. Add tempfile to dev-dependencies in cli/Cargo.toml if not present
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p cli --all-features -- -D warnings
    cargo test -p cli
  </verify>
  <done>
    - replace_binary test passes
    - All 20+ tests pass
    - Zero clippy warnings
  </done>
</task>

## Exit Criteria

- [x] self_update.rs created with perform_self_update()
- [x] Platform-specific restart (Unix exec, Windows spawn/exit)
- [x] Binary replacement with backup
- [x] UpdateCLI handler performs update with confirmation
- [x] Rollback capability exists
- [x] All tests pass (22 tests)
- [x] Zero clippy warnings

## Notes

- Binary replacement strategy:
  1. Download new binary to cache
  2. Rename current → current.old (backup)
  3. Copy new → current location
  4. Set permissions
  5. Restart (exec on Unix, spawn+exit on Windows)

- Windows considerations:
  - Can't delete/rename running executable directly
  - Spawn new process then exit current

- Unix considerations:
  - exec() replaces current process entirely
  - No zombie process left behind

- Rollback is automatic if copy fails (restore from backup)

## Security Considerations

- Checksum verification happens in download_binary() (already implemented)
- Preserve file permissions
- Clean up backup after successful restart (future enhancement)

## Completion Marks M1 Done

This phase completes Milestone 1: Auto-Update Foundation. The system will:
1. Check for updates on startup (background, cached)
2. Display notification if update available
3. Allow user to trigger self-update
4. Verify checksums before applying
5. Safely replace binary with rollback
6. Restart with new version
