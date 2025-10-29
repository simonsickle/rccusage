use super::{ModelBreakdown, ModelName, ProjectPath, SessionId, TokenCounts};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

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

    #[serde(rename = "tokenCounts")]
    pub token_counts: TokenCounts,

    #[serde(rename = "costUSD")]
    pub cost_usd: Decimal,

    pub models: Vec<String>,

    #[serde(
        rename = "usageLimitResetTime",
        skip_serializing_if = "Option::is_none"
    )]
    pub usage_limit_reset_time: Option<DateTime<Utc>>,
}

impl SessionBlock {
    /// Calculate the total number of tokens in this block
    pub fn total_tokens(&self) -> u64 {
        self.token_counts.total()
    }
}
