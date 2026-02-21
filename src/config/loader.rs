use super::types::Config;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub trait ConfigLoader {
    fn load() -> Result<Config>;
    fn init_config() -> Result<PathBuf>;
    fn config_path() -> PathBuf;
    fn print(&self) -> Result<()>;
    fn check(&self) -> Result<()>;
}

impl ConfigLoader for Config {
    fn load() -> Result<Config> {
        let path = Self::config_path();

        if !path.exists() {
            return Ok(Config::default());
        }

        let contents = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Config = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        Ok(config)
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
        println!("{}", content);
        Ok(())
    }

    fn check(&self) -> Result<()> {
        use std::collections::HashSet;

        if self.segments.is_empty() {
            anyhow::bail!("No segments configured");
        }

        let mut seen_ids: HashSet<&str> = HashSet::new();
        for segment in &self.segments {
            if !seen_ids.insert(segment.id.as_str()) {
                anyhow::bail!("Duplicate segment ID: {}", segment.id);
            }
        }

        let valid_ids = ["token_usage", "weekly_usage", "mcp_usage"];
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
