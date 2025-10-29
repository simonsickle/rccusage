use anyhow::{Context, Result};
use subprocess::{Exec, Redirection};

/// Apply jq filter to JSON string
pub fn apply_jq_filter(json: &str, filter: &str) -> Result<String> {
    let output = Exec::cmd("jq")
        .arg(filter)
        .stdin(json)
        .stdout(Redirection::Pipe)
        .stderr(Redirection::Pipe)
        .capture()
        .context("Failed to execute jq command")?;

    if !output.success() {
        let stderr = output.stderr_str();
        anyhow::bail!("jq filter failed: {}", stderr);
    }

    Ok(output.stdout_str())
}

/// Check if jq is available on the system
pub fn is_jq_available() -> bool {
    Exec::cmd("jq")
        .arg("--version")
        .stdout(Redirection::None)
        .stderr(Redirection::None)
        .join()
        .map(|status| status.success())
        .unwrap_or(false)
}