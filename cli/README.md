# saorsa-cli

`saorsa-cli` is the lightweight bootstrapper for the Saorsa workspace. It can:

- launch the unified `saorsa` TUI
- download/update first-party binaries from GitHub Releases
- run `sb` and `sdisk` directly
- execute installed plugins

## Build

From the workspace root:

```bash
cargo build --release -p cli
```

The binary will be written to `target/release/saorsa-cli`.

## Usage

### Interactive menu

```bash
saorsa-cli
```

Menu entries include:

- Launch Saorsa
- Run `sb`
- Run `sdisk`
- Update binaries
- Update `saorsa-cli`
- Settings
- Plugins
- Exit

Navigation:

- `↑/↓` or `j/k` — move
- `Enter` / `Space` — select
- `q` / `Esc` — quit

### Direct-run mode

```bash
saorsa-cli --run sb [args...]
saorsa-cli --run sdisk [args...]
```

`--run` currently supports `sb` and `sdisk`.

### Plugin execution

```bash
saorsa-cli --plugin rg -- foo src
saorsa-cli --plugin fd -- Cargo .
```

## Command-line flags

- `--no-update-check` — disable startup update checks
- `--use-system` — prefer binaries already on your `PATH`
- `--force-download` — force re-download of release artifacts
- `-v, --verbose` — enable verbose logging
- `-r, --run <tool>` — run `sb` or `sdisk` directly
- `--plugin <name>` — execute a plugin
- trailing args after `--run` or `--plugin` are forwarded to the selected tool/plugin

## Configuration

The config file is stored in the platform config directory under `saorsa-cli/config.toml`.

Typical locations:

- macOS: `~/Library/Application Support/saorsa-cli/config.toml`
- Linux: `${XDG_CONFIG_HOME:-~/.config}/saorsa-cli/config.toml`
- Windows: `%APPDATA%\saorsa-cli\config.toml`

Example:

```toml
[github]
owner = "saorsa-labs"
repo = "saorsa-cli"
check_prerelease = false

[cache]
directory = null
auto_clean = false
max_versions = 3

[behavior]
auto_update_check = true
use_system_binaries = false
prefer_local_build = false
```

Legacy configs that still point to `dirvine/saorsa-cli` are migrated in memory to `saorsa-labs/saorsa-cli`.

## Binary cache

Downloaded binaries are stored under the platform cache directory in `saorsa-cli/binaries/`.

Typical locations:

- macOS: `~/Library/Caches/saorsa-cli/binaries/`
- Linux: `${XDG_CACHE_HOME:-~/.cache}/saorsa-cli/binaries/`
- Windows: `%LOCALAPPDATA%\saorsa-cli\cache\binaries\`

## Releases

For a local release validation pass:

```bash
./scripts/create-release.sh vX.Y.Z
```

That script validates a local release build and can optionally push a tag. GitHub Actions produces the final signed release artifacts after the tag is pushed.

## License

MIT OR Apache-2.0
