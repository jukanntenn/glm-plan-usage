# Error Handling

> How errors are handled in this project.

---

## Core Philosophy

**Graceful degradation**: the plugin must never cause Claude Code to crash. Every error path either falls back to a safe default or silently produces no output.

---

## Dual Error Strategy

| Layer | Crate | Use When |
|-------|-------|----------|
| **Application** | `anyhow` | Config loading, file I/O, general errors — where context matters more than type |
| **Domain** | `thiserror` | API communication — where callers need to match on specific error variants |

### When to Use Each

- **`anyhow::Result<T>`** — Most functions (config, CLI commands, orchestration)
- **`Result<T, ApiError>`** — API layer only, where typed errors enable specific handling
- **`Option<T>`** — Segments, where failure means the segment simply doesn't appear

---

## Error Type Hierarchy

```
anyhow::Error          ← Application-level (config, file I/O)
    ├── Config::load() → anyhow::Result<Config>
    ├── Config::init_config() → anyhow::Result<PathBuf>
    └── Config::check() → anyhow::Result<()>

ApiError (thiserror)   ← Domain-specific (API communication)
    ├── MissingEnvVar(String)
    ├── HttpError(String)
    ├── ApiResponse(String)
    ├── ParseError(String)
    └── PlatformDetectionFailed
```

---

## Propagation Patterns

### Context-Aware Errors (anyhow)

Use `.with_context()` for file I/O and config parsing:

```rust
fs::read_to_string(&path)
    .with_context(|| format!("Failed to read config: {}", path.display()))?;
```

### Graceful Degradation (Option)

Use `ok()?` chains in segments — all errors become `None`:

```rust
fn fetch_usage_stats(&self) -> Option<UsageStats> {
    GlmApiClient::from_env().ok()?.fetch_usage_stats().ok()
}
```

### Cache Fallback

On fetch failure, return stale cached data:

```rust
let stale = cache.as_ref().map(|(stats, _)| stats.clone());
let fetched = fetch();
fetched.or(stale)
```

---

## main() Error Handling

### Non-Fatal Errors (status line generation)

Use defaults, log only if verbose, never crash:

```rust
let config = match Config::load() {
    Ok(cfg) => cfg,
    Err(e) => {
        if args.verbose { eprintln!("Warning: {}", e); }
        Config::default()
    }
};
```

### Fatal Errors (CLI commands: init, check)

Explicit commands should exit with error code:

```rust
Err(e) => {
    eprintln!("Error: {}", e);
    std::process::exit(1);
}
```

---

## Retry Logic

3 attempts with 100ms delay between retries, configurable via `api.retry_attempts`.

---

## Forbidden Patterns

- `.unwrap()` or `.expect()` in production code paths
- Panic for expected errors (missing config, API failures)
- `Box<dyn Error>` directly — use `anyhow::Error` or `thiserror`
- Silently ignore errors without at least logging in verbose mode
- Create custom error types when `anyhow::Error` suffices
