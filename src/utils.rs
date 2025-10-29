use std::env;
use std::path::PathBuf;

/// Expand tilde in path to home directory
pub fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with("~") {
        if let Ok(home) = env::var("HOME") {
            return PathBuf::from(path.replacen("~", &home, 1));
        }
    }
    PathBuf::from(path)
}

/// Check if running in CI environment
pub fn is_ci() -> bool {
    env::var("CI").is_ok() || env::var("GITHUB_ACTIONS").is_ok()
}

/// Get log level from environment
pub fn get_log_level() -> String {
    env::var("LOG_LEVEL").unwrap_or_else(|_| {
        if is_ci() {
            "info".to_string()
        } else {
            "warn".to_string()
        }
    })
}

/// Format duration in human-readable format
pub fn format_duration(seconds: i64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    }
}