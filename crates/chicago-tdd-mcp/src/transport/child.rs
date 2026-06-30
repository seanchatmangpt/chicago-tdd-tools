//! Child-process transport helpers.
//!
//! Thin wrappers around `rmcp`'s `TokioChildProcess` used by `McpServerHarness`.

use std::time::Duration;

use rmcp::transport::child_process::TokioChildProcess;
use rmcp::{serve_client, service::RunningService, RoleClient};

use crate::error::McpTestError;

/// Spawn `cmd` as a child process and return a connected rmcp client service.
///
/// The child's stdin and stdout are piped; the rmcp protocol handshake is NOT
/// performed here — callers must call `initialize()` on the returned service.
///
/// # Errors
///
/// Returns [`McpTestError::Io`] if the process cannot be spawned,
/// [`McpTestError::Rmcp`] if the rmcp layer fails, or
/// [`McpTestError::Timeout`] if `spawn_timeout` elapses before the service is ready.
pub async fn connect_child(
    cmd: tokio::process::Command,
    spawn_timeout: Duration,
) -> Result<RunningService<RoleClient, ()>, McpTestError> {
    let transport = TokioChildProcess::new(cmd).map_err(McpTestError::Io)?;
    let client = tokio::time::timeout(spawn_timeout, serve_client((), transport))
        .await
        .map_err(|_| McpTestError::Timeout(spawn_timeout))?
        .map_err(|e| McpTestError::Rmcp(e.to_string()))?;
    Ok(client)
}
