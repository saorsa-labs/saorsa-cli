# Saorsa

Saorsa is a Rust workspace for a keyboard-first terminal toolkit:

- `saorsa` — unified tabbed TUI workspace
- `saorsa-cli` — bootstrapper, updater, and plugin launcher
- `sb` — standalone Markdown browser/editor
- `sdisk` — standalone disk usage and cleanup utility

The workspace also includes reusable crates for UI, plugin loading, and tab adapters.

## Workspace components

| Binary / Crate | Path | Purpose |
| --- | --- | --- |
| `saorsa` | `crates/saorsa` | Unified TUI with Files, Disk, Git, and Plugins tabs |
| `saorsa-cli` | `cli` | Downloads/updates binaries, launches tools, runs plugins |
| `sb` | `sb` | Markdown browser/editor reused inside the Files tab |
| `sdisk` | `sdisk` | Disk analyzer and cleanup helper reused inside the Disk tab |
| `saorsa-cli-core` | `crates/saorsa-cli-core` | Plugin loader, message bus, run history |
| `saorsa-ui` | `crates/saorsa-ui` | Shared Ratatui UI primitives |
| `saorsa-sb` / `saorsa-disk` / `saorsa-git` | `crates/*` | Adapters that bridge standalone tools into `saorsa` |

## Install

### Signed install script

```bash
curl -fsSL https://github.com/saorsa-labs/saorsa-cli/releases/latest/download/saorsa-install.sh | bash
```

### Verify before running

```bash
curl -fsSLO https://github.com/saorsa-labs/saorsa-cli/releases/latest/download/saorsa-install.sh
curl -fsSLO https://github.com/saorsa-labs/saorsa-cli/releases/latest/download/saorsa-install.sh.asc
gpg --import docs/signing/saorsa-public.asc
gpg --verify saorsa-install.sh.asc saorsa-install.sh
bash saorsa-install.sh
```

When run, the install script:

- detects your OS/architecture and selects the matching release artifact
- downloads the archive plus its detached signature from GitHub Releases
- imports `docs/signing/saorsa-public.asc` if needed
- verifies the **archive signature** with GPG
- installs `saorsa`, `saorsa-cli`, `sb`, and `sdisk` into `/usr/local/bin` or `~/.local/bin`

Useful overrides:

- `SAORSA_VERSION=v0.4.0`
- `SAORSA_TARGET=x86_64-unknown-linux-musl`
- `SAORSA_PREFIX=$HOME/bin`

## Build from source

```bash
git clone https://github.com/saorsa-labs/saorsa-cli
cd saorsa-cli
cargo build --release
```

Built binaries appear in `target/release/`:

- `saorsa`
- `saorsa-cli`
- `sb`
- `sdisk`

## Run

```bash
saorsa-cli                 # bootstrap menu
saorsa                     # unified TUI
saorsa-cli --run sb        # run sb directly
saorsa-cli --run sdisk     # run sdisk directly
saorsa-cli --plugin rg -- foo src
```

All binaries support `-h/--help`.

## Keyboard notes

### `saorsa-cli`

- `↑/↓` or `j/k` — move
- `Enter` / `Space` — select
- `q` / `Esc` — exit
- plugin menu supports execute, refresh, directory listing, and run-history summary

### `saorsa`

- `Ctrl+Q` / `Ctrl+C` — quit
- `Tab` / `Shift+Tab` — next/previous tab
- `Alt+1..9` — jump to tab

Tab-specific highlights:

- **Files** — press `?` for the in-app cheat sheet; common actions include open/toggle, create file, delete, save, link insertion, and raw editor mode.
- **Disk** — `j/k` or arrows navigate, `o` overview, `l` largest entries, `s` stale items, `r` refresh.
- **Git** — `j/k` navigate, `Enter` / `Space` stage or unstage selection, `s` stage all, `u` unstage all, `r` refresh, `l/h` switch between status and diff panes.
- **Plugins** — `↑/↓` select, `Enter` run, `r` reload, `h/?` help, `i` details, `d` plugin directories, `c`/`Esc` close the info panel.

## Plugins

Saorsa discovers `saorsa-plugin.toml` manifests in:

- `~/.saorsa/plugins`
- `${XDG_DATA_HOME:-~/.local/share}/saorsa/plugins`
- `/usr/local/share/saorsa/plugins`
- `./plugins`

See `docs/PLUGINS.md` for the manifest format and plugin authoring notes.

### Built-in search plugins

First-party built-ins are bundled for:

- `fd` — file discovery (requires `fd` on your `PATH`)
- `rg` — content search (requires `rg` on your `PATH`)

The CLI offers an interactive argument builder for both.

### Security note

Plugins currently run **unsandboxed** with the same privileges as the current user. Only install plugins you trust.

## Validation

Recommended checks:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo run --bin saorsa
```

For local hardening work, also consider running a stricter non-test clippy pass that denies `panic`, `unwrap`, and `expect` in production code.

## Release helper

`./scripts/create-release.sh vX.Y.Z` performs a **local release validation pass**:

- builds the workspace in release mode
- verifies all four binaries exist
- creates a local test archive
- smoke-checks binary `--version` output
- optionally creates and pushes a Git tag so GitHub Actions can build release assets

The GitHub release artifacts themselves are produced by `.github/workflows/release.yml` after the tag is pushed.

## Project layout

```text
saorsa-cli/
├── Cargo.toml
├── cli/
├── crates/
│   ├── saorsa
│   ├── saorsa-cli-core
│   ├── saorsa-disk
│   ├── saorsa-git
│   ├── saorsa-sb
│   └── saorsa-ui
├── sb/
├── sdisk/
├── docs/
├── scripts/
└── workspace-hack/
```

## License

Dual-licensed under MIT or Apache-2.0.
