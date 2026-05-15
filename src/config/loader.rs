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

        let Ok(config): Result<Config, _> = toml::from_str(&value.to_string()) else {
            return Self::default();
        };

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

        let config: Config = toml::from_str(&result.value.to_string())
            .with_context(|| "Failed to deserialize migrated config")?;

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
}
