# Logging

> How output is handled in this project.

---

## Design Philosophy

**Silent by default, verbose on demand.**

This is a CLI tool that outputs to Claude Code's status bar. There is no traditional logging framework — no log files, no log levels, no structured logging.

---

## Output Channels

| Channel    | Purpose                       | Format                         |
| ---------- | ----------------------------- | ------------------------------ |
| **stdout** | Status line output (primary)  | ANSI colored text, single line |
| **stderr** | Error messages (verbose only) | Plain text                     |

Normal operation produces zero stderr output.

---

## When to Use stderr

### With `--verbose` check (non-fatal errors)

Config loading failures, stdin parse errors, environment variable issues:

```rust
if args.verbose {
    eprintln!("Warning: Failed to load config: {}. Using defaults.", e);
}
```

### Unconditionally (fatal errors, CLI commands only)

`init` and `check` commands that fail:

```rust
eprintln!("Error initializing config: {}", e);
std::process::exit(1);
```

### Never

- During normal status line generation
- For API failures (use graceful degradation → return `None`)
- For cache misses
- For successful operations (no news is good news)

---

## Graceful Degradation Over Logging

Instead of logging errors, segments return `None`:

```rust
fn collect(&self, _input: &InputData, config: &Config) -> Option<SegmentData> {
    let stats = self.cache
        .get_or_fetch(config.cache.ttl_seconds, || self.fetch_usage_stats())?;
    // ... format and return
}
```

The segment simply doesn't appear in the status bar when something fails. Claude Code continues working.

---

## Secrets Handling

| Variable               | Rule                                  |
| ---------------------- | ------------------------------------- |
| `ANTHROPIC_AUTH_TOKEN` | **Never** print, even in verbose mode |
| `ANTHROPIC_BASE_URL`   | Safe to reference in error messages   |

Config file may contain API endpoints (safe to reference) but should not contain credentials by design.

---

## Forbidden Patterns

- `println!` for status output (status line must be a single line without trailing newline)
- Unconditional stderr output during normal operation
- Logging API tokens or credentials
- Adding `log`, `slog`, or `tracing` dependencies without justification
- Writing log files to disk

---

## Common Mistakes

1. **Printing errors unconditionally** → Always check `args.verbose` first
2. **Using `println!` for status** → Status line should use stdout write without newline
3. **Logging API responses** → May contain sensitive data, and creates noise
4. **Adding logging frameworks** → Overkill for a simple CLI tool

---

## Future: Adding Traditional Logging

If logging becomes necessary (e.g., daemon mode), add `tracing` as an optional feature — not default behavior. Don't break the current silent-by-default CLI model.
