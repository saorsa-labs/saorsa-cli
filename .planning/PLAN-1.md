# PLAN-1: Version Infrastructure

**Phase**: 1 of 5
**Milestone**: M1 - Auto-Update Foundation
**Created**: 2026-01-16

## Overview

Establish the foundational version infrastructure for the auto-update system:
- Add semantic versioning support
- Ensure all binaries have `--version` flags
- Create version comparison utilities
- Add version tracking to configuration

## Prerequisites

- [x] GSD-Hybrid planning initialized
- [x] Interview decisions recorded in STATE.md
- [x] Existing codebase explored

## Current State

| Binary | Has --version | Has clap |
|--------|---------------|----------|
| saorsa | Yes | Yes (4.5) |
| sdisk | Yes | Yes (4.5) |
| sb | No | No |

## Tasks

<task type="auto" priority="p0">
  <n>Add version dependencies and create version.rs module</n>
  <files>
    cli/Cargo.toml,
    cli/src/version.rs,
    cli/src/lib.rs (if exists) or cli/src/main.rs
  </files>
  <action>
    1. Add dependencies to cli/Cargo.toml:
       ```toml
       # Version management
       semver = "1.0"
       chrono = { version = "0.4", features = ["serde"] }
       ```

    2. Create cli/src/version.rs with:
       - VersionInfo struct containing version, build_date, target
       - parse_version() function using semver
       - compare_versions() returning Ordering
       - is_newer() helper that returns bool
       - get_current_version() returning VersionInfo
       - Format: use env!("CARGO_PKG_VERSION") for current version

    3. Add `mod version;` to cli/src/main.rs

    4. Export version module for use by other components
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p cli --all-features -- -D warnings
    cargo build -p cli
    cargo test -p cli
  </verify>
  <done>
    - semver and chrono dependencies added
    - version.rs module compiles
    - Version comparison logic tested
    - No clippy warnings
  </done>
</task>

<task type="auto" priority="p1">
  <n>Add clap and --version flag to sb binary</n>
  <files>
    sb/Cargo.toml,
    sb/src/main.rs
  </files>
  <action>
    1. Add clap dependency to sb/Cargo.toml:
       ```toml
       clap = { version = "4.5", features = ["derive"] }
       ```

    2. Update sb/src/main.rs:
       - Add `use clap::Parser;`
       - Create Args struct with #[derive(Parser)]:
         ```rust
         #[derive(Parser, Debug)]
         #[command(
             name = "sb",
             about = "Terminal Markdown Browser/Editor with Git integration",
             version,
             author
         )]
         struct Args {
             /// Root directory to browse (defaults to current directory)
             #[arg(default_value = ".")]
             root: PathBuf,
         }
         ```
       - Replace `std::env::args().nth(1)...` with `Args::parse()`
       - Use `args.root` instead of manual argument parsing

    3. Ensure existing functionality unchanged (just moved to clap)
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p sb --all-features -- -D warnings
    cargo build -p sb
    cargo test -p sb
    ./target/debug/sb --version
    ./target/debug/sb --help
  </verify>
  <done>
    - sb --version shows "sb 0.3.12"
    - sb --help shows usage
    - sb [path] still works as before
    - No clippy warnings
  </done>
</task>

<task type="auto" priority="p1">
  <n>Add version tracking to Config</n>
  <files>
    cli/src/config.rs
  </files>
  <action>
    1. Add version tracking fields to Config struct:
       ```rust
       #[derive(Debug, Clone, Serialize, Deserialize)]
       pub struct VersionTracking {
           /// Currently installed version (from last update)
           pub installed_version: Option<String>,
           /// Timestamp of last update check
           pub last_check: Option<chrono::DateTime<chrono::Utc>>,
           /// Version that was skipped by user
           pub skipped_version: Option<String>,
       }
       ```

    2. Add to main Config struct:
       ```rust
       #[serde(default)]
       pub version: VersionTracking,
       ```

    3. Implement Default for VersionTracking

    4. Add helper methods:
       - should_check_update() -> bool (based on last_check and 1-hour cache)
       - mark_checked() -> updates last_check to now
       - update_installed_version(version: &str)

    5. Ensure backwards compatibility with existing config files
       (use #[serde(default)] for new fields)
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p cli --all-features -- -D warnings
    cargo build -p cli
    cargo test -p cli
  </verify>
  <done>
    - VersionTracking struct added to config
    - Existing config files load without error (backwards compatible)
    - Helper methods implemented
    - No clippy warnings
  </done>
</task>

## Exit Criteria

- [ ] All three binaries respond to `--version` flag
- [ ] Version comparison logic works correctly
- [ ] Config can track installed versions and check timestamps
- [ ] All tests pass
- [ ] Zero clippy warnings

## Notes

- sb currently uses raw `std::env::args()` - migrating to clap
- saorsa and sdisk already have clap with version support
- Using chrono for timestamp management (serde feature for config persistence)
- Config uses serde default to maintain backwards compatibility

## Next Phase

Phase 2: Checksum Generation
- Update release workflow to generate SHA256 checksums
- Add checksum verification to downloader
