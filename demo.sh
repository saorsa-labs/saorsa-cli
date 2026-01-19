#!/bin/bash

# Demo script for Saorsa CLI

echo "==================================="
echo "    Saorsa CLI Demonstration"
echo "==================================="
echo

# Build the CLI
echo "1. Building the CLI tool..."
cargo build --package cli --quiet
echo "✅ Build complete"
echo

# Show help
echo "2. CLI Help:"
./target/debug/saorsa --help
echo

# Run sdisk info through CLI
echo "3. Running sdisk info command through CLI:"
echo "   Command: ./target/debug/saorsa --use-system --run sdisk info"
./target/debug/saorsa --use-system --run sdisk info
echo

# Show configuration location
echo "4. Configuration location:"
echo "   Config will be stored at: ~/.config/saorsa-cli/config.toml"
echo "   Binaries cached at: ~/Library/Caches/saorsa-cli/binaries/"
echo

echo "==================================="
echo "To run the interactive menu, use:"
echo "  ./target/debug/saorsa"
echo
echo "To run tools directly:"
echo "  ./target/debug/saorsa-cli --run sb [args]"
echo "  ./target/debug/saorsa-cli --run sdisk [args]"
echo "===================================="
