use crate::aggregation::identify_session_blocks;
use crate::commands::BlocksArgs;
use crate::data_loader::load_usage_entries;
use crate::output::{output_json, table};
use crate::pricing::PricingFetcher;
use anyhow::Result;
use chrono::{Duration, Utc};
use tracing::info;

pub async fn run(args: BlocksArgs) -> Result<()> {
    let options = args.common.to_common_options();
    let pricing_fetcher = PricingFetcher::new(options.offline);

    info!("Loading usage data...");
    let entries = load_usage_entries(&options, &pricing_fetcher).await?;

    if entries.is_empty() {
        if options.json {
            println!("[]");
        } else {
            println!("No usage data found");
        }
        return Ok(());
    }

    info!("Identifying session blocks...");
    let mut blocks = identify_session_blocks(entries, args.token_limit);

    // Filter blocks based on flags
    if args.active {
        // Show only active block
        blocks.retain(|b| b.is_active);
    } else if args.recent {
        // Show blocks from last 3 days
        let cutoff = Utc::now() - Duration::days(3);
        blocks.retain(|b| b.start_time >= cutoff || b.is_active);
    }

    if blocks.is_empty() {
        if options.json {
            println!("[]");
        } else {
            println!("No session blocks found for the specified period");
        }
        return Ok(());
    }

    // Output results
    if options.json {
        output_json(&blocks, options.jq.as_deref())?;
    } else {
        table::output_blocks_table(&blocks, args.token_limit, options.compact)?;
    }

    Ok(())
}