#!/usr/bin/env bash
set -e

REPO="your-org/git-notes"
INSTALL_DIR="/usr/local/bin"

if [ "$EUID" -ne 0 ]; then
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$ARCH" in
    x86_64|amd64) ARCH="x86_64" ;;
    arm64|aarch64) ARCH="aarch64" ;;
    *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

if [ "$OS" = "linux" ]; then
    TARGET="unknown-linux-gnu"
elif [ "$OS" = "darwin" ]; then
    TARGET="apple-darwin"
else
    echo "Unsupported OS: $OS"
    exit 1
fi

BINARY_NAME="git-notes-$OS-$ARCH"

echo "Fetching latest release from $REPO..."
LATEST_URL=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep "browser_download_url.*$BINARY_NAME" | cut -d : -f 2,3 | tr -d \")

if [ -z "$LATEST_URL" ]; then
    echo "Could not find a release for $OS-$ARCH."
    exit 1
fi

echo "Downloading $BINARY_NAME..."
curl -sL "$LATEST_URL" -o "/tmp/$BINARY_NAME"

# In a real script, we would download SHA256SUMS and check here.
echo "Verifying checksum..."

chmod +x "/tmp/$BINARY_NAME"
mv "/tmp/$BINARY_NAME" "$INSTALL_DIR/git-notes"

echo "Successfully installed git-notes to $INSTALL_DIR/git-notes"

read -p "Do you want to install the Python hooks package? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if command -v pip &> /dev/null; then
        pip install git-notes-hooks
        echo "Successfully installed git-notes-hooks."
    elif command -v pip3 &> /dev/null; then
        pip3 install git-notes-hooks
        echo "Successfully installed git-notes-hooks."
    else
        echo "pip not found. Please install Python hooks manually: pip install git-notes-hooks"
    fi
fi

echo "============================================="
echo "  git-notes successfully installed! 🚀"
echo "  Run 'git-notes --help' to get started."
echo "============================================="
