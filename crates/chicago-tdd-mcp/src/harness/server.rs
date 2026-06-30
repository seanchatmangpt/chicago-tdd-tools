//! `McpServerHarness` — spawn an MCP server binary and drive it with typed requests.

use std::time::Duration;

use rmcp::{
    model::{CallToolRequestParams, CallToolResult, Prompt, Resource, Tool},
    service::RunningService,
    RoleClient,
};
use tokio::process::Command;

use crate::error::McpTestError;
use crate::transport::child::connect_child;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_SPAWN_TIMEOUT: Duration = Duration::from_secs(10);

/// Builder for [`McpServerHarness`].
pub struct McpServerHarnessBuilder {
    cmd: Command,
    timeout: Duration,
    spawn_timeout: Duration,
}

impl McpServerHarnessBuilder {
    /// Create a builder from a `tokio::process::Command` pointing at the MCP server binary.
    #[must_use]
    pub fn new(cmd: Command) -> Self {
        Self { cmd, timeout: DEFAULT_TIMEOUT, spawn_timeout: DEFAULT_SPAWN_TIMEOUT }
    }

    /// Override the per-request timeout.
    #[must_use]
    pub fn timeout(mut self, d: Duration) -> Self {
        self.timeout = d;
        self
    }

    /// Override the timeout applied while spawning and connecting to the child process.
    #[must_use]
    pub fn spawn_timeout(mut self, d: Duration) -> Self {
        self.spawn_timeout = d;
        self
    }

    /// Spawn the server and perform the MCP `initialize` handshake.
    ///
    /// # Errors
    ///
    /// Returns [`McpTestError`] if the process fails to spawn or the handshake times out.
    pub async fn spawn(self) -> Result<McpServerHarness, McpTestError> {
        let client = connect_child(self.cmd, self.spawn_timeout).await?;
        Ok(McpServerHarness { client, timeout: self.timeout })
    }
}

/// A running MCP server connection.
///
/// Obtained via [`McpServerHarnessBuilder::spawn`].
pub struct McpServerHarness {
    pub(crate) client: RunningService<RoleClient, ()>,
    timeout: Duration,
}

impl McpServerHarness {
    /// List the tools the server exposes.
    ///
    /// # Errors
    ///
    /// Returns [`McpTestError::Timeout`] or [`McpTestError::Rmcp`] on failure.
    pub async fn tools_list(&self) -> Result<Vec<Tool>, McpTestError> {
        let result = tokio::time::timeout(self.timeout, self.client.list_tools(Default::default()))
            .await
            .map_err(|_| McpTestError::Timeout(self.timeout))?
            .map_err(|e| McpTestError::Rmcp(e.to_string()))?;
        Ok(result.tools.into_iter().collect())
    }

    /// List the prompts the server exposes.
    ///
    /// # Errors
    ///
    /// Returns [`McpTestError::Timeout`] or [`McpTestError::Rmcp`] on failure.
    pub async fn prompts_list(&self) -> Result<Vec<Prompt>, McpTestError> {
        let result = tokio::time::timeout(self.timeout, self.client.list_all_prompts())
            .await
            .map_err(|_| McpTestError::Timeout(self.timeout))?
            .map_err(|e| McpTestError::Rmcp(e.to_string()))?;
        Ok(result)
    }

    /// List the resources the server exposes.
    ///
    /// # Errors
    ///
    /// Returns [`McpTestError::Timeout`] or [`McpTestError::Rmcp`] on failure.
    pub async fn resources_list(&self) -> Result<Vec<Resource>, McpTestError> {
        let result = tokio::time::timeout(self.timeout, self.client.list_all_resources())
            .await
            .map_err(|_| McpTestError::Timeout(self.timeout))?
            .map_err(|e| McpTestError::Rmcp(e.to_string()))?;
        Ok(result)
    }

    /// Call a named tool with JSON arguments.
    ///
    /// # Errors
    ///
    /// Returns [`McpTestError::Timeout`] or [`McpTestError::Rmcp`] on failure.
    pub async fn call_tool(
        &self,
        name: &str,
        args: serde_json::Value,
    ) -> Result<CallToolResult, McpTestError> {
        let mut params = CallToolRequestParams::new(name.to_string());
        if let Some(obj) = args.as_object() {
            params = params.with_arguments(obj.clone().into_iter().collect());
        }
        let result = tokio::time::timeout(self.timeout, self.client.call_tool(params))
            .await
            .map_err(|_| McpTestError::Timeout(self.timeout))?
            .map_err(|e| McpTestError::Rmcp(e.to_string()))?;
        Ok(result)
    }

    /// Gracefully shut down the server.
    ///
    /// Cancels the rmcp service (which closes stdin, triggering the server to exit).
    pub async fn shutdown(self) {
        let _ = self.client.cancel().await;
    }
}
