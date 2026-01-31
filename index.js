#!/usr/bin/env node

const path = require('path');
const os = require('os');
const fs = require('fs');
const { spawnSync } = require('child_process');

// Determine platform and architecture
const platform = os.platform();
const arch = os.arch();

// Map Node.js platform names to Rust target triples
const platformMap = {
  'linux': 'unknown-linux-gnu',
  'darwin': 'apple-darwin',
  'win32': 'pc-windows-msvc'
};

const archMap = {
  'x64': 'x86_64',
  'arm64': 'aarch64'
};

// Determine binary name
const rustPlatform = platformMap[platform] || platform;
const rustArch = archMap[arch] || arch;
const targetTriple = `${rustArch}-${rustPlatform}`;

// Binary name
const binaryName = platform === 'win32' ? 'glm-plan-usage.exe' : 'glm-plan-usage';

// Path to the binary
const binaryPath = path.join(__dirname, 'lib', targetTriple, binaryName);

// Check if binary exists
if (!fs.existsSync(binaryPath)) {
  console.error(`Error: Binary not found for platform ${platform} and architecture ${arch}`);
  console.error(`Expected path: ${binaryPath}`);
  console.error('\nPlease report this issue at: https://github.com/your-username/glm-plan-usage/issues');
  process.exit(1);
}

// Make binary executable on Unix-like systems
if (platform !== 'win32') {
  try {
    fs.chmodSync(binaryPath, 0o755);
  } catch (err) {
    // Ignore permission errors
  }
}

// Spawn the binary with arguments
const args = process.argv.slice(2);
const result = spawnSync(binaryPath, args, {
  stdio: 'inherit',
  env: { ...process.env }
});

// Exit with the same code as the binary
process.exit(result.status || 0);
