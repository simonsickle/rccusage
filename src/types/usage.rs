use super::{
    DailyDate, MessageId, ModelName, MonthlyDate, ProjectPath,
    RequestId, SessionId, TokenCounts, WeeklyDate,
};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Raw JSONL entry structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,

    #[serde(rename = "sessionId", skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,

    pub timestamp: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    pub message: MessageData,

    #[serde(rename = "costUSD", skip_serializing_if = "Option::is_none")]
    pub cost_usd: Option<f64>,

    #[serde(rename = "requestId", skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,

    #[serde(rename = "isApiErrorMessage", skip_serializing_if = "Option::is_none")]
    pub is_api_error_message: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageData {
    pub usage: TokenCounts,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<ContentItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// Loaded and processed usage entry
#[derive(Debug, Clone)]
pub struct LoadedUsageEntry {
    pub timestamp: DateTime<Utc>,
    pub model: ModelName,
    pub tokens: TokenCounts,
    pub cost: Decimal,
    pub session_id: Option<SessionId>,
    pub request_id: Option<RequestId>,
    pub message_id: Option<MessageId>,
    pub project: Option<String>,
    pub version: Option<String>,
}

impl LoadedUsageEntry {
    /// Create a unique hash for deduplication
    pub fn unique_hash(&self) -> String {
        match (&self.message_id, &self.request_id) {
            (Some(msg_id), Some(req_id)) => format!("{}:{}", msg_id.0, req_id.0),
            (Some(msg_id), None) => format!("{}:", msg_id.0),
            (None, Some(req_id)) => format!(":{}", req_id.0),
            (None, None) => String::new(),
        }
    }
}

/// Model breakdown for aggregated data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelBreakdown {
    #[serde(rename = "modelName")]
    pub model_name: ModelName,

    #[serde(rename = "inputTokens")]
    pub input_tokens: u64,

    #[serde(rename = "outputTokens")]
    pub output_tokens: u64,

    #[serde(rename = "cacheCreationTokens")]
    pub cache_creation_tokens: u64,

    #[serde(rename = "cacheReadTokens")]
    pub cache_read_tokens: u64,

    pub cost: Decimal,
}

impl ModelBreakdown {
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens + self.cache_creation_tokens + self.cache_read_tokens
    }
}

/// Daily usage aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyUsage {
    pub date: DailyDate,

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

    #[serde(rename = "modelsUsed")]
    pub models_used: Vec<ModelName>,

    #[serde(rename = "modelBreakdowns")]
    pub model_breakdowns: Vec<ModelBreakdown>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

impl DailyUsage {
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens + self.cache_creation_tokens + self.cache_read_tokens
    }
}

/// Monthly usage aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyUsage {
    pub date: MonthlyDate,

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

    #[serde(rename = "modelsUsed")]
    pub models_used: Vec<ModelName>,

    #[serde(rename = "modelBreakdowns")]
    pub model_breakdowns: Vec<ModelBreakdown>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

impl MonthlyUsage {
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens + self.cache_creation_tokens + self.cache_read_tokens
    }
}

/// Weekly usage aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyUsage {
    pub date: WeeklyDate,

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

    #[serde(rename = "modelsUsed")]
    pub models_used: Vec<ModelName>,

    #[serde(rename = "modelBreakdowns")]
    pub model_breakdowns: Vec<ModelBreakdown>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

impl WeeklyUsage {
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens + self.cache_creation_tokens + self.cache_read_tokens
    }
}