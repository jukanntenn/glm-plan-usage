# GitHub Actions CI/CD

This directory contains the automated workflows for building, testing, and publishing `glm-plan-usage`.

## Workflows

### CI Workflow (`ci.yml`)

Runs on every push to `main` and on all pull requests.

**Jobs:**
- **test**: Runs `cargo test`, `cargo clippy`, and `cargo fmt --check` on stable and nightly Rust
- **lint-npm**: Validates the `package.json` configuration
- **version-sync**: Ensures `Cargo.toml` and `package.json` versions are synchronized

### Release Workflow (`release.yml`)

Triggers on:
- Tag pushes matching `v*` (e.g., `v1.0.0`)
- GitHub release creation

**Jobs:**
1. **sync-version**: Syncs the Cargo.toml version to package.json
2. **build**: Builds release binaries for all supported platforms:
   - `x86_64-unknown-linux-gnu` (Ubuntu)
   - `aarch64-unknown-linux-gnu` (ARM64 Linux, via cross)
   - `x86_64-apple-darwin` (Intel macOS)
   - `aarch64-apple-darwin` (Apple Silicon macOS)
   - `x86_64-pc-windows-msvc` (Windows)
3. **package-npm**: Packages and publishes to npm
4. **create-github-release**: Creates a GitHub release with binary archives

## Required Secrets

Add the following secrets to your GitHub repository settings:

| Secret | Description | How to Get |
|--------|-------------|------------|
| `NPM_TOKEN` | npm authentication token for publishing | `npm token create --access=public` |

## Setup Instructions

### 1. Configure npm token

```bash
# Create an npm access token
npm token create --access=public

# Copy the token and add it to GitHub Secrets as NPM_TOKEN
```

### 2. Release a new version

```bash
# Update version in Cargo.toml
# Bump version: 0.0.1 -> 0.0.2

# Commit the change
git add Cargo.toml
git commit -m "bump: version 0.0.2"

# Create and push tag
git tag v0.0.2
git push origin main --tags
```

The release workflow will:
1. Automatically sync the version to `package.json`
2. Build binaries for all platforms
3. Publish to npm
4. Create a GitHub release with binary attachments

## Platform Build Details

### Linux x86_64
- **OS**: ubuntu-latest
- **Tool**: native cargo build

### Linux ARM64
- **OS**: ubuntu-latest
- **Tool**: cross-rs for cross-compilation

### macOS Intel (x86_64)
- **OS**: macos-13 (Intel runner)
- **Tool**: native cargo build

### macOS Apple Silicon (ARM64)
- **OS**: macos-latest (ARM runner)
- **Tool**: native cargo build

### Windows x86_64
- **OS**: windows-latest
- **Tool**: native cargo build

## Artifacts

Each build produces artifacts that are:
1. Packaged into the npm package (under `lib/`)
2. Attached to the GitHub release as `.tar.gz` or `.zip` files

## Troubleshooting

### Build fails for ARM Linux
The ARM64 Linux build uses `cross-rs` which requires Docker. If the build fails:
- Check that Docker is available on the runner
- Review the cross compilation logs

### npm publish fails
- Verify `NPM_TOKEN` secret is set correctly
- Ensure you have publish permissions for the package
- Check if the version already exists on npm (delete or bump version)

### Version mismatch
The CI will fail if `Cargo.toml` and `package.json` versions don't match.
- Bump version in `Cargo.toml`
- The release workflow auto-syncs to `package.json`

### Permission denied on npm publish
Ensure your npm token has the correct permissions:
```bash
npm token list
```

Look for a token with "Automation" or "Publish" permissions.
