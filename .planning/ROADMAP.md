# ROADMAP.md - Auto-Update System

## Milestone 1: Auto-Update Foundation ✅ COMPLETE

Build the core components for version checking and update notification.

### Phase 1: Version Infrastructure
**Status**: Complete
**Estimated Tasks**: 3 (completed)

- [x] Add `semver` and `chrono` dependencies to cli/Cargo.toml
- [x] Create `cli/src/version.rs` with version parsing and comparison
- [x] Add `--version` flag to sb binary (sb/src/main.rs) - saorsa/sdisk already had it
- [x] Add version tracking fields to Config struct

### Phase 2: Checksum Generation
**Status**: Complete
**Estimated Tasks**: 3 (completed)

- [x] Update release.yml to generate SHA256 checksums (CHECKSUMS.txt)
- [x] Add checksum parsing and verification to downloader.rs
- [x] Implement download verification with comprehensive tests

### Phase 3: Update Checker
**Status**: Complete
**Estimated Tasks**: 3 (completed)

- [x] Create `cli/src/updater.rs` with UpdateChecker struct
- [x] Implement background check task (async, non-blocking)
- [x] Add check result caching with 1-hour TTL (via config.should_check_for_updates())
- [x] Store last_check_timestamp in config (via VersionState)
- [x] Integrate checker into main.rs startup (Arc<RwLock<Config>>, tokio::spawn)

### Phase 4: Update Notification
**Status**: Complete
**Estimated Tasks**: 3 (completed)

- [x] Add update_available field to Menu struct
- [x] Implement status line hint in menu header (yellow)
- [x] Add "Update CLI" menu option when update found
- [x] Highlight UpdateCLI item in yellow

### Phase 5: Self-Update & Restart
**Status**: Complete
**Estimated Tasks**: 3 (completed)

- [x] Implement binary replacement logic (self_update.rs)
- [x] Add restart prompt after self-update (dialoguer confirmation)
- [x] Handle platform-specific restart (Unix exec, Windows spawn)
- [x] Tests for binary replacement (3 tests)

---

## Milestone 2: Polish & Edge Cases (Future)

- [ ] Rollback capability
- [ ] Update history logging
- [ ] Skip version preference
- [ ] Offline mode handling
- [ ] Proxy support

---

## Completion Log

| Phase | Completed | Notes |
|-------|-----------|-------|
| Phase 1 | 2026-01-16 | Version infrastructure - semver, clap for sb, version tracking in config |
| Phase 2 | 2026-01-16 | Checksum generation - release.yml generates CHECKSUMS.txt, downloader verifies |
| Phase 3 | 2026-01-16 | Update checker - updater.rs, background task, Arc<RwLock<Config>>, notification |
| Phase 4 | 2026-01-16 | Update notification - MenuChoice::UpdateCLI, yellow header/menu highlights |
| Phase 5 | 2026-01-16 | Self-update - binary replacement, platform restart, user confirmation |
