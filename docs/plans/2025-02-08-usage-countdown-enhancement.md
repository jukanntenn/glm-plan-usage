# Token Usage & Countdown Enhancement Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enhance the GLM plan usage status bar to display detailed token usage with reset countdown and raw MCP usage counts.

**Architecture:** Extend the existing API client and segment formatter to parse additional fields from the GLM API (`nextResetTime`) and format them into a compact, human-readable display with time delta formatting.

**Tech Stack:**
- Rust (edition 2021)
- serde (JSON deserialization)
- ureq (HTTP client)
- std::time (timestamp calculations)

---

## Context & Prerequisites

**Current Behavior:**
- Displays: `T:32% M:20%` (percentages only)
- API endpoint: `https://open.bigmodel.cn/api/monitor/usage/quota/limit`
- Data cached for 5 minutes

**Target Behavior:**
- Displays: `T:32% (13.05M/40.00M) ⏱️1:44 M:20/100`
- Token: percentage + actual usage (M/K units) + countdown (HH:MM)
- MCP: raw count (used/limit)

**Key API Response Fields** (from `docs/api-research.md`):
```json
{
  "type": "TOKENS_LIMIT",
  "usage": 40000000,
  "currentValue": 13050812,
  "percentage": 32,
  "nextResetTime": 1770565089893  // ← NEW: milliseconds timestamp
}

{
  "type": "TIME_LIMIT",
  "usage": 100,
  "currentValue": 20,
  "percentage": 20
  // ← No reset time for MCP
}
```

---

## Task 1: Extend API Types to Support Reset Time

**Files:**
- Modify: `src/api/types.rs:58-66` (QuotaLimitItem)
- Modify: `src/api/types.rs:108-114` (QuotaUsage)

**Step 1: Add `next_reset_time` field to `QuotaLimitItem`**

Open `src/api/types.rs` and locate the `QuotaLimitItem` struct (around line 58).

Add the new field:
```rust
#[derive(Debug, Deserialize, Clone)]
pub struct QuotaLimitItem {
    #[serde(rename = "type")]
    pub quota_type: String,
    pub usage: i64,
    #[serde(rename = "currentValue")]
    pub current_value: i64,
    pub percentage: i32,
    #[serde(rename = "nextResetTime", default)]
    pub next_reset_time: Option<i64>, // Millisecond timestamp
}
```

**Step 2: Add `reset_at` field to `QuotaUsage`**

Locate the `QuotaUsage` struct (around line 108).

Update to:
```rust
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct QuotaUsage {
    pub used: i64,
    pub limit: i64,
    pub percentage: u8,
    pub reset_at: Option<i64>, // Second-level timestamp (converted from ms)
}
```

**Step 3: Verify compilation**

Run: `cargo check`

Expected: No errors (new fields are `Option` so existing code still compiles)

**Step 4: Commit**

```bash
git add src/api/types.rs
git commit -m "feat(api): add nextResetTime field support to QuotaLimitItem"
```

---

## Task 2: Update API Client to Map Reset Time

**Files:**
- Modify: `src/api/client.rs:93-122` (try_fetch_usage_stats function)

**Step 1: Map nextResetTime with conversion to seconds**

Open `src/api/client.rs` and locate the token usage mapping (around line 93).

Update the token_usage mapping:
```rust
// Extract token usage (TOKENS_LIMIT)
let token_usage = quota_response
    .data
    .limits
    .iter()
    .find(|item| item.quota_type == "TOKENS_LIMIT")
    .map(|item| QuotaUsage {
        used: item.current_value,
        limit: item.usage,
        percentage: item.percentage.clamp(0, 100) as u8,
        reset_at: item.next_reset_time.map(|ms| ms / 1000), // Convert ms to seconds
    });
```

**Step 2: Update MCP usage to explicitly set reset_at as None**

Update the mcp_usage mapping (around line 106):
```rust
// Extract tool usage (TIME_LIMIT)
let mcp_usage = quota_response
    .data
    .limits
    .iter()
    .find(|item| item.quota_type == "TIME_LIMIT")
    .map(|item| QuotaUsage {
        used: item.current_value,
        limit: item.usage,
        percentage: item.percentage.clamp(0, 100) as u8,
        reset_at: None, // MCP has no reset time
    });
```

**Step 3: Verify compilation**

Run: `cargo check`

Expected: No errors

**Step 4: Commit**

```bash
git add src/api/client.rs
git commit -m "feat(api): map nextResetTime and convert to seconds"
```

---

## Task 3: Add Token Formatting Helper

**Files:**
- Modify: `src/core/segments/glm_usage.rs` (add after imports, around line 7)

**Step 1: Add format_tokens function**

Open `src/core/segments/glm_usage.rs` and add this function after the imports (around line 7, before the `GlmUsageSegment` struct):

```rust
/// Format token count with appropriate units (M/K/raw)
fn format_tokens(count: i64) -> String {
    if count >= 1_000_000 {
        format!("{:.2}M", count as f64 / 1_000_000.0)
    } else if count >= 10_000 {
        format!("{:.1}K", count as f64 / 1_000.0)
    } else {
        format!("{}", count)
    }
}
```

**Step 2: Verify compilation**

Run: `cargo check`

Expected: No errors (function is not yet used)

**Step 3: Commit**

```bash
git add src/core/segments/glm_usage.rs
git commit -m "feat(segment): add token formatting helper"
```

---

## Task 4: Add Countdown Calculation Function

**Files:**
- Modify: `src/core/segments/glm_usage.rs` (add after format_tokens, around line 18)

**Step 1: Add SystemTime import**

At the top of the file, update the imports:
```rust
use super::Segment;
use crate::api::{GlmApiClient, UsageStats};
use crate::config::{Config, InputData};
use crate::core::segments::{SegmentData, SegmentStyle};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH}; // Add SystemTime and UNIX_EPOCH
```

**Step 2: Add format_countdown function**

Add this function after `format_tokens`:

```rust
/// Calculate countdown to reset time and format as HH:MM
fn format_countdown(reset_at: i64) -> Option<String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_secs() as i64;

    let remaining = reset_at.saturating_sub(now);

    if remaining <= 0 {
        return Some("0:00".to_string());
    }

    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;

    Some(format!("{}:{:02}", hours, minutes))
}
```

**Step 3: Verify compilation**

Run: `cargo check`

Expected: No errors (function is not yet used)

**Step 4: Commit**

```bash
git add src/core/segments/glm_usage.rs
git commit -m "feat(segment): add countdown calculation function"
```

---

## Task 5: Update format_stats to Show Detailed Info

**Files:**
- Modify: `src/core/segments/glm_usage.rs:60-76` (format_stats function)

**Step 1: Replace format_stats implementation**

Locate the `format_stats` function (around line 60) and replace it entirely:

```rust
fn format_stats(stats: &UsageStats) -> String {
    let mut parts = Vec::new();

    // Token usage with countdown
    if let Some(token) = &stats.token_usage {
        let countdown = token.reset_at
            .and_then(|t| format_countdown(t))
            .unwrap_or_else(|| "--:--".to_string());

        parts.push(format!(
            "T:{}% ({}/{}) ⏱️{}",
            token.percentage,
            format_tokens(token.used),
            format_tokens(token.limit),
            countdown
        ));
    }

    // MCP raw count
    if let Some(mcp) = &stats.mcp_usage {
        parts.push(format!("M:{}/{}", mcp.used, mcp.limit));
    }

    if parts.is_empty() {
        String::new()
    } else {
        parts.join(" ")
    }
}
```

**Step 2: Verify compilation**

Run: `cargo check`

Expected: No errors

**Step 3: Build binary**

Run: `cargo build --release`

Expected: Binary built successfully

**Step 4: Test with real API**

Run: `echo '{"model": {"id": "test"}}' | ./target/release/glm-plan-usage`

Expected output (example):
```
T:32% (13.05M/40.00M) ⏱️1:44 M:20/100
```

**Step 5: Commit**

```bash
git add src/core/segments/glm_usage.rs
git commit -m "feat(segment): update format_stats to show usage and countdown"
```

---

## Task 6: Manual Testing & Verification

**Files:**
- No file changes (testing only)

**Step 1: Test normal output**

Run: `echo '{"model": {"id": "test"}}' | ./target/release/glm-plan-usage`

Expected:
- Token percentage shown
- Token count in M/K units
- Countdown in HH:MM format
- MCP raw count

**Step 2: Test cache behavior**

Run: `echo '{"model": {"id": "test"}}' | ./target/release/glm-plan-usage` twice

Expected: Second call uses cache (faster response)

**Step 3: Test with --verbose flag**

Run: `echo '{"model": {"id": "test"}}' | ./target/release/glm-plan-usage --verbose`

Expected: Errors printed to stderr if any

**Step 4: Test with --no-cache flag**

Run: `echo '{"model": {"id": "test"}}' | ./target/release/glm-plan-usage --no-cache`

Expected: Fresh API call each time

**Step 5: Verify color coding**

Check output colors:
- 0-79%: Green
- 80-94%: Yellow
- 95-100%: Red

**Step 6: Create test documentation**

Create `docs/plans/2025-02-08-test-results.md` with your findings:

```markdown
# Test Results - Usage Countdown Enhancement

## Test Environment
- Date: [fill in date]
- API Endpoint: https://open.bigmodel.cn/api/monitor/usage/quota/limit

## Test Results

### Normal Output
✅ PASS / FAIL
Output: `T:32% (13.05M/40.00M) ⏱️1:44 M:20/100`

### Cache Behavior
✅ PASS / FAIL

### Edge Cases
- Countdown < 1 min: `0:00`
- Countdown expired: `0:00`
- No resetTime from API: `--:--`
- Token < 10K: raw count
- Token > 1M: M units

## Known Issues
[Document any issues found]
```

**Step 7: Commit test results**

```bash
git add docs/plans/2025-02-08-test-results.md
git commit -m "test: add manual test results for countdown feature"
```

---

## Task 7: Update Documentation

**Files:**
- Modify: `CLAUDE.md` (add to "Color-coded Warning Levels" section)

**Step 1: Update CLAUDE.md with new output format**

Locate the "Color-coded Warning Levels" section in `CLAUDE.md` (around line 200) and add after it:

```markdown
### Output Format

The status bar displays:
- **Token Usage**: `T:32% (13.05M/40.00M) ⏱️1:44`
  - Percentage used
  - Actual usage/limit (auto-formatted in M/K units)
  - Countdown to reset (HH:MM format)
- **MCP Usage**: `M:20/100`
  - Raw count (used/limit)

**Token Unit Formatting:**
- ≥ 1,000,000: Shows "M" (millions), e.g., "13.05M"
- ≥ 10,000: Shows "K" (thousands), e.g., "250.5K"
- < 10,000: Shows raw count, e.g., "5000"

**Countdown Behavior:**
- Only TOKENS_LIMIT shows countdown (from API `nextResetTime` field)
- TIME_LIMIT (MCP) has no countdown
- Format: HH:MM (e.g., "1:44" = 1 hour 44 minutes)
- Expired: "0:00"
- API missing field: "--:--"
```

**Step 2: Commit documentation**

```bash
git add CLAUDE.md
git commit -m "docs: update output format documentation"
```

---

## Task 8: Version Bump (If Preparing for Release)

**Files:**
- Modify: `Cargo.toml` (version field)
- Modify: `npm/main/package.json` (version field)

**Step 1: Bump Cargo.toml version**

Open `Cargo.toml` and update:
```toml
[package]
name = "glm-plan-usage"
version = "0.0.3"  # Increment from current
```

**Step 2: Bump npm package.json version**

Open `npm/main/package.json` and update:
```json
{
  "name": "@jukanntenn/glm-plan-usage",
  "version": "0.0.3"  # Must match Cargo.toml
}
```

**Step 3: Commit version bump**

```bash
git add Cargo.toml npm/main/package.json
git commit -m "chore: bump version to 0.0.3"
```

---

## Completion Checklist

- [ ] Task 1: API types extended with `next_reset_time`
- [ ] Task 2: API client maps and converts timestamps
- [ ] Task 3: Token formatting helper added
- [ ] Task 4: Countdown calculation function added
- [ ] Task 5: format_stats shows detailed usage + countdown
- [ ] Task 6: Manual testing completed and documented
- [ ] Task 7: Documentation updated
- [ ] Task 8: Version bumped (if releasing)

---

## Success Criteria

✅ **Output displays correctly**: `T:32% (13.05M/40.00M) ⏱️1:44 M:20/100`

✅ **Countdown accurate**: Time remaining matches API reset time

✅ **Graceful degradation**: Shows `--:--` when API doesn't return reset time

✅ **No breaking changes**: Old configurations still work

✅ **Colors work**: High usage still shows red/yellow appropriately

---

## References

- API Research: `docs/api-research.md`
- Current Architecture: `CLAUDE.md`
- API Types: `src/api/types.rs`
- Client Implementation: `src/api/client.rs`
- Segment Logic: `src/core/segments/glm_usage.rs`
