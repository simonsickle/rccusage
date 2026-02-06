use log::LevelFilter;
use std::env;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

/// Initialize logging based on VERBOSE environment variable
/// Silent by default. Set VERBOSE=true for info-level logs,
/// or use LOG_LEVEL=0..4 for fine-grained control.
pub fn init_logger() {
    let log_level = if let Some(level) = env::var("LOG_LEVEL")
        .ok()
        .and_then(|s| s.parse::<u8>().ok())
    {
        level
    } else if env::var("VERBOSE").is_ok_and(|v| v == "true" || v == "1") {
        2 // info
    } else {
        0 // silent by default
    };

    let level_filter = match log_level {
        0 => LevelFilter::Off,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        4.. => LevelFilter::Trace,
    };

    // Set up env_logger for the log crate
    env_logger::Builder::from_default_env()
        .filter_level(level_filter)
        .init();

    // Also set up tracing subscriber for tracing crate
    let filter = match log_level {
        0 => EnvFilter::new("off"),
        1 => EnvFilter::new("warn"),
        2 => EnvFilter::new("info"),
        3 => EnvFilter::new("debug"),
        4.. => EnvFilter::new("trace"),
    };

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default tracing subscriber failed");
}

/// Log macros that respect LOG_LEVEL
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        log::info!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        log::warn!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        log::debug!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        log::error!($($arg)*);
    };
}
