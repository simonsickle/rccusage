use crate::aggregation::aggregate_daily;
use crate::commands::DailyArgs;
use crate::data_loader::load_usage_entries;
use crate::live::LiveMonitor;
use crate::output::{output_json, table};
use crate::pricing::PricingFetcher;
use anyhow::Result;
use tracing::info;

pub async fn run(args: DailyArgs) -> Result<()> {
    let mut options = args.common.to_common_options();

    // If all_time flag is set, clear date filters
    if args.all_time {
        options.since = None;
        options.until = None;
    }

    // If watch mode is enabled, use live monitoring
    if args.watch {
        let monitor = LiveMonitor::new();
        let args_clone = args.clone();

        monitor.watch(move || {
            let runtime = tokio::runtime::Runtime::new()?;
            runtime.block_on(run_once(args_clone.clone()))
        })?;
    } else {
        run_once(args).await?;
    }

    Ok(())
}

async fn run_once(args: DailyArgs) -> Result<()> {
    let mut options = args.common.to_common_options();

    // If all_time flag is set, clear date filters
    if args.all_time {
        options.since = None;
        options.until = None;
    }

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

    info!("Aggregating daily usage...");
    let daily_usage = aggregate_daily(entries, options.order);

    if daily_usage.is_empty() {
        if options.json {
            println!("[]");
        } else {
            println!("No usage data found for the specified period");
        }
        return Ok(());
    }

    // Output results
    if options.json {
        output_json(&daily_usage, options.jq.as_deref())?;
    } else {
        table::output_daily_table(&daily_usage, options.compact)?;
    }

    Ok(())
}
