# ARCHITECTURE

## System shape
A small CLI plugin with layered modules:
1. entrypoint/CLI handling
2. config loading and stdin input parsing
3. API client and response types
4. status-line composition and segment rendering

## Main execution flow
1. Parse CLI flags (`src/cli.rs`)
2. Optionally initialize config file (`src/main.rs`)
3. Load config or fall back to defaults
4. Read stdin payload from Claude Code
5. Parse JSON into `InputData`; fall back to empty input on parse failure
6. Build `StatusLineGenerator` with `GlmUsageSegment`
7. Segment fetches usage data from GLM API, using cache when valid
8. Generator formats ANSI text and prints to stdout

## Core responsibilities by layer
- `main.rs`: orchestration and graceful fallback behavior
- `config/`: persisted configuration schema and loader/init logic
- `api/`: platform detection, HTTP client, API DTOs, error types
- `core/statusline.rs`: segment loop and final string formatting
- `core/segments/glm_usage.rs`: business logic for usage fetch, cache, text format, severity color

## Design patterns in use
- Trait-based segment abstraction via `Segment`
- Builder-style composition for `StatusLineGenerator::add_segment`
- Value DTOs for config and API payloads
- Thin library crate exporting internal modules for reuse/tests

## State management
- Stateless process overall
- Only durable state is config file under user home
- Only runtime state is in-memory cache stored inside `GlmUsageSegment`

## Error strategy
- Plugin favors graceful degradation over hard failure
- Missing/invalid config -> defaults
- Invalid stdin JSON -> empty input payload
- API failures -> cached value if available, otherwise no segment output
- Verbose CLI mode prints more stderr diagnostics

## Delivery architecture
- Rust binary is the source of truth
- Node/npm layer is a distribution and launcher wrapper, not business logic
