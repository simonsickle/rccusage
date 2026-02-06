use crate::commands::StatuslineArgs;
use anyhow::Result;
use serde::Deserialize;
use std::io::{self, Read};

const MAX_INPUT_SIZE: u64 = 1024 * 1024; // 1MB

#[derive(Deserialize)]
struct StatusInput {
    model: Option<ModelInfo>,
    context_window: Option<ContextWindow>,
    cost: Option<CostInfo>,
}

#[derive(Deserialize)]
struct ModelInfo {
    display_name: Option<String>,
}

#[derive(Deserialize)]
struct ContextWindow {
    used_percentage: Option<f64>,
}

#[derive(Deserialize)]
struct CostInfo {
    total_cost_usd: Option<f64>,
    total_duration_ms: Option<u64>,
    total_lines_added: Option<u64>,
    total_lines_removed: Option<u64>,
}

pub async fn run(args: StatuslineArgs) -> Result<()> {
    let mut input = String::new();
    io::stdin()
        .take(MAX_INPUT_SIZE)
        .read_to_string(&mut input)?;

    let data: StatusInput = serde_json::from_str(&input).unwrap_or(StatusInput {
        model: None,
        context_window: None,
        cost: None,
    });

    let model = data
        .model
        .as_ref()
        .and_then(|m| m.display_name.as_deref())
        .unwrap_or("?");

    let cost = data
        .cost
        .as_ref()
        .and_then(|c| c.total_cost_usd)
        .unwrap_or(0.0);

    let ctx_pct = data
        .context_window
        .as_ref()
        .and_then(|c| c.used_percentage)
        .unwrap_or(0.0) as u64;

    let lines_added = data
        .cost
        .as_ref()
        .and_then(|c| c.total_lines_added)
        .unwrap_or(0);

    let lines_removed = data
        .cost
        .as_ref()
        .and_then(|c| c.total_lines_removed)
        .unwrap_or(0);

    let duration_ms = data
        .cost
        .as_ref()
        .and_then(|c| c.total_duration_ms)
        .unwrap_or(0);

    let duration_str = format_duration(duration_ms);
    let ctx_bar = context_bar(ctx_pct);

    match args.format.as_str() {
        "minimal" => {
            print!("${:.2}", cost);
        }
        _ => {
            print!(
                "[{}] ${:.2} | {}{}% | +{}/−{} | {}",
                model, cost, ctx_bar, ctx_pct, lines_added, lines_removed, duration_str
            );
        }
    }

    Ok(())
}

fn context_bar(pct: u64) -> &'static str {
    match pct {
        0..=25 => "░░░░ ",
        26..=50 => "▓░░░ ",
        51..=75 => "▓▓░░ ",
        76..=90 => "▓▓▓░ ",
        _ => "▓▓▓▓ ",
    }
}

fn format_duration(ms: u64) -> String {
    let secs = ms / 1000;
    if secs < 60 {
        format!("{}s", secs)
    } else {
        let mins = secs / 60;
        let remaining_secs = secs % 60;
        format!("{}m{}s", mins, remaining_secs)
    }
}
