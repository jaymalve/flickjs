#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
LINTER_DIR="$(dirname "$SCRIPT_DIR")"
BINARIES_DIR="$SCRIPT_DIR/binaries"

# All supported targets
TARGETS=(
  "aarch64-apple-darwin"
  "x86_64-apple-darwin"
  "x86_64-unknown-linux-gnu"
  "aarch64-unknown-linux-gnu"
  "x86_64-pc-windows-msvc"
)

mkdir -p "$BINARIES_DIR"

echo "Building flick-scan for all targets..."
echo "Note: Cross-compilation requires appropriate toolchains."
echo "      Install 'cross' (cargo install cross) for Linux/Windows targets."
echo ""

for target in "${TARGETS[@]}"; do
  echo "--- Building for $target ---"

  # Determine binary name
  if [[ "$target" == *"windows"* ]]; then
    src_binary="flick-scan.exe"
    dest_binary="flick-scan-${target}.exe"
  else
    src_binary="flick-scan"
    dest_binary="flick-scan-${target}"
  fi

  # Use cross for non-native targets, cargo for native
  if rustup target list --installed | grep -q "$target"; then
    echo "  Using cargo (target installed)"
    (cd "$LINTER_DIR" && cargo build --release --target "$target")
  else
    echo "  Using cross (target not installed natively)"
    if ! command -v cross &> /dev/null; then
      echo "  ERROR: 'cross' not found. Install with: cargo install cross"
      echo "  Skipping $target"
      continue
    fi
    (cd "$LINTER_DIR" && cross build --release --target "$target")
  fi

  cp "$LINTER_DIR/target/$target/release/$src_binary" "$BINARIES_DIR/$dest_binary"
  echo "  -> $dest_binary"
done

echo ""
echo "Done! Binaries in $BINARIES_DIR:"
ls -lh "$BINARIES_DIR"
echo ""
echo "To publish: cd $SCRIPT_DIR && npm publish --access public"
