#!/bin/bash

# Script to validate a release build locally and optionally push a release tag.
# Usage: ./scripts/create-release.sh [version]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get version from argument or prompt
VERSION=${1:-}
if [ -z "$VERSION" ]; then
    echo -e "${YELLOW}Enter release version (e.g., v0.1.0):${NC}"
    read VERSION
fi

# Validate version format
if [[ ! "$VERSION" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo -e "${RED}Error: Version must be in format v0.0.0${NC}"
    exit 1
fi

echo -e "${GREEN}Creating release ${VERSION}...${NC}"

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d ".github" ]; then
    echo -e "${RED}Error: Must run from repository root${NC}"
    exit 1
fi

# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    echo -e "${YELLOW}Warning: You have uncommitted changes${NC}"
    echo "Do you want to continue? (y/n)"
    read -r response
    if [[ ! "$response" =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Build all binaries in release mode
echo -e "${GREEN}Building release binaries...${NC}"
cargo build --release --workspace

# Verify all binaries exist
echo -e "${GREEN}Verifying binaries...${NC}"
for binary in saorsa saorsa-cli sb sdisk; do
    if [ ! -f "target/release/$binary" ]; then
        echo -e "${RED}Error: Binary $binary not found${NC}"
        exit 1
    fi
    echo "  ✓ $binary"
done

# Get current platform for local testing
PLATFORM=$(uname -s)
ARCH=$(uname -m)

case "$PLATFORM" in
    Darwin)
        if [ "$ARCH" = "arm64" ]; then
            TARGET="aarch64-apple-darwin"
        else
            TARGET="x86_64-apple-darwin"
        fi
        ;;
    Linux)
        if [ "$ARCH" = "aarch64" ]; then
            TARGET="aarch64-unknown-linux-gnu"
        else
            TARGET="x86_64-unknown-linux-gnu"
        fi
        ;;
    *)
        echo -e "${RED}Unsupported platform: $PLATFORM${NC}"
        exit 1
        ;;
esac

# Create local test archive
echo -e "${GREEN}Creating test archive for $TARGET...${NC}"
cd target/release
tar czf ../../saorsa-cli-${TARGET}-test.tar.gz saorsa saorsa-cli sb sdisk
cd ../..
echo -e "  ✓ Created saorsa-cli-${TARGET}-test.tar.gz"

# Test the binaries
echo -e "${GREEN}Testing binaries...${NC}"
./target/release/saorsa --version || echo "  ⚠ saorsa version check failed"
./target/release/saorsa-cli --version || echo "  ⚠ saorsa-cli version check failed"
./target/release/sb --version || echo "  ⚠ sb version check failed"
./target/release/sdisk --version || echo "  ⚠ sdisk version check failed"

# Create git tag
echo -e "${GREEN}Creating git tag ${VERSION}...${NC}"
echo "Do you want to create and push the tag? (y/n)"
read -r response
if [[ "$response" =~ ^[Yy]$ ]]; then
    git tag -a "$VERSION" -m "Release $VERSION

Includes:
- saorsa: Unified tabbed terminal workspace
- saorsa-cli: Bootstrapper and updater
- sb: Saorsa Browser - Terminal Markdown Browser/Editor
- sdisk: Saorsa Disk - Disk cleanup utility"
    
    echo -e "${GREEN}Tag created locally${NC}"
    echo "Push tag to trigger GitHub Actions release? (y/n)"
    read -r response
    if [[ "$response" =~ ^[Yy]$ ]]; then
        git push origin "$VERSION"
        echo -e "${GREEN}Tag pushed! GitHub Actions will now build and create the release.${NC}"
        echo -e "${YELLOW}Monitor the build at: https://github.com/saorsa-labs/saorsa-cli/actions${NC}"
    else
        echo -e "${YELLOW}Tag created but not pushed. Push later with: git push origin $VERSION${NC}"
    fi
else
    echo -e "${YELLOW}Tag not created. You can create it manually with:${NC}"
    echo "  git tag -a $VERSION -m \"Release $VERSION\""
    echo "  git push origin $VERSION"
fi

echo -e "${GREEN}Done!${NC}"