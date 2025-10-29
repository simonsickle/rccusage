use crate::aggregation::aggregate_sessions;
use crate::commands::SessionArgs;
use crate::data_loader::load_usage_entries;
use crate::output::{output_json, table};
use crate::pricing::PricingFetcher;
use anyhow::Result;
use chrono::{Duration, Utc};
use tracing::info;

pub async fn run(args: SessionArgs) -> Result<()> {
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

    info!("Aggregating session usage...");
    let mut session_usage = aggregate_sessions(entries, options.order);

    // Filter by recent days if specified
    if let Some(days) = args.recent_days {
        let cutoff_date = Utc::now().date_naive() - Duration::days(days.into());
        session_usage.retain(|s| s.last_activity >= cutoff_date);
    }

    if session_usage.is_empty() {
        if options.json {
            println!("[]");
        } else {
            println!("No session data found for the specified period");
        }
        return Ok(());
    }

    // Output results
    if options.json {
        output_json(&session_usage, options.jq.as_deref())?;
    } else {
        table::output_session_table(&session_usage, options.compact)?;
    }

    Ok(())
}
