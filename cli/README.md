# Saorsa CLI (Bootstrapper)

`saorsa-cli` is the lightweight bootstrapper and downloader for the Saorsa ecosystem. It
detects your platform, fetches first-party binaries, and presents an interactive menu with
“Launch Saorsa” as the primary action so you can get into the full TUI workspace quickly.

## Features

- **Launch Saorsa**: One-keystroke option to install/launch the unified Saorsa TUI
- **Interactive Menu**: Terminal UI for launching `sb`, `sdisk`, or managing settings when the full TUI isn't available yet
- **Automatic Downloads**: Automatically downloads binaries from GitHub releases
- **Platform Detection**: Detects OS and architecture to download appropriate binaries
- **Binary Caching**: Caches downloaded binaries for faster subsequent launches
- **Update Management**: Check for and download newer versions of tools
- **Direct Execution**: Run tools directly without menu using `--run` flag

## Installation

Build from source:
```bash
cargo build --release --package cli
```

The binary will be available at `target/release/saorsa-cli`.

## Usage

### Interactive Menu Mode

Run without arguments to launch the interactive menu:
```bash
saorsa-cli
```

Menu options:
- **Launch Saorsa**: Install (if needed) and start the full TUI workspace
- **Run Saorsa Browser (sb)**: Launch the markdown browser/editor
- **Run Saorsa Disk (sdisk)**: Launch the disk cleanup utility
- **Update Binaries**: Download latest versions of tools
- **Settings**: View current configuration
- **Exit**: Close the menu

Navigation:
- Use arrow keys or `j/k` to navigate
- Press Enter or Space to select
- Press `q` or Esc to exit

### Direct Run Mode

Run a specific tool directly (helpful for CI or scripting):
```bash
# Run sb (Saorsa Browser)
saorsa-cli --run sb [args...]

# Run sdisk
saorsa-cli --run sdisk [args...]

# Once Saorsa is installed, run it directly:
saorsa
```

### Command-Line Options

- `--no-update-check`: Disable automatic update checks
- `--use-system`: Use system-installed binaries instead of downloading
- `--force-download`: Force re-download of binaries
- `-v, --verbose`: Enable verbose logging
- `-r, --run <tool>`: Run a specific tool directly (sb or sdisk)

## Configuration

Configuration is stored in `~/.config/saorsa-cli/config.toml`:

```toml
[github]
owner = "dirvine"
repo = "saorsa-cli"
check_prerelease = false

[cache]
directory = null  # Uses default cache directory
auto_clean = false
max_versions = 3

[behavior]
auto_update_check = true
use_system_binaries = false
prefer_local_build = false
```

## Binary Storage

Downloaded binaries are cached in:
- macOS: `~/Library/Caches/saorsa-cli/binaries/`
- Linux: `~/.cache/saorsa-cli/binaries/`
- Windows: `%LOCALAPPDATA%\saorsa-cli\cache\binaries\`

## Platform Support

The CLI automatically detects and downloads binaries for:
- macOS (x86_64, aarch64/ARM64)
- Linux (x86_64, aarch64)
- Windows (x86_64, aarch64)

## Building Releases

To build binaries for distribution:

```bash
# Build all workspace members
cargo build --release

# The binaries will be at:
# - target/release/sb
# - target/release/sdisk
# - target/release/saorsa-cli
```

## License

MIT OR Apache-2.0
