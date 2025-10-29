pub mod table;

use anyhow::{Context, Result};
use serde::Serialize;
use serde_json;
use std::process::{Command, Stdio};

/// Output data as JSON, optionally filtered through jq
pub fn output_json<T: Serialize>(data: &T, jq_expression: Option<&str>) -> Result<()> {
    let json = serde_json::to_string_pretty(data)?;

    if let Some(expr) = jq_expression {
        // Pass through jq for filtering
        let mut jq = Command::new("jq")
            .arg(expr)
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .spawn()
            .context("Failed to spawn jq process")?;

        if let Some(mut stdin) = jq.stdin.take() {
            use std::io::Write;
            stdin.write_all(json.as_bytes())?;
        }

        jq.wait().context("Failed to wait for jq")?;
    } else {
        // Direct output
        println!("{}", json);
    }

    Ok(())
}