#!/usr/bin/env bash
set -euo pipefail

OWNER="saorsa-labs"
REPO="saorsa-cli"
PROJECT="$OWNER/$REPO"
KEY_URL="${SAORSA_KEY_URL:-https://raw.githubusercontent.com/$PROJECT/main/docs/signing/saorsa-public.asc}"
TMP_DIR="$(mktemp -d 2>/dev/null || mktemp -d -t saorsa)"

cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

log() {
  printf '[saorsa] %s\n' "$*"
}

fail() {
  printf '[saorsa] error: %s\n' "$*" >&2
  exit 1
}

need() {
  command -v "$1" >/dev/null 2>&1 || fail "missing required command: $1"
}

need curl
need tar
need gpg

detect_target() {
  if [ -n "${SAORSA_TARGET:-}" ]; then
    echo "$SAORSA_TARGET"
    return
  fi

  uname_s="$(uname -s)"
  uname_m="$(uname -m)"
  case "$uname_s" in
    Linux)
      case "$uname_m" in
        x86_64|amd64) echo "x86_64-unknown-linux-gnu" ;;
        arm64|aarch64) echo "aarch64-unknown-linux-musl" ;;
        *) fail "unsupported Linux architecture: $uname_m" ;;
      esac
      ;;
    Darwin)
      case "$uname_m" in
        x86_64) echo "x86_64-apple-darwin" ;;
        arm64) echo "aarch64-apple-darwin" ;;
        *) fail "unsupported macOS architecture: $uname_m" ;;
      esac
      ;;
    *)
      fail "unsupported operating system: $uname_s"
      ;;
  esac
}

VERSION="${SAORSA_VERSION:-latest}"
TARGET="$(detect_target)"
ASSET="saorsa-cli-${TARGET}.tar.gz"
SIGNATURE="${ASSET}.asc"

if [ "$VERSION" = "latest" ]; then
  DOWNLOAD_BASE="https://github.com/${PROJECT}/releases/latest/download"
else
  DOWNLOAD_BASE="https://github.com/${PROJECT}/releases/download/${VERSION}"
fi

download() {
  local url="$1"
  local dest="$2"
  log "downloading ${url}"
  curl -fsSL "$url" -o "$dest" || fail "failed to download $url"
}

ARCHIVE_PATH="$TMP_DIR/$ASSET"
SIG_PATH="$TMP_DIR/$SIGNATURE"

download "${DOWNLOAD_BASE}/${ASSET}" "$ARCHIVE_PATH"
download "${DOWNLOAD_BASE}/${SIGNATURE}" "$SIG_PATH"

log "ensuring signing key is installed"
if ! gpg --list-keys "david@saorsalabs.com" >/dev/null 2>&1; then
  curl -fsSL "$KEY_URL" -o "$TMP_DIR/saorsa.pub" || fail "failed to download signing key"
  gpg --batch --import "$TMP_DIR/saorsa.pub" >/dev/null 2>&1 || fail "failed to import signing key"
fi

log "verifying archive signature"
gpg --batch --verify "$SIG_PATH" "$ARCHIVE_PATH" >/dev/null 2>&1 || fail "signature verification failed"

log "extracting archive"
tar -xzf "$ARCHIVE_PATH" -C "$TMP_DIR"

install_dir="${SAORSA_PREFIX:-/usr/local/bin}"
if [ ! -w "$install_dir" ]; then
  install_dir="$HOME/.local/bin"
  mkdir -p "$install_dir"
fi

log "installing binaries to ${install_dir}"
for bin in saorsa saorsa-cli sb sdisk; do
  if [ -f "$TMP_DIR/$bin" ]; then
    install -m 0755 "$TMP_DIR/$bin" "$install_dir/$bin"
  fi
done

if ! printf '%s' "$PATH" | tr ':' '\n' | grep -qx "$install_dir"; then
  log "note: add ${install_dir} to your PATH if you have not already"
fi

log "installation complete. try running 'saorsa' or 'saorsa-cli'."
