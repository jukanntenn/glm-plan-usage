const fs = require('fs');
const path = require('path');
const os = require('os');
const { execSync } = require('child_process');

// Silent mode detection
const silent = process.env.npm_config_loglevel === 'silent' ||
               process.env.GLM_PLAN_USAGE_SKIP_POSTINSTALL === '1';

if (!silent) {
  console.log('🚀 Setting up GLM Plan Usage...');
}

try {
  const platform = process.platform;
  const arch = process.arch;

  // Create integration directory
  const integrationDir = path.join(os.homedir(), '.claude', 'glm-plan-usage');
  fs.mkdirSync(integrationDir, { recursive: true });

  // Detect platform (same logic as bin/glm-plan-usage.js)
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
    console.log('✨ GLM Plan Usage is ready!');
    console.log(`📍 Location: ${targetPath}`);
  }
} catch (error) {
  // Silent failure - don't break installation
  if (!silent) {
    console.log('Note: Could not auto-configure');
    console.log('The tool will still work.');
  }
}

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

    // If we can't detect, default to glibc
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
  }

  return platformKey;
}

/**
 * Find binary path across different package managers (npm, yarn, pnpm)
 */
function findBinaryPath(packageName, binaryName) {
  const possiblePaths = [
    // npm/yarn: nested in node_modules
    path.join(__dirname, '..', '..', 'node_modules', packageName, binaryName),

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
