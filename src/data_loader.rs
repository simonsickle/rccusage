use crate::pricing::PricingFetcher;
use crate::types::*;
use crate::utils;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use dashmap::DashSet;
use glob::glob;
use rust_decimal::prelude::*;
use serde_json;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Default Claude data directories to search
const DEFAULT_CLAUDE_PATHS: &[&str] = &[
    "~/.config/claude/projects", // New default location
    "~/.claude/projects",         // Old default location
];

/// Get all Claude data directories to search
pub fn get_claude_data_dirs() -> Vec<PathBuf> {
    // Check for custom paths from environment variable
    if let Ok(custom_paths) = std::env::var("CLAUDE_CONFIG_DIR") {
        return custom_paths
            .split(',')
            .map(|p| {
                let path = PathBuf::from(p.trim());
                if path.ends_with("projects") {
                    path
                } else {
                    path.join("projects")
                }
            })
            .filter(|p| p.exists())
            .collect();
    }

    // Use default paths
    DEFAULT_CLAUDE_PATHS
        .iter()
        .map(|p| utils::expand_tilde(p))
        .filter(|p| p.exists())
        .collect()
}

/// Find all JSONL files in Claude data directories
pub async fn find_jsonl_files() -> Result<Vec<PathBuf>> {
    let dirs = get_claude_data_dirs();
    if dirs.is_empty() {
        return Ok(Vec::new());
    }

    let mut all_files = Vec::new();

    for dir in dirs {
        let pattern = format!("{}/**/*.jsonl", dir.display());
        for entry in glob(&pattern).context("Failed to read glob pattern")? {
            if let Ok(path) = entry {
                all_files.push(path);
            }
        }
    }

    Ok(all_files)
}

/// Extract project name from file path
pub fn extract_project_name(file_path: &Path) -> String {
    // Path structure: .../projects/{project}/{sessionId}.jsonl
    let components: Vec<_> = file_path.components().collect();
    let projects_idx = components
        .iter()
        .rposition(|c| c.as_os_str() == "projects");

    if let Some(idx) = projects_idx {
        if idx + 1 < components.len() {
            return components[idx + 1]
                .as_os_str()
                .to_string_lossy()
                .to_string();
        }
    }

    "unknown".to_string()
}

/// Stream JSONL file line by line (PR #706 fix - prevents memory issues with large files)
pub fn stream_jsonl_file<F>(file_path: &Path, mut process_line: F) -> Result<()>
where
    F: FnMut(&str, usize) -> Result<()>,
{
    let file = File::open(file_path)
        .with_context(|| format!("Failed to open file: {}", file_path.display()))?;

    let reader = BufReader::new(file);
    let mut line_number = 0;

    for line_result in reader.lines() {
        line_number += 1;

        match line_result {
            Ok(line) => {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    // Silently skip malformed lines
                    let _ = process_line(trimmed, line_number);
                }
            }
            Err(_e) => {
                // Silently skip lines that can't be read
            }
        }
    }

    Ok(())
}

/// Parse a single JSONL entry into LoadedUsageEntry
pub async fn parse_usage_entry(
    data: &UsageData,
    project: String,
    cost_mode: CostMode,
    pricing_fetcher: &PricingFetcher,
) -> Result<LoadedUsageEntry> {
    // Parse timestamp
    let timestamp = DateTime::parse_from_rfc3339(&data.timestamp)
        .context("Failed to parse timestamp")?
        .with_timezone(&Utc);

    // Get model name
    let model = ModelName::new(
        data.message
            .model
            .as_ref()
            .cloned()
            .unwrap_or_else(|| "unknown".to_string()),
    );

    // Calculate cost based on mode
    let cost = match cost_mode {
        CostMode::Display => {
            // Always use pre-calculated cost, default to 0
            Decimal::from_f64(data.cost_usd.unwrap_or(0.0)).unwrap_or_else(|| Decimal::ZERO)
        }
        CostMode::Calculate => {
            // Always calculate from tokens
            pricing_fetcher
                .calculate_cost(&model, &data.message.usage)
                .await
                .unwrap_or_else(|_| Decimal::ZERO)
        }
        CostMode::Auto => {
            // Use pre-calculated if available, otherwise calculate
            if let Some(cost_usd) = data.cost_usd {
                Decimal::from_f64(cost_usd).unwrap_or_else(|| Decimal::ZERO)
            } else {
                pricing_fetcher
                    .calculate_cost(&model, &data.message.usage)
                    .await
                    .unwrap_or_else(|_| Decimal::ZERO)
            }
        }
    };

    Ok(LoadedUsageEntry {
        timestamp,
        model,
        tokens: data.message.usage.clone(),
        cost,
        session_id: data.session_id.as_ref().map(|s| SessionId::new(s.clone())),
        request_id: data.request_id.as_ref().map(|r| RequestId::new(r.clone())),
        message_id: data.message.id.as_ref().map(|m| MessageId::new(m.clone())),
        project: Some(project),
        version: data.version.clone(),
    })
}

/// Load all usage entries from JSONL files with streaming and deduplication
pub async fn load_usage_entries(
    options: &CommonOptions,
    pricing_fetcher: &PricingFetcher,
) -> Result<Vec<LoadedUsageEntry>> {
    let files = find_jsonl_files().await?;
    let seen_hashes = Arc::new(DashSet::new());
    let mut all_entries = Vec::new();

    for file_path in files {
        let project = extract_project_name(&file_path);

        // Filter by project if specified
        if let Some(ref target_project) = options.project {
            if project != *target_project {
                continue;
            }
        }

        let mut file_entries = Vec::new();
        let fetcher = pricing_fetcher.clone();
        let mode = options.mode;

        // Stream file line by line (PR #706 fix)
        stream_jsonl_file(&file_path, |line, _line_num| {
            // Parse JSON line
            if let Ok(data) = serde_json::from_str::<UsageData>(line) {
                // Skip API error messages
                if data.is_api_error_message.unwrap_or(false) {
                    return Ok(());
                }

                // Create entry synchronously for now (can optimize later with async streaming)
                let entry_future = parse_usage_entry(&data, project.clone(), mode, &fetcher);
                if let Ok(entry) = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(entry_future)
                }) {
                    // Deduplication check
                    let hash = entry.unique_hash();
                    if !hash.is_empty() && seen_hashes.contains(&hash) {
                        return Ok(());
                    }

                    if !hash.is_empty() {
                        seen_hashes.insert(hash);
                    }

                    // Date filtering
                    let entry_date = entry.timestamp.date_naive();
                    if let Some(since) = options.since {
                        if entry_date < since {
                            return Ok(());
                        }
                    }
                    if let Some(until) = options.until {
                        if entry_date > until {
                            return Ok(());
                        }
                    }

                    file_entries.push(entry);
                }
            }

            Ok(())
        })?;

        all_entries.extend(file_entries);
    }

    // Sort by timestamp
    all_entries.sort_by_key(|e| e.timestamp);

    Ok(all_entries)
}
