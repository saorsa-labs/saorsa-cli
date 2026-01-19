# Claude Engineering Notes

This file briefs Claude-based agents on how to work inside the Saorsa workspace. Keep responses
focused on actionable engineering guidance and mirror the keyboard-first philosophy of the project.

## Mission Snapshot

- Deliver a single front-end (`crates/saorsa`) backed by reusable headless tools (`sb`, `sdisk`).
- Keep the bootstrapper (`cli`) lean: downloads, launches, plugin execution, self-update.
- Maintain a trusted first-party plugin catalog; sandboxing is **not** implemented yet, so docs and UI
  must repeat that plugins inherit full user privileges.
- Enforce plugin integrity: manifests require a lowercase `sha256` hash of the compiled library and
  releases should be signed via GPG as documented in `docs/PLUGINS.md`.
- Preserve the "world-class" Ratatui UX: low-latency drawing, tabbed layout, and the EdTUI editor for Markdown.
- Ship releases via the signed `saorsa-install.sh` helper that lives on every GitHub Release; docs must show both the quick `curl | bash` path and the GPG-verification flow using `docs/signing/saorsa-public.asc`.

## Architecture Map

- `crates/saorsa-ui`: Ratatui foundation (tab bar, status bar, layout helpers, message bus integration).
- `crates/saorsa-cli-core`: shared `Message`, `AppCoordinator`, plugin loader (`PluginManager`), and history tracking.
- `crates/saorsa`: binary that wires Files (`saorsa-sb`), Disk, Git, and Plugins tabs together.
- `cli`: bootstrapper menu plus downloader, updater, and plugin runner.
- `sb` + `sdisk`: standalone binaries reused inside Saorsa via adapters (`saorsa-sb`, `saorsa-disk`).
- `docs/PLUGINS.md`: authoritative guide for dynamic plugins (`saorsa-plugin.toml`).

## UX + Keybinding Expectations

- Bootstrapper menu: `Up/Down` or `j/k`, `Enter`/`Space`, `q`/`Esc`, plus plugin submenu parity (execute, details, refresh, directory list).
- Saorsa TUI global shortcuts: `Ctrl+Q/C`, `Tab` / `Shift+Tab`, `Alt+1-9`, message bus dispatch for everything else.
- Files tab (sb): EdTUI 0.11 (vim handler) powers editing; `?` opens a cheat sheet, `:` command mode, `Space` multi-select, `r` raw editor toggle.
- Disk/Git tabs: arrow navigation + `Enter` to act, `Backspace` to bubble up, `Space` toggles selection in Git lists.
- Plugins tab: `Up/Down`, `Enter` runs, `r` reloads manifests, `h/?` opens help, `i` shows plugin details + history, `d` lists search directories, `c`/`Esc` closes info overlays, and the footer keeps the full-trust warning visible.

When adding new features, ensure **every** action has a reachable keybinding and is documented either in `README.md` or an in-app overlay.

## Production Readiness Checklist

1. `cargo fmt --all`
2. `cargo clippy --all-features --all-targets -- -D clippy::panic -D clippy::unwrap_used -D clippy::expect_used`
3. `cargo test --all`
4. `cargo run --bin saorsa` - flip through every tab and run at least one plugin
5. `./scripts/create-release.sh vX.Y.Z` when tagging releases
6. Audit plugin directories (`~/.saorsa/plugins`, `./plugins`) before shipping because plugins execute unsandboxed native code.

## Ratatui / EdTUI Guidance

- Follow the Ratatui component architecture (render + state separation, derived from https://ratatui.rs) for any new UI code.
- Prefer EdTUI widgets (https://docs.rs/edtui) for advanced text editing instead of rolling custom textareas; wrap them via helper structs like `sb::editor::MainEditor` so tabs stay agnostic.
- Keep draw calls under ~10 ms; if a view needs background IO, move it to a worker thread and push updates through the message bus.

## Common Tasks

- **Docs**: README + CLAUDE should stay aligned with the collapsed-front-end architecture and plugin warnings.
- **Distribution**: Attach `saorsa-install.sh` + `.asc` signatures to every release, and update README instructions to reference `https://github.com/saorsa-labs/saorsa-cli/releases/latest/download/saorsa-install.sh`.
- **Plugins**: use `saorsa_cli_core::PluginManager::load()`; manifests live next to `.so/.dylib/.dll` files, `entry_symbol` defaults to `_plugin_init`.
- **Plugin integrity**: any plugin missing `sha256` will be rejected; update manifests during release
  packaging and follow the GPG instructions in `docs/PLUGINS.md` (secrets: `GPG_PRIVATE_KEY`,
  `GPG_PASSPHRASE`).
- **Menu parity**: The Saorsa Plugins tab now mirrors the CLI plugin menu (execute, help/details, directory listing, refresh). Keep both surfaces in sync when adding new plugin actions.

## Open Questions / Follow-ups

- Long term sandboxing model for plugins (signatures? WASI wrappers?).
- Should `saorsa-cli` eventually embed the full TUI once downloads finish instead of spawning a second binary?
- How to surface plugin/log output without overwhelming the status bar? Consider a transient console pane.

Stay disciplined about ASCII output, structured logging via `tracing`, and zero `unwrap`/`expect` in production code.
