//! Entry point for the GLM Plan Usage plugin.
//!
//! This binary reads Claude Code's stdin to receive usage data,
//! fetches GLM API statistics, and outputs a formatted status line.

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod api;
mod cli;
mod config;
mod core;
mod setup;
mod util;

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
            Commands::Update => handle_update(),
            Commands::Setup { with_ccline } => handle_setup(with_ccline),
        }
        return;
    }

    let mut config = Config::load();

    if args.no_cache {
        config.cache.enabled = false;
    }

    let input_text = match read_stdin() {
        Ok(text) => text,
        Err(e) => {
            if args.verbose {
                eprintln!("Error reading stdin: {e}");
            }
            return;
        }
    };

    let input: InputData = match serde_json::from_str(&input_text) {
        Ok(data) => data,
        Err(e) => {
            if args.verbose {
                eprintln!("Error parsing input JSON: {e}");
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

    let output = StatusLineGenerator::generate(&config, &segments);

    if !output.is_empty() {
        print!("{output}");
    }
}

/// Initialize a new config file at the default location.
fn handle_init() {
    let config_path = Config::config_path();
    if config_path.exists() {
        println!("Config already exists at {}", config_path.display());
        println!("Run `glm-plan-usage update` to migrate to latest format.");
        return;
    }
    match Config::init_config() {
        Ok(path) => println!("Created config at {}", path.display()),
        Err(e) => {
            eprintln!("Error initializing config: {e}");
            std::process::exit(1);
        }
    }
}

/// Print the current configuration to stdout.
fn handle_print() {
    let config = Config::load();
    if let Err(e) = config.print() {
        eprintln!("Error printing config: {e}");
        std::process::exit(1);
    }
}

/// Validate the current configuration and report any errors.
fn handle_check() {
    let config_path = Config::config_path();
    if !config_path.exists() {
        eprintln!("Config file not found at {}", config_path.display());
        std::process::exit(1);
    }
    let config = Config::load();
    if let Err(e) = config.check() {
        eprintln!("Configuration invalid: {e}");
        std::process::exit(1);
    }
    println!("✓ Configuration valid");
}

/// Update the config file to the latest format version.
fn handle_update() {
    match Config::load_for_update() {
        Ok((_, None)) => println!("Config created"),
        Ok((_, Some(r))) if r.changes == 0 => println!("Already up to date"),
        Ok((_, Some(r))) => println!("Config migrated ({} changes)", r.changes),
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

/// Configure Claude Code statusline settings.
fn handle_setup(with_ccline: bool) {
    if let Err(e) = setup::run(with_ccline) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

/// Read all input from stdin.
fn read_stdin() -> Result<String, std::io::Error> {
    use std::io::Read;
    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

/// Collect active segments with their data for rendering.
fn collect_segments(config: &Config, input: &InputData) -> Vec<(SegmentConfig, core::SegmentData)> {
    let mut results = Vec::new();

    let shared_cache = api::SharedCache::new();
    let token_segment = TokenUsageSegment::with_cache(shared_cache.clone());
    let weekly_segment = WeeklyUsageSegment::with_cache(shared_cache.clone());
    let mcp_segment = McpUsageSegment::with_cache(shared_cache.clone());

    let segment_lookup: [(&str, &dyn Segment); 3] = [
        ("token_usage", &token_segment),
        ("weekly_usage", &weekly_segment),
        ("mcp_usage", &mcp_segment),
    ];

    for seg_config in &config.segments {
        if !seg_config.enabled {
            continue;
        }

        let data = segment_lookup
            .iter()
            .find(|(id, _)| *id == seg_config.id.as_str())
            .and_then(|(_, seg)| seg.collect(input, config));

        if let Some(d) = data {
            results.push((seg_config.clone(), d));
        }
    }

    results
}
