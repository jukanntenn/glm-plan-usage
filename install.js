#!/usr/bin/env node

const path = require('path');
const os = require('os');
const fs = require('fs');

const platform = os.platform();

console.log('Installing glm-plan-usage...');

// No post-install steps needed on Windows
if (platform === 'win32') {
  console.log('Installation complete!');
  process.exit(0);
}

// On Unix-like systems, ensure the binary is executable
const platformMap = {
  'linux': 'unknown-linux-gnu',
  'darwin': 'apple-darwin'
};

const archMap = {
  'x64': 'x86_64',
  'arm64': 'aarch64'
};

const rustPlatform = platformMap[platform];
const rustArch = archMap[os.arch()];
const targetTriple = `${rustArch}-${rustPlatform}`;
const binaryPath = path.join(__dirname, 'lib', targetTriple, 'glm-plan-usage');

try {
  fs.chmodSync(binaryPath, 0o755);
  console.log('✓ Binary is executable');
} catch (err) {
  console.warn('Warning: Could not set executable permission:', err.message);
}

console.log('Installation complete!');
