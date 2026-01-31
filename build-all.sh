#!/bin/bash

# Build script for all target platforms

set -e

echo "Building glm-plan-usage for all platforms..."

# Define targets
TARGETS=(
  "x86_64-unknown-linux-gnu"
  "aarch64-unknown-linux-gnu"
  "x86_64-apple-darwin"
  "aarch64-apple-darwin"
  "x86_64-pc-windows-msvc"
)

# Check if cross-compilation tools are installed
echo "Checking for cross-compilation tools..."

for target in "${TARGETS[@]}"; do
  echo "Building for $target..."

  # Install target if not already installed
  rustup target add "$target" 2>/dev/null || true

  # Build for target
  if [[ "$target" == *"linux"* ]]; then
    cargo build --release --target "$target"
  elif [[ "$target" == *"darwin"* ]]; then
    # macOS builds require special handling
    if [[ "$OSTYPE" == "darwin"* ]]; then
      cargo build --release --target "$target"
    else
      echo "⚠ Skipping $target (can only build on macOS)"
    fi
  elif [[ "$target" == *"windows"* ]]; then
    cargo build --release --target "$target"
  fi
done

echo ""
echo "✓ Build complete!"
echo ""
echo "Next steps:"
echo "  1. npm run package"
echo "  2. npm test"
echo "  3. npm publish"
