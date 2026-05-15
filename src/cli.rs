//! CLI argument definitions using clap derive macros.

use clap::{Parser, Subcommand};

/// GLM plan usage plugin for Claude Code.
///
/// Displays GLM (ZHIPU/ZAI) coding plan usage statistics
/// in the Claude Code status bar.
#[derive(Parser, Debug)]
#[command(name = "glm-plan-usage")]
#[command(about = "Display GLM plan usage statistics in Claude Code status bar", long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Verbose output
    #[arg(long, global = true)]
    pub verbose: bool,

    /// Disable cache
    #[arg(long, global = true)]
    pub no_cache: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize the configuration file.
    Init,

    /// Print the current configuration.
    Print,

    /// Validate the configuration file.
    Check,

    /// Update or migrate the configuration file.
    Update,
}
