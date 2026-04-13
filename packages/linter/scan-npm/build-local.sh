#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
LINTER_DIR="$(dirname "$SCRIPT_DIR")"
BINARIES_DIR="$SCRIPT_DIR/binaries"

# Detect current platform
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS-$ARCH" in
  Darwin-arm64)  TARGET="aarch64-apple-darwin" ;;
  Darwin-x86_64) TARGET="x86_64-apple-darwin" ;;
  Linux-x86_64)  TARGET="x86_64-unknown-linux-gnu" ;;
  Linux-aarch64) TARGET="aarch64-unknown-linux-gnu" ;;
  *)
    echo "Unsupported platform: $OS-$ARCH"
    exit 1
    ;;
esac

echo "Building flick-scan for $TARGET..."

mkdir -p "$BINARIES_DIR"
(cd "$LINTER_DIR" && cargo build --release --target "$TARGET")

DEST="$BINARIES_DIR/flick-scan-${TARGET}"
cp "$LINTER_DIR/target/$TARGET/release/flick-scan" "$DEST"

echo "Binary copied to: $DEST"
echo ""
echo "Test with: cd $SCRIPT_DIR && node bin.js --help"
