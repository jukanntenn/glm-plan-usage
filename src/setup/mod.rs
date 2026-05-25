//! Setup command for configuring Claude Code statusline settings.

mod script;
mod settings;

use anyhow::Result;
use std::path::PathBuf;

/// Run the setup command.
pub fn run(with_ccline: bool) -> Result<()> {
    let settings_path = claude_settings_path();

    let mut doc = settings::read_or_create(&settings_path)?;

    let current_command = doc.statusline_command();

    if let Some(ref cmd) = current_command {
        if is_our_command(cmd) {
            if with_ccline && !is_combined_command(cmd) {
                println!("ℹ Statusline is configured for glm-plan-usage (simple mode).");
                println!("  Reconfiguring for combined mode with `CCometixLine`...");
            } else if !with_ccline && is_combined_command(cmd) {
                println!("ℹ Statusline is configured for combined mode.");
                println!("  Reconfiguring for simple mode...");
            } else {
                println!("ℹ Statusline already configured for glm-plan-usage. No changes needed.");
                return Ok(());
            }
        } else {
            print_conflict_guidance(with_ccline);
            return Ok(());
        }
    }

    if with_ccline {
        let ccline_path = ccline_binary_path();
        if !ccline_path.exists() {
            eprintln!("✗ CCometixLine not found at {}", ccline_path.display());
            eprintln!("  Install CCometixLine first: https://github.com/Haleclipse/CCometixLine");
            std::process::exit(1);
        }

        let script_path = combined_script_path();
        script::write_combined_script(&script_path)?;

        doc.set_statusline_command(&combined_command_path());
        settings::write(&settings_path, &doc)?;

        println!(
            "✓ Created combined statusline script at {}",
            script_path.display()
        );
        println!(
            "✓ Configured Claude Code statusline in {}",
            settings_path.display()
        );
    } else {
        doc.set_statusline_command(&glm_command_path());
        settings::write(&settings_path, &doc)?;

        println!(
            "✓ Configured Claude Code statusline in {}",
            settings_path.display()
        );
    }

    println!("  Restart Claude Code to see changes.");
    Ok(())
}

/// Path to Claude Code's global settings file.
fn claude_settings_path() -> PathBuf {
    dirs::home_dir()
        .expect("No home directory found")
        .join(".claude")
        .join("settings.json")
}

/// Path to the glm-plan-usage binary for statusline command.
fn glm_command_path() -> String {
    if cfg!(windows) {
        "$HOME/.claude/glm-plan-usage/glm-plan-usage.exe".to_string()
    } else {
        "~/.claude/glm-plan-usage/glm-plan-usage".to_string()
    }
}

/// Path to the combined script for statusline command.
fn combined_command_path() -> String {
    if cfg!(windows) {
        "powershell.exe -File $HOME/.claude/status-line-combined.ps1".to_string()
    } else {
        "~/.claude/status-line-combined.sh".to_string()
    }
}

/// Path where the combined script will be written.
fn combined_script_path() -> PathBuf {
    let home = dirs::home_dir().expect("No home directory found");
    if cfg!(windows) {
        home.join(".claude").join("status-line-combined.ps1")
    } else {
        home.join(".claude").join("status-line-combined.sh")
    }
}

/// Path to the `CCometixLine` binary.
fn ccline_binary_path() -> PathBuf {
    let home = dirs::home_dir().expect("No home directory found");
    if cfg!(windows) {
        home.join(".claude").join("ccline").join("ccline.exe")
    } else {
        home.join(".claude").join("ccline").join("ccline")
    }
}

/// Check if a command path belongs to us (glm-plan-usage or combined script).
fn is_our_command(cmd: &str) -> bool {
    cmd.contains("glm-plan-usage") || cmd.contains("status-line-combined")
}

/// Check if a command path is the combined script.
fn is_combined_command(cmd: &str) -> bool {
    cmd.contains("status-line-combined")
}

/// Print guidance when a custom statusline is already configured.
fn print_conflict_guidance(with_ccline: bool) {
    println!("⚠ Statusline is already configured with a custom command.");
    println!("  To avoid overwriting your configuration, no changes were made.");
    println!();
    println!("  To add glm-plan-usage alongside your existing setup, add this to your");
    println!("  Claude Code settings.json (~/.claude/settings.json) manually:");
    println!();

    if with_ccline {
        println!("  Run `glm-plan-usage setup` (without --with-ccline) for simple mode,");
        println!("  or refer to the README for combined setup instructions.");
    } else {
        let cmd = glm_command_path();
        let snippet = format!(
            concat!(
                "  {{\n",
                "    \"statusLine\": {{\n",
                "      \"type\": \"command\",\n",
                "      \"command\": \"{cmd}\",\n",
                "      \"padding\": 0\n",
                "    }}\n",
                "  }}"
            ),
            cmd = cmd,
        );
        println!("{snippet}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_is_our_command_glm() {
        assert!(is_our_command("~/.claude/glm-plan-usage/glm-plan-usage"));
        assert!(is_our_command(
            "/home/user/.claude/glm-plan-usage/glm-plan-usage"
        ));
    }

    #[test]
    fn test_is_our_command_combined() {
        assert!(is_our_command("~/.claude/status-line-combined.sh"));
        assert!(is_our_command("/home/user/.claude/status-line-combined.sh"));
    }

    #[test]
    fn test_is_our_command_foreign() {
        assert!(!is_our_command("/usr/bin/custom-tool"));
        assert!(!is_our_command("~/.claude/ccline/ccline"));
    }

    #[test]
    fn test_is_combined_command() {
        assert!(is_combined_command("~/.claude/status-line-combined.sh"));
        assert!(is_combined_command(
            "powershell.exe -File $HOME/.claude/status-line-combined.ps1"
        ));
        assert!(!is_combined_command(
            "~/.claude/glm-plan-usage/glm-plan-usage"
        ));
    }

    #[test]
    fn test_settings_doc_read_write() {
        let dir = std::env::temp_dir().join("glm-plan-usage-test-setup-settings");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("settings.json");

        let _ = fs::remove_file(&path);

        // Create new
        let mut doc = settings::read_or_create(&path).unwrap();
        assert!(doc.statusline_command().is_none());

        doc.set_statusline_command("~/.claude/glm-plan-usage/glm-plan-usage");
        settings::write(&path, &doc).unwrap();

        // Read back
        let doc2 = settings::read_or_create(&path).unwrap();
        assert_eq!(
            doc2.statusline_command().unwrap(),
            "~/.claude/glm-plan-usage/glm-plan-usage"
        );

        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_settings_doc_preserves_existing_content() {
        let dir = std::env::temp_dir().join("glm-plan-usage-test-setup-preserve");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("settings.json");

        // Write initial content with other keys
        let initial = r#"{"someKey": "someValue", "otherKey": 42}"#;
        fs::write(&path, initial).unwrap();

        let mut doc = settings::read_or_create(&path).unwrap();
        doc.set_statusline_command("~/.claude/glm-plan-usage/glm-plan-usage");
        settings::write(&path, &doc).unwrap();

        // Read back and verify other keys preserved
        let content = fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["someKey"], "someValue");
        assert_eq!(parsed["otherKey"], 42);
        assert_eq!(
            parsed["statusLine"]["command"],
            "~/.claude/glm-plan-usage/glm-plan-usage"
        );

        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_settings_doc_merge_statusline_keys() {
        let dir = std::env::temp_dir().join("glm-plan-usage-test-setup-merge");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("settings.json");

        // Write with existing statusLine that has an extra key
        let initial = r#"{"statusLine": {"type": "command", "command": "old-cmd", "padding": 0, "customKey": true}}"#;
        fs::write(&path, initial).unwrap();

        let mut doc = settings::read_or_create(&path).unwrap();
        doc.set_statusline_command("~/.claude/glm-plan-usage/glm-plan-usage");
        settings::write(&path, &doc).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(
            parsed["statusLine"]["command"],
            "~/.claude/glm-plan-usage/glm-plan-usage"
        );
        // customKey preserved by the merge (set_statusline_command only touches type/command/padding)
        assert_eq!(parsed["statusLine"]["customKey"], true);

        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir_all(&dir);
    }
}
