#!/bin/bash
set -e

# ========================================
# Get script directory and project root
# ========================================
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Color output (for local display only, not in workflow)
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default local registry URL
LOCAL_REGISTRY="${NPM_REGISTRY:-http://localhost:4873/}"

# Get version from Cargo.toml
VERSION=$(grep '^version = ' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/' | tr -d '"')

if [ -z "$VERSION" ]; then
  echo -e "${RED}Error: Unable to extract version from Cargo.toml${NC}"
  exit 1
fi

# Define all paths
TARGET_DIR="$PROJECT_ROOT/target"
NPM_DIR="$PROJECT_ROOT/npm"
EXTRACTED_DIR="$NPM_DIR/extracted"
NPM_PUBLISH_DIR="$PROJECT_ROOT/npm-publish"
NPM_SCRIPTS_DIR="$NPM_DIR/scripts"
NPM_PLATFORMS_DIR="$NPM_DIR/platforms"

# Cleanup function
cleanup() {
  echo ""
  echo "=== Cleaning up intermediate directories ==="
  rm -rf "$EXTRACTED_DIR"
  rm -rf "$NPM_PUBLISH_DIR"
  echo "✓ Cleaned up: $EXTRACTED_DIR"
  echo "✓ Cleaned up: $NPM_PUBLISH_DIR"
}

# Trap to ensure cleanup runs on exit
trap cleanup EXIT

echo "=========================================="
echo "Publishing to local NPM registry"
echo "=========================================="
echo "Project Root: $PROJECT_ROOT"
echo "Registry: $LOCAL_REGISTRY"
echo "Version: $VERSION"
echo ""

# ========================================
# Step: Extract binaries from target directory
# (Simulates "Extract binaries from archives" in workflow)
# ========================================
rm -rf "$EXTRACTED_DIR"
mkdir -p "$EXTRACTED_DIR"

echo "=== Extracting binaries from target directory ==="

# Platform mapping (same as workflow artifact names → extracted names)
declare -A ARTIFACT_MAP=(
  ["x86_64-apple-darwin"]="darwin-x64"
  ["aarch64-apple-darwin"]="darwin-arm64"
  ["x86_64-unknown-linux-gnu"]="linux-x64"
  ["x86_64-unknown-linux-musl"]="linux-x64-musl"
  ["aarch64-unknown-linux-gnu"]="linux-arm64"
  ["aarch64-unknown-linux-musl"]="linux-arm64-musl"
  ["x86_64-pc-windows-gnu"]="win32-x64"
)

for RUST_TARGET in "${!ARTIFACT_MAP[@]}"; do
  NPM_PLATFORM="${ARTIFACT_MAP[$RUST_TARGET]}"

  if [ "$NPM_PLATFORM" = "win32-x64" ]; then
    BINARY_NAME="glm-plan-usage.exe"
    EXTRACTED_NAME="glm-plan-usage-win32-x64.exe"
  else
    BINARY_NAME="glm-plan-usage"
    EXTRACTED_NAME="glm-plan-usage-$NPM_PLATFORM"
  fi

  SOURCE_PATH="$TARGET_DIR/$RUST_TARGET/release/$BINARY_NAME"

  if [ -f "$SOURCE_PATH" ]; then
    cp "$SOURCE_PATH" "$EXTRACTED_DIR/$EXTRACTED_NAME"
    echo "✓ Extracted: $RUST_TARGET → $EXTRACTED_NAME"
  else
    echo "⚠ Skipped: $RUST_TARGET (not found)"
  fi
done

echo ""
echo "=== List extracted files ==="
ls -la "$EXTRACTED_DIR/"
echo ""

# ========================================
# Step: Prepare NPM packages
# (Exactly matches workflow "Prepare NPM packages")
# ========================================
echo "=== Prepare NPM packages ==="
echo "# Prepare packages with version management"
cd "$NPM_DIR"
node "$NPM_SCRIPTS_DIR/prepare-packages.js" "$VERSION"

echo ""
echo "# Copy binaries to platform directories"

# macOS x64
if [ -f "$EXTRACTED_DIR/glm-plan-usage-darwin-x64" ]; then
  cp "$EXTRACTED_DIR/glm-plan-usage-darwin-x64" "$NPM_PUBLISH_DIR/darwin-x64/glm-plan-usage"
  echo "✓ Copied: darwin-x64"
fi

# macOS ARM64
if [ -f "$EXTRACTED_DIR/glm-plan-usage-darwin-arm64" ]; then
  cp "$EXTRACTED_DIR/glm-plan-usage-darwin-arm64" "$NPM_PUBLISH_DIR/darwin-arm64/glm-plan-usage"
  echo "✓ Copied: darwin-arm64"
fi

# Linux x64
if [ -f "$EXTRACTED_DIR/glm-plan-usage-linux-x64" ]; then
  cp "$EXTRACTED_DIR/glm-plan-usage-linux-x64" "$NPM_PUBLISH_DIR/linux-x64/glm-plan-usage"
  echo "✓ Copied: linux-x64"
fi

# Linux musl (static)
if [ -f "$EXTRACTED_DIR/glm-plan-usage-linux-x64-musl" ]; then
  cp "$EXTRACTED_DIR/glm-plan-usage-linux-x64-musl" "$NPM_PUBLISH_DIR/linux-x64-musl/glm-plan-usage"
  echo "✓ Copied: linux-x64-musl"
fi

# Linux ARM64
if [ -f "$EXTRACTED_DIR/glm-plan-usage-linux-arm64" ]; then
  cp "$EXTRACTED_DIR/glm-plan-usage-linux-arm64" "$NPM_PUBLISH_DIR/linux-arm64/glm-plan-usage"
  echo "✓ Copied: linux-arm64"
fi

# Linux ARM64 musl (static)
if [ -f "$EXTRACTED_DIR/glm-plan-usage-linux-arm64-musl" ]; then
  cp "$EXTRACTED_DIR/glm-plan-usage-linux-arm64-musl" "$NPM_PUBLISH_DIR/linux-arm64-musl/glm-plan-usage"
  echo "✓ Copied: linux-arm64-musl"
fi

# Windows
if [ -f "$EXTRACTED_DIR/glm-plan-usage-win32-x64.exe" ]; then
  cp "$EXTRACTED_DIR/glm-plan-usage-win32-x64.exe" "$NPM_PUBLISH_DIR/win32-x64/glm-plan-usage.exe"
  echo "✓ Copied: win32-x64"
fi

echo ""
echo "# Set executable permissions for Unix binaries"
chmod +x "$NPM_PUBLISH_DIR/darwin-x64/glm-plan-usage" 2>/dev/null || echo "  (darwin-x64 not found)"
chmod +x "$NPM_PUBLISH_DIR/darwin-arm64/glm-plan-usage" 2>/dev/null || echo "  (darwin-arm64 not found)"
chmod +x "$NPM_PUBLISH_DIR/linux-x64/glm-plan-usage" 2>/dev/null || echo "  (linux-x64 not found)"
chmod +x "$NPM_PUBLISH_DIR/linux-x64-musl/glm-plan-usage" 2>/dev/null || echo "  (linux-x64-musl not found)"
chmod +x "$NPM_PUBLISH_DIR/linux-arm64/glm-plan-usage" 2>/dev/null || echo "  (linux-arm64 not found)"
chmod +x "$NPM_PUBLISH_DIR/linux-arm64-musl/glm-plan-usage" 2>/dev/null || echo "  (linux-arm64-musl not found)"

echo ""
echo "# Verify packages"
echo "Package structure:"
find "$NPM_PUBLISH_DIR" -name "package.json" -exec echo "=== {} ===" \; -exec head -5 {} \;

echo ""

# ========================================
# Step: Unpublish existing packages (for local testing)
# ========================================
echo "=== Unpublishing existing packages from local registry ==="

# Unpublish main package first (dependencies will be removed)
echo "📦 Unpublishing @jukanntenn/glm-plan-usage..."
npm unpublish --force @jukanntenn/glm-plan-usage --registry "$LOCAL_REGISTRY" 2>/dev/null || echo "  (package not found or already unpublished)"

# Unpublish all platform packages
for platform in darwin-x64 darwin-arm64 linux-x64 linux-x64-musl linux-arm64 linux-arm64-musl win32-x64; do
  echo "📦 Unpublishing @jukanntenn/glm-plan-usage-$platform..."
  npm unpublish --force "@jukanntenn/glm-plan-usage-$platform" --registry "$LOCAL_REGISTRY" 2>/dev/null || echo "  (not found)"
done

echo "✓ Cleanup completed"
echo ""

# ========================================
# Step: Publish platform packages to NPM
# (Matches workflow "Publish platform packages to NPM")
# ========================================
echo "=== Publish platform packages to local NPM registry ==="

PUBLISHED_PLATFORMS=()
for platform in darwin-x64 darwin-arm64 linux-x64 linux-x64-musl linux-arm64 linux-arm64-musl win32-x64; do
  if [ -d "$NPM_PUBLISH_DIR/$platform" ]; then
    echo "📦 Publishing @jukanntenn/glm-plan-usage-$platform"
    (cd "$NPM_PUBLISH_DIR/$platform" && npm publish --registry "$LOCAL_REGISTRY" 2>/dev/null) && {
      echo "✅ Published @jukanntenn/glm-plan-usage-$platform"
      PUBLISHED_PLATFORMS+=("$platform")
    } || {
      echo "⚠ Failed to publish @jukanntenn/glm-plan-usage-$platform"
    }
  fi
done

echo ""

# ========================================
# Step: Wait for NPM registry
# (Matches workflow "Wait for NPM registry")
# ========================================
echo "=== Wait for NPM registry ==="
echo "⏳ Waiting for platform packages to be available on NPM..."
# Use shorter wait time for local registry
sleep 2

echo ""

# ========================================
# Step: Publish main package to NPM
# (Matches workflow "Publish main package to NPM")
# ========================================
echo "=== Publish main package to local NPM registry ==="

cd "$NPM_PUBLISH_DIR/main"

# In workflow, prepare-packages.js already updated versions and dependencies
# We just need to override the registry for local testing
echo "📦 Publishing @jukanntenn/glm-plan-usage"

# Temporarily set registry config for publish
if npm publish --registry "$LOCAL_REGISTRY" --access public 2>/dev/null; then
  echo "✅ Published @jukanntenn/glm-plan-usage"
  echo ""
  echo "🎉 NPM packages published successfully!"

  # List published platforms
  echo ""
  echo "Published platforms:"
  for platform in "${PUBLISHED_PLATFORMS[@]}"; do
    echo "  ✓ $platform"
  done

  echo ""
  echo "Install with: npm install -g @jukanntenn/glm-plan-usage --registry=$LOCAL_REGISTRY"
else
  echo "⚠ Failed to publish @jukanntenn/glm-plan-usage"
  exit 1
fi

echo ""
echo "✨ Done!"
