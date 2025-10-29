use crate::types::*;
use anyhow::Result;
use colored::*;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, presets::UTF8_BORDERS_ONLY, Cell, Color, ContentArrangement, Table};
use rust_decimal::prelude::*;
use terminal_size::{terminal_size, Width};

/// Get terminal width
fn get_terminal_width() -> usize {
    terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or(120) // Default width if can't detect
}

/// Format cost as currency
fn format_cost(cost: Decimal) -> String {
    if cost >= Decimal::from(1000) {
        format!("${:.0}", cost)
    } else if cost >= Decimal::from(100) {
        format!("${:.1}", cost)
    } else {
        format!("${:.2}", cost)
    }
}

/// Format tokens with K/M/B suffix
fn format_tokens_compact(tokens: u64) -> String {
    if tokens >= 1_000_000_000 {
        format!("{:.1}B", tokens as f64 / 1_000_000_000.0)
    } else if tokens >= 1_000_000 {
        format!("{:.1}M", tokens as f64 / 1_000_000.0)
    } else if tokens >= 10_000 {
        format!("{:.0}K", tokens as f64 / 1_000.0)
    } else if tokens >= 1_000 {
        format!("{:.1}K", tokens as f64 / 1_000.0)
    } else {
        tokens.to_string()
    }
}

/// Abbreviate model name for compact display
fn abbreviate_model_name(name: &str) -> String {
    // Common patterns to abbreviate
    let name = name
        .replace("claude-", "")
        .replace("-20250929", "")
        .replace("-20251001", "")
        .replace("-20250805", "")
        .replace("-4-5", "-4.5")
        .replace("-4-1", "-4.1");

    // Further abbreviations
    match name.as_str() {
        s if s.contains("sonnet-4.5") => "S4.5".to_string(),
        s if s.contains("haiku-4.5") => "H4.5".to_string(),
        s if s.contains("opus-4.1") => "O4.1".to_string(),
        s if s.contains("sonnet") => "Sonnet".to_string(),
        s if s.contains("haiku") => "Haiku".to_string(),
        s if s.contains("opus") => "Opus".to_string(),
        _ => name.to_string(),
    }
}

/// Output daily usage as compact table
pub fn output_daily_table(data: &[DailyUsage], force_compact: bool) -> Result<()> {
    let width = get_terminal_width();
    let compact_mode = force_compact || width < 120;

    let mut table = Table::new();

    // Use more compact preset for narrow terminals
    if compact_mode {
        table.load_preset(UTF8_BORDERS_ONLY);
    } else {
        table.load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS);
    }

    // Set content arrangement for better wrapping
    table.set_content_arrangement(ContentArrangement::Dynamic);

    // Headers based on terminal width
    if compact_mode {
        // Ultra-compact mode for narrow terminals
        table.set_header(vec![
            Cell::new("Date").fg(Color::Blue),
            Cell::new("Tokens").fg(Color::Blue),
            Cell::new("Cost").fg(Color::Blue),
            Cell::new("Models").fg(Color::Blue),
        ]);
    } else {
        // Normal mode with more details
        table.set_header(vec![
            Cell::new("Date").fg(Color::Blue),
            Cell::new("In").fg(Color::Blue),
            Cell::new("Out").fg(Color::Blue),
            Cell::new("Cache").fg(Color::Blue),
            Cell::new("Total").fg(Color::Blue),
            Cell::new("Cost").fg(Color::Green),
            Cell::new("Models").fg(Color::Blue),
        ]);
    }

    let mut total_input = 0u64;
    let mut total_output = 0u64;
    let mut total_cache = 0u64;
    let mut total_cost = Decimal::ZERO;

    for usage in data {
        let total_tokens = usage.total_tokens();
        let cache_tokens = usage.cache_creation_tokens + usage.cache_read_tokens;

        total_input += usage.input_tokens;
        total_output += usage.output_tokens;
        total_cache += cache_tokens;
        total_cost += usage.total_cost;

        // Format models list compactly
        let models_str = usage
            .models_used
            .iter()
            .map(|m| abbreviate_model_name(&m.to_string()))
            .collect::<Vec<_>>()
            .join(", ");

        if compact_mode {
            // Ultra-compact: combine all tokens into one column
            let tokens_str = format!("{}↑ {}↓ {}◆",
                format_tokens_compact(usage.input_tokens),
                format_tokens_compact(usage.output_tokens),
                format_tokens_compact(cache_tokens)
            );

            table.add_row(vec![
                Cell::new(usage.date.to_string()),
                Cell::new(tokens_str),
                Cell::new(format_cost(usage.total_cost)).fg(Color::Green),
                Cell::new(models_str).fg(Color::Cyan),
            ]);
        } else {
            // Normal mode with separate columns
            table.add_row(vec![
                Cell::new(usage.date.to_string()),
                Cell::new(format_tokens_compact(usage.input_tokens)),
                Cell::new(format_tokens_compact(usage.output_tokens)),
                Cell::new(format_tokens_compact(cache_tokens)).fg(Color::Grey),
                Cell::new(format_tokens_compact(total_tokens)).fg(Color::Yellow),
                Cell::new(format_cost(usage.total_cost)).fg(Color::Green),
                Cell::new(models_str).fg(Color::Cyan),
            ]);
        }
    }

    // Add totals row
    let total_all = total_input + total_output + total_cache;

    if compact_mode {
        let tokens_str = format!("{}↑ {}↓ {}◆",
            format_tokens_compact(total_input),
            format_tokens_compact(total_output),
            format_tokens_compact(total_cache)
        );

        table.add_row(vec![
            Cell::new("TOTAL").fg(Color::Yellow),
            Cell::new(tokens_str).fg(Color::Yellow),
            Cell::new(format_cost(total_cost)).fg(Color::Green),
            Cell::new("").fg(Color::Yellow),
        ]);
    } else {
        table.add_row(vec![
            Cell::new("TOTAL").fg(Color::Yellow),
            Cell::new(format_tokens_compact(total_input)).fg(Color::Yellow),
            Cell::new(format_tokens_compact(total_output)).fg(Color::Yellow),
            Cell::new(format_tokens_compact(total_cache)).fg(Color::Yellow),
            Cell::new(format_tokens_compact(total_all)).fg(Color::Yellow),
            Cell::new(format_cost(total_cost)).fg(Color::Green),
            Cell::new(""),
        ]);
    }

    println!("{}", table);
    Ok(())
}

/// Output monthly usage as table
pub fn output_monthly_table(data: &[MonthlyUsage], force_compact: bool) -> Result<()> {
    let width = get_terminal_width();
    let compact_mode = force_compact || width < 100;

    let mut table = Table::new();

    if compact_mode {
        table.load_preset(UTF8_BORDERS_ONLY);
    } else {
        table.load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS);
    }

    table.set_content_arrangement(ContentArrangement::Dynamic);

    if compact_mode {
        table.set_header(vec![
            Cell::new("Month").fg(Color::Blue),
            Cell::new("Tokens").fg(Color::Blue),
            Cell::new("Cost").fg(Color::Blue),
        ]);
    } else {
        table.set_header(vec![
            Cell::new("Month").fg(Color::Blue),
            Cell::new("Input").fg(Color::Blue),
            Cell::new("Output").fg(Color::Blue),
            Cell::new("Cache").fg(Color::Blue),
            Cell::new("Total").fg(Color::Blue),
            Cell::new("Cost").fg(Color::Green),
        ]);
    }

    let mut total_cost = Decimal::ZERO;
    let mut total_tokens = 0u64;

    for usage in data {
        let tokens = usage.total_tokens();
        let cache = usage.cache_creation_tokens + usage.cache_read_tokens;
        total_cost += usage.total_cost;
        total_tokens += tokens;

        if compact_mode {
            let tokens_str = format!("{}↑ {}↓",
                format_tokens_compact(usage.input_tokens),
                format_tokens_compact(usage.output_tokens)
            );

            table.add_row(vec![
                Cell::new(usage.date.to_string()),
                Cell::new(tokens_str),
                Cell::new(format_cost(usage.total_cost)).fg(Color::Green),
            ]);
        } else {
            table.add_row(vec![
                Cell::new(usage.date.to_string()),
                Cell::new(format_tokens_compact(usage.input_tokens)),
                Cell::new(format_tokens_compact(usage.output_tokens)),
                Cell::new(format_tokens_compact(cache)).fg(Color::Grey),
                Cell::new(format_tokens_compact(tokens)).fg(Color::Yellow),
                Cell::new(format_cost(usage.total_cost)).fg(Color::Green),
            ]);
        }
    }

    // Add total row
    if compact_mode {
        table.add_row(vec![
            Cell::new("TOTAL").fg(Color::Yellow),
            Cell::new(format_tokens_compact(total_tokens)).fg(Color::Yellow),
            Cell::new(format_cost(total_cost)).fg(Color::Green),
        ]);
    } else {
        table.add_row(vec![
            Cell::new("TOTAL").fg(Color::Yellow),
            Cell::new(""),
            Cell::new(""),
            Cell::new(""),
            Cell::new(format_tokens_compact(total_tokens)).fg(Color::Yellow),
            Cell::new(format_cost(total_cost)).fg(Color::Green),
        ]);
    }

    println!("{}", table);
    Ok(())
}

/// Output weekly usage as table
pub fn output_weekly_table(data: &[WeeklyUsage], force_compact: bool) -> Result<()> {
    // Similar to monthly but with week formatting
    output_monthly_table(&data.iter().map(|w| MonthlyUsage {
        date: MonthlyDate::from_datetime(chrono::Utc::now()),  // Placeholder
        input_tokens: w.input_tokens,
        output_tokens: w.output_tokens,
        cache_creation_tokens: w.cache_creation_tokens,
        cache_read_tokens: w.cache_read_tokens,
        total_cost: w.total_cost,
        models_used: w.models_used.clone(),
        model_breakdowns: w.model_breakdowns.clone(),
        project: w.project.clone(),
    }).collect::<Vec<_>>(), force_compact)
}

/// Output session usage as table
pub fn output_session_table(data: &[SessionUsage], force_compact: bool) -> Result<()> {
    let width = get_terminal_width();
    let compact_mode = force_compact || width < 120;

    let mut table = Table::new();

    if compact_mode {
        table.load_preset(UTF8_BORDERS_ONLY);
    } else {
        table.load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS);
    }

    table.set_content_arrangement(ContentArrangement::Dynamic);

    if compact_mode {
        table.set_header(vec![
            Cell::new("Session").fg(Color::Blue),
            Cell::new("Msgs").fg(Color::Blue),
            Cell::new("Cost").fg(Color::Blue),
            Cell::new("Last").fg(Color::Blue),
        ]);
    } else {
        table.set_header(vec![
            Cell::new("Session").fg(Color::Blue),
            Cell::new("Msgs").fg(Color::Blue),
            Cell::new("Tkns").fg(Color::Blue),
            Cell::new("Cost").fg(Color::Green),
            Cell::new("First").fg(Color::Blue),
            Cell::new("Last").fg(Color::Blue),
        ]);
    }

    let mut total_cost = Decimal::ZERO;
    let mut total_messages = 0u64;

    for session in data {
        total_cost += session.total_cost;
        total_messages += 1; // Count sessions instead of messages for now

        // Truncate long session IDs
        let session_id_str = session.session_id.0.clone();
        let session_id = if session_id_str.len() > 30 && compact_mode {
            format!("{}...", &session_id_str[..27])
        } else if session_id_str.len() > 50 {
            format!("{}...", &session_id_str[..47])
        } else {
            session_id_str
        };

        if compact_mode {
            table.add_row(vec![
                Cell::new(session_id).fg(Color::Cyan),
                Cell::new("1"), // TODO: add message count to SessionUsage
                Cell::new(format_cost(session.total_cost)).fg(Color::Green),
                Cell::new(session.last_activity.format("%m/%d").to_string()),
            ]);
        } else {
            table.add_row(vec![
                Cell::new(session_id).fg(Color::Cyan),
                Cell::new("1"), // TODO: add message count to SessionUsage
                Cell::new(format_tokens_compact(session.total_tokens())),
                Cell::new(format_cost(session.total_cost)).fg(Color::Green),
                Cell::new(session.last_activity.format("%Y-%m-%d").to_string()),
                Cell::new(session.last_activity.format("%Y-%m-%d").to_string()),
            ]);
        }
    }

    // Add total row
    if compact_mode {
        table.add_row(vec![
            Cell::new("TOTAL").fg(Color::Yellow),
            Cell::new(total_messages.to_string()).fg(Color::Yellow),
            Cell::new(format_cost(total_cost)).fg(Color::Green),
            Cell::new(""),
        ]);
    } else {
        table.add_row(vec![
            Cell::new("TOTAL").fg(Color::Yellow),
            Cell::new(total_messages.to_string()).fg(Color::Yellow),
            Cell::new(""),
            Cell::new(format_cost(total_cost)).fg(Color::Green),
            Cell::new(""),
            Cell::new(""),
        ]);
    }

    println!("{}", table);
    Ok(())
}

/// Output blocks usage as table
pub fn output_blocks_table(data: &[SessionBlock], token_limit: Option<u64>, force_compact: bool) -> Result<()> {
    let width = get_terminal_width();
    let compact_mode = force_compact || width < 100;

    let mut table = Table::new();

    if compact_mode {
        table.load_preset(UTF8_BORDERS_ONLY);
    } else {
        table.load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS);
    }

    table.set_content_arrangement(ContentArrangement::Dynamic);

    table.set_header(vec![
        Cell::new("Block").fg(Color::Blue),
        Cell::new("Period").fg(Color::Blue),
        Cell::new("Usage").fg(Color::Blue),
        Cell::new("Cost").fg(Color::Green),
        Cell::new("Status").fg(Color::Blue),
    ]);

    for block in data {
        let usage_pct = if let Some(limit) = token_limit {
            let pct = (block.total_tokens() as f64 / limit as f64 * 100.0).min(100.0);
            format!("{:.1}%", pct)
        } else {
            format_tokens_compact(block.total_tokens())
        };

        let status = if block.is_active {
            "● Active".green().to_string()
        } else {
            "○ Complete".dimmed().to_string()
        };

        let period = format!("{} - {}",
            block.start_time.format("%m/%d %H:%M"),
            block.end_time.format("%m/%d %H:%M")
        );

        table.add_row(vec![
            Cell::new(block.id.clone()),
            Cell::new(period),
            Cell::new(usage_pct),
            Cell::new(format_cost(block.cost_usd)).fg(Color::Green),
            Cell::new(status),
        ]);
    }

    println!("{}", table);
    Ok(())
}

/// Output statusline in compact format
pub fn output_statusline(data: &StatuslineData) -> Result<()> {
    // Ultra-compact one-line status
    let status = format!(
        "{}  {} {}  {} {}  {}",
        format!("▶ {}", data.active_block_id).cyan(),
        format_tokens_compact(data.block_tokens).yellow(),
        format!("({}%)", data.block_usage_pct).dimmed(),
        format_cost(data.block_cost).green(),
        format!("◆ {}", format_tokens_compact(data.day_tokens)).white(),
        format_cost(data.day_cost).green(),
    );

    println!("{}", status);
    Ok(())
}

// StatuslineData struct for the statusline command
pub struct StatuslineData {
    pub active_block_id: String,
    pub block_tokens: u64,
    pub block_usage_pct: u32,
    pub block_cost: Decimal,
    pub day_tokens: u64,
    pub day_cost: Decimal,
}