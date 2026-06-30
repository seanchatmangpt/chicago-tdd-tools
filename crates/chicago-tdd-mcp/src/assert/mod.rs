//! Assertion helpers for MCP tool call responses.

pub mod error_scenarios;
pub mod schema;

use rmcp::model::{CallToolResult, ContentBlock};
use serde_json::Value;

use crate::error::McpTestError;

/// Assert that a `CallToolResult` contains a well-formed content array.
///
/// Checks:
/// - `content` is non-empty
/// - every `Text` item has a non-empty `text` field
/// - every `Image` item has non-empty `data` and `mimeType`
/// - `isError` is `false` or absent (use [`assert_tool_error`] for error responses)
///
/// # Panics
///
/// Panics with a descriptive message if any check fails.
pub fn assert_content_array(result: &CallToolResult) {
    assert!(!result.content.is_empty(), "expected non-empty content array, got empty");

    if result.is_error == Some(true) {
        panic!(
            "expected successful tool result (isError=false), but isError=true; content: {:?}",
            result.content
        );
    }

    for (i, item) in result.content.iter().enumerate() {
        match item {
            ContentBlock::Text(t) => {
                assert!(!t.text.is_empty(), "content[{i}]: Text item has empty text field");
            }
            ContentBlock::Image(img) => {
                assert!(!img.data.is_empty(), "content[{i}]: Image item has empty data field");
                assert!(
                    !img.mime_type.is_empty(),
                    "content[{i}]: Image item has empty mimeType field"
                );
            }
            // Resource and Audio items are allowed through; structure checked by rmcp types.
            _ => {}
        }
    }
}

/// Assert that a `CallToolResult` is an error response (`isError = true`).
///
/// # Panics
///
/// Panics if `isError` is not `true`.
pub fn assert_tool_error(result: &CallToolResult) {
    assert_eq!(result.is_error, Some(true), "expected isError=true, got {:?}", result.is_error);
}

/// Helpers for asserting on MCP JSON-RPC error responses.
pub struct McpErrorAssert;

impl McpErrorAssert {
    /// Assert that `response` is a JSON-RPC error with code `code`.
    ///
    /// `response` should be the raw JSON value of the `error` field in a JSON-RPC response.
    ///
    /// # Panics
    ///
    /// Panics with the full response JSON if the code does not match.
    pub fn expect_code(response: &Value, code: i32) {
        let actual = response
            .get("error")
            .and_then(|e| e.get("code"))
            .and_then(Value::as_i64)
            .map(|c| c as i32);
        assert_eq!(
            actual,
            Some(code),
            "expected JSON-RPC error code {code}, got {actual:?}\nfull response: {}",
            serde_json::to_string_pretty(response).unwrap_or_default()
        );
    }

    /// Assert that the `error.message` field contains `substring` (case-insensitive).
    ///
    /// # Panics
    ///
    /// Panics if the message is absent or does not contain `substring`.
    pub fn expect_message_contains(response: &Value, substring: &str) {
        let msg = response
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(Value::as_str)
            .unwrap_or("");
        assert!(
            msg.to_lowercase().contains(&substring.to_lowercase()),
            "expected error.message to contain {substring:?}, got {msg:?}"
        );
    }
}

/// Validate `args` against `tool`'s `inputSchema` using JSON Schema validation.
///
/// Requires the `testing` feature (pulls in the `jsonschema` crate).
///
/// # Errors
///
/// Returns [`McpTestError::SchemaViolation`] if validation fails.
#[cfg(feature = "testing")]
pub fn assert_tool_input_valid(tool: &rmcp::model::Tool, args: &Value) -> Result<(), McpTestError> {
    let schema_value =
        serde_json::to_value(tool.input_schema.as_ref()).map_err(McpTestError::ParseError)?;

    let compiled = jsonschema::validator_for(&schema_value)
        .map_err(|e| McpTestError::SchemaViolation(format!("invalid inputSchema: {e}")))?;

    compiled
        .validate(args)
        .map_err(|e| McpTestError::SchemaViolation(e.to_string()))
}
