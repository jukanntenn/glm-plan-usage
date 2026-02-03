const fs = require('fs');
const path = require('path');

// Get version from git tag or command line
const version = process.env.GITHUB_REF?.replace('refs/tags/v', '') || process.argv[2];
if (!version) {
  console.error('Error: Version not provided');
  console.error('Usage: node prepare-packages.js <version>');
  console.error('Or set GITHUB_REF environment variable');
  process.exit(1);
}

console.log(`Preparing npm packages for version ${version}...`);

// Define all platforms
const platforms = [
  'darwin-x64',
  'darwin-arm64',
  'linux-x64',
  'linux-arm64',
  'win32-x64'
];

// Rust target to npm platform mapping
const targetToPlatform = {
  'x86_64-apple-darwin': 'darwin-x64',
  'aarch64-apple-darwin': 'darwin-arm64',
  'x86_64-unknown-linux-gnu': 'linux-x64',
  'aarch64-unknown-linux-gnu': 'linux-arm64',
  'x86_64-pc-windows-msvc': 'win32-x64'
};

// Prepare platform packages
platforms.forEach(platform => {
  const sourceDir = path.join(__dirname, '..', 'platforms', platform);
  const targetDir = path.join(__dirname, '..', '..', '..', 'npm-publish', platform);

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

  console.log(`✓ Prepared glm-plan-usage-${platform} v${version}`);
});

// Prepare main package
const mainSource = path.join(__dirname, '..', 'main');
const mainTarget = path.join(__dirname, '..', '..', '..', 'npm-publish', 'main');

// Copy main package files
fs.cpSync(mainSource, mainTarget, { recursive: true });

// Update main package.json
const mainPackageJsonPath = path.join(mainTarget, 'package.json');
const mainPackageJson = JSON.parse(fs.readFileSync(mainPackageJsonPath, 'utf8'));

mainPackageJson.version = version;

// Update optionalDependencies versions
if (mainPackageJson.optionalDependencies) {
  Object.keys(mainPackageJson.optionalDependencies).forEach(dep => {
    if (dep.startsWith('glm-plan-usage-')) {
      mainPackageJson.optionalDependencies[dep] = version;
    }
  });
}

fs.writeFileSync(
  mainPackageJsonPath,
  JSON.stringify(mainPackageJson, null, 2) + '\n'
);

console.log(`✓ Prepared glm-plan-usage v${version}`);
console.log('\nAll packages prepared in npm-publish/');
console.log('\nNext steps:');
console.log('1. Copy binaries to npm-publish/{platform}/ directories');
console.log('2. Publish platform packages first');
console.log('3. Publish main package last');
