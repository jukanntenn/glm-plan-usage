mod api;
mod cli;
mod config;
mod core;

use clap::Parser;
use cli::Commands;
use config::{Config, ConfigLoader, InputData, SegmentConfig};
use core::{McpUsageSegment, Segment, StatusLineGenerator, TokenUsageSegment, WeeklyUsageSegment};

fn main() {
    let args = cli::Args::parse();

    if let Some(command) = args.command {
        match command {
            Commands::Init => handle_init(),
            Commands::Print => handle_print(),
            Commands::Check => handle_check(),
        }
        return;
    }

    let mut config = match Config::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            if args.verbose {
                eprintln!("Warning: Failed to load config: {}. Using defaults.", e);
            }
            Config::default()
        }
    };

    if args.no_cache {
        config.cache.enabled = false;
    }

    let input_text = match read_stdin() {
        Ok(text) => text,
        Err(e) => {
            if args.verbose {
                eprintln!("Error reading stdin: {}", e);
            }
            return;
        }
    };

    let input: InputData = match serde_json::from_str(&input_text) {
        Ok(data) => data,
        Err(e) => {
            if args.verbose {
                eprintln!("Error parsing input JSON: {}", e);
            }
            InputData {
                model: None,
                workspace: None,
                transcript_path: None,
                cost_info: None,
            }
        }
    };

    let segments = collect_segments(&config, &input);

    let output = StatusLineGenerator::generate(&config, segments);

    if !output.is_empty() {
        print!("{}", output);
    }
}

fn handle_init() {
    let config_path = Config::config_path();
    if config_path.exists() {
        println!("Config already exists at {}", config_path.display());
        return;
    }
    match Config::init_config() {
        Ok(path) => println!("Created config at {}", path.display()),
        Err(e) => {
            eprintln!("Error initializing config: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_print() {
    let config = Config::load().unwrap_or_else(|_| Config::default());
    if let Err(e) = config.print() {
        eprintln!("Error printing config: {}", e);
        std::process::exit(1);
    }
}

fn handle_check() {
    let config_path = Config::config_path();
    if !config_path.exists() {
        eprintln!("Config file not found at {}", config_path.display());
        std::process::exit(1);
    }
    let config = match Config::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Configuration invalid: {}", e);
            std::process::exit(1);
        }
    };
    if let Err(e) = config.check() {
        eprintln!("Configuration invalid: {}", e);
        std::process::exit(1);
    }
    println!("✓ Configuration valid");
}

fn read_stdin() -> Result<String, std::io::Error> {
    use std::io::Read;
    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn collect_segments(config: &Config, input: &InputData) -> Vec<(SegmentConfig, core::SegmentData)> {
    let mut results = Vec::new();

    let shared_cache = api::SharedCache::new();
    let token_segment = TokenUsageSegment::with_cache(shared_cache.clone());
    let weekly_segment = WeeklyUsageSegment::with_cache(shared_cache.clone());
    let mcp_segment = McpUsageSegment::with_cache(shared_cache.clone());

    for seg_config in &config.segments {
        if !seg_config.enabled {
            continue;
        }

        let data = match seg_config.id.as_str() {
            "token_usage" => token_segment.collect(input, config),
            "weekly_usage" => weekly_segment.collect(input, config),
            "mcp_usage" => mcp_segment.collect(input, config),
            _ => None,
        };

        if let Some(d) = data {
            results.push((seg_config.clone(), d));
        }
    }

    results
}
