use assert_cmd::Command;
use std::path::PathBuf;
use tempfile::TempDir;

pub fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn fixture_path(name: &str) -> PathBuf {
    project_root().join("e2e").join("fixtures").join(name)
}

pub fn read_fixture(name: &str) -> String {
    std::fs::read_to_string(fixture_path(name))
        .unwrap_or_else(|e| panic!("Failed to read fixture '{name}': {e}"))
}

/// Creates a temp directory to use as HOME, optionally writing config.toml.
pub fn temp_home_with_config(config_content: Option<&str>) -> TempDir {
    let dir = TempDir::new().unwrap();
    if let Some(content) = config_content {
        let config_dir = dir.path().join(".claude").join("glm-plan-usage");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(config_dir.join("config.toml"), content).unwrap();
    }
    dir
}

/// Creates a `Command` for the binary with HOME isolated to a temp dir.
pub fn bin_cmd(home: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("glm-plan-usage").unwrap();
    cmd.env("HOME", home.path());
    cmd
}

/// Minimal valid config in ASCII mode for predictable assertions.
pub const ASCII_CONFIG: &str = r##"
[style]
mode = "ascii"
separator = " | "

[[segments]]
id = "token_usage"
enabled = true
[segments.icon]
emoji = "🪙"
ascii = "$"
[segments.options]

[[segments]]
id = "weekly_usage"
enabled = true
[segments.icon]
emoji = "🗓️"
ascii = "*"
[segments.options]

[[segments]]
id = "mcp_usage"
enabled = true
[segments.icon]
emoji = "🌐"
ascii = "#"
[segments.options]
"##;
