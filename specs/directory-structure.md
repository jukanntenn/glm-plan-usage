# Directory Structure

> How source code is organized in this project.

---

## Architecture Style

Domain-driven module organization with clear separation of concerns. Each domain owns a directory with `mod.rs` as the export layer.

---

## Directory Layout

```
src/
├── main.rs              # Entry point, stdin parsing, CLI command handling
├── lib.rs               # Library interface, module exports
├── cli.rs               # CLI argument definitions (clap derive)
├── config/
│   ├── mod.rs           # Module exports
│   ├── types.rs         # All configuration structs
│   └── loader.rs        # Config file loading/parsing
├── api/
│   ├── mod.rs           # Module exports
│   ├── client.rs        # GlmApiClient (HTTP, auth, retry)
│   ├── cache.rs         # SharedCache (Arc<Mutex<>> TTL cache)
│   └── types.rs         # API response types, ApiError, Platform enum
└── core/
    ├── mod.rs           # Module exports
    ├── statusline.rs    # StatusLineGenerator (segment orchestration)
    └── segments/
        ├── mod.rs       # Segment trait, SegmentData
        ├── token_usage.rs  # TokenUsageSegment
        ├── weekly_usage.rs # WeeklyUsageSegment
        └── mcp_usage.rs    # McpUsageSegment
```

---

## Domain Responsibilities

| Domain | Responsibility |
|--------|---------------|
| `config/` | Configuration loading, validation, TOML serialization |
| `api/` | External API communication, caching, error types |
| `core/` | Core business logic, status line generation |
| `core/segments/` | Pluggable status bar segments |

---

## Adding a New Segment

1. Create `src/core/segments/<name>.rs` implementing the `Segment` trait
2. Export from `src/core/segments/mod.rs`: `pub mod <name>; pub use <name>::<Struct>;`
3. Register in `main.rs:collect_segments()` with the segment ID string

**See:** `src/main.rs:collect_segments()` for registration pattern

---

## Adding a New Domain Module

1. Create `src/<domain>/mod.rs`
2. Create `src/<domain>/<implementation>.rs`
3. Export public types via `mod.rs`: `pub use <impl>::<Type>;`
4. Import in `src/main.rs` or `src/lib.rs`: `mod <domain>;`

---

## Module Export Pattern

Each domain directory uses `mod.rs` to control its public API:

- Internal implementation files are private (`mod types;` without `pub`)
- Only public types are re-exported (`pub use types::*;`)
- This keeps internal implementation hidden while exposing a clean API surface

---

## Naming Conventions

| Convention | Usage | Example |
|------------|-------|---------|
| `snake_case.rs` | All Rust files | `token_usage.rs`, `statusline.rs` |
| `mod.rs` | Module exports | `src/api/mod.rs` |
| `PascalCase` | Types, structs, enums | `SegmentData`, `ApiError` |
| `snake_case` | Functions, methods | `collect_segments()`, `get_or_fetch()` |

---

## Import Patterns

**External crates** (alphabetical within group):
```rust
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
```

**Internal modules** (crate-relative, never `super::`):
```rust
use crate::api::{GlmApiClient, SharedCache};
use crate::config::{Config, InputData};
```

---

## Forbidden Patterns

- Place implementation directly in `mod.rs` (unless trivial)
- Create deep nesting beyond 3 levels
- Mix unrelated functionality in a single module
- Import from sibling modules using `super::<sibling>::` — use `crate::` paths
- Create circular dependencies between domains (`api` ↔ `core`)

---

## Common Mistakes

1. **Forgetting to export new modules** from `mod.rs` → Module exists but can't be imported
2. **Cyclic dependencies** → Use shared types in a third location
3. **Large `main.rs`** → Keep main minimal, move logic to domain modules
