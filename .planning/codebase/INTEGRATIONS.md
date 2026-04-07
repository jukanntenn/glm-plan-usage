# INTEGRATIONS

## Claude Code integration
- Intended to run as Claude Code `statusLine.command`
- Consumes stdin JSON with model/workspace/cost context (`src/main.rs`, `src/config/types.rs`)
- Produces a single formatted status line string
- README documents standalone and combined-status-line setup

## Environment-variable integration
- `ANTHROPIC_AUTH_TOKEN` is required for API auth
- `ANTHROPIC_BASE_URL` selects provider/base URL and defaults to ZHIPU endpoint family

## External HTTP integration
- GLM/ZAI quota endpoint: `{base_url}/monitor/usage/quota/limit` after ZHIPU URL normalization (`src/api/client.rs`)
- Auth model: Bearer token header
- Platform detection based on base URL host markers (`src/api/types.rs`)

## Filesystem integration
- Reads/writes config at `~/.claude/glm-plan-usage/config.toml` (`src/config/loader.rs`)
- npm postinstall copies or links installed binary into `~/.claude/glm-plan-usage/`

## npm integration
- Main package: `@jukanntenn/glm-plan-usage`
- Platform packages provide per-OS binary payloads
- Node wrapper resolves local installed binary and executes it
- Packaging script rewrites versions and optional dependency pins for release publishing

## CI/CD integration
- GitHub Actions CI validates tests, formatting, lint, and release builds
- Release workflow publishes GitHub release artifacts and npm packages
- npm publishing depends on `NPM_TOKEN` secret in GitHub Actions

## User-facing integration points
- Claude Code `settings.json` statusLine command
- Manual shell/powershell wrapper scripts for combining with other status line tools
