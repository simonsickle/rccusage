mod aggregation;
mod commands;
mod config;
mod data_loader;
mod live;
mod logger;
mod output;
mod pricing;
mod types;
mod utils;

use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging based on LOG_LEVEL env var
    logger::init_logger();

    // Load config file if present (currently unused, reserved for future use)
    let _config = config::Config::load().unwrap_or_default();

    // Parse CLI arguments and run command
    let cli = commands::Cli::parse();
    cli.run().await
}
