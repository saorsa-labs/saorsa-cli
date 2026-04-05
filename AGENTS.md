# Repository Guidelines

## Project Structure & Module Organization
- Rust workspace with standalone binaries in `cli/`, `sb/`, and `sdisk/`.
- Reusable crates live under `crates/`:
  - `crates/saorsa` — unified tabbed TUI
  - `crates/saorsa-cli-core` — plugin loader, message bus, shared runtime
  - `crates/saorsa-ui` — shared UI components
  - `crates/saorsa-sb`, `crates/saorsa-disk`, `crates/saorsa-git` — tab adapters
- Shared workspace config is in root `Cargo.toml`.
- CI workflows live in `.github/workflows/`; release helpers live in `scripts/`.

## Build, Test, and Development Commands
- Build workspace: `cargo build --release`
- Run unified TUI: `cargo run --bin saorsa`
- Run bootstrapper: `cargo run --bin saorsa-cli`
- Run sb: `cargo run --bin sb -- /path/to/notes`
- Run sdisk: `cargo run --bin sdisk -- info`
- Non-interactive sdisk example: `cargo run --bin sdisk -- --non-interactive stale --path /tmp`
- Test all: `cargo test --all`
- Format: `cargo fmt --all -- --check`
- CI lint baseline: `cargo clippy --all-targets --all-features -- -D warnings`
- Preferred production-code hardening pass: run an additional clippy invocation that denies `panic`, `unwrap`, and `expect` in non-test targets.
- Local release validation: `./scripts/create-release.sh vX.Y.Z`

## Coding Style & Naming Conventions
- Rust 2021 with `rustfmt` defaults.
- Use `snake_case` for functions/files, `CamelCase` for types, `SCREAMING_SNAKE_CASE` for constants.
- Keep modules focused and testable; avoid one-letter identifiers.
- Prefer `tracing` for diagnostics.

## Testing Guidelines
- Tests live both inline (`mod tests`) and in dedicated test modules such as `sb/src/tests/`.
- Add tests for bug fixes and new behavior.
- Keep filesystem-heavy tests deterministic and isolated with temp directories.

## Commit & Pull Request Guidelines
- Use concise imperative commit messages; type prefixes like `fix:`, `feat:`, `docs:`, and `ci:` are welcome.
- PRs should describe behavior changes, test coverage, and any doc updates.
- Include screenshots or terminal recordings for visible TUI changes when helpful.

## Security & Configuration Tips
- Do not commit secrets or generated binaries.
- `saorsa-cli` stores config in the platform config directory under `saorsa-cli/config.toml`.
- Plugins run unsandboxed with full user privileges; only install trusted plugins.
- If working on `sb` video support, keep the `ffmpeg` requirement documented.
