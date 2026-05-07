---
name: release
description: >
  Execute the glm-plan-usage release process. Use this skill when the user wants to
  publish a new version, create a release, bump version numbers, or says things like
  "release v0.2.0", "publish a new version", "cut a release", "ship it", or "prepare
  release". Also use when the user mentions CHANGELOG updates combined with version
  bumping in this Rust + NPM project.
---

# Release Process

Safe release workflow for glm-plan-usage (Rust CLI, multi-platform NPM package). Every step validates before proceeding; failures report the problem + suggestion then STOP — never auto-fix.

## Prerequisites

Verify clean tree and identify current state:

```bash
git status                    # must be clean
grep '^version = ' Cargo.toml # current version
git describe --tags --abbrev=0  # last tag
```

## Steps 1-5: Automated Phase

### Step 1: Quality Checks

```bash
cargo test && cargo fmt -- --check && cargo clippy -- -D warnings
```

Fail → report which check failed (test/fmt/clippy) + likely cause, STOP.

### Step 2: Version Bump

1. Show commits since last tag: `git log --oneline <last_tag>..HEAD`
2. Ask user to confirm new version (suggest semver bump: patch=fixes, minor=features, major=breaking)
3. Update exactly TWO files:
   - `Cargo.toml`: `version = "x.y.z"`
   - `npm/main/package.json`: `"version": "x.y.z"`
4. Sync `Cargo.lock` with the new version:
   ```bash
   cargo check
   ```
5. Verify all three files show the same new version (`Cargo.toml`, `npm/main/package.json`, `Cargo.lock`)

Do NOT touch `npm/platforms/*/package.json` or `optionalDependencies` in npm/main/package.json — `prepare-packages.js` handles those at release time.

### Step 3: Local Build

```bash
cargo build --release
```

Fail → report error + likely cause, STOP. Pass → `target/release/glm-plan-usage` exists.

### Step 4: Local Publish Verification

```bash
NPM_REGISTRY=http://192.168.5.50:4873/ ./npm/publish-local.sh
```

This reads version from Cargo.toml, runs `prepare-packages.js`, copies binaries, publishes all packages to local verdaccio. Missing cross-compile binaries are expected (only current platform builds locally).

Fail → report which platform failed + error output, STOP.

### Step 5: Update CHANGELOG & Verify READMEs

**CHANGELOG**: `git log --oneline <last_tag>..HEAD` → derive user-visible changes, draft entry matching existing format, present to user for review, then insert at TOP:

```
## [X.Y.Z] - YYYY-MM-DD

### Added
- ...
### Fixed
- ...
```

Omit empty sections. Keep `# Changelog` header + blank line. Only user-visible changes.

**README consistency check**: Compare `README.md`, `README_en.md`, and `npm/main/README.md` for conflicting or contradictory information (Features, configuration examples, paths, display examples, etc.). NPM README is a subset — it need not cover everything, but what it covers must agree with the main READMEs.

If inconsistencies found, report them to the user and STOP.

1. `git log --oneline <last_tag>..HEAD` — derive user-visible changes
2. Draft entry matching existing format, present to user for review, then insert at TOP:

   ```
   ## [X.Y.Z] - YYYY-MM-DD

   ### Added
   - ...
   ### Fixed
   - ...
   ```

   Omit empty sections. Keep `# Changelog` header + blank line. Only user-visible changes.

---

## PAUSE — User Confirmation Required

Present summary:

```
Steps 1-5 complete:
- Version: X.Y.Z (Cargo.toml + npm/main/package.json)
- Local build + publish: passed
- CHANGELOG: updated
- READMEs: verified in sync

Ready to commit and publish? Confirm to continue.
```

Do NOT proceed until user confirms.

---

## Steps 6-10: Commit & Publish Phase

### Step 6: Verify Release Workflow

```bash
grep -A5 'Extract release notes' .github/workflows/release.yml
```

Must have: CHANGELOG extraction step + `body_path` in `action-gh-release`. If missing, explain gap and ask user whether to fix.

### Step 7: Commit

```bash
git add Cargo.toml Cargo.lock npm/main/package.json CHANGELOG.md README.md README_en.md npm/main/README.md .github/workflows/release.yml
git commit -m "chore: release vX.Y.Z"
```

Verify: `git log --oneline -1` shows commit, `git status` is clean.
Hook failure → report output, STOP. Never `--no-verify`.

### Step 8: Tag

```bash
git tag -a vX.Y.Z -m "Release vX.Y.Z"
```

Verify: `git tag -l vX.Y.Z` exists. Tag exists → report, STOP.

### Step 9: Push

Confirm with user first — **destructive, visible to others**:

```bash
git push && git push --tags
```

Then provide monitoring URL: `https://github.com/jukanntenn/glm-plan-usage/actions/workflows/release.yml`
Rejected → report error, STOP. Never force push.

### Step 10: Post-Release Verification (manual)

Provide user with:

1. **GitHub Release**: check body matches CHANGELOG + 7 binaries attached
   → `https://github.com/jukanntenn/glm-plan-usage/releases/tag/vX.Y.Z`
2. **NPM install**: `npm install -g @jukanntenn/glm-plan-usage && glm-plan-usage --version`
3. **Rollback options**: edit Release manually, `npm unpublish` (72h window), delete+recreate tag
