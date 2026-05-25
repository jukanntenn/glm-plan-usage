//! Claude Code settings.json read/write operations.

use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::path::Path;

/// A mutable wrapper around a parsed settings.json document.
pub struct SettingsDoc {
    value: Value,
}

impl SettingsDoc {
    /// Get the current `statusLine.command` value, if any.
    pub fn statusline_command(&self) -> Option<String> {
        self.value
            .get("statusLine")
            .and_then(|v| v.get("command"))
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string)
    }

    /// Set `statusLine.command` to the given value, preserving other keys.
    pub fn set_statusline_command(&mut self, command: &str) {
        let status_line = self
            .value
            .as_object_mut()
            .unwrap()
            .entry("statusLine")
            .or_insert_with(|| Value::Object(serde_json::Map::new()));

        let obj = status_line.as_object_mut().unwrap();
        obj.insert("type".to_string(), Value::String("command".to_string()));
        obj.insert("command".to_string(), Value::String(command.to_string()));
        obj.insert("padding".to_string(), Value::Number(0.into()));
    }
}

/// Read an existing settings.json or create a new empty document.
pub fn read_or_create(path: &Path) -> Result<SettingsDoc> {
    if path.exists() {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let value: Value = serde_json::from_str(&contents)
            .with_context(|| format!("Failed to parse {}", path.display()))?;
        Ok(SettingsDoc { value })
    } else {
        let parent = path.parent().with_context(|| "Invalid settings path")?;
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
        Ok(SettingsDoc {
            value: Value::Object(serde_json::Map::new()),
        })
    }
}

/// Write the settings document back to disk with pretty formatting.
pub fn write(path: &Path, doc: &SettingsDoc) -> Result<()> {
    let contents =
        serde_json::to_string_pretty(&doc.value).context("Failed to serialize settings.json")?;
    fs::write(path, &contents).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}
