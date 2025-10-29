use crate::types::*;
use chrono::{DateTime, Duration, Utc};
use indexmap::IndexMap;
use itertools::Itertools;
use rust_decimal::prelude::*;
use std::collections::{HashMap, HashSet};

/// Aggregate usage entries by day
pub fn aggregate_daily(
    entries: Vec<LoadedUsageEntry>,
    order: SortOrder,
) -> Vec<DailyUsage> {
    let mut daily_map: IndexMap<DailyDate, Vec<LoadedUsageEntry>> = IndexMap::new();

    // Group entries by date
    for entry in entries {
        let date = DailyDate::from_datetime(entry.timestamp);
        daily_map.entry(date).or_insert_with(Vec::new).push(entry);
    }

    // Convert to DailyUsage structs
    let mut results: Vec<_> = daily_map
        .into_iter()
        .map(|(date, entries)| aggregate_entries_to_daily(date, entries))
        .collect();

    // Sort by date
    match order {
        SortOrder::Asc => results.sort_by_key(|d| d.date.clone()),
        SortOrder::Desc => results.sort_by_key(|d| std::cmp::Reverse(d.date.clone())),
    }

    results
}

/// Aggregate usage entries by month
pub fn aggregate_monthly(
    entries: Vec<LoadedUsageEntry>,
    order: SortOrder,
) -> Vec<MonthlyUsage> {
    let mut monthly_map: IndexMap<MonthlyDate, Vec<LoadedUsageEntry>> = IndexMap::new();

    // Group entries by month
    for entry in entries {
        let date = MonthlyDate::from_datetime(entry.timestamp);
        monthly_map.entry(date).or_insert_with(Vec::new).push(entry);
    }

    // Convert to MonthlyUsage structs
    let mut results: Vec<_> = monthly_map
        .into_iter()
        .map(|(date, entries)| aggregate_entries_to_monthly(date, entries))
        .collect();

    // Sort by date
    match order {
        SortOrder::Asc => results.sort_by_key(|m| m.date.clone()),
        SortOrder::Desc => results.sort_by_key(|m| std::cmp::Reverse(m.date.clone())),
    }

    results
}

/// Aggregate usage entries by week
pub fn aggregate_weekly(
    entries: Vec<LoadedUsageEntry>,
    order: SortOrder,
) -> Vec<WeeklyUsage> {
    let mut weekly_map: IndexMap<WeeklyDate, Vec<LoadedUsageEntry>> = IndexMap::new();

    // Group entries by week
    for entry in entries {
        let date = WeeklyDate::from_datetime(entry.timestamp);
        weekly_map.entry(date).or_insert_with(Vec::new).push(entry);
    }

    // Convert to WeeklyUsage structs
    let mut results: Vec<_> = weekly_map
        .into_iter()
        .map(|(date, entries)| aggregate_entries_to_weekly(date, entries))
        .collect();

    // Sort by date
    match order {
        SortOrder::Asc => results.sort_by_key(|w| w.date.clone()),
        SortOrder::Desc => results.sort_by_key(|w| std::cmp::Reverse(w.date.clone())),
    }

    results
}

/// Aggregate usage entries by session
pub fn aggregate_sessions(
    entries: Vec<LoadedUsageEntry>,
    order: SortOrder,
) -> Vec<SessionUsage> {
    let mut session_map: IndexMap<(SessionId, ProjectPath), Vec<LoadedUsageEntry>> = IndexMap::new();

    // Group entries by session and project
    for entry in entries {
        if let Some(session_id) = entry.session_id.clone() {
            let project_path = ProjectPath::new(
                entry
                    .project
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
            );
            let key = (session_id, project_path);
            session_map.entry(key).or_insert_with(Vec::new).push(entry);
        }
    }

    // Convert to SessionUsage structs
    let mut results: Vec<_> = session_map
        .into_iter()
        .map(|((session_id, project_path), entries)| {
            aggregate_entries_to_session(session_id, project_path, entries)
        })
        .collect();

    // Sort by last activity
    match order {
        SortOrder::Asc => results.sort_by_key(|s| s.last_activity),
        SortOrder::Desc => results.sort_by_key(|s| std::cmp::Reverse(s.last_activity)),
    }

    results
}

/// Identify 5-hour billing blocks from entries
pub fn identify_session_blocks(
    mut entries: Vec<LoadedUsageEntry>,
    token_limit: Option<u64>,
) -> Vec<SessionBlock> {
    if entries.is_empty() {
        return Vec::new();
    }

    // Sort by timestamp
    entries.sort_by_key(|e| e.timestamp);

    let mut blocks = Vec::new();
    let block_duration = Duration::hours(5);
    let now = Utc::now();

    let mut current_block_entries = Vec::new();
    let mut current_block_start: Option<DateTime<Utc>> = None;

    for entry in entries {
        // Floor timestamp to hour
        use chrono::{Timelike};
        let entry_hour = entry.timestamp
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap();

        if let Some(block_start) = current_block_start {
            // Check if entry is within current block
            if entry_hour < block_start + block_duration {
                current_block_entries.push(entry);
            } else {
                // Create block for current entries
                if !current_block_entries.is_empty() {
                    blocks.push(create_session_block(
                        block_start,
                        std::mem::take(&mut current_block_entries),
                        now,
                        token_limit,
                    ));
                }

                // Check for gap
                if entry_hour > block_start + block_duration {
                    // Create gap block
                    let gap_start = block_start + block_duration;
                    let gap_end = entry_hour;
                    blocks.push(SessionBlock {
                        id: gap_start.to_rfc3339(),
                        start_time: gap_start,
                        end_time: gap_end,
                        actual_end_time: None,
                        is_active: false,
                        is_gap: Some(true),
                        token_counts: TokenCounts::default(),
                        cost_usd: Decimal::ZERO,
                        models: Vec::new(),
                        usage_limit_reset_time: None,
                    });
                }

                // Start new block
                current_block_start = Some(entry_hour);
                current_block_entries.push(entry);
            }
        } else {
            // First block
            current_block_start = Some(entry_hour);
            current_block_entries.push(entry);
        }
    }

    // Create final block
    if let Some(block_start) = current_block_start {
        if !current_block_entries.is_empty() {
            blocks.push(create_session_block(
                block_start,
                current_block_entries,
                now,
                token_limit,
            ));
        }
    }

    blocks
}

/// Helper function to create a session block
fn create_session_block(
    start_time: DateTime<Utc>,
    entries: Vec<LoadedUsageEntry>,
    now: DateTime<Utc>,
    token_limit: Option<u64>,
) -> SessionBlock {
    let end_time = start_time + Duration::hours(5);
    let is_active = entries
        .last()
        .map(|e| now - e.timestamp < Duration::minutes(30))
        .unwrap_or(false);

    let actual_end_time = if !is_active {
        entries.last().map(|e| e.timestamp)
    } else {
        None
    };

    // Aggregate tokens and costs
    let mut token_counts = TokenCounts::default();
    let mut total_cost = Decimal::ZERO;
    let mut models = HashSet::new();

    for entry in &entries {
        token_counts.add(&entry.tokens);
        total_cost += entry.cost;
        models.insert(entry.model.as_str().to_string());
    }

    // Calculate usage limit reset time if near limit
    let usage_limit_reset_time = if let Some(limit) = token_limit {
        if token_counts.total() >= (limit * 80 / 100) {
            Some(end_time)
        } else {
            None
        }
    } else {
        None
    };

    SessionBlock {
        id: start_time.to_rfc3339(),
        start_time,
        end_time,
        actual_end_time,
        is_active,
        is_gap: Some(false),
        token_counts,
        cost_usd: total_cost,
        models: models.into_iter().sorted().collect(),
        usage_limit_reset_time,
    }
}

/// Helper to aggregate entries to DailyUsage
fn aggregate_entries_to_daily(date: DailyDate, entries: Vec<LoadedUsageEntry>) -> DailyUsage {
    let (tokens, cost, models, breakdowns) = aggregate_tokens_and_cost(entries);

    DailyUsage {
        date,
        input_tokens: tokens.input_tokens,
        output_tokens: tokens.output_tokens,
        cache_creation_tokens: tokens.cache_creation_tokens,
        cache_read_tokens: tokens.cache_read_tokens,
        total_cost: cost,
        models_used: models,
        model_breakdowns: breakdowns,
        project: None,
    }
}

/// Helper to aggregate entries to MonthlyUsage
fn aggregate_entries_to_monthly(date: MonthlyDate, entries: Vec<LoadedUsageEntry>) -> MonthlyUsage {
    let (tokens, cost, models, breakdowns) = aggregate_tokens_and_cost(entries);

    MonthlyUsage {
        date,
        input_tokens: tokens.input_tokens,
        output_tokens: tokens.output_tokens,
        cache_creation_tokens: tokens.cache_creation_tokens,
        cache_read_tokens: tokens.cache_read_tokens,
        total_cost: cost,
        models_used: models,
        model_breakdowns: breakdowns,
        project: None,
    }
}

/// Helper to aggregate entries to WeeklyUsage
fn aggregate_entries_to_weekly(date: WeeklyDate, entries: Vec<LoadedUsageEntry>) -> WeeklyUsage {
    let (tokens, cost, models, breakdowns) = aggregate_tokens_and_cost(entries);

    WeeklyUsage {
        date,
        input_tokens: tokens.input_tokens,
        output_tokens: tokens.output_tokens,
        cache_creation_tokens: tokens.cache_creation_tokens,
        cache_read_tokens: tokens.cache_read_tokens,
        total_cost: cost,
        models_used: models,
        model_breakdowns: breakdowns,
        project: None,
    }
}

/// Helper to aggregate entries to SessionUsage
fn aggregate_entries_to_session(
    session_id: SessionId,
    project_path: ProjectPath,
    entries: Vec<LoadedUsageEntry>,
) -> SessionUsage {
    let (tokens, cost, models, breakdowns) = aggregate_tokens_and_cost(entries.clone());

    // Get unique versions
    let versions: Vec<String> = entries
        .iter()
        .filter_map(|e| e.version.clone())
        .unique()
        .collect();

    // Get last activity date
    let last_activity = entries
        .iter()
        .map(|e| e.timestamp.date_naive())
        .max()
        .unwrap_or_else(|| Utc::now().date_naive());

    SessionUsage {
        session_id,
        project_path,
        input_tokens: tokens.input_tokens,
        output_tokens: tokens.output_tokens,
        cache_creation_tokens: tokens.cache_creation_tokens,
        cache_read_tokens: tokens.cache_read_tokens,
        total_cost: cost,
        last_activity,
        versions,
        models_used: models,
        model_breakdowns: breakdowns,
    }
}

/// Helper to aggregate tokens and costs from entries
fn aggregate_tokens_and_cost(
    entries: Vec<LoadedUsageEntry>,
) -> (AggregatedTokenCounts, Decimal, Vec<ModelName>, Vec<ModelBreakdown>) {
    let mut total_tokens = AggregatedTokenCounts::default();
    let mut total_cost = Decimal::ZERO;
    let mut model_map: HashMap<ModelName, (AggregatedTokenCounts, Decimal)> = HashMap::new();

    for entry in entries {
        // Add to totals
        total_tokens.add_from_raw(&entry.tokens);
        total_cost += entry.cost;

        // Add to model-specific totals
        let model_entry = model_map
            .entry(entry.model.clone())
            .or_insert_with(|| (AggregatedTokenCounts::default(), Decimal::ZERO));
        model_entry.0.add_from_raw(&entry.tokens);
        model_entry.1 += entry.cost;
    }

    // Create model breakdowns, sorted by cost descending
    let mut breakdowns: Vec<ModelBreakdown> = model_map
        .into_iter()
        .map(|(model, (tokens, cost))| ModelBreakdown {
            model_name: model,
            input_tokens: tokens.input_tokens,
            output_tokens: tokens.output_tokens,
            cache_creation_tokens: tokens.cache_creation_tokens,
            cache_read_tokens: tokens.cache_read_tokens,
            cost,
        })
        .collect();

    breakdowns.sort_by(|a, b| b.cost.cmp(&a.cost));

    let models: Vec<ModelName> = breakdowns.iter().map(|b| b.model_name.clone()).collect();

    (total_tokens, total_cost, models, breakdowns)
}