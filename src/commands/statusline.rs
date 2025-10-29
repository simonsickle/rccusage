use crate::aggregation::identify_session_blocks;
use crate::commands::StatuslineArgs;
use crate::data_loader::load_usage_entries;
use crate::output::output_json;
use crate::pricing::PricingFetcher;
use anyhow::Result;
use rust_decimal::prelude::*;
use serde_json::json;
use tracing::info;

pub async fn run(args: StatuslineArgs) -> Result<()> {
    let options = args.common.to_common_options();
    let pricing_fetcher = PricingFetcher::new(options.offline);

    info!("Loading usage data...");
    let entries = load_usage_entries(&options, &pricing_fetcher).await?;

    // Find active block
    let blocks = identify_session_blocks(entries, None);
    let active_block = blocks.iter().find(|b| b.is_active);

    if options.json {
        let status = if let Some(block) = active_block {
            json!({
                "active": true,
                "tokens": block.total_tokens(),
                "cost": block.cost_usd.to_f64().unwrap_or(0.0),
                "models": block.models,
                "start_time": block.start_time.to_rfc3339(),
                "end_time": block.end_time.to_rfc3339(),
            })
        } else {
            json!({
                "active": false,
                "tokens": 0,
                "cost": 0.0,
                "models": [],
            })
        };

        output_json(&status, args.common.jq.as_deref())?;
    } else {
        // Compact text output for shell prompts
        if let Some(block) = active_block {
            let tokens = block.total_tokens();
            let cost = block.cost_usd;

            match args.format.as_str() {
                "compact" => {
                    // Compact format: "1.2K tokens | $0.05"
                    let tokens_str = format_token_count(tokens);
                    let cost_str = format_cost_compact(cost);
                    print!("{} | {}", tokens_str, cost_str);
                }
                "minimal" => {
                    // Minimal format: just cost
                    print!("{}", format_cost_compact(cost));
                }
                "tokens" => {
                    // Just token count
                    print!("{}", format_token_count(tokens));
                }
                _ => {
                    // Default to compact
                    let tokens_str = format_token_count(tokens);
                    let cost_str = format_cost_compact(cost);
                    print!("{} | {}", tokens_str, cost_str);
                }
            }
        } else {
            // No active block
            match args.format.as_str() {
                "minimal" => print!("$0.00"),
                "tokens" => print!("0"),
                _ => print!("No active session"),
            }
        }
    }

    Ok(())
}

fn format_token_count(tokens: u64) -> String {
    if tokens >= 1_000_000 {
        format!("{:.1}M tokens", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{:.1}K tokens", tokens as f64 / 1_000.0)
    } else {
        format!("{} tokens", tokens)
    }
}

fn format_cost_compact(cost: Decimal) -> String {
    format!("${:.2}", cost)
}
