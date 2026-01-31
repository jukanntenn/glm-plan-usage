#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

console.log('Packaging glm-plan-usage for npm...');

// Create lib directory
const libDir = path.join(__dirname, 'lib');
if (!fs.existsSync(libDir)) {
  fs.mkdirSync(libDir, { recursive: true });
}

// Define targets to package
const targets = [
  { triple: 'x86_64-unknown-linux-gnu', binary: 'glm-plan-usage' },
  { triple: 'aarch64-unknown-linux-gnu', binary: 'glm-plan-usage' },
  { triple: 'x86_64-apple-darwin', binary: 'glm-plan-usage' },
  { triple: 'aarch64-apple-darwin', binary: 'glm-plan-usage' },
  { triple: 'x86_64-pc-windows-msvc', binary: 'glm-plan-usage.exe' }
];

// Package each target
targets.forEach(target => {
  const targetDir = path.join(libDir, target.triple);

  // Create target directory
  if (!fs.existsSync(targetDir)) {
    fs.mkdirSync(targetDir, { recursive: true });
  }

  // Source binary path
  const sourcePath = path.join(__dirname, 'target', target.triple, 'release', target.binary);

  // Destination binary path
  const destPath = path.join(targetDir, target.binary);

  // Check if binary exists
  if (!fs.existsSync(sourcePath)) {
    console.warn(`⚠ Warning: Binary not found for ${target.triple}`);
    console.warn(`  Expected: ${sourcePath}`);
    console.warn(`  Run: cargo build --release --target ${target.triple}`);
    return;
  }

  // Copy binary
  fs.copyFileSync(sourcePath, destPath);

  // Make executable on Unix-like systems
  if (!target.binary.endsWith('.exe')) {
    fs.chmodSync(destPath, 0o755);
  }

  console.log(`✓ Packaged ${target.triple}`);
});

// Copy package.json
console.log('\n✓ Packaging complete!');
console.log('\nTo publish to npm:');
console.log('  1. npm run package');
console.log('  2. npm publish');
