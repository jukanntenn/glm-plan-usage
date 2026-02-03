#!/usr/bin/env node
const { spawnSync } = require('child_process');
const { execSync } = require('child_process');
const path = require('path');
const fs = require('fs');
const os = require('os');

// Priority 1: Use integration path if exists (e.g., ~/.claude/glm-plan-usage/glm-plan-usage)
const integrationPath = path.join(
  os.homedir(),
  '.claude',
  'glm-plan-usage',
  process.platform === 'win32' ? 'glm-plan-usage.exe' : 'glm-plan-usage'
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

// Platform detection logic
let platformKey = detectPlatform(platform, arch);

const packageMap = {
  'darwin-x64': 'glm-plan-usage-darwin-x64',
  'darwin-arm64': 'glm-plan-usage-darwin-arm64',
  'linux-x64': 'glm-plan-usage-linux-x64',
  'linux-arm64': 'glm-plan-usage-linux-arm64',
  'win32-x64': 'glm-plan-usage-win32-x64',
};

const packageName = packageMap[platformKey];
const binaryName = platform === 'win32' ? 'glm-plan-usage.exe' : 'glm-plan-usage';

// Find binary path (handles npm, yarn, pnpm)
const binaryPath = findBinaryPath(packageName, binaryName);

if (!binaryPath || !fs.existsSync(binaryPath)) {
  console.error(`Error: Binary not found for ${platformKey}`);
  console.error(`Package: ${packageName}`);
  console.error('Please try reinstalling: npm install -g glm-plan-usage');
  process.exit(1);
}

const result = spawnSync(binaryPath, process.argv.slice(2), {
  stdio: 'inherit',
  shell: false
});

process.exit(result.status || 0);

/**
 * Detect Linux libc type (glibc vs musl)
 */
function getLibcInfo() {
  try {
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

    // If we can't detect, default to glibc (more common)
    return { type: 'glibc' };
  } catch (e) {
    // If detection fails, default to glibc
    return { type: 'glibc' };
  }
}

/**
 * Detect platform key with libc detection for Linux
 */
function detectPlatform(platform, arch) {
  let platformKey = `${platform}-${arch}`;

  if (platform === 'linux') {
    const libcInfo = getLibcInfo();

    // For now, we only build glibc versions
    // If you add musl builds, add detection logic here
    if (libcInfo.type === 'musl') {
      // Fall back to glibc if musl not available
      // Or implement musl package selection
    }
  }

  return platformKey;
}

/**
 * Find binary path across different package managers (npm, yarn, pnpm)
 */
function findBinaryPath(packageName, binaryName) {
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
}
