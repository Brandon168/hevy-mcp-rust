#!/bin/sh
# hevy-mcp-rust installer

set -e

REPO="Brandon168/hevy-mcp-rust"
EXE_NAME="hevy-mcp"
INSTALL_DIR="/usr/local/bin"

# Detect OS and Architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
  darwin)  PLATFORM="macos" ;;
  linux)   PLATFORM="linux" ;;
  *)       echo "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
  x86_64)  ARCH_NAME="x86_64" ;;
  arm64|aarch64) ARCH_NAME="aarch64" ;;
  *)       echo "Unsupported Architecture: $ARCH"; exit 1 ;;
esac

# Construct Tag/URL
TAG=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep -Po '"tag_name": "\K.*?(?=")')
if [ -z "$TAG" ]; then
    # Fallback if grep -Po is not available (macOS default grep)
    TAG=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | head -n 1 | cut -d '"' -f 4)
fi

FILENAME="${EXE_NAME}-${PLATFORM}-${ARCH_NAME}.tar.gz"
URL="https://github.com/$(echo $REPO)/releases/download/$(echo $TAG)/$(echo $FILENAME)"

echo "Downloading $EXE_NAME $TAG for $PLATFORM-$ARCH_NAME..."

# Temporary directory for download
TMP_DIR=$(mktemp -d)
curl -L "$URL" -o "$TMP_DIR/$FILENAME"

# Extract
tar -xzf "$TMP_DIR/$FILENAME" -C "$TMP_DIR"

# Install
echo "Installing to $INSTALL_DIR/$EXE_NAME..."
if [ -w "$INSTALL_DIR" ]; then
    mv "$TMP_DIR/$EXE_NAME" "$INSTALL_DIR/"
else
    echo "Requires sudo to install to $INSTALL_DIR"
    sudo mv "$TMP_DIR/$EXE_NAME" "$INSTALL_DIR/"
fi

# Cleanup
rm -rf "$TMP_DIR"

echo "Successfully installed $EXE_NAME!"
echo "Run it using: $EXE_NAME --help"
