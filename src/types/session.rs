use super::{LoadedUsageEntry, ModelBreakdown, ModelName, ProjectPath, SessionId, TokenCounts};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Session-based usage aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUsage {
    #[serde(rename = "sessionId")]
    pub session_id: SessionId,

    #[serde(rename = "projectPath")]
    pub project_path: ProjectPath,

    #[serde(rename = "inputTokens")]
    pub input_tokens: u64,

    #[serde(rename = "outputTokens")]
    pub output_tokens: u64,

    #[serde(rename = "cacheCreationTokens")]
    pub cache_creation_tokens: u64,

    #[serde(rename = "cacheReadTokens")]
    pub cache_read_tokens: u64,

    #[serde(rename = "totalCost")]
    pub total_cost: Decimal,

    #[serde(rename = "lastActivity")]
    pub last_activity: NaiveDate,

    pub versions: Vec<String>,

    #[serde(rename = "modelsUsed")]
    pub models_used: Vec<ModelName>,

    #[serde(rename = "modelBreakdowns")]
    pub model_breakdowns: Vec<ModelBreakdown>,
}

impl SessionUsage {
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens + self.cache_creation_tokens + self.cache_read_tokens
    }
}

/// 5-hour billing block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionBlock {
    pub id: String, // ISO timestamp of block start

    #[serde(rename = "startTime")]
    pub start_time: DateTime<Utc>,

    #[serde(rename = "endTime")]
    pub end_time: DateTime<Utc>,

    #[serde(rename = "actualEndTime", skip_serializing_if = "Option::is_none")]
    pub actual_end_time: Option<DateTime<Utc>>,

    #[serde(rename = "isActive")]
    pub is_active: bool,

    #[serde(rename = "isGap", skip_serializing_if = "Option::is_none")]
    pub is_gap: Option<bool>,

    #[serde(skip)] // Don't serialize entries to JSON
    pub entries: Vec<LoadedUsageEntry>,

    #[serde(rename = "tokenCounts")]
    pub token_counts: TokenCounts,

    #[serde(rename = "costUSD")]
    pub cost_usd: Decimal,

    pub models: Vec<String>,

    #[serde(rename = "usageLimitResetTime", skip_serializing_if = "Option::is_none")]
    pub usage_limit_reset_time: Option<DateTime<Utc>>,
}

impl SessionBlock {
    /// Calculate the total number of tokens in this block
    pub fn total_tokens(&self) -> u64 {
        self.token_counts.total()
    }

    /// Check if this block is near the token limit (80% threshold)
    pub fn is_near_limit(&self, token_limit: u64) -> bool {
        let usage_ratio = self.total_tokens() as f64 / token_limit as f64;
        usage_ratio >= 0.8
    }

    /// Calculate the projected end time based on current usage rate
    pub fn projected_end_time(&self, token_limit: u64) -> Option<DateTime<Utc>> {
        if !self.is_active || self.entries.is_empty() {
            return None;
        }

        let total_tokens = self.total_tokens();
        if total_tokens == 0 || total_tokens >= token_limit {
            return Some(Utc::now());
        }

        // Calculate usage rate
        let first_entry = self.entries.first()?;
        let last_entry = self.entries.last()?;
        let duration = last_entry.timestamp - first_entry.timestamp;

        if duration.num_seconds() <= 0 {
            return None;
        }

        let tokens_per_second = total_tokens as f64 / duration.num_seconds() as f64;
        let remaining_tokens = token_limit - total_tokens;
        let seconds_remaining = remaining_tokens as f64 / tokens_per_second;

        Some(Utc::now() + chrono::Duration::seconds(seconds_remaining as i64))
    }
}