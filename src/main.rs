mod api;
mod cli;
mod config;
mod core;
mod gsd;

use clap::Parser;
use config::{Config, ConfigLoader, InputData};
use core::{GlmUsageSegment, StatusLineGenerator};

fn main() {
    // Load .env file from config directory
    let config_dir = dirs::home_dir()
        .expect("No home directory found")
        .join(".claude")
        .join("glm-plan-usage");
    let env_path = config_dir.join(".env");

    if env_path.exists() {
        let _ = dotenvy::from_path(&env_path);
    }

    // Parse CLI arguments
    let args = cli::Args::parse();

    // Handle --fix-gsd flag
    if args.fix_gsd {
        match gsd::fix_integration(args.verbose) {
            Ok(changed) => {
                if changed {
                    eprintln!("GSD statusline updated successfully!");
                    eprintln!("Restart Claude Code to see GLM usage in status bar.");
                } else {
                    eprintln!("GSD statusline is already up to date.");
                }
                return;
            }
            Err(e) => {
                eprintln!("Error fixing GSD integration: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Handle --setup flag
    if args.setup {
        match setup_wizard() {
            Ok(_) => return,
            Err(e) => {
                eprintln!("Error during setup: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Handle --init flag
    if args.init {
        match Config::init_config() {
            Ok(path) => {
                eprintln!("Initialized config at: {}", path.display());
                return;
            }
            Err(e) => {
                eprintln!("Error initializing config: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Load configuration
    let mut config = match Config::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            if args.verbose {
                eprintln!("Warning: Failed to load config: {}. Using defaults.", e);
            }
            Config::default()
        }
    };

    // Apply CLI overrides
    if args.no_cache {
        config.cache.enabled = false;
    }

    // Read input from stdin
    let input_text = match read_stdin() {
        Ok(text) => text,
        Err(e) => {
            if args.verbose {
                eprintln!("Error reading stdin: {}", e);
            }
            return;
        }
    };

    // Parse input JSON
    let input: InputData = match serde_json::from_str(&input_text) {
        Ok(data) => data,
        Err(e) => {
            if args.verbose {
                eprintln!("Error parsing input JSON: {}", e);
            }
            // Continue with empty input
            InputData {
                model: None,
                workspace: None,
                transcript_path: None,
                cost_info: None,
            }
        }
    };

    // Create status line generator
    let generator = StatusLineGenerator::new().add_segment(Box::new(GlmUsageSegment::new()));

    // Generate output
    let output = generator.generate(&input, &config);

    // Print to stdout
    if !output.is_empty() {
        print!("{}", output);
    }
}

/// Interactive setup wizard
fn setup_wizard() -> Result<(), anyhow::Error> {
    use dialoguer::{Confirm, FuzzySelect, Input};
    use std::path::PathBuf;

    println!("\n=== GLM Plan Usage Setup Wizard ===\n");

    // Step 1: Select platform
    let platforms = vec![
        "智谱 AI (open.bigmodel.cn)",
        "Z.ai (api.z.ai)",
    ];

    println!("选择你的 GLM 平台:");
    let selection = FuzzySelect::new()
        .items(&platforms)
        .default(0)
        .interact()?;

    let base_url = match selection {
        0 => "https://open.bigmodel.cn/api/anthropic",
        1 => "https://api.z.ai/api/anthropic",
        _ => unreachable!(),
    };

    println!("✓ 已选择：{}\n", platforms[selection]);

    // Step 2: Ask for API token
    println!("请输入你的 API Token (ANTHROPIC_AUTH_TOKEN):");
    let token: String = Input::new()
        .with_prompt("Token")
        .interact_text()?;

    // Validate token is not empty
    if token.trim().is_empty() {
        return Err(anyhow::anyhow!("Token 不能为空"));
    }

    println!("✓ Token 已保存\n");

    // Step 3: Confirm configuration
    let env_path = PathBuf::from(".env");
    let env_content = format!(
        "ANTHROPIC_AUTH_TOKEN={}\nANTHROPIC_BASE_URL={}\n",
        token.trim(),
        base_url
    );

    println!("将创建以下配置:");
    println!("  平台：{}", base_url);
    println!("  Token: {}...", &token[..std::cmp::min(8, token.len())]);
    println!();

    let confirm = Confirm::new()
        .with_prompt("是否保存配置到 .env 文件？")
        .default(true)
        .interact()?;

    if confirm {
        std::fs::write(&env_path, &env_content)?;
        println!("✓ 配置已保存到：{}", env_path.display());
        println!("\n使用前请执行：source .env 或将变量添加到 shell 配置中");
    } else {
        println!("配置预览:");
        println!("{}", env_content);
    }

    // Step 4: Initialize plugin config
    let init_plugin = Confirm::new()
        .with_prompt("是否同时初始化插件配置文件？")
        .default(true)
        .interact()?;

    if init_plugin {
        let config_path = Config::init_config()?;
        println!("✓ 插件配置已初始化：{}", config_path.display());
    }

    println!("\n=== Setup Complete ===\n");
    println!("运行以下命令测试:");
    println!("  export ANTHROPIC_AUTH_TOKEN=\"{}\"", &token[..std::cmp::min(8, token.len())]);
    println!("  export ANTHROPIC_BASE_URL=\"{}\"", base_url);
    println!("  echo '{{\"model\":{{\"id\":\"test\"}}}}' | glm-plan-usage");

    Ok(())
}

fn read_stdin() -> Result<String, std::io::Error> {
    use std::io::Read;
    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}
