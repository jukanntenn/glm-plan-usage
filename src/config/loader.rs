//! Configuration loading and file operations.
//!
//! This module provides the `ConfigLoader` trait for loading,
//! validating, and managing plugin configuration.

use super::migration::{self, MigrationResult};
use super::template;
use super::types::Config;
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Valid segment identifiers.
const VALID_SEGMENT_IDS: &[&str] = &["token_usage", "weekly_usage", "mcp_usage"];

/// Configuration loading and management trait.
pub trait ConfigLoader {
    /// Load configuration, returning defaults if file doesn't exist.
    fn load() -> Config;
    /// Load configuration with migration info for update.
    ///
    /// # Errors
    ///
    /// Returns an error if the config file cannot be read, parsed, or migrated.
    fn load_for_update() -> Result<(Config, Option<MigrationResult>)>;
    /// Initialize configuration file and return its path.
    ///
    /// # Errors
    ///
    /// Returns an error if the config directory cannot be created or the file cannot be written.
    fn init_config() -> Result<PathBuf>;
    /// Get the configuration file path.
    fn config_path() -> PathBuf;
    /// Print configuration to stdout.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be serialized to TOML.
    fn print(&self) -> Result<()>;
    /// Validate configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if segments are empty, contain duplicate IDs, or invalid segment IDs.
    fn check(&self) -> Result<()>;
}

impl ConfigLoader for Config {
    fn load() -> Config {
        let path = Self::config_path();

        if !path.exists() {
            return Self::default();
        }

        let Ok(contents) = fs::read_to_string(&path) else {
            return Self::default();
        };

        let Ok(raw): Result<toml::Value, _> = toml::from_str(&contents) else {
            return Self::default();
        };

        let MigrationResult { value, changes } = migration::migrate(raw);

        if changes > 0 {
            // Silent write-back; graceful degradation on failure
            let _ = write_migrated_config(&path, &value);
        }

        let config = deserialize_migrated_value(&value);
        config.merge_default_segments()
    }

    fn load_for_update() -> Result<(Config, Option<MigrationResult>)> {
        let path = Self::config_path();

        if !path.exists() {
            Self::init_config()?;
            return Ok((Self::default(), None));
        }

        let contents = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let raw: toml::Value = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        let result = migration::migrate(raw);

        if result.changes > 0 {
            write_migrated_config(&path, &result.value)?;
        }

        let config = deserialize_migrated_value(&result.value);

        Ok((config.merge_default_segments(), Some(result)))
    }

    fn init_config() -> Result<PathBuf> {
        let config_path = Self::config_path();
        let config_dir = config_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid config path"))?;

        fs::create_dir_all(config_dir).with_context(|| {
            format!(
                "Failed to create config directory: {}",
                config_dir.display()
            )
        })?;

        let config_template = include_str!("../config_template.toml");

        fs::write(&config_path, config_template)
            .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;

        Ok(config_path)
    }

    fn config_path() -> PathBuf {
        dirs::home_dir()
            .expect("No home directory found")
            .join(".claude")
            .join("glm-plan-usage")
            .join("config.toml")
    }

    fn print(&self) -> Result<()> {
        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;
        println!("{content}");
        Ok(())
    }

    fn check(&self) -> Result<()> {
        if self.segments.is_empty() {
            anyhow::bail!("No segments configured");
        }

        let mut seen_ids: HashSet<&str> = HashSet::new();
        for segment in &self.segments {
            if !seen_ids.insert(segment.id.as_str()) {
                anyhow::bail!("Duplicate segment ID: {}", segment.id);
            }
        }

        let valid_ids = VALID_SEGMENT_IDS;
        for segment in &self.segments {
            if !valid_ids.contains(&segment.id.as_str()) {
                anyhow::bail!(
                    "Invalid segment ID: {}. Valid IDs: {:?}",
                    segment.id,
                    valid_ids
                );
            }
        }

        Ok(())
    }
}

/// Deserializes a migrated `toml::Value` into a `Config`.
///
/// Uses `toml::to_string` instead of `Value::to_string` because the latter
/// produces inline table syntax (`{ key = value }`) which is invalid at the
/// TOML document level.
fn deserialize_migrated_value(value: &toml::Value) -> Config {
    if value.as_table().is_some_and(toml::map::Map::is_empty) {
        return Config::default();
    }
    let Ok(toml_str) = toml::to_string(value) else {
        return Config::default();
    };
    toml::from_str(&toml_str).unwrap_or_default()
}

/// Writes migrated config back to file if changes were made.
fn write_migrated_config(path: &Path, value: &toml::Value) -> Result<()> {
    let content = template::generate_overlay(value);
    crate::util::atomic_write(path, &content)
        .with_context(|| format!("Failed to write migrated config to {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SegmentConfig;

    #[test]
    fn test_check_valid() {
        let config = Config::default();
        assert!(config.check().is_ok());
    }

    #[test]
    fn test_check_empty_segments() {
        let config = Config {
            segments: vec![],
            ..Config::default()
        };
        assert!(config.check().is_err());
    }

    #[test]
    fn test_check_duplicate_id() {
        let config = Config {
            segments: vec![SegmentConfig::token_usage(), SegmentConfig::token_usage()],
            ..Config::default()
        };
        assert!(config.check().is_err());
    }

    #[test]
    fn test_check_invalid_id() {
        let config = Config {
            segments: vec![SegmentConfig {
                id: "nonexistent".to_string(),
                ..SegmentConfig::token_usage()
            }],
            ..Config::default()
        };
        assert!(config.check().is_err());
    }

    // --- Regression tests for toml::Value::to_string() bug ---
    // toml::Value::to_string() produces inline table syntax ({ key = value })
    // which is invalid at the TOML document level, causing deserialization failure.

    #[test]
    fn test_deserialize_empty_table_returns_defaults() {
        let empty = toml::Value::Table(toml::map::Map::new());
        let config = deserialize_migrated_value(&empty);
        assert_eq!(config.style.mode, crate::config::DisplayMode::Auto);
        assert_eq!(config.style.separator, " | ");
        assert_eq!(config.api.timeout_ms, 5000);
        assert_eq!(config.cache.enabled, true);
    }

    #[test]
    fn test_deserialize_preserves_custom_values_after_migration() {
        // Simulate a v0.2.0 config where mode="ascii" and timeout_ms=3000 are custom
        let raw: toml::Value = toml::from_str(
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

        let MigrationResult { value, .. } = migration::migrate(raw);
        let config = deserialize_migrated_value(&value);

        assert_eq!(
            config.style.mode,
            crate::config::DisplayMode::Ascii,
            "custom mode=ascii must survive migration + deserialize"
        );
        assert_eq!(
            config.api.timeout_ms, 3000,
            "custom timeout_ms=3000 must survive migration + deserialize"
        );
        // Defaults should fill in
        assert_eq!(config.style.separator, " | ");
        assert_eq!(config.cache.ttl_seconds, 300);
    }

    #[test]
    fn test_deserialize_preserves_segment_order_after_migration() {
        // Segments with non-default values (enabled=false) so they survive stripping
        let raw: toml::Value = toml::from_str(
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
id = "mcp_usage"
enabled = false
[segments.icon]
emoji = "🌐"
ascii = "#"
[segments.options]
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

        let MigrationResult { value, .. } = migration::migrate(raw);
        let config = deserialize_migrated_value(&value);
        let merged = config.merge_default_segments();

        let ids: Vec<&str> = merged.segments.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(
            ids,
            vec!["mcp_usage", "token_usage", "weekly_usage"],
            "user segment order must be preserved after migration + deserialize"
        );
        // Verify the non-default value also survived
        let mcp_seg = merged
            .segments
            .iter()
            .find(|s| s.id == "mcp_usage")
            .unwrap();
        assert!(!mcp_seg.enabled, "enabled=false must survive");
    }

    #[test]
    fn test_deserialize_non_default_segment_enabled_survives() {
        let raw: toml::Value = toml::from_str(
            r##"
[style]
mode = "emoji"
separator = " | "
[api]
timeout_ms = 5000
retry_attempts = 2
[cache]
enabled = true
ttl_seconds = 300
[[segments]]
id = "token_usage"
enabled = false
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

        let MigrationResult { value, .. } = migration::migrate(raw);
        let config = deserialize_migrated_value(&value);

        let token_seg = config
            .segments
            .iter()
            .find(|s| s.id == "token_usage")
            .expect("token_usage segment must exist");
        assert!(
            !token_seg.enabled,
            "enabled=false must survive migration + deserialize"
        );
        assert_eq!(
            config.style.mode,
            crate::config::DisplayMode::Emoji,
            "custom mode=emoji must survive"
        );
    }

    #[test]
    fn test_toml_value_to_string_would_fail() {
        // Guard against accidentally reverting to Value::to_string().
        // This test documents WHY we use toml::to_string() instead.
        let raw: toml::Value = toml::from_str(
            r#"
[style]
mode = "ascii"
[api]
timeout_ms = 3000
"#,
        )
        .unwrap();

        let MigrationResult { value, .. } = migration::migrate(raw);

        // Value::to_string() produces inline table syntax, invalid at document level
        let bad_output = value.to_string();
        assert!(
            bad_output.starts_with('{'),
            "regression guard: Value::to_string() should still produce inline syntax \
             (if this changes, the deserialize_migrated_value fix may no longer be needed)"
        );
        assert!(
            toml::from_str::<Config>(&bad_output).is_err(),
            "regression guard: inline table syntax from Value::to_string() must fail \
             deserialization (otherwise we don't need the workaround)"
        );

        // But toml::to_string() produces valid TOML
        let good_output = toml::to_string(&value).unwrap();
        assert!(
            !good_output.starts_with('{'),
            "toml::to_string() must produce standard TOML"
        );
        assert!(
            toml::from_str::<Config>(&good_output).is_ok(),
            "toml::to_string() output must deserialize successfully"
        );
    }
}
