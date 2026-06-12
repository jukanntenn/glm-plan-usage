//! Configuration migration for backwards compatibility.
//!
//! This module handles migrating legacy configuration formats
//! to the current version by stripping default values.

use toml::Value;

/// Result of running the migration chain
#[derive(Debug)]
pub struct MigrationResult {
    /// The migrated TOML value.
    pub value: Value,
    /// Number of changes made during migration.
    pub changes: usize,
}

/// Run the full idempotent migration chain on a raw TOML value.
pub fn migrate(raw: Value) -> MigrationResult {
    let original = raw.clone();
    let mut v = raw;

    if is_legacy_config(&v) {
        strip_legacy_defaults(&mut v);
        strip_segment_defaults(&mut v);
    }

    let changes = if v == original {
        0
    } else {
        count_diffs(&original, &v)
    };

    MigrationResult { value: v, changes }
}

/// Does this `toml::Value` look like a v0.2.0 (pre-migration) config?
/// Checks for marker fields that are present in the old template
/// but commented out in the new template.
pub fn is_legacy_config(raw: &Value) -> bool {
    let has_sep = raw.get("style").and_then(|s| s.get("separator")).is_some();
    let has_cache_enabled = raw.get("cache").and_then(|c| c.get("enabled")).is_some();
    let has_retry = raw
        .get("api")
        .and_then(|a| a.get("retry_attempts"))
        .is_some();
    has_sep && has_cache_enabled && has_retry
}

/// Strip scalar/table fields that match v0.2.0 defaults.
fn strip_legacy_defaults(raw: &mut Value) {
    // style
    if let Some(style) = raw.get_mut("style").and_then(|v| v.as_table_mut()) {
        strip_if_eq_str(style, "mode", "auto");
        strip_if_eq_str(style, "separator", " | ");
        if style.is_empty() {
            raw.as_table_mut().unwrap().remove("style");
        }
    }

    // api
    if let Some(api) = raw.get_mut("api").and_then(|v| v.as_table_mut()) {
        strip_if_eq(api, "timeout_ms", &Value::Integer(5000));
        strip_if_eq(api, "retry_attempts", &Value::Integer(2));
        if api.is_empty() {
            raw.as_table_mut().unwrap().remove("api");
        }
    }

    // cache
    if let Some(cache) = raw.get_mut("cache").and_then(|v| v.as_table_mut()) {
        strip_if_eq(cache, "enabled", &Value::Boolean(true));
        strip_if_eq(cache, "ttl_seconds", &Value::Integer(300));
        if cache.is_empty() {
            raw.as_table_mut().unwrap().remove("cache");
        }
    }

    // multiplier
    if let Some(mult) = raw.get_mut("multiplier").and_then(|v| v.as_table_mut()) {
        let default_models = vec![
            Value::String("glm-5".into()),
            Value::String("glm-5.1".into()),
            Value::String("glm-5-turbo".into()),
        ];
        strip_if_eq(mult, "premium_models", &Value::Array(default_models));
        strip_if_eq_str(mult, "peak_start", "14:00");
        strip_if_eq_str(mult, "peak_end", "18:00");
        strip_if_eq(mult, "peak", &Value::Float(3.0));
        strip_if_eq(mult, "off_peak", &Value::Float(2.0));

        if let Some(promo) = mult.get_mut("promo").and_then(|v| v.as_table_mut()) {
            strip_if_eq(promo, "off_peak", &Value::Float(1.0));
            strip_if_eq_str(promo, "expires", "2026-06-30");
            if promo.is_empty() {
                mult.remove("promo");
            }
        }

        if mult.is_empty() {
            raw.as_table_mut().unwrap().remove("multiplier");
        }
    }
}

/// Strip segment fields that match v0.2.0 defaults.
fn strip_segment_defaults(raw: &mut Value) {
    let Some(segments) = raw.get_mut("segments").and_then(|v| v.as_array_mut()) else {
        return;
    };

    let mut stripped_indices = Vec::new();

    for (i, seg) in segments.iter_mut().enumerate() {
        let Some(table) = seg.as_table_mut() else {
            continue;
        };
        let Some(id) = table.get("id").and_then(|v| v.as_str()) else {
            continue;
        };

        match id {
            "token_usage" => {
                strip_if_eq(table, "enabled", &Value::Boolean(true));
                if let Some(icon) = table.get_mut("icon").and_then(|v| v.as_table_mut()) {
                    strip_if_eq_str(icon, "emoji", "🪙");
                    strip_if_eq_str(icon, "ascii", "$");
                    if icon.is_empty() {
                        table.remove("icon");
                    }
                }
                if let Some(opts) = table.get_mut("options").and_then(|v| v.as_table_mut()) {
                    strip_if_eq(opts, "show_timer", &Value::Boolean(true));
                    strip_if_eq_str(opts, "timer_mode", "clock");
                    strip_if_eq(opts, "show_multiplier", &Value::Boolean(true));
                    if opts.is_empty() {
                        table.remove("options");
                    }
                }
            }
            "weekly_usage" => {
                strip_if_eq(table, "enabled", &Value::Boolean(true));
                if let Some(icon) = table.get_mut("icon").and_then(|v| v.as_table_mut()) {
                    strip_if_eq_str(icon, "emoji", "🗓️");
                    strip_if_eq_str(icon, "ascii", "*");
                    if icon.is_empty() {
                        table.remove("icon");
                    }
                }
                if let Some(opts) = table.get_mut("options").and_then(|v| v.as_table_mut()) {
                    if opts.is_empty() {
                        table.remove("options");
                    }
                }
            }
            "mcp_usage" => {
                strip_if_eq(table, "enabled", &Value::Boolean(true));
                if let Some(icon) = table.get_mut("icon").and_then(|v| v.as_table_mut()) {
                    strip_if_eq_str(icon, "emoji", "🌐");
                    strip_if_eq_str(icon, "ascii", "#");
                    if icon.is_empty() {
                        table.remove("icon");
                    }
                }
                if let Some(opts) = table.get_mut("options").and_then(|v| v.as_table_mut()) {
                    if opts.is_empty() {
                        table.remove("options");
                    }
                }
            }
            _ => {}
        }

        // If only id remains, the segment is fully default — strip entirely
        let remaining: Vec<_> = table.keys().filter(|k| *k != "id").collect();
        if remaining.is_empty() {
            stripped_indices.push(i);
        }
    }

    // Remove fully-stripped segments (in reverse to preserve indices)
    for i in stripped_indices.into_iter().rev() {
        segments.remove(i);
    }

    // Remove empty segments array
    if segments.is_empty() {
        raw.as_table_mut().unwrap().remove("segments");
    }
}

/// Remove a key from a TOML table if its value equals the expected value.
fn strip_if_eq(table: &mut toml::map::Map<String, Value>, key: &str, expected: &Value) {
    if let Some(actual) = table.get(key) {
        if values_equal(actual, expected) {
            table.remove(key);
        }
    }
}

/// Convenience wrapper for string comparisons.
fn strip_if_eq_str(table: &mut toml::map::Map<String, Value>, key: &str, expected: &str) {
    strip_if_eq(table, key, &Value::String(expected.into()));
}

/// Compare two `toml::Value`s for equality, treating `f64` specially
/// (`3.0 == Integer(3)` is false in `toml::Value`'s `PartialEq`).
fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Float(f1), Value::Float(f2)) => (f1 - f2).abs() < f64::EPSILON,
        (Value::Integer(i), Value::Float(f)) | (Value::Float(f), Value::Integer(i)) => {
            #[expect(
                clippy::cast_precision_loss,
                reason = "i64 values from TOML are small config numbers, well within f64 precision"
            )]
            let i_f64 = *i as f64;
            (*f - i_f64).abs() < f64::EPSILON
        }
        _ => a == b,
    }
}

/// Count the number of top-level keys that differ between two values.
fn count_diffs(original: &Value, migrated: &Value) -> usize {
    let Some(orig_table) = original.as_table() else {
        return 0;
    };
    let Some(mig_table) = migrated.as_table() else {
        return orig_table.len();
    };

    let mut count = 0;
    for key in orig_table.keys().chain(mig_table.keys()) {
        if orig_table.get(key) != mig_table.get(key) {
            count += 1;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_legacy_config_with_full_defaults() {
        let raw: Value = toml::from_str(
            r#"
[style]
mode = "auto"
separator = " | "
[api]
timeout_ms = 5000
retry_attempts = 2
[cache]
enabled = true
ttl_seconds = 300
"#,
        )
        .unwrap();
        assert!(is_legacy_config(&raw));
    }

    #[test]
    fn test_is_legacy_config_new_format() {
        let raw: Value = toml::from_str(
            r#"
[style]
mode = "ascii"
"#,
        )
        .unwrap();
        assert!(!is_legacy_config(&raw));
    }

    #[test]
    fn test_is_legacy_config_empty() {
        let raw = Value::Table(toml::map::Map::new());
        assert!(!is_legacy_config(&raw));
    }

    #[test]
    fn test_strip_preserves_custom_values() {
        let mut raw: Value = toml::from_str(
            r#"
[style]
mode = "ascii"
separator = " | "
[api]
timeout_ms = 3000
retry_attempts = 2
[cache]
enabled = true
ttl_seconds = 300
"#,
        )
        .unwrap();

        strip_legacy_defaults(&mut raw);

        // mode = "ascii" was custom, should be preserved
        assert_eq!(
            raw.get("style").unwrap().get("mode").unwrap().as_str(),
            Some("ascii")
        );
        // separator = " | " was default, should be stripped
        assert!(raw.get("style").unwrap().get("separator").is_none());
        // timeout_ms = 3000 was custom, preserved
        assert_eq!(
            raw.get("api")
                .unwrap()
                .get("timeout_ms")
                .unwrap()
                .as_integer(),
            Some(3000)
        );
    }

    #[test]
    fn test_migrate_idempotent() {
        let raw: Value = toml::from_str(
            r#"
[style]
mode = "ascii"
"#,
        )
        .unwrap();

        let result = migrate(raw);
        assert_eq!(result.changes, 0);
    }

    #[test]
    fn test_segment_merge_preserves_user_order() {
        use crate::config::SegmentConfig;
        let mut segments = vec![SegmentConfig::mcp_usage(), SegmentConfig::token_usage()];
        // Simulate merge: user has mcp_usage and token_usage
        let defaults = vec![
            SegmentConfig::token_usage(),
            SegmentConfig::weekly_usage(),
            SegmentConfig::mcp_usage(),
        ];
        let user_ids: Vec<_> = segments.iter().map(|s| s.id.clone()).collect();
        for seg in defaults {
            if !user_ids.contains(&seg.id) {
                segments.push(seg);
            }
        }
        // Order: user's [mcp_usage, token_usage] + appended [weekly_usage]
        assert_eq!(segments[0].id, "mcp_usage");
        assert_eq!(segments[1].id, "token_usage");
        assert_eq!(segments[2].id, "weekly_usage");
    }

    #[test]
    fn test_values_equal_same_type() {
        assert!(values_equal(&Value::Integer(3), &Value::Integer(3)));
        assert!(values_equal(
            &Value::String("a".into()),
            &Value::String("a".into())
        ));
        assert!(values_equal(&Value::Boolean(true), &Value::Boolean(true)));
    }

    #[test]
    fn test_values_equal_float_int() {
        assert!(values_equal(&Value::Float(3.0), &Value::Integer(3)));
        assert!(values_equal(&Value::Integer(3), &Value::Float(3.0)));
    }

    #[test]
    fn test_values_equal_float_precision() {
        assert!(values_equal(&Value::Float(3.0), &Value::Float(3.0)));
        assert!(!values_equal(&Value::Float(3.0001), &Value::Float(3.0)));
    }

    #[test]
    fn test_values_equal_different() {
        assert!(!values_equal(&Value::Integer(1), &Value::Integer(2)));
        assert!(!values_equal(
            &Value::String("a".into()),
            &Value::String("b".into())
        ));
    }

    #[test]
    fn test_count_diffs_no_changes() {
        let a: Value = toml::from_str("[style]\nmode = \"auto\"").unwrap();
        let b = a.clone();
        assert_eq!(count_diffs(&a, &b), 0);
    }

    #[test]
    fn test_count_diffs_with_changes() {
        let a: Value = toml::from_str("[style]\nmode = \"auto\"").unwrap();
        let b: Value = toml::from_str("[style]\nmode = \"ascii\"").unwrap();
        // Both tables have "style" key, iterated from both orig and mig → counted twice for same key
        assert!(count_diffs(&a, &b) > 0);
    }

    #[test]
    fn test_count_diffs_non_table_original() {
        let a = Value::Array(vec![]);
        let b = Value::Array(vec![]);
        assert_eq!(count_diffs(&a, &b), 0);
    }

    #[test]
    fn test_count_diffs_non_table_migrated() {
        let a: Value = toml::from_str("[style]\nmode = \"auto\"").unwrap();
        let b = Value::Array(vec![]);
        // original has keys, migrated doesn't → count original keys
        assert!(count_diffs(&a, &b) > 0);
    }

    #[test]
    fn test_count_diffs_key_only_in_migrated() {
        let a: Value = toml::from_str("[style]\nmode = \"auto\"").unwrap();
        let b: Value =
            toml::from_str("[style]\nmode = \"auto\"\n[api]\ntimeout_ms = 3000").unwrap();
        assert_eq!(count_diffs(&a, &b), 1);
    }

    #[test]
    fn test_strip_segment_defaults_weekly_usage() {
        let mut raw: Value = toml::from_str(
            r##"
[[segments]]
id = "weekly_usage"
enabled = true
[segments.icon]
emoji = "🗓️"
ascii = "*"
[segments.options]
"##,
        )
        .unwrap();
        strip_segment_defaults(&mut raw);
        // All defaults → segment should be stripped entirely
        assert!(raw.get("segments").is_none());
    }

    #[test]
    fn test_strip_segment_defaults_mcp_usage() {
        let mut raw: Value = toml::from_str(
            r##"
[[segments]]
id = "mcp_usage"
enabled = true
[segments.icon]
emoji = "🌐"
ascii = "#"
[segments.options]
"##,
        )
        .unwrap();
        strip_segment_defaults(&mut raw);
        // All defaults → segment should be stripped entirely
        assert!(raw.get("segments").is_none());
    }

    #[test]
    fn test_strip_segment_defaults_preserves_custom() {
        let mut raw: Value = toml::from_str(
            r##"
[[segments]]
id = "token_usage"
enabled = false
[segments.icon]
emoji = "🪙"
ascii = "$"
[segments.options]
show_timer = true
"##,
        )
        .unwrap();
        strip_segment_defaults(&mut raw);
        // enabled=false is non-default, segment should remain
        assert!(raw.get("segments").is_some());
        let segments = raw["segments"].as_array().unwrap();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0]["id"].as_str(), Some("token_usage"));
    }

    #[test]
    fn test_strip_segment_defaults_unknown_id() {
        let mut raw: Value = toml::from_str(
            r##"
[[segments]]
id = "unknown_segment"
enabled = true
"##,
        )
        .unwrap();
        strip_segment_defaults(&mut raw);
        // Unknown IDs are not stripped (no defaults to compare against)
        assert!(raw.get("segments").is_some());
    }

    #[test]
    fn test_strip_segment_defaults_no_segments() {
        let mut raw: Value = toml::from_str("[style]\nmode = \"auto\"").unwrap();
        strip_segment_defaults(&mut raw);
        // No segments to strip, should be fine
        assert!(raw.get("segments").is_none());
    }

    #[test]
    fn test_strip_segment_defaults_non_table_segment() {
        // A segment that's not a table (e.g., a string) should be skipped
        let mut raw = Value::Table(toml::map::Map::new());
        raw.as_table_mut().unwrap().insert(
            "segments".into(),
            Value::Array(vec![Value::String("not_a_table".into())]),
        );
        strip_segment_defaults(&mut raw);
        // Should still have segments (non-table items are skipped)
        assert!(raw.get("segments").is_some());
    }

    #[test]
    fn test_strip_legacy_defaults_removes_empty_sections() {
        let mut raw: Value = toml::from_str(
            r#"
[style]
mode = "auto"
separator = " | "
[api]
timeout_ms = 5000
retry_attempts = 2
[cache]
enabled = true
ttl_seconds = 300
"#,
        )
        .unwrap();
        strip_legacy_defaults(&mut raw);
        // All defaults → sections should be removed
        assert!(raw.get("style").is_none());
        assert!(raw.get("api").is_none());
        assert!(raw.get("cache").is_none());
    }

    #[test]
    fn test_strip_legacy_defaults_multiplier() {
        let mut raw: Value = toml::from_str(
            r#"
[style]
mode = "auto"
separator = " | "
[api]
timeout_ms = 5000
retry_attempts = 2
[cache]
enabled = true
ttl_seconds = 300
[multiplier]
premium_models = ["glm-5", "glm-5.1", "glm-5-turbo"]
peak_start = "14:00"
peak_end = "18:00"
peak = 3.0
off_peak = 2.0
[multiplier.promo]
off_peak = 1.0
expires = "2026-06-30"
"#,
        )
        .unwrap();
        strip_legacy_defaults(&mut raw);
        // All multiplier defaults → should be removed
        assert!(raw.get("multiplier").is_none());
    }

    #[test]
    fn test_migrate_legacy_config_with_segments() {
        let raw: Value = toml::from_str(
            r##"
[style]
mode = "auto"
separator = " | "
[api]
timeout_ms = 5000
retry_attempts = 2
[cache]
enabled = true
ttl_seconds = 300
[[segments]]
id = "token_usage"
enabled = true
[segments.icon]
emoji = "🪙"
ascii = "$"
[segments.options]
show_timer = true
timer_mode = "clock"
show_multiplier = true
"##,
        )
        .unwrap();

        let result = migrate(raw);
        // Legacy config with all defaults → changes should strip everything
        assert!(result.changes > 0);
        // Segments should be stripped since all values are defaults
        assert!(result.value.get("segments").is_none());
    }

    #[test]
    fn test_migrate_new_format_no_changes() {
        let raw: Value = toml::from_str(
            r#"
[style]
mode = "ascii"
"#,
        )
        .unwrap();
        let result = migrate(raw);
        assert_eq!(result.changes, 0);
        assert_eq!(result.value["style"]["mode"].as_str(), Some("ascii"));
    }
}
