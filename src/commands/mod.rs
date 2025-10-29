pub mod blocks;
pub mod daily;
pub mod monthly;
pub mod session;
pub mod statusline;
pub mod weekly;

use crate::types::{CommonOptions, CostMode, SortOrder};
use anyhow::Result;
use chrono::NaiveDate;
use clap::{Parser, Subcommand};

/// Usage analysis tool for Claude Code
#[derive(Parser, Debug)]
#[command(name = "ccusage")]
#[command(version)]
#[command(about = "Fast usage analysis tool for Claude Code", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Show daily usage report
    Daily(DailyArgs),

    /// Show monthly usage report
    Monthly(MonthlyArgs),

    /// Show weekly usage report
    Weekly(WeeklyArgs),

    /// Show session-based usage report
    Session(SessionArgs),

    /// Show 5-hour billing blocks usage report
    Blocks(BlocksArgs),

    /// Show compact status line (Beta)
    Statusline(StatuslineArgs),
}

/// Common arguments shared across commands
#[derive(Parser, Debug, Clone)]
pub struct CommonArgs {
    /// Output format as JSON instead of table
    #[arg(long)]
    pub json: bool,

    /// Cost calculation mode
    #[arg(long, value_enum, default_value_t = CostMode::Auto)]
    pub mode: CostMode,

    /// Start date filter (YYYYMMDD format)
    #[arg(long, value_parser = parse_date)]
    pub since: Option<NaiveDate>,

    /// End date filter (YYYYMMDD format)
    #[arg(long, value_parser = parse_date)]
    pub until: Option<NaiveDate>,

    /// Sort order
    #[arg(long, value_enum, default_value_t = SortOrder::Asc)]
    pub order: SortOrder,

    /// Timezone for date grouping (e.g., "America/New_York")
    #[arg(long, env = "TZ")]
    pub timezone: Option<String>,

    /// Use offline pricing only
    #[arg(long)]
    pub offline: bool,

    /// Filter by project name
    #[arg(long)]
    pub project: Option<String>,

    /// jq expression for JSON filtering
    #[arg(long)]
    pub jq: Option<String>,

    /// Force compact display mode (auto-detected by default)
    #[arg(long)]
    pub compact: bool,
}

impl CommonArgs {
    pub fn to_common_options(&self) -> CommonOptions {
        CommonOptions {
            json: self.json,
            mode: self.mode,
            since: self.since,
            until: self.until,
            order: self.order,
            timezone: self.timezone.clone(),
            offline: self.offline,
            project: self.project.clone(),
            jq: self.jq.clone(),
            compact: self.compact,
        }
    }
}

/// Arguments for daily command
#[derive(Parser, Debug, Clone)]
pub struct DailyArgs {
    #[command(flatten)]
    pub common: CommonArgs,

    /// Show breakdown by project
    #[arg(long)]
    pub by_project: bool,

    /// Show all daily data ever (no date filtering)
    #[arg(long)]
    pub all_time: bool,

    /// Enable live monitoring mode (watch for file changes)
    #[arg(long)]
    pub watch: bool,
}

/// Arguments for monthly command
#[derive(Parser, Debug)]
pub struct MonthlyArgs {
    #[command(flatten)]
    pub common: CommonArgs,

    /// Show breakdown by project
    #[arg(long)]
    pub by_project: bool,

    /// Show all monthly data ever (no date filtering)
    #[arg(long)]
    pub all_time: bool,
}

/// Arguments for weekly command
#[derive(Parser, Debug)]
pub struct WeeklyArgs {
    #[command(flatten)]
    pub common: CommonArgs,

    /// Show breakdown by project
    #[arg(long)]
    pub by_project: bool,

    /// Show all weekly data ever (no date filtering)
    #[arg(long)]
    pub all_time: bool,
}

/// Arguments for session command
#[derive(Parser, Debug)]
pub struct SessionArgs {
    #[command(flatten)]
    pub common: CommonArgs,

    /// Show only sessions with activity in last N days
    #[arg(long)]
    pub recent_days: Option<u32>,

    /// Show all sessions ever (no date filtering)
    #[arg(long, conflicts_with = "recent_days")]
    pub all_time: bool,
}

/// Arguments for blocks command
#[derive(Parser, Debug)]
pub struct BlocksArgs {
    #[command(flatten)]
    pub common: CommonArgs,

    /// Show only the active block with projections
    #[arg(long)]
    pub active: bool,

    /// Show blocks from the last 3 days (including active)
    #[arg(long)]
    pub recent: bool,

    /// Token limit for quota warnings (number or "max")
    #[arg(long, value_parser = parse_token_limit)]
    pub token_limit: Option<u64>,
}

/// Arguments for statusline command
#[derive(Parser, Debug)]
pub struct StatuslineArgs {
    #[command(flatten)]
    pub common: CommonArgs,

    /// Format for statusline output
    #[arg(long, default_value = "compact")]
    pub format: String,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        match self.command {
            Commands::Daily(args) => daily::run(args).await,
            Commands::Monthly(args) => monthly::run(args).await,
            Commands::Weekly(args) => weekly::run(args).await,
            Commands::Session(args) => session::run(args).await,
            Commands::Blocks(args) => blocks::run(args).await,
            Commands::Statusline(args) => statusline::run(args).await,
        }
    }
}

/// Parse date from YYYYMMDD format
fn parse_date(s: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(s, "%Y%m%d")
        .map_err(|e| format!("Invalid date format (expected YYYYMMDD): {}", e))
}

/// Parse token limit (number or "max")
fn parse_token_limit(s: &str) -> Result<u64, String> {
    if s.to_lowercase() == "max" {
        Ok(u64::MAX)
    } else {
        s.parse::<u64>()
            .map_err(|e| format!("Invalid token limit: {}", e))
    }
}