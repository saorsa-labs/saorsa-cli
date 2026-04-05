# sb

`sb` is the standalone Markdown browser/editor used by the Saorsa Files tab.

It focuses on fast keyboard-first note and code browsing in the terminal, with:

- file tree navigation
- rendered Markdown preview
- raw editor mode
- syntax-highlighted code preview
- Git-aware status and diff integration
- image preview and ffmpeg-backed video playback
- file creation, copy, move, and delete actions

Repository: <https://github.com/saorsa-labs/saorsa-cli>

## Requirements

- Rust **1.82+** to build from source
- `ffmpeg` on your `PATH` for video playback

Examples:

```bash
# macOS
brew install ffmpeg

# Ubuntu / Debian
sudo apt-get install ffmpeg
```

## Build and run

From the workspace root:

```bash
cargo build --release -p sb
cargo run --release -p sb -- /path/to/notes
```

Or run the built binary directly:

```bash
./target/release/sb /path/to/notes
```

Helpful development commands:

```bash
cargo test -p sb
RUST_LOG=debug cargo run -p sb -- /path/to/notes
```

## Core behaviors

### File tree + preview/editor

`sb` uses a left-hand file tree and a right-hand preview/editor area.

- Directories can be expanded/collapsed from the tree
- Files open into the right-hand preview/editor pane
- Markdown is rendered for reading, while code files get syntax-highlighted preview
- Git diffs are shown when available for tracked files

### Editing

`sb` supports two editing styles:

- **preview mode** for browsing rendered content and quick line-oriented actions
- **raw editor mode** for direct text editing

### Media

- images render inline when supported by the terminal
- video playback is available through `ffmpeg`
- Markdown video links use the inline syntax ``[video](clip.mp4)``

### File operations

From the file tree you can:

- create files
- copy and move entries
- delete files/directories with confirmation
- open files externally with your system opener or `$EDITOR`

## Keybindings

Press `?` in the app for the current in-app cheat sheet.

Common keys:

### Global

- `Tab` / `Shift+Tab` — cycle focus
- `?` — toggle help
- `q` or `Esc` — quit / back out of overlays
- `Ctrl+B` or `F9` — toggle the file tree
- `Ctrl+S` — save current file
- `F2` or `Ctrl+I` — insert link via file picker

### File tree

- `↑/↓/←/→` or `j/k/h/l` — navigate
- `Enter` — open file / toggle directory
- `n` — create file
- `d` — delete selection
- `F5` — copy
- `F6` — move/rename
- `s` — add/remove current item to multi-selection
- `Ctrl+A` — select all visible entries
- `o` — open externally
- `r` — refresh tree

### Preview/editor

- `j/k` or arrows — move through preview
- `e` — enter raw editor mode
- `i` — begin line edit from preview
- `Ctrl+R` — switch to raw editor mode
- `PageUp` / `PageDown` — faster preview movement

### Video playback

- `Space` — pause/resume
- `s` — stop playback
- `Ctrl+V` — toggle autoplay

### Pane sizing

- `Ctrl+,` or `Ctrl+-` — narrow left pane
- `Ctrl+.` or `Ctrl+=` — widen left pane
- mouse drag on separator — resize with the mouse

## Notes on safety

The crate contains security helpers for path validation and file-size limits, but the current implementation is still an evolving terminal app rather than a hardened sandbox. Treat it as a local trusted-user tool.

## License

Dual-licensed under MIT or Apache-2.0 (`LICENSE-MIT` / `LICENSE-APACHE`).
