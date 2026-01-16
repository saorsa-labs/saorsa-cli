# PLAN-3: Update Checker

**Phase**: 3 of 5
**Milestone**: M1 - Auto-Update Foundation
**Created**: 2026-01-16

## Overview

Implement the background update checker that runs asynchronously on startup:
- Create UpdateChecker struct with async check logic
- Spawn non-blocking background task
- Use existing downloader to fetch latest release
- Compare versions using version.rs
- Cache results with 1-hour TTL via config.version_state
- Integrate into main.rs startup flow

## Prerequisites

- [x] Phase 1 completed (version.rs with comparison logic)
- [x] Phase 2 completed (downloader with checksum verification)
- [x] Config has version_state with `should_check_for_updates()` method
- [x] Args has `--no-update-check` flag

## Current State

| Component | Status |
|-----------|--------|
| version.rs | Has `update_available()`, `parse_version()` |
| config.rs | Has `VersionState` with cache fields |
| downloader.rs | Has `get_latest_release()` |
| main.rs | Has `--no-update-check` flag, loads config |

## Tasks

<task type="auto" priority="p0">
  <n>Create updater.rs with UpdateChecker struct</n>
  <files>
    cli/src/updater.rs,
    cli/src/main.rs
  </files>
  <action>
    1. Create cli/src/updater.rs with:
       ```rust
       //! Background update checker for the auto-update system.

       use crate::config::Config;
       use crate::downloader::Downloader;
       use crate::version;
       use std::sync::Arc;
       use tokio::sync::RwLock;

       /// Result of an update check.
       #[derive(Debug, Clone)]
       pub struct UpdateCheckResult {
           /// Whether an update is available
           pub update_available: bool,
           /// The latest version string (e.g., "0.3.13")
           pub latest_version: Option<String>,
           /// The current version string
           pub current_version: String,
           /// Human-readable message
           pub message: Option<String>,
       }

       /// Background update checker.
       pub struct UpdateChecker {
           config: Arc<RwLock<Config>>,
           downloader: Arc<Downloader>,
       }

       impl UpdateChecker {
           pub fn new(config: Arc<RwLock<Config>>, downloader: Arc<Downloader>) -> Self {
               Self { config, downloader }
           }

           /// Check for updates (respects cache TTL).
           pub async fn check(&self) -> Option<UpdateCheckResult> {
               // Check if we should even run (respects --no-update-check and cache)
               {
                   let config = self.config.read().await;
                   if !config.should_check_for_updates() {
                       // Return cached result if available
                       if let Some(latest) = config.get_latest_version() {
                           let current = env!("CARGO_PKG_VERSION");
                           let update_available = version::update_available(current, latest)
                               .unwrap_or(false);
                           return Some(UpdateCheckResult {
                               update_available,
                               latest_version: Some(latest.clone()),
                               current_version: current.to_string(),
                               message: None,
                           });
                       }
                       return None;
                   }
               }

               // Perform the actual check
               self.perform_check().await
           }

           /// Actually perform the update check (bypasses cache).
           async fn perform_check(&self) -> Option<UpdateCheckResult> {
               let current = env!("CARGO_PKG_VERSION");

               match self.downloader.get_latest_release().await {
                   Ok(release) => {
                       let latest = release.tag_name.trim_start_matches('v');
                       let update_available = version::update_available(current, latest)
                           .unwrap_or(false);

                       // Update cache
                       {
                           let mut config = self.config.write().await;
                           config.record_update_check(Some(latest.to_string()));
                           // Save config (ignore errors for background task)
                           let _ = config.save();
                       }

                       Some(UpdateCheckResult {
                           update_available,
                           latest_version: Some(latest.to_string()),
                           current_version: current.to_string(),
                           message: if update_available {
                               Some(format!("Update available: {} -> {}", current, latest))
                           } else {
                               None
                           },
                       })
                   }
                   Err(e) => {
                       tracing::debug!("Update check failed: {}", e);
                       None
                   }
               }
           }
       }
       ```

    2. Add `mod updater;` to cli/src/main.rs after other mod declarations

    3. Export UpdateChecker and UpdateCheckResult for use by menu
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p cli --all-features -- -D warnings
    cargo build -p cli
  </verify>
  <done>
    - updater.rs created with UpdateChecker struct
    - Uses Arc<RwLock<Config>> for thread-safe config access
    - Respects cache TTL via should_check_for_updates()
    - Module added to main.rs
    - No clippy warnings
  </done>
</task>

<task type="auto" priority="p1">
  <n>Integrate update checker into main.rs startup</n>
  <files>
    cli/src/main.rs
  </files>
  <action>
    1. Update main() to wrap config in Arc<RwLock<>>:
       ```rust
       use std::sync::Arc;
       use tokio::sync::RwLock;

       // After loading config
       let config = Arc::new(RwLock::new(config));
       ```

    2. Wrap downloader in Arc:
       ```rust
       let downloader = Arc::new(Downloader::new(...)?);
       ```

    3. Create UpdateChecker and spawn background task:
       ```rust
       use crate::updater::UpdateChecker;

       // Create update checker
       let update_checker = UpdateChecker::new(
           Arc::clone(&config),
           Arc::clone(&downloader),
       );

       // Spawn background update check (non-blocking)
       let update_result = Arc::new(RwLock::new(None));
       let update_result_clone = Arc::clone(&update_result);
       tokio::spawn(async move {
           if let Some(result) = update_checker.check().await {
               *update_result_clone.write().await = Some(result);
           }
       });
       ```

    4. Update functions that use config/downloader to handle Arc<RwLock<>>:
       - check_binaries() needs &Config, extract with read().await
       - update_binaries() needs &Downloader, use Arc directly
       - Pass update_result to menu for status display

    5. Show update notification if available:
       ```rust
       // Before main menu loop, check for update result
       if let Some(result) = update_result.read().await.as_ref() {
           if result.update_available {
               println!("📦 {}", result.message.as_deref().unwrap_or("Update available!"));
           }
       }
       ```
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p cli --all-features -- -D warnings
    cargo build -p cli
    cargo run -p cli -- --help
    cargo run -p cli -- --version
  </verify>
  <done>
    - Config wrapped in Arc<RwLock<>> for thread-safe access
    - Downloader wrapped in Arc for sharing
    - Background update check spawned on startup
    - Update notification printed before menu
    - All existing functionality preserved
    - No clippy warnings
  </done>
</task>

<task type="auto" priority="p2">
  <n>Add update checker tests</n>
  <files>
    cli/src/updater.rs
  </files>
  <action>
    1. Add unit tests to updater.rs:
       ```rust
       #[cfg(test)]
       mod tests {
           use super::*;

           #[test]
           fn test_update_check_result_display() {
               let result = UpdateCheckResult {
                   update_available: true,
                   latest_version: Some("0.4.0".to_string()),
                   current_version: "0.3.12".to_string(),
                   message: Some("Update available: 0.3.12 -> 0.4.0".to_string()),
               };

               assert!(result.update_available);
               assert_eq!(result.latest_version, Some("0.4.0".to_string()));
               assert!(result.message.is_some());
           }

           #[test]
           fn test_update_check_result_no_update() {
               let result = UpdateCheckResult {
                   update_available: false,
                   latest_version: Some("0.3.12".to_string()),
                   current_version: "0.3.12".to_string(),
                   message: None,
               };

               assert!(!result.update_available);
               assert!(result.message.is_none());
           }
       }
       ```

    2. Note: Integration tests for UpdateChecker require mocking
       the downloader (deferred to future phase if needed)
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p cli --all-features -- -D warnings
    cargo test -p cli
  </verify>
  <done>
    - UpdateCheckResult tests added
    - All tests pass
    - No clippy warnings
  </done>
</task>

## Exit Criteria

- [x] updater.rs created with UpdateChecker struct
- [x] Background check spawned on startup (non-blocking)
- [x] Cache respected (1-hour TTL via should_check_for_updates)
- [x] Update notification shown before menu if available
- [x] Existing functionality unchanged
- [x] All tests pass (19 tests)
- [x] Zero clippy warnings

## Notes

- Using `Arc<RwLock<Config>>` allows background task to update cache
- Background task is fire-and-forget (errors logged, not propagated)
- Update check respects `--no-update-check` CLI flag
- Cache prevents API rate limiting (1-hour TTL)
- Config is saved after update check to persist cache

## Thread Safety Considerations

- Config needs RwLock for write access from background task
- Downloader is read-only after creation, Arc suffices
- UpdateCheckResult stored in Arc<RwLock<>> for menu access

## Next Phase

Phase 4: Update Notification
- Add update status to menu display
- Add "Update Available" menu option
- Implement download with progress
