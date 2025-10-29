use crate::types::{CostMode, SortOrder};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Configuration file structure for ccusage.config.json
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// Default cost calculation mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<CostMode>,

    /// Default sort order
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<SortOrder>,

    /// Default timezone
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,

    /// Default offline mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offline: Option<bool>,

    /// Default project filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,

    /// Custom Claude data directories
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claude_dirs: Option<Vec<String>>,

    /// Default output format (json or table)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,

    /// Log level (0-4)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_level: Option<u8>,
}

impl Config {
    /// Load config from default locations
    pub fn load() -> Result<Self> {
        // Check for config file in these locations (in order):
        // 1. ./ccusage.config.json (current directory)
        // 2. ~/.config/ccusage/config.json
        // 3. ~/.ccusage/config.json

        let config_paths = vec![
            PathBuf::from("./ccusage.config.json"),
            dirs::config_dir()
                .map(|d| d.join("ccusage").join("config.json"))
                .unwrap_or_default(),
            dirs::home_dir()
                .map(|d| d.join(".ccusage").join("config.json"))
                .unwrap_or_default(),
        ];

        for path in config_paths {
            if path.exists() {
                return Self::load_from_file(&path);
            }
        }

        // No config file found, return default
        Ok(Self::default())
    }

    /// Load config from specific file
    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Config = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        // Apply config-based environment variables if set
        if let Some(log_level) = config.log_level {
            std::env::set_var("LOG_LEVEL", log_level.to_string());
        }

        if let Some(ref tz) = config.timezone {
            std::env::set_var("TZ", tz);
        }

        if let Some(ref dirs) = config.claude_dirs {
            std::env::set_var("CLAUDE_CONFIG_DIR", dirs.join(","));
        }

        Ok(config)
    }
}