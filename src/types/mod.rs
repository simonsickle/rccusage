use chrono::{DateTime, Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod session;
pub mod tokens;
pub mod usage;

pub use session::*;
pub use tokens::*;
pub use usage::*;

/// Branded type for model names
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModelName(pub String);

impl ModelName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ModelName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Branded type for session IDs
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub String);

impl SessionId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

/// Branded type for request IDs
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RequestId(pub String);

impl RequestId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

/// Branded type for message IDs
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(pub String);

impl MessageId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

/// Branded type for project paths
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectPath(pub PathBuf);

impl ProjectPath {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self(path.into())
    }
}

impl std::fmt::Display for ProjectPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

/// Date type for daily aggregation (YYYY-MM-DD)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct DailyDate(pub NaiveDate);

impl DailyDate {
    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        Self(dt.date_naive())
    }
}

impl std::fmt::Display for DailyDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.format("%Y-%m-%d"))
    }
}

/// Date type for monthly aggregation (YYYY-MM)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct MonthlyDate {
    pub year: i32,
    pub month: u32,
}

impl MonthlyDate {
    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        Self {
            year: dt.year(),
            month: dt.month(),
        }
    }
}

impl std::fmt::Display for MonthlyDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}", self.year, self.month)
    }
}

/// Date type for weekly aggregation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct WeeklyDate(pub NaiveDate);

impl WeeklyDate {
    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        // Get the ISO week start (Monday)
        let weekday = dt.weekday().num_days_from_monday();
        let week_start = dt.date_naive() - chrono::Duration::days(weekday as i64);
        Self(week_start)
    }
}

impl std::fmt::Display for WeeklyDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.format("%Y-%m-%d"))
    }
}

/// Cost calculation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
pub enum CostMode {
    /// Use pre-calculated costUSD when available, otherwise calculate from tokens
    Auto,
    /// Always calculate costs from token counts using model pricing
    Calculate,
    /// Always use pre-calculated costUSD values, show 0 for missing costs
    Display,
}

impl Default for CostMode {
    fn default() -> Self {
        Self::Auto
    }
}

/// Sort order for results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl Default for SortOrder {
    fn default() -> Self {
        Self::Asc
    }
}

/// Common options for all commands
#[derive(Debug, Clone)]
pub struct CommonOptions {
    pub json: bool,
    pub mode: CostMode,
    pub since: Option<NaiveDate>,
    pub until: Option<NaiveDate>,
    pub order: SortOrder,
    pub offline: bool,
    pub project: Option<String>,
    pub jq: Option<String>,
    pub compact: bool,
}
