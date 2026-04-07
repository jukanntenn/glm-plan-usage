# CONCERNS

## High-priority concerns

### 1. Sparse behavioral tests
The repository has CI, fmt, clippy, and build checks, but little visible automated verification of core runtime behavior. Regressions in API parsing, cache behavior, or output formatting could slip through.

### 2. Config/runtime drift
`ApiConfig` exposes timeout and retry knobs in config, but `GlmApiClient` currently uses fixed timeout/retry values in code. This can confuse maintainers and users because configuration surface exceeds actual behavior.

### 3. Documentation drift risk
`CLAUDE.md` is detailed and useful, but it includes implementation details that can become stale as code changes. The richer the narrative doc, the higher the maintenance burden.

## Medium-priority concerns

### 4. Single-segment concentration
Most business value currently lives in one segment implementation. That keeps the repo simple, but also concentrates formatting, cache, fallback, and API concerns in one file.

### 5. Silent failure paths
The plugin intentionally degrades gracefully, but some failures return no output with limited visibility unless verbose mode is used. Good for user experience, weaker for troubleshooting.

### 6. Packaging complexity
The npm distribution layer has non-trivial platform/libc detection, postinstall behavior, and release-time package generation. This is a meaningful maintenance surface relative to the small Rust core.

## Lower-priority concerns

### 7. Unused or legacy response types
`src/api/types.rs` contains response structs that do not appear central to the current single-endpoint flow, which may indicate historical residue.

### 8. Sync HTTP in status-line path
Synchronous requests are acceptable for a small CLI, but any future latency increase could impact status-line responsiveness. Current cache mitigates this.

## Recommended follow-up areas
- Add targeted unit tests around the core behavior
- Reconcile config schema with actual runtime usage
- Periodically verify `CLAUDE.md` and README examples against real code paths
- Consider splitting formatting/cache/API concerns further only if feature scope grows
