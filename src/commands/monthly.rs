use crate::aggregation::aggregate_monthly;
use crate::commands::MonthlyArgs;
use crate::data_loader::load_usage_entries;
use crate::output::{output_json, table};
use crate::pricing::PricingFetcher;
use anyhow::Result;
use tracing::info;

pub async fn run(args: MonthlyArgs) -> Result<()> {
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

    info!("Aggregating monthly usage...");
    let monthly_usage = aggregate_monthly(entries, options.order);

    if monthly_usage.is_empty() {
        if options.json {
            println!("[]");
        } else {
            println!("No usage data found for the specified period");
        }
        return Ok(());
    }

    // Output results
    if options.json {
        output_json(&monthly_usage, options.jq.as_deref())?;
    } else {
        table::output_monthly_table(&monthly_usage, options.compact)?;
    }

    Ok(())
}
