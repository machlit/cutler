#!/usr/bin/env bash

set -e

BINARY="cutler"
INSTALL_DIR="/usr/local/bin"
MANPAGE_DIR="/usr/local/share/man/man1"
OS="$(uname -s)"
ARCH="$(uname -m)"

# Only macOS is supported
if [[ "$OS" != "Darwin" ]]; then
  echo "❌ This script can only run on macOS. Detected: $OS"
  exit 1
fi

# Determine ARCH_PREFIX based on architecture
if [[ "$ARCH" == "x86_64" ]]; then
  ARCH_PREFIX="x86_64"
elif [[ "$ARCH" == "arm64" ]]; then
  ARCH_PREFIX="arm64"
else
  echo "❌ Unsupported architecture: $ARCH"
  exit 1
fi

echo "🔒 Uninstalling $BINARY binary from $INSTALL_DIR..."
if [ -f "$INSTALL_DIR/$BINARY" ]; then
  sudo rm "$INSTALL_DIR/$BINARY"
  echo "✅ Removed $INSTALL_DIR/$BINARY"
else
  echo "⚠️  $INSTALL_DIR/$BINARY not found"
fi

echo "📖 Uninstalling manpage from $MANPAGE_DIR..."

if [ -f "$MANPAGE_DIR/$BINARY.1.gz" ]; then
  sudo rm "$MANPAGE_DIR/$BINARY.1.gz"
  echo "✅ Removed $MANPAGE_DIR/$BINARY.1.gz"
else
  echo "⚠️ Compressed manpage for $BINARY not found in $MANPAGE_DIR"
fi

if [ -f "$MANPAGE_DIR/$BINARY.1" ]; then
  sudo rm "$MANPAGE_DIR/$BINARY.1"
  echo "✅ Removed $MANPAGE_DIR/$BINARY.1"
else
  echo "⚠️ Manpage for $BINARY not found in $MANPAGE_DIR"
fi

echo "🎉 Uninstallation complete."
echo "Note that the snapshot file in the configuration directory will not be removed.\nYou may remove it yourself if needed."
