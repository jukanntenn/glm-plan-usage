# Quality

> Code quality standards for the glm-plan-usage plugin.

---

## Core Principles

- **Graceful degradation** — Never crash Claude Code. All segments return `Option<SegmentData>`, never panic.
- **Simplicity over abstraction** — Avoid over-engineering. Three similar lines of code are better than a premature abstraction.
- **Type safety** — Leverage Rust's type system. Use enums for known variants, `Option` for nullable values.
- **Silent by default** — No stderr output unless `--verbose`. API failures return `None`, not errors.

---

## Forbidden Patterns

| Pattern                          | Why                         | Alternative                           |
| -------------------------------- | --------------------------- | ------------------------------------- |
| `.unwrap()` on external input    | Can crash Claude Code       | `?`, `.ok()`, or graceful degradation |
| `.expect()` with generic message | Unhelpful error             | `anyhow::anyhow!` with context        |
| Panic in library code            | Breaks graceful degradation | Return `Result` or `Option`           |
| `#![allow(...)]` at crate level  | Hides real warnings         | Fix underlying issues                 |
| Hardcoded paths                  | Not portable                | Use `dirs::` crate                    |
| Blocking operations in segments  | Freezes Claude Code         | Keep segments fast                    |

---

## Required Patterns

### Graceful Degradation

All segments must return `Option<SegmentData>`, using `ok()?` chains:

```rust
fn collect(&self, _input: &InputData, _config: &Config) -> Option<SegmentData> {
    self.fetch_data().ok()?.parse().ok()
}
```

### Serde Defaults

All config fields must have `#[serde(default)]` to allow adding new fields without breaking existing configs.

### Trait-Based Extensibility

Use the `Segment` trait for pluggable behavior — enables adding segments without modifying core code.

### Context-Aware Errors

Use `.with_context()` for better error messages in config and file operations.

---

## Linting and Formatting

```bash
cargo fmt                      # Format code (required before commit)
cargo clippy -- -D warnings    # Run linter (required before commit)
```

Configuration: Default rustfmt settings (no `rustfmt.toml`).

---

## Code Organization

- Each domain has its own directory with `mod.rs`
- Re-export public types via `pub use`
- Keep implementation private, expose minimal API
- Prefer small, focused functions (< 30 lines)
- Extract complex logic into named functions

### Comment Style

- **Doc comments** (`///`) for public API
- **Regular comments** (`//`) only for non-obvious implementation notes
- **No TODO comments** — use task tracking instead

---

## Testing

**Current state**: No formal tests. Recommended additions:

1. **Unit tests**: `Platform::detect_from_url()`, color calculation, countdown formatting
2. **Integration tests**: Config loading, segment registration, full status line generation

Test pattern:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        assert_eq!(Platform::detect_from_url("https://api.z.ai/foo"), Some(Platform::Zai));
    }
}
```

---

## Review Checklist

### Functionality

- [ ] Works with default config (no config file needed)
- [ ] Handles API failures gracefully (returns None/uses cache)
- [ ] `--verbose` flag shows useful error info

### Code Quality

- [ ] `cargo fmt` and `cargo clippy` pass
- [ ] No `#[allow(dead_code)]` in production code
- [ ] No `.unwrap()` on external input
- [ ] Secrets never printed

### Architecture

- [ ] New segments implement `Segment` trait
- [ ] Config fields have `#[serde(default)]`
- [ ] Errors use `anyhow::Error` or domain types
- [ ] No circular dependencies between domains

---

## Performance

- Keep segments fast — status line is called frequently
- Use cache for API calls — avoid redundant HTTP requests
- Minimal allocations in hot paths — status line generation
