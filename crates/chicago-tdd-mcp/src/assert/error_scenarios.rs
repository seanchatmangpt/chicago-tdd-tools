//! Pre-built error injection scenarios for MCP servers.
//!
//! Each function exercises a specific JSON-RPC or MCP error path and returns
//! the raw JSON response for assertion with [`super::McpErrorAssert`].

use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::time::timeout;

/// JSON-RPC error codes covered by this module.
pub mod codes {
    /// Parse error — malformed JSON.
    pub const PARSE_ERROR: i32 = -32700;
    /// Invalid request — missing `jsonrpc` field.
    pub const INVALID_REQUEST: i32 = -32600;
    /// Method not found.
    pub const METHOD_NOT_FOUND: i32 = -32601;
    /// Invalid params.
    pub const INVALID_PARAMS: i32 = -32602;
}

/// Send a non-JSON line to the server and return the raw response.
///
/// The server should respond with a `-32700` parse error.
///
/// # Errors
///
/// Returns an `anyhow::Error` if spawning or I/O fails.
pub async fn send_malformed_json(mut cmd: Command) -> anyhow::Result<Value> {
    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null());

    let mut child = cmd.spawn()?;
    let mut stdin = child.stdin.take().expect("piped stdin");
    let stdout = child.stdout.take().expect("piped stdout");

    stdin.write_all(b"not valid json\n").await?;
    stdin.flush().await?;

    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    reader.read_line(&mut line).await?;

    let _ = child.kill().await;
    Ok(serde_json::from_str(line.trim())?)
}

/// Send a valid JSON object that is missing the `jsonrpc` field.
///
/// The server should respond with a `-32600` invalid request error.
///
/// # Errors
///
/// Returns an error if spawning or I/O fails.
pub async fn send_invalid_request(mut cmd: Command) -> anyhow::Result<Value> {
    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null());

    let mut child = cmd.spawn()?;
    let mut stdin = child.stdin.take().expect("piped stdin");
    let stdout = child.stdout.take().expect("piped stdout");

    let bad_req = json!({"id": 1, "method": "tools/list"});
    stdin.write_all(format!("{bad_req}\n").as_bytes()).await?;
    stdin.flush().await?;

    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    reader.read_line(&mut line).await?;

    let _ = child.kill().await;
    Ok(serde_json::from_str(line.trim())?)
}

/// Call an unknown method and return the raw response.
///
/// The server should respond with a `-32601` method-not-found error.
///
/// # Errors
///
/// Returns an error if the harness fails.
pub async fn call_unknown_method(
    harness: &crate::harness::server::McpServerHarness,
) -> anyhow::Result<Value> {
    // tools_list succeeds; we probe a non-existent tool instead.
    let result = harness.call_tool("__nonexistent_tool__", serde_json::Value::Null).await;

    match result {
        Ok(r) => Ok(serde_json::to_value(r)?),
        Err(crate::error::McpTestError::Rmcp(msg)) => {
            // rmcp surfaces server errors as Rmcp variants; wrap in a
            // synthetic JSON-RPC error envelope for McpErrorAssert.
            Ok(json!({
                "error": {
                    "code": codes::METHOD_NOT_FOUND,
                    "message": msg
                }
            }))
        }
        Err(e) => Err(anyhow::anyhow!("unexpected error: {e}")),
    }
}

/// Send a `tools/call` with wrong argument types for `tool_name`.
///
/// # Errors
///
/// Returns an error if the harness fails unexpectedly.
pub async fn call_tool_invalid_params(
    harness: &crate::harness::server::McpServerHarness,
    tool_name: &str,
) -> anyhow::Result<Value> {
    // Pass an integer where a string would be expected — deliberately wrong.
    let bad_args = json!({"__wrong_type__": 12345});
    let result = harness.call_tool(tool_name, bad_args).await;

    match result {
        Ok(r) if r.is_error == Some(true) => Ok(serde_json::to_value(r)?),
        Ok(r) => Ok(json!({
            "error": { "code": codes::INVALID_PARAMS, "message": "server accepted invalid params", "result": r }
        })),
        Err(crate::error::McpTestError::Rmcp(msg)) => Ok(json!({
            "error": { "code": codes::INVALID_PARAMS, "message": msg }
        })),
        Err(e) => Err(anyhow::anyhow!("unexpected error: {e}")),
    }
}

/// Send a tools/call request to a fresh server WITHOUT first performing the MCP initialize handshake.
///
/// A conformant MCP server should either:
/// - Return a JSON-RPC error (any code)
/// - Close the connection immediately
///
/// Returns `Some(Value)` with the JSON-RPC response if the server replied,
/// or `None` if the server closed the connection without responding.
///
/// # Errors
///
/// Returns an error only if spawning the process or I/O fails unexpectedly.
pub async fn call_before_initialize(mut cmd: Command) -> anyhow::Result<Option<Value>> {
    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null());

    let mut child = cmd.spawn()?;
    let mut stdin = child.stdin.take().expect("piped stdin");
    let stdout = child.stdout.take().expect("piped stdout");

    // Send tools/call without any initialize first
    let call_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {"name": "echo", "arguments": {"message": "test"}}
    });
    stdin.write_all(format!("{call_req}\n").as_bytes()).await?;
    stdin.flush().await?;
    drop(stdin);

    let mut reader = BufReader::new(stdout);
    let mut line = String::new();

    // Use a short timeout — server may close the pipe immediately
    match timeout(std::time::Duration::from_secs(2), reader.read_line(&mut line)).await {
        Ok(Ok(0)) | Err(_) => {
            // Server closed pipe (0 bytes) or timed out — connection refused
            let _ = child.kill().await;
            return Ok(None);
        }
        Ok(Ok(_)) => {}
        Ok(Err(e)) => return Err(e.into()),
    }

    let _ = child.kill().await;
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    Ok(Some(serde_json::from_str(trimmed)?))
}
