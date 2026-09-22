#!/bin/sh
set -e

# git-notes universal installer for Linux & macOS
VERSION="v0.1.0"
REPO="isaim0011/git-notes"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64) BINARY="git-notes-linux-x86_64" ;;
      *) echo "Unsupported Linux architecture: $ARCH" >&2; exit 1 ;;
    esac
    ;;
  Darwin)
    case "$ARCH" in
      arm64) BINARY="git-notes-macos-aarch64" ;;
      x86_64) BINARY="git-notes-macos-x86_64" ;;
      *) echo "Unsupported macOS architecture: $ARCH" >&2; exit 1 ;;
    esac
    ;;
  *)
    echo "Unsupported OS: $OS" >&2
    exit 1
    ;;
esac

URL="https://github.com/${REPO}/releases/download/${VERSION}/${BINARY}"
INSTALL_DIR="${HOME}/.local/bin"
mkdir -p "$INSTALL_DIR"

echo "==> Downloading git-notes (${VERSION}) for ${OS} ${ARCH}..."
if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$URL" -o "${INSTALL_DIR}/git-notes"
elif command -v wget >/dev/null 2>&1; then
  wget -qO "${INSTALL_DIR}/git-notes" "$URL"
else
  echo "Error: curl or wget required" >&2
  exit 1
fi

chmod +x "${INSTALL_DIR}/git-notes"

echo "==> Installed git-notes to ${INSTALL_DIR}/git-notes"
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    echo "Notice: Add ${INSTALL_DIR} to your PATH by adding this to your ~/.bashrc or ~/.zshrc:"
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    ;;
esac

echo "==> Run 'git-notes --help' to get started!"
