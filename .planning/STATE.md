# STATE.md - Auto-Update System

**Last Updated**: 2026-01-16
**Current Milestone**: M1 - Auto-Update Foundation
**Current Phase**: Phase 5 - Self-Update & Restart (COMPLETED)
**Current Milestone**: M1 - Auto-Update Foundation (COMPLETED)

## Interview Decisions

| Category | Decision | Rationale |
|----------|----------|-----------|
| Check Timing | On startup (non-blocking) | Background check doesn't delay CLI startup |
| Apply Mode | Prompt before download | User controls bandwidth/timing |
| Channels | Stable only | No prereleases unless config changed |
| Verification | SHA256 checksum | Balance security and simplicity |
| Notification | Status line hint | Non-intrusive, always visible |
| Self-Update | Restart required | Clean replacement, avoids process conflicts |
| Granularity | All binaries together | Simpler UX, consistent versions |
| Version Flag | All binaries | `--version` on sb, sdisk, saorsa |
| Checksum Source | Release body/notes | Standard GitHub release pattern |
| Cache Duration | 1 hour | Reasonable API rate limiting |

## Codebase Foundation

**Already Exists:**
- `cli/src/downloader.rs` - GitHub release fetching & binary download
- `cli/src/config.rs` - TOML configuration with `auto_update_check` setting
- `cli/src/platform.rs` - Platform detection (OS/arch)
- `cli/src/runner.rs` - Binary execution
- GitHub release workflow with signed macOS binaries

**Needs Building:**
- (M1 Complete - all core functionality implemented)

## Completed This Session

- [x] Explored existing codebase
- [x] Conducted user interview
- [x] Created planning structure
- [x] Phase 1 plan created (PLAN-1.md)
- [x] Task 1: Added semver/chrono deps + created version.rs
- [x] Task 2: Added clap and --version to sb binary
- [x] Task 3: Added version tracking to Config
- [x] Phase 2 Task 1: Added SHA256 checksum generation to release.yml
- [x] Phase 2 Task 2: Added checksum parsing/verification to downloader.rs
- [x] Phase 2 Task 3: Added checksum verification tests
- [x] Phase 3 Task 1: Created updater.rs with UpdateChecker struct
- [x] Phase 3 Task 2: Integrated update checker into main.rs (Arc wrappers, background task)
- [x] Phase 3 Task 3: Added update checker tests
- [x] Phase 4 Task 1: Added MenuChoice::UpdateCLI, dynamic menu items
- [x] Phase 4 Task 2: Added yellow update notification in header/menu
- [x] Phase 4 Task 3: Update status refresh in menu loop
- [x] Phase 5 Task 1: Created self_update.rs with binary replacement
- [x] Phase 5 Task 2: Integrated self-update into UpdateCLI handler
- [x] Phase 5 Task 3: Added self-update tests (3 tests)

## Decisions Made

1. Use `semver` crate for version parsing
2. Store installed versions in config.toml
3. Cache update check results with timestamp
4. Parse SHA256 checksums from release body

## Blockers

None currently.

## Handoff Context

**MILESTONE 1 COMPLETE: Auto-Update Foundation**

The auto-update system is fully implemented:
1. Background update check on startup (non-blocking, cached 1hr)
2. Version comparison via semver
3. Checksum verification (SHA256)
4. Update notification in menu (yellow highlight)
5. Self-update with user confirmation
6. Safe binary replacement with backup
7. Platform-specific restart (Unix exec, Windows spawn)
8. 22 tests passing, zero clippy warnings

Future enhancements (M2):
- Rollback capability
- Update history logging
- Skip version preference
- Offline mode handling
