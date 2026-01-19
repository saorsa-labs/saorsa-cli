# Saorsa

Saorsa is a world-class terminal workspace built on Ratatui 0.30 and EdTUI 0.11. It unifies
Markdown knowledge work, disk insights, Git telemetry, and trusted plugins inside a single binary,
while `saorsa-cli` handles bootstrap downloads, updates, and standalone execution on systems that
do not yet have the full TUI installed.

## Workspace Components

| Binary / Crate | Path | Role | Highlights |
| --- | --- | --- | --- |
| `saorsa` | `crates/saorsa` | Unified tabbed TUI | Files (sb), Disk, Git, Plugins tabs share one compositor + status bar |
| `saorsa-cli` | `cli` | Bootstrapper/downloader | Ratatui menu for installs, updates, plugin launcher, self-update |
| `sb` | `sb` | Headless Markdown browser/editor | Uses EdTUI 0.11, git-aware file tree, media preview, reusable as Saorsa tab |
| `sdisk` | `sdisk` | Disk usage analyzer | Streaming walkers plus Saorsa tab via `saorsa-disk` |
| `saorsa-ui` | `crates/saorsa-ui` | UI toolkit | Component layout, theming, tab + status widgets |
| `saorsa-cli-core` | `crates/saorsa-cli-core` | Core runtime | Message bus, plugin loader, history ledger |
| `saorsa-sb` / `saorsa-disk` / `saorsa-git` | `crates/*` | Tab adapters | Bridge the standalone tools into the unified app |

## UX Pillars

- **Modern Ratatui patterns** - Layout + event handling follow the 0.30 component architecture
  (stateful tabs, `Message` bus) so redraws stay sub-5 ms even with multiple panes.
- **EdTUI editing** - The Markdown editor is powered by EdTUI 0.11 (vim mode, syntax highlighting,
  mouse support) for world-class text ergonomics.
- **Consistent chrome** - `saorsa-ui` supplies a shared tab bar + status bar, so Files/Disk/Git/Plugins
  all expose the same help hints and focus semantics.
- **Plugin-aware from the start** - Both the bootstrapper and the TUI share `PluginManager`, run
  history, and on-screen trust warnings. First-party plugins live in `~/.saorsa/plugins`.

## Quick Start

### Install via signed script (recommended)

```bash
curl -fsSL https://github.com/saorsa-labs/saorsa-cli/releases/latest/download/saorsa-install.sh | bash
```

For a fully verified install, grab the detached signature and check it before piping to `bash`:

```bash
curl -fsSLO https://github.com/saorsa-labs/saorsa-cli/releases/latest/download/saorsa-install.sh
curl -fsSLO https://github.com/saorsa-labs/saorsa-cli/releases/latest/download/saorsa-install.sh.asc
gpg --import docs/signing/saorsa-public.asc
gpg --verify saorsa-install.sh.asc saorsa-install.sh
bash saorsa-install.sh
```

What the script does:

- Detects your OS/architecture and selects the correct **signed** release artifact
- Downloads the tarball plus its `.asc` signature from GitHub Releases (latest tag by default)
- Imports `docs/signing/saorsa-public.asc` (if not already present) and verifies both the install script and archive with GPG
- Installs `saorsa`, `saorsa-cli`, `sb`, and `sdisk` into `/usr/local/bin` (or `~/.local/bin` if not writable)

Advanced flags:

- `SAORSA_VERSION=v0.4.0` pins to a specific tag.
- `SAORSA_TARGET=x86_64-unknown-linux-musl` overrides the detected target triple.
- `SAORSA_PREFIX=$HOME/bin` installs into a custom directory.

### Install prebuilt binaries manually

1. Download the latest release artifacts from GitHub.
2. Put `saorsa` and/or `saorsa-cli` somewhere on your `$PATH`.
3. Launch `saorsa-cli` once - it will download missing tools into the cache (or use system binaries if configured).

### Build everything from source

```bash
git clone https://github.com/dirvine/saorsa-cli
cd saorsa-cli
cargo build --release          # builds the workspace (saorsa + tabs + headless binaries)
```

Binaries appear under `target/release/` (`saorsa`, `saorsa-cli`, `sb`, `sdisk`).

### Run it

```bash
saorsa-cli              # bootstrap menu (downloads + updates + plugin runner)
saorsa                  # full TUI once installed
saorsa-cli --run sb     # invoke headless sb directly
saorsa-cli --run sdisk  # invoke sdisk directly
```

All binaries accept `-h/--help` for additional flags.

## Keyboard-first Menus

### Bootstrapper (`saorsa-cli`)

- `↑/↓` or `j/k` navigate
- `Enter` or `Space` activate highlighted action
- `q` / `Esc` returns to the shell
- Plugins menu mirrors CLI functionality (execute, details, refresh, directory listing)

### Saorsa TUI

- `Ctrl+Q` or `Ctrl+C` quits from anywhere
- `Tab` / `Shift+Tab` cycle tabs, `Alt+1..9` jumps directly
- Global status hints live in the footer (`?:help  q:quit`)
- Each tab adds mode-specific bindings (press `?` inside the Files tab to see the sb keymap)

**Files (sb) tab** - Dual-pane browser + EdTUI editor. `?` shows in-app cheat sheet, `:` enters command mode,
`r` toggles raw editor, `Space` multi-selects.

**Disk tab** - Arrow keys move between sections, `Enter` drills down, `Backspace` goes up.

**Git tab** - Navigate staged/unstaged lists with arrows, `Space` toggles selection, `c` creates commit drafts.

**Plugins tab** - `↑/↓` select, `Enter` runs, `r` reloads search paths, `h/?` shows plugin help, `i` shows metadata + run stats, `d` lists directories, `c`/`Esc` closes info panels. A status footer reminds users that plugins run with full trust.

## Plugins

Saorsa loads manifests named `saorsa-plugin.toml` from:

- `~/.saorsa/plugins`
- `${XDG_DATA_HOME:-~/.local/share}/saorsa/plugins`
- `/usr/local/share/saorsa/plugins`
- `./plugins` (handy while developing)

See `docs/PLUGINS.md` for manifest format, Rust skeleton, and troubleshooting tips. Plugins currently run unsandboxed with the same privileges as `saorsa`, so only install trusted code.

### Plugin Security

- Every manifest must include a lowercase `sha256` checksum of its shared library. The loader re-hashes the binary at runtime and aborts on mismatches.
- First-party releases are built from this monorepo and published on GitHub with both the manifest hash and an optional GPG detached signature. The instructions for generating hashes, signing artifacts, and uploading `.asc` signatures live in `docs/PLUGINS.md`.
- The public verification key is checked into `docs/signing/saorsa-public.asc`; import it before verifying signatures.
- If you side-load a plugin, ensure you audit the manifest + source and update the hash yourself—unsigned plugins are rejected by default.

## Production Readiness Checklist

- `cargo fmt --all` - formatting gate
- `cargo clippy --all-targets --all-features -- -D warnings` - lint with panic/unwrap forbid rules
- `cargo test --all` - workspace tests (including sb + sdisk)
- `cargo run --bin saorsa` - manual smoke test of every tab + plugin menu
- `./scripts/create-release.sh vX.Y.Z` - release helper (tags + assets)
- Verify plugin directories (especially `./plugins`) before cutting a release; the runtime does not sandbox dynamic libraries.

## Project Layout

```
saorsa-cli/
├── Cargo.toml
├── cli/                     # saorsa-cli bootstrapper
├── crates/
│   ├── saorsa               # unified TUI main binary
│   ├── saorsa-cli-core      # plugin manager, message bus, theme
│   ├── saorsa-ui            # reusable UI primitives
│   ├── saorsa-sb            # Files tab adapter
│   ├── saorsa-disk          # Disk tab adapter
│   └── saorsa-git           # Git tab adapter
├── sb/                      # standalone Markdown app reused inside Saorsa
├── sdisk/                   # standalone disk analyzer reused inside Saorsa
├── docs/PLUGINS.md          # plugin authoring guide
├── scripts/create-release.sh
└── README.md
```

## Contributing

1. Fork + clone the repo
2. Create a feature branch (`git checkout -b feat/my-change`)
3. Make changes, run fmt/clippy/tests
4. Open a PR with screenshots/GIFs for UI tweaks and describe plugin/API changes

## License

Dual-licensed under MIT or Apache 2.0.

## Maintainer

David Irvine - david.irvine@saorsa.net (@dirvine)
