# NPM Multi-Platform Binary Distribution - Implementation Details

This document describes the implementation strategy for distributing compiled Rust binaries via npm with automatic platform detection and post-install setup.

---

## Table of Contents

1. [Overview](#overview)
2. [Directory Structure](#directory-structure)
3. [Package Architecture](#package-architecture)
4. [Platform Detection](#platform-detection)
5. [Release Workflow](#release-workflow)
6. [Implementation Patterns](#implementation-patterns)
7. [Common Pitfalls and Solutions](#common-pitfalls-and-solutions)
8. [Customization Guide](#customization-guide)

---

## Overview

### The Problem

Distributing platform-specific binaries (Rust, Go, C++) via npm presents several challenges:

1. **Multi-platform support**: Need separate binaries for Windows, macOS, Linux (x64/ARM), with musl vs glibc variants
2. **Installation size**: Users shouldn't download binaries for platforms they don't use
3. **Integration**: Need to set up binaries in specific locations for tool integration
4. **Package manager differences**: npm, yarn, pnpm have different node_modules structures

### The Solution

Use npm's **optionalDependencies** pattern combined with:
- Platform-specific binary packages (one per platform)
- A main orchestrator package that users install
- Post-install scripts for automatic setup
- Smart binary resolution with fallbacks

---

## Directory Structure

```
npm/
├── main/                          # Main package (@cometix/yourtool)
│   ├── bin/
│   │   └── yourtool.js            # Entry point - delegates to platform binary
│   ├── scripts/
│   │   └── postinstall.js         # Runs after install - sets up integrations
│   ├── package.json               # Main manifest with optionalDependencies
│   └── README.md                  # User documentation
│
├── platforms/                     # Platform-specific templates
│   ├── darwin-x64/
│   │   └── package.json           # macOS Intel binary package manifest
│   ├── darwin-arm64/
│   │   └── package.json           # macOS Apple Silicon binary package manifest
│   ├── linux-x64/
│   │   └── package.json           # Linux x64 (glibc) binary package manifest
│   ├── linux-x64-musl/
│   │   └── package.json           # Linux x64 (musl) binary package manifest
│   ├── linux-arm64/
│   │   └── package.json           # Linux ARM64 (glibc) binary package manifest
│   ├── linux-arm64-musl/
│   │   └── package.json           # Linux ARM64 (musl) binary package manifest
│   └── win32-x64/
│       └── package.json           # Windows x64 binary package manifest
│
└── scripts/
    └── prepare-packages.js        # Release preparation script
```

### Important Notes

- The `platforms/` directory contains **templates** only
- At release time, these are copied to `npm-publish/` with versions injected
- Compiled binaries are added to `npm-publish/{platform}/` during release
- The `npm/` directory stays in source control; `npm-publish/` is transient

---

## Package Architecture

### 1. Main Package (npm/main/package.json)

```json
{
  "name": "@cometix/yourtool",
  "version": "0.0.0",
  "description": "Your tool description",
  "bin": {
    "yourtool": "./bin/yourtool.js"
  },
  "scripts": {
    "postinstall": "node scripts/postinstall.js"
  },
  "optionalDependencies": {
    "@cometix/yourtool-darwin-x64": "0.0.0",
    "@cometix/yourtool-darwin-arm64": "0.0.0",
    "@cometix/yourtool-linux-x64": "0.0.0",
    "@cometix/yourtool-linux-x64-musl": "0.0.0",
    "@cometix/yourtool-linux-arm64": "0.0.0",
    "@cometix/yourtool-linux-arm64-musl": "0.0.0",
    "@cometix/yourtool-win32-x64": "0.0.0"
  },
  "repository": {
    "type": "git",
    "url": "https://github.com/your-org/your-repo"
  },
  "keywords": ["your", "keywords"],
  "author": "Your Name",
  "license": "MIT",
  "engines": {
    "node": ">=14.0.0"
  }
}
```

**Key Points:**
- `version: "0.0.0"` - Placeholder, replaced during release
- `bin` - Points to the JavaScript wrapper, not the binary directly
- `postinstall` - Sets up integrations after installation
- `optionalDependencies` - npm installs ONLY the matching platform

### 2. Platform Packages (npm/platforms/*/package.json)

```json
{
  "name": "@cometix/yourtool-linux-x64",
  "version": "0.0.0",
  "description": "Linux x64 binary for YourTool",
  "files": ["yourtool"],
  "os": ["linux"],
  "cpu": ["x64"],
  "repository": {
    "type": "git",
    "url": "https://github.com/your-org/your-repo"
  },
  "author": "Your Name",
  "license": "MIT"
}
```

**Key Points:**
- `files` - Lists ONLY the binary file to be published
- `os` and `cpu` - Ensure npm only installs on matching platforms
- No `bin` field - the binary is used by the main package

---

## Platform Detection

### Entry Point Binary Resolution

The `bin/yourtool.js` file implements priority-based binary resolution:

```javascript
#!/usr/bin/env node
const { spawnSync } = require('child_process');
const path = require('path');
const fs = require('fs');
const os = require('os');

// Priority 1: Use integration path if exists (e.g., ~/.claude/yourtool/yourtool)
const integrationPath = path.join(
  os.homedir(),
  '.claude',                    // Customize for your integration
  'yourtool',
  process.platform === 'win32' ? 'yourtool.exe' : 'yourtool'
);

if (fs.existsSync(integrationPath)) {
  const result = spawnSync(integrationPath, process.argv.slice(2), {
    stdio: 'inherit',
    shell: false
  });
  process.exit(result.status || 0);
}

// Priority 2: Use npm package binary
const platform = process.platform;
const arch = process.arch;

// Platform detection logic (see below)
let platformKey = detectPlatform(platform, arch);

const packageMap = {
  'darwin-x64': '@cometix/yourtool-darwin-x64',
  'darwin-arm64': '@cometix/yourtool-darwin-arm64',
  'linux-x64': '@cometix/yourtool-linux-x64',
  'linux-x64-musl': '@cometix/yourtool-linux-x64-musl',
  'linux-arm64': '@cometix/yourtool-linux-arm64',
  'linux-arm64-musl': '@cometix/yourtool-linux-arm64-musl',
  'win32-x64': '@cometix/yourtool-win32-x64',
};

const packageName = packageMap[platformKey];
const binaryName = platform === 'win32' ? 'yourtool.exe' : 'yourtool';
const binaryPath = path.join(__dirname, '..', 'node_modules', packageName, binaryName);

if (!fs.existsSync(binaryPath)) {
  console.error(`Error: Binary not found at ${binaryPath}`);
  console.error('Please try reinstalling: npm install -g @cometix/yourtool');
  process.exit(1);
}

const result = spawnSync(binaryPath, process.argv.slice(2), {
  stdio: 'inherit',
  shell: false
});

process.exit(result.status || 0);
```

### Linux libc Detection

On Linux, you need to distinguish between **glibc** and **musl**:

```javascript
function getLibcInfo() {
  try {
    const { execSync } = require('child_process');
    const lddOutput = execSync('ldd --version 2>/dev/null || echo ""', {
      encoding: 'utf8',
      timeout: 1000
    });

    // Check for musl explicitly
    if (lddOutput.includes('musl')) {
      return { type: 'musl' };
    }

    // Parse glibc version: "ldd (GNU libc) 2.35" format
    const match = lddOutput.match(/(?:GNU libc|GLIBC).*?(\d+)\.(\d+)/);
    if (match) {
      const major = parseInt(match[1]);
      const minor = parseInt(match[2]);
      return { type: 'glibc', major, minor };
    }

    // If we can't detect, default to musl for safety (more portable)
    return { type: 'musl' };
  } catch (e) {
    // If detection fails, default to musl (more portable)
    return { type: 'musl' };
  }
}
```

**Why default to musl?**
- musl binaries are more portable (statically linked)
- musl binaries work on glibc systems but not vice versa
- Fallback to the safer option minimizes user issues

### Platform Selection Logic

```javascript
function detectPlatform(platform, arch) {
  let platformKey = `${platform}-${arch}`;

  if (platform === 'linux') {
    const libcInfo = getLibcInfo();

    if (arch === 'arm64') {
      // ARM64 Linux: choose based on libc type and version
      if (libcInfo.type === 'musl' ||
          (libcInfo.type === 'glibc' && (libcInfo.major < 2 || (libcInfo.major === 2 && libcInfo.minor < 35)))) {
        platformKey = 'linux-arm64-musl';
      } else {
        platformKey = 'linux-arm64';
      }
    } else {
      // x64 Linux: choose based on libc type and version
      if (libcInfo.type === 'musl' ||
          (libcInfo.type === 'glibc' && (libcInfo.major < 2 || (libcInfo.major === 2 && libcInfo.minor < 35)))) {
        platformKey = 'linux-x64-musl';
      }
    }
  }

  return platformKey;
}
```

---

## Release Workflow

### Overview

The release workflow has three phases:

1. **Build**: Compile binaries for all platforms
2. **Prepare**: Create npm packages with correct versions
3. **Publish**: Publish to npm registry

### GitHub Actions Workflow

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:

permissions:
  contents: write
  id-token: write

jobs:
  build:
    name: Build for ${{ matrix.target }}
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          # Define your target platforms here
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-22.04
            name: yourtool-linux-x64.tar.gz
          - target: x86_64-unknown-linux-musl
            os: ubuntu-latest
            name: yourtool-linux-x64-static.tar.gz
          # ... add more platforms

    steps:
      - uses: actions/checkout@v4

      # Build your binary here
      - name: Build binary
        run: |
          # Your build commands
          cargo build --release --target ${{ matrix.target }}

      # Package the binary
      - name: Package
        run: |
          mkdir -p dist
          cp target/${{ matrix.target }}/release/yourtool dist/yourtool
          cd dist
          tar czf ../${{ matrix.name }} yourtool

      # Upload artifact
      - uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.name }}
          path: ${{ matrix.name }}

  release:
    name: Create Release and Publish NPM
    runs-on: ubuntu-latest
    needs: build
    if: startsWith(github.ref, 'refs/tags/')

    steps:
      - uses: actions/checkout@v4

      # Download all build artifacts
      - uses: actions/download-artifact@v4
        with:
          path: artifacts

      # Extract binaries from archives
      - name: Extract binaries
        run: |
          mkdir -p extracted
          tar -xzf artifacts/yourtool-linux-x64.tar.gz/yourtool-linux-x64.tar.gz -C extracted
          mv extracted/yourtool extracted/yourtool-linux-x64
          # ... repeat for all platforms

      # Prepare NPM packages
      - name: Prepare NPM packages
        run: |
          # Run prepare script - creates npm-publish/ directory
          node npm/scripts/prepare-packages.js

          # Copy binaries to platform directories
          cp extracted/yourtool-linux-x64 npm-publish/linux-x64/yourtool
          cp extracted/yourtool-darwin-x64 npm-publish/darwin-x64/yourtool
          # ... repeat for all platforms

          # Set executable permissions
          chmod +x npm-publish/*/yourtool

      # Setup Node.js
      - uses: actions/setup-node@v4
        with:
          node-version: '18'
          registry-url: 'https://registry.npmjs.org'

      # Publish platform packages first
      - name: Publish platform packages
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
        run: |
          for platform in darwin-x64 darwin-arm64 linux-x64 linux-x64-musl linux-arm64 linux-arm64-musl win32-x64; do
            echo "📦 Publishing @cometix/yourtool-$platform"
            cd npm-publish/$platform
            npm publish --access public
            cd ../..
          done

      # Wait for npm registry propagation
      - name: Wait for NPM registry
        run: sleep 30

      # Publish main package last
      - name: Publish main package
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
        run: |
          cd npm-publish/main
          npm publish --access public
```

### Prepare Packages Script

The `npm/scripts/prepare-packages.js` script is responsible for:

```javascript
const fs = require('fs');
const path = require('path');

// Get version from git tag or command line
const version = process.env.GITHUB_REF?.replace('refs/tags/v', '') || process.argv[2];
if (!version) {
  console.error('Error: Version not provided');
  process.exit(1);
}

// Define all platforms
const platforms = [
  'darwin-x64',
  'darwin-arm64',
  'linux-x64',
  'linux-x64-musl',
  'linux-arm64',
  'linux-arm64-musl',
  'win32-x64'
];

// Prepare platform packages
platforms.forEach(platform => {
  const sourceDir = path.join(__dirname, '..', 'platforms', platform);
  const targetDir = path.join(__dirname, '..', '..', 'npm-publish', platform);

  // Create directory
  fs.mkdirSync(targetDir, { recursive: true });

  // Read template package.json
  const templatePath = path.join(sourceDir, 'package.json');
  const packageJson = JSON.parse(fs.readFileSync(templatePath, 'utf8'));

  // Update version
  packageJson.version = version;

  // Write to target directory
  fs.writeFileSync(
    path.join(targetDir, 'package.json'),
    JSON.stringify(packageJson, null, 2) + '\n'
  );

  console.log(`✓ Prepared @cometix/yourtool-${platform} v${version}`);
});

// Prepare main package
const mainSource = path.join(__dirname, '..', 'main');
const mainTarget = path.join(__dirname, '..', '..', 'npm-publish', 'main');

// Copy main package files
fs.cpSync(mainSource, mainTarget, { recursive: true });

// Update main package.json
const mainPackageJsonPath = path.join(mainTarget, 'package.json');
const mainPackageJson = JSON.parse(fs.readFileSync(mainPackageJsonPath, 'utf8'));

mainPackageJson.version = version;

// Update optionalDependencies versions
if (mainPackageJson.optionalDependencies) {
  Object.keys(mainPackageJson.optionalDependencies).forEach(dep => {
    if (dep.startsWith('@cometix/yourtool-')) {
      mainPackageJson.optionalDependencies[dep] = version;
    }
  });
}

fs.writeFileSync(
  mainPackageJsonPath,
  JSON.stringify(mainPackageJson, null, 2) + '\n'
);

console.log(`✓ Prepared @cometix/yourtool v${version}`);
```

---

## Implementation Patterns

### Pattern 1: Post-Install Integration Setup

The `postinstall.js` script handles automatic integration:

```javascript
const fs = require('fs');
const path = require('path');
const os = require('os');

// Silent mode detection
const silent = process.env.npm_config_loglevel === 'silent' ||
               process.env.YOURTOOL_SKIP_POSTINSTALL === '1';

if (!silent) {
  console.log('🚀 Setting up YourTool...');
}

try {
  const platform = process.platform;
  const arch = process.arch;

  // Create integration directory
  const integrationDir = path.join(os.homedir(), '.your-app', 'yourtool');
  fs.mkdirSync(integrationDir, { recursive: true });

  // Detect platform (same logic as bin/yourtool.js)
  let platformKey = detectPlatform(platform, arch);

  const packageMap = {
    'darwin-x64': '@cometix/yourtool-darwin-x64',
    // ... other platforms
  };

  const packageName = packageMap[platformKey];
  const binaryName = platform === 'win32' ? 'yourtool.exe' : 'yourtool';
  const targetPath = path.join(integrationDir, binaryName);

  // Find the binary in node_modules
  const sourcePath = findBinaryPath(packageName, binaryName);

  if (!sourcePath) {
    if (!silent) {
      console.log('Binary package not installed, skipping setup');
    }
    process.exit(0);
  }

  // Copy or link the binary
  if (platform === 'win32') {
    fs.copyFileSync(sourcePath, targetPath);
  } else {
    // Try hard link first, fallback to copy
    try {
      if (fs.existsSync(targetPath)) {
        fs.unlinkSync(targetPath);
      }
      fs.linkSync(sourcePath, targetPath);
    } catch {
      fs.copyFileSync(sourcePath, targetPath);
    }
    fs.chmodSync(targetPath, '755');
  }

  if (!silent) {
    console.log('✨ YourTool is ready!');
    console.log(`📍 Location: ${targetPath}`);
  }
} catch (error) {
  // Silent failure - don't break installation
  if (!silent) {
    console.log('Note: Could not auto-configure');
    console.log('The tool will still work.');
  }
}
```

### Pattern 2: Multi-Package Manager Binary Finding

Support npm, yarn, and pnpm:

```javascript
const findBinaryPath = (packageName, binaryName) => {
  const possiblePaths = [
    // npm/yarn: nested in node_modules
    path.join(__dirname, '..', 'node_modules', packageName, binaryName),

    // pnpm: try require.resolve first
    (() => {
      try {
        const packagePath = require.resolve(packageName + '/package.json');
        return path.join(path.dirname(packagePath), binaryName);
      } catch {
        return null;
      }
    })(),

    // pnpm: flat structure fallback with version detection
    (() => {
      const currentPath = __dirname;
      const pnpmMatch = currentPath.match(/(.+\.pnpm)[\\/]([^\\//]+)[\\/]/);
      if (pnpmMatch) {
        const pnpmRoot = pnpmMatch[1];
        const packageNameEncoded = packageName.replace('/', '+');

        try {
          const pnpmContents = fs.readdirSync(pnpmRoot);
          const packagePattern = new RegExp(`^${packageNameEncoded.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}@`);
          const matchingPackage = pnpmContents.find(dir => packagePattern.test(dir));

          if (matchingPackage) {
            return path.join(pnpmRoot, matchingPackage, 'node_modules', packageName, binaryName);
          }
        } catch {
          // Fallback
        }
      }
      return null;
    })()
  ].filter(p => p !== null);

  for (const testPath of possiblePaths) {
    if (fs.existsSync(testPath)) {
      return testPath;
    }
  }
  return null;
};
```

### Pattern 3: Silent Mode for CI/CD

Allow postinstall to run silently:

```javascript
const silent = process.env.npm_config_loglevel === 'silent' ||
               process.env.YOURTOOL_SKIP_POSTINSTALL === '1';

// Use this to suppress output in CI environments
if (!silent) {
  console.log('🚀 Setting up...');
}
```

---

## Common Pitfalls and Solutions

### Pitfall 1: Platform Package Not Found

**Problem:** Users get "Binary not found" errors with pnpm.

**Solution:** The `findBinaryPath()` function above handles multiple package manager layouts. Test with npm, yarn, and pnpm.

### Pitfall 2: Binary Execution Permissions

**Problem:** Unix binaries aren't executable after install.

**Solution:** Always set permissions in postinstall:

```javascript
fs.chmodSync(targetPath, '755');
```

### Pitfall 3: npm Registry Propagation

**Problem:** Main package installs before platform packages are available.

**Solution:** Always publish platform packages first, then wait:

```yaml
- name: Publish platform packages
  run: # publish all platforms

- name: Wait for NPM registry
  run: sleep 30  # Give npm time to propagate

- name: Publish main package
  run: # publish main
```

### Pitfall 4: Linux musl vs glibc

**Problem:** musl binary doesn't work on old glibc systems.

**Solution:** Compile both variants and detect at runtime. Default to musl for unknown systems (more portable).

### Pitfall 5: Postinstall Failures

**Problem:** Postinstall errors break the entire installation.

**Solution:** Always wrap in try-catch and fail silently:

```javascript
try {
  // Postinstall logic
} catch (error) {
  // Silent failure - don't break installation
  if (!silent) {
    console.log('Note: Could not auto-configure');
  }
  process.exit(0);  // Exit successfully even on failure
}
```

---

## Customization Guide

### Step 1: Update Package Names

Find and replace:
- `@cometix/ccline` → `@yourorg/yourtool`
- `ccline` → `yourtool`

### Step 2: Define Your Platforms

Decide which platforms you need:

| Platform | Target Triple | When Needed |
|----------|---------------|-------------|
| darwin-x64 | x86_64-apple-darwin | macOS Intel |
| darwin-arm64 | aarch64-apple-darwin | macOS Apple Silicon |
| linux-x64 | x86_64-unknown-linux-gnu | Linux (glibc) |
| linux-x64-musl | x86_64-unknown-linux-musl | Linux (musl/Alpine) |
| linux-arm64 | aarch64-unknown-linux-gnu | Linux ARM (glibc) |
| linux-arm64-musl | aarch64-unknown-linux-musl | Linux ARM (musl) |
| win32-x64 | x86_64-pc-windows-gnu | Windows 64-bit |

### Step 3: Adjust Integration Path

Change the integration directory:

```javascript
// In postinstall.js and bin/yourtool.js
const integrationDir = path.join(os.homedir(), '.your-app', 'yourtool');
```

### Step 4: Update Release Workflow

Modify the build matrix in `.github/workflows/release.yml` to match your build system (Rust, Go, etc.).

### Step 5: Test Locally

Before releasing:

```bash
# 1. Build your binaries locally
# 2. Manually create npm-publish/ structure
# 3. Test install from local directory
npm install -g ./npm-publish/main

# 4. Verify binary works
yourtool --version
```

---

## Checklist for New Projects

- [ ] Create `npm/main/` with package.json, bin/, scripts/postinstall.js
- [ ] Create `npm/platforms/*/package.json` for each target platform
- [ ] Create `npm/scripts/prepare-packages.js`
- [ ] Implement platform detection in bin/yourtool.js
- [ ] Implement postinstall integration setup
- [ ] Create GitHub Actions release workflow
- [ ] Add NPM_TOKEN to repository secrets
- [ ] Test local installation
- [ ] Test with npm, yarn, and pnpm
- [ ] Test on all target platforms
- [ ] Verify postinstall doesn't break on failure
- [ ] Add silent mode support

---

## Additional Resources

- npm optionalDependencies: https://docs.npmjs.com/cli/v10/configuring-npm/package-json#optionaldependencies
- npm bin field: https://docs.npmjs.com/cli/v10/configuring-npm/package-json#bin
- Node.js child_process: https://nodejs.org/api/child_process.html
- GitHub Actions workflows: https://docs.github.com/en/actions
