# NPM Publishing Guide for glm-plan-usage

This guide explains how to publish the GLM Usage Status plugin to npm, making it easy for users to install via `npm install -g glm-plan-usage`.

## Table of Contents
1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Building for All Platforms](#building-for-all-platforms)
4. [Packaging](#packaging)
5. [Testing](#testing)
6. [Publishing](#publishing)
7. [Post-Publishing](#post-publishing)
8. [Troubleshooting](#troubleshooting)

---

## Overview

The glm-plan-usage plugin is a Rust application that we distribute via npm. This approach combines:
- **Performance**: Native Rust binary
- **Distribution**: npm's extensive registry
- **Installation**: Simple `npm install` command

### How It Works

1. Rust binary is compiled for multiple platforms
2. Binaries are bundled in an npm package
3. Node.js wrapper executes the appropriate binary
4. Users install via npm with automatic platform detection

---

## Prerequisites

### Required Tools

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js and npm
# Install from https://nodejs.org/ or use nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install --lts

# Cross-compilation tools (optional, for building on one platform)
# Linux
sudo apt install gcc-mingw-w64-x86-64  # For Windows builds

# macOS (already included with Xcode)
xcode-select --install
```

### Verify Installation

```bash
rustc --version  # Should be 1.70+
node --version   # Should be 14+
npm --version    # Should be 6+
```

### npm Account

```bash
# Create npm account at https://www.npmjs.com/signup

# Login to npm
npm login
```

---

## Building for All Platforms

### Option 1: Build on Each Platform (Recommended)

Build on each target platform separately for best results:

**On Linux**:
```bash
# Install cross-compilation tools
sudo apt install gcc-x86-64-linux-gnu gcc-aarch64-linux-gnu

# Add targets
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu

# Build
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
```

**On macOS**:
```bash
# Add targets
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# Build (Intel Mac)
cargo build --release --target x86_64-apple-darwin

# Build (Apple Silicon Mac)
cargo build --release --target aarch64-apple-darwin
```

**On Windows**:
```bash
# Add target
rustup target add x86_64-pc-windows-msvc

# Build
cargo build --release --target x86_64-pc-windows-msvc
```

### Option 2: Automated Build Script

Use the provided build script (limited by host platform):

```bash
chmod +x build-all.sh
./build-all.sh
```

**Note**: The build script can only build for the current platform family. For a full release, build on each platform.

### Option 3: GitHub Actions (Recommended for CI/CD)

Create `.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            binary: glm-plan-usage
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
            binary: glm-plan-usage
          - os: macos-latest
            target: x86_64-apple-darwin
            binary: glm-plan-usage
          - os: macos-12
            target: aarch64-apple-darwin
            binary: glm-plan-usage
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            binary: glm-plan-usage.exe

    steps:
      - name: Checkout
        uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: ${{ matrix.target }}
          override: true

      - name: Build
        run: cargo build --release --target ${{ matrix.target }}

      - name: Upload binary
        uses: actions/upload-artifact@v3
        with:
          name: ${{ matrix.target }}
          path: target/${{ matrix.target }}/release/${{ matrix.binary }}
```

---

## Packaging

After building binaries for all platforms:

### Step 1: Run Package Script

```bash
npm run package
```

This script:
1. Creates `lib/` directory
2. Copies each binary to `lib/{target-triple}/`
3. Sets executable permissions
4. Verifies all binaries are present

### Step 2: Verify Package Structure

```bash
# Check directory structure
find lib -type f

# Expected output:
# lib/x86_64-unknown-linux-gnu/glm-plan-usage
# lib/aarch64-unknown-linux-gnu/glm-plan-usage
# lib/x86_64-apple-darwin/glm-plan-usage
# lib/aarch64-apple-darwin/glm-plan-usage
# lib/x86_64-pc-windows-msvc/glm-plan-usage.exe
```

### Step 3: Check File Sizes

```bash
du -sh lib/*/*
```

Expected sizes:
- Linux: ~3.1MB
- macOS: ~2.8MB
- Windows: ~3.0MB

### Step 4: Verify npm/main/package.json

```bash
cat npm/main/package.json | grep -A 5 '"files"'
```

Should include:
```json
"files": [
  "bin",
  "lib",
  "README.md",
  "LICENSE"
]
```

---

## Testing

### Test Locally Before Publishing

```bash
# Run test suite
npm test

# Manual test
echo '{"model":{"id":"test"}}' | npm start
```

### Test with Local npm Registry (Optional)

```bash
# Create a local registry for testing
npx verdaccio

# Publish to local registry
npm publish --registry http://localhost:4873

# Test installation from local registry
npm install -g glm-plan-usage --registry http://localhost:4873
```

### Test on Different Platforms

Before publishing to npm, test on each platform:

1. **Linux**: Test on Ubuntu/Debian
2. **macOS**: Test on Intel and Apple Silicon
3. **Windows**: Test on Windows 10/11

```bash
# Install and test
npm install -g glm-plan-usage
glm-plan-usage --help
echo '{"model":{"id":"test"}}' | glm-plan-usage
```

---

## Publishing

### Step 1: Update Version

Edit `npm/main/package.json` and `Cargo.toml`:

```bash
# Update version numbers
npm version patch  # 0.1.0 → 0.1.1
npm version minor  # 0.1.0 → 0.2.0
npm version major  # 0.1.0 → 1.0.0
```

### Step 2: Update CHANGELOG

Create/update `CHANGELOG.md`:

```markdown
# Changelog

## [0.1.0] - 2025-01-30

### Added
- Initial npm release
- GLM usage tracking (tokens and MCP)
- Color-coded warnings
- Caching system
- Multi-platform support
```

### Step 3: Pre-Publish Checklist

```bash
# ✓ All binaries built
ls lib/*/*

# ✓ Version numbers updated
cat package.json | grep version
cat Cargo.toml | grep version

# ✓ Tests passing
npm test

# ✓ README updated
cat README.md | grep npm
```

### Step 4: Dry Run

Test publishing without actually publishing:

```bash
npm publish --dry-run
```

This shows what would be published without uploading to npm.

### Step 5: Publish to npm

```bash
# Publish to public npm registry
npm publish

# With tagged version
npm publish --tag beta

# With OTP (if enabled)
npm publish --otp <code>
```

### Step 6: Verify Publication

```bash
# Check if package is published
npm view glm-plan-usage

# View package info
npm info glm-plan-usage

# Open in browser
npm repo glm-plan-usage
```

---

## Post-Publishing

### Update Repository

```bash
# Add git tag
git tag -a v0.1.0 -m "Release v0.1.0"

# Push tags
git push origin v0.1.0

# Create GitHub release
gh release create v0.1.0 --notes "Release v0.1.0"
```

### Update Documentation

Update `README.md` with installation instructions:

```markdown
## Installation

### Via npm (Recommended)

```bash
npm install -g glm-plan-usage
```

### Manual Installation

See [MANUAL_INSTALL.md](MANUAL_INSTALL.md)
```

### Announce Release

- Update GitHub releases
- Post to relevant communities
- Update documentation

---

## Troubleshooting

### Issue: Binary Not Found

**Error**: `Error: Binary not found for platform linux and architecture x64`

**Solution**:
```bash
# Rebuild for missing platform
cargo build --release --target x86_64-unknown-linux-gnu

# Re-run package script
npm run package
```

### Issue: Permission Denied

**Error**: `EACCES: permission denied`

**Solution**:
```bash
# Make binary executable
chmod +x lib/x86_64-unknown-linux-gnu/glm-plan-usage

# Re-package
npm run package
```

### Issue: Package Too Large

**Error**: `Package size exceeds limit (50MB)`

**Solution**:
```bash
# Strip binaries
strip target/*/release/glm-plan-usage

# Rebuild with optimizations
cargo build --release

# Check sizes
du -sh lib/*/*
```

### Issue: Wrong Platform Binary

**Error**: Plugin crashes or doesn't work

**Solution**:
```bash
# Verify binary for your platform
npm install -g glm-plan-usage
which glm-plan-usage
file $(which glm-plan-usage)

# Should show correct platform (e.g., ELF 64-bit LSB for Linux)
```

### Issue: npm Publish Fails

**Error**: `403 Forbidden` or `E403`

**Solutions**:
```bash
# Check if package name is taken
npm view glm-plan-usage

# If taken, choose a different name
# Update npm/main/package.json with scoped package
npm publish --access public
```

---

## Best Practices

### Version Management

- Follow Semantic Versioning (semver)
- Update CHANGELOG for each release
- Tag git commits with version numbers

### Security

- Never include API tokens in package
- Use `.npmignore` to exclude sensitive files
- Enable two-factor authentication on npm

### Testing

- Test on all supported platforms before release
- Use `--dry-run` before publishing
- Verify installation with `npm install -g`

### Documentation

- Keep README up to date
- Include installation instructions
- Provide troubleshooting guide

---

## Automation Script

Complete release automation:

```bash
#!/bin/bash
# release.sh - Automate the release process

set -e

echo "🚀 Starting release process..."

# 1. Run tests
echo "📋 Running tests..."
npm test

# 2. Build binaries
echo "🔨 Building binaries..."
cargo build --release

# 3. Package
echo "📦 Packaging..."
npm run package

# 4. Dry run
echo "🔍 Dry run..."
npm publish --dry-run

# 5. Prompt for confirmation
read -p "Publish to npm? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]
then
    # 6. Publish
    echo "📤 Publishing..."
    npm publish

    # 7. Tag and push
    VERSION=$(node -p "require('./npm/main/package.json').version")
    git tag -a "v$VERSION" -m "Release v$VERSION"
    git push origin "v$VERSION"

    echo "✅ Release complete!"
else
    echo "❌ Release cancelled"
fi
```

Make it executable:
```bash
chmod +x release.sh
./release.sh
```

---

## Summary

### Quick Reference

```bash
# 1. Build for current platform
cargo build --release

# 2. Package for npm
npm run package

# 3. Test
npm test

# 4. Publish
npm publish

# 5. Verify
npm info glm-plan-usage
```

### Platform Support Matrix

| Platform | Architecture | Status | Binary |
|----------|-------------|--------|--------|
| Linux | x64 (x86_64) | ✅ | glm-plan-usage |
| Linux | arm64 (aarch64) | ✅ | glm-plan-usage |
| macOS | Intel (x86_64) | ✅ | glm-plan-usage |
| macOS | Apple Silicon (aarch64) | ✅ | glm-plan-usage |
| Windows | x64 (x86_64) | ✅ | glm-plan-usage.exe |

### Support

For issues or questions:
- GitHub: https://github.com/jukanntenn/glm-plan-usage/issues
- npm: https://www.npmjs.com/package/glm-plan-usage

---

**Happy Publishing!** 🎉
