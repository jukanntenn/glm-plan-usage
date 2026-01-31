use clap::Parser;

/// GLM usage status plugin for Claude Code
#[derive(Parser, Debug)]
#[command(name = "glm-plan-usage")]
#[command(about = "Display GLM usage statistics in Claude Code status bar", long_about = None)]
pub struct Args {
    /// Initialize configuration file
    #[arg(long)]
    pub init: bool,

    /// Verbose output
    #[arg(long)]
    pub verbose: bool,

    /// Disable cache
    #[arg(long)]
    pub no_cache: bool,
}
