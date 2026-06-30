//! Snapshot assertions for MCP tool list and prompt list output.
//!
//! Uses `insta` to pin the exact schema of a server's tool list so that
//! accidental field renames or type changes are caught at review time.

use std::collections::BTreeMap;

use rmcp::model::Tool;
use serde_json::Value;

use crate::error::McpTestError;
#[cfg(feature = "snapshot-testing")]
use crate::harness::server::McpServerHarness;

/// Fetch `tools/list`, canonicalize, strip `serverInfo.version`, and compare
/// against a stored `insta` snapshot.
///
/// Call this from a test annotated with `#[cfg(feature = "snapshot-testing")]`.
///
/// The snapshot file is written to `tests/snapshots/` on first run and must be
/// committed. Update snapshots by running with `INSTA_UPDATE=always`.
///
/// # Errors
///
/// Returns [`McpTestError`] if the tool list cannot be fetched.
#[cfg(feature = "snapshot-testing")]
pub async fn assert_tool_list_snapshot(
    harness: &McpServerHarness,
    snapshot_name: &str,
) -> Result<(), McpTestError> {
    let tools = harness.tools_list().await?;
    let canonical = canonicalize_tools(&tools)?;
    insta::assert_json_snapshot!(snapshot_name, canonical);
    Ok(())
}

/// Canonicalize a list of tools into a stable, sorted JSON representation.
///
/// - Tools are sorted by name.
/// - All JSON objects have their keys sorted (via `BTreeMap`).
/// - `serverInfo.version` fields are omitted (they change every release).
///
/// # Errors
///
/// Returns [`McpTestError::ParseError`] if a tool cannot be serialized.
pub fn canonicalize_tools(tools: &[Tool]) -> Result<Value, McpTestError> {
    let mut sorted: Vec<&Tool> = tools.iter().collect();
    sorted.sort_by(|a, b| a.name.cmp(&b.name));

    let values: Vec<Value> = sorted
        .into_iter()
        .map(|t| {
            let raw = serde_json::to_value(t)?;
            Ok(sort_keys(raw))
        })
        .collect::<Result<_, serde_json::Error>>()?;

    Ok(Value::Array(values))
}

/// Recursively sort all object keys in a JSON value.
fn sort_keys(v: Value) -> Value {
    match v {
        Value::Object(map) => {
            let sorted: BTreeMap<String, Value> =
                map.into_iter().map(|(k, v)| (k, sort_keys(v))).collect();
            Value::Object(sorted.into_iter().collect())
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(sort_keys).collect()),
        other => other,
    }
}
