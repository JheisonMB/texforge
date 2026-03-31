#!/bin/sh
# install.sh — download and install texforge + tectonic from GitHub Releases
# Usage: curl -fsSL https://raw.githubusercontent.com/JheisonMB/texforge/main/install.sh | sh
set -eu

REPO="JheisonMB/texforge"
BINARY="texforge"
TECTONIC_VERSION="0.15.0+20251006"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
TEXFORGE_DIR="$HOME/.texforge/bin"

info() { printf '  \033[1;34m%s\033[0m %s\n' "$1" "$2"; }
error() { printf '  \033[1;31merror:\033[0m %s\n' "$1" >&2; exit 1; }

# --- detect OS ---
OS="$(uname -s)"
case "$OS" in
  Linux*)  OS_TARGET="unknown-linux-musl" ;;
  Darwin*) OS_TARGET="apple-darwin" ;;
  *)       error "Unsupported OS: $OS (only Linux and macOS are supported)" ;;
esac

# --- detect arch ---
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64|amd64)   ARCH_TARGET="x86_64" ;;
  arm64|aarch64)   ARCH_TARGET="aarch64" ;;
  *)               error "Unsupported architecture: $ARCH" ;;
esac

TARGET="${ARCH_TARGET}-${OS_TARGET}"
info "platform" "$TARGET"

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

# ============================================================
# 1. Install tectonic (LaTeX engine)
# ============================================================

if command -v tectonic >/dev/null 2>&1; then
  info "tectonic" "already installed ($(tectonic --version 2>/dev/null || echo 'unknown version'))"
elif [ -x "$TEXFORGE_DIR/tectonic" ]; then
  info "tectonic" "already installed at $TEXFORGE_DIR/tectonic"
else
  info "tectonic" "installing v${TECTONIC_VERSION}..."

  TECTONIC_ARCHIVE="tectonic-${TECTONIC_VERSION}-${TARGET}.tar.gz"
  TECTONIC_URL="https://github.com/tectonic-typesetting/tectonic/releases/download/continuous/${TECTONIC_ARCHIVE}"

  info "download" "$TECTONIC_URL"
  HTTP_CODE=$(curl -fSL -w '%{http_code}' -o "$TMPDIR/$TECTONIC_ARCHIVE" "$TECTONIC_URL" 2>/dev/null) || true
  [ "$HTTP_CODE" = "200" ] || error "Tectonic download failed (HTTP $HTTP_CODE). URL:\n  $TECTONIC_URL"

  tar xzf "$TMPDIR/$TECTONIC_ARCHIVE" -C "$TMPDIR"
  [ -f "$TMPDIR/tectonic" ] || error "Tectonic binary not found in archive"

  mkdir -p "$TEXFORGE_DIR"
  mv "$TMPDIR/tectonic" "$TEXFORGE_DIR/tectonic"
  chmod +x "$TEXFORGE_DIR/tectonic"
  info "installed" "$TEXFORGE_DIR/tectonic"
fi

# ============================================================
# 2. Install texforge
# ============================================================

# --- resolve latest version ---
if [ -n "${VERSION:-}" ]; then
  TAG="v$VERSION"
  info "version" "$TAG (pinned)"
else
  TAG=$(curl -fsSL -o /dev/null -w '%{url_effective}' "https://github.com/$REPO/releases/latest" | rev | cut -d'/' -f1 | rev)
  [ -z "$TAG" ] && error "Could not resolve latest release tag"
  info "version" "$TAG (latest)"
fi

# --- download ---
ARCHIVE="${BINARY}-${TAG}-${TARGET}.tar.gz"
URL="https://github.com/$REPO/releases/download/${TAG}/${ARCHIVE}"

info "download" "$URL"
HTTP_CODE=$(curl -fSL -w '%{http_code}' -o "$TMPDIR/$ARCHIVE" "$URL" 2>/dev/null) || true
[ "$HTTP_CODE" = "200" ] || error "Download failed (HTTP $HTTP_CODE). Check that $TAG exists for $TARGET at:\n  $URL"

# --- extract ---
tar xzf "$TMPDIR/$ARCHIVE" -C "$TMPDIR"
[ -f "$TMPDIR/$BINARY" ] || error "Binary not found in archive"

# --- install ---
mkdir -p "$INSTALL_DIR"
mv "$TMPDIR/$BINARY" "$INSTALL_DIR/$BINARY"
chmod +x "$INSTALL_DIR/$BINARY"
info "installed" "$INSTALL_DIR/$BINARY"

# ============================================================
# 3. Ensure PATH includes both directories
# ============================================================

PATHS_TO_ADD=""
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *) PATHS_TO_ADD="$INSTALL_DIR" ;;
esac
case ":$PATH:" in
  *":$TEXFORGE_DIR:"*) ;;
  *) PATHS_TO_ADD="$PATHS_TO_ADD $TEXFORGE_DIR" ;;
esac

if [ -n "$PATHS_TO_ADD" ]; then
  for dir in $PATHS_TO_ADD; do
    export PATH="$dir:$PATH"
  done

  for profile in "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.profile"; do
    if [ -f "$profile" ]; then
      for dir in $PATHS_TO_ADD; do
        if ! grep -q "export PATH=\"$dir:\$PATH\"" "$profile" 2>/dev/null; then
          printf '\n# Added by texforge installer\nexport PATH="%s:$PATH"\n' "$dir" >> "$profile"
          info "updated" "$profile"
        fi
      done
    fi
  done
fi

# ============================================================
# 4. Verify
# ============================================================

info "done" "$($INSTALL_DIR/$BINARY --version 2>/dev/null || echo "$BINARY installed")"
TECTONIC_BIN=$(command -v tectonic 2>/dev/null || echo "$TEXFORGE_DIR/tectonic")
info "tectonic" "$($TECTONIC_BIN --version 2>/dev/null || echo "installed")"
echo ""
info "ready" "Run 'texforge new my-project' to get started!"
