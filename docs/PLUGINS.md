# Saorsa Plugin Guide

This document explains how to package and install dynamic plugins that can be
discovered by both the `saorsa` TUI and the `saorsa-cli` bootstrapper.

## Directory Layout

Saorsa searches the following folders at startup (in this order):

1. `~/.saorsa/plugins`
2. `${XDG_DATA_HOME:-~/.local/share}/saorsa/plugins`
3. `/usr/local/share/saorsa/plugins`
4. `./plugins` (relative to the current working directory – useful during development)

Each plugin lives in its own folder and must contain:

- A manifest named `saorsa-plugin.toml`
- A shared library (`.so`, `.dylib`, or `.dll`) referenced by the manifest

Example:

```
~/.saorsa/plugins/
└── sample-plugin/
    ├── saorsa-plugin.toml
    └── libsample_plugin.dylib
```

## Manifest Format

```toml
# saorsa-plugin.toml
name        = "sample"
version     = "0.1.0"
description = "Demonstrates the Saorsa plugin API"
author      = "Jane Doe <jane@example.com>"
library     = "libsample_plugin.dylib"  # relative to this manifest
help        = "Prints a friendly greeting."
entry_symbol = "_plugin_init"           # optional, defaults to this value
sha256 = "0123456789abcdef..."          # required checksum of the shared library
```

### Field reference

| Field        | Required | Description                                                                 |
|--------------|----------|-----------------------------------------------------------------------------|
| `name`       | ✅       | Unique identifier shown in the UI                                           |
| `version`    | ✅       | Semantic version string                                                      |
| `description`| ✅       | Short summary for menus                                                      |
| `author`     | ✅       | Attribution text                                                             |
| `library`    | ✅       | Relative or absolute path to the compiled dynamic library                   |
| `help`       | ❌       | Longer usage text shown in the CLI/TUI                                      |
| `entry_symbol` | ❌     | Constructor symbol; override only if you renamed `_plugin_init`             |
| `sha256`     | ✅       | Lowercase SHA-256 hash (no spaces) of the compiled library file             |

## Rust Plugin Skeleton

```rust
use anyhow::Result;
use saorsa_cli_core::{declare_plugin, Plugin, PluginContext};

struct Sample;

impl Plugin for Sample {
    fn name(&self) -> &str { "sample" }
    fn description(&self) -> &str { "Demonstrates the Saorsa plugin API" }
    fn version(&self) -> &str { "0.1.0" }
    fn author(&self) -> &str { "Jane Doe" }
    fn help(&self) -> &str { "Prints a greeting." }

    fn execute(&self, _args: &[String], _ctx: PluginContext<'_>) -> Result<()> {
        println!("Hello from sample plugin!");
        Ok(())
    }
}

declare_plugin!(Sample, || Sample);
```

Compile the library using `cd plugin-dir && cargo build --release` and copy the
resulting `.so/.dylib/.dll` next to the manifest.

## Testing a Plugin

1. Drop the directory into one of the search paths above.
2. Run `saorsa-cli --plugin sample` or open the `Plugins` tab inside `saorsa`.
3. Use the built-in refresh action if you modify binaries in place.

## Troubleshooting

- **Plugin not listed** – ensure the manifest name is exactly `saorsa-plugin.toml`
  and the shared library path is valid.
- **Duplicate name** – each plugin’s `name` must be unique across search paths.
- **Crashes on execution** – check the application logs (`RUST_LOG=debug`) for
  the `PluginLoadFailed` error which includes the dynamic loader message.

## Integrity & Signing

Saorsa now enforces first-party integrity for every plugin. Each manifest must
include a `sha256` entry that matches the compiled shared library. During load,
the runtime hashes the `.so/.dylib/.dll` and refuses to execute if the checksum
does not match.

### Computing the checksum

```bash
# After building your plugin in release mode
PLUGIN=target/release/libsample_plugin.dylib   # adjust for your platform
shasum -a 256 "$PLUGIN" | awk '{print $1}'
```

Copy the resulting lowercase hex string into `saorsa-plugin.toml` under `sha256`.
If you script releases, add a step that rewrites the manifest automatically so
the published archives and the hash in git always match.

### Release signing with GPG (recommended)

1. Generate a maintainer key (one-time):
   ```bash
   gpg --full-generate-key
   gpg --armor --export you@example.com > saorsa-public.asc
   ```
   Commit the **public** key (we store ours at `docs/signing/saorsa-public.asc`)
   so CI and users can import it for verification.
2. Export the private key for CI and store it as GitHub secrets:
   ```bash
   gpg --armor --export-secret-keys you@example.com > saorsa-private.asc
   ```
   - `GPG_PRIVATE_KEY`: contents of `saorsa-private.asc`
   - `GPG_PASSPHRASE`: the passphrase you set.
3. Update your release workflow to import the key and sign plugin archives:
   ```yaml
   - name: Import GPG key
     run: |
       echo "$GPG_PRIVATE_KEY" | gpg --batch --import
   - name: Sign plugin bundle
     run: |
       gpg --batch --yes --pinentry-mode loopback \
           --passphrase "$GPG_PASSPHRASE" \
           --detach-sign --armor plugins/sample-plugin.tar.gz
   ```
4. Upload both the artifact and its `.asc` signature to the GitHub release. The
   bootstrapper can treat any asset signed by this key (and matching the hash in
   the manifest) as trusted.

With the checksum + GPG approach in place, we can safely treat GitHub releases
from this monorepo as the single source of truth for first-party plugins.
