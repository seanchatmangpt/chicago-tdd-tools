//! In-process MCP transport backed by `tokio::io::duplex`.
//!
//! `McpPipe` lets you test an `rmcp` server without spawning a subprocess:
//! the server and client live in the same process, connected by a memory pipe.
//!
//! # Example
//!
//! ```ignore
//! use chicago_tdd_mcp::transport::pipe::McpPipe;
//!
//! #[tokio::test]
//! async fn echo_tool_round_trip() {
//!     let mut pipe = McpPipe::new_in_process(EchoServer::new()).await.unwrap();
//!     let result = pipe.call_tool("echo", serde_json::json!({"message": "hi"})).await.unwrap();
//!     assert!(!result.content.is_empty());
//! }
//! ```

use std::time::Duration;

use rmcp::{
    model::{CallToolResult, Tool},
    serve_client,
    service::RunningService,
    RoleClient, ServerHandler, ServiceExt,
};
use tokio::io::duplex;

use crate::error::McpTestError;

const PIPE_BUF: usize = 65_536;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// An in-process MCP client connected to a server via a memory pipe.
pub struct McpPipe {
    client: RunningService<RoleClient, ()>,
    timeout: Duration,
}

impl McpPipe {
    /// Spawn `server` in a background task and return a connected client.
    ///
    /// # Errors
    ///
    /// Returns [`McpTestError::Rmcp`] if the rmcp initialize handshake fails.
    pub async fn new_in_process<S>(server: S) -> Result<Self, McpTestError>
    where
        S: ServerHandler + Send + 'static,
    {
        let (server_io, client_io) = duplex(PIPE_BUF);

        // `serve()` returns a `RunningService` that drives the server loop.
        // We must keep it alive by awaiting `.waiting()` — dropping it cancels the task.
        tokio::spawn(async move {
            match server.serve(server_io).await {
                Ok(running) => {
                    if let Err(e) = running.waiting().await {
                        eprintln!("[chicago-tdd-mcp] pipe server task error: {e}");
                    }
                }
                Err(e) => eprintln!("[chicago-tdd-mcp] pipe server init error: {e}"),
            }
        });

        let client = serve_client((), client_io)
            .await
            .map_err(|e| McpTestError::Rmcp(e.to_string()))?;

        Ok(Self { client, timeout: DEFAULT_TIMEOUT })
    }

    /// Override the per-operation timeout (default: 5 s).
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// List the tools exposed by the server.
    ///
    /// # Errors
    ///
    /// Returns [`McpTestError::Timeout`] or [`McpTestError::Rmcp`] on failure.
    pub async fn list_tools(&self) -> Result<Vec<Tool>, McpTestError> {
        let result = tokio::time::timeout(self.timeout, self.client.list_tools(Default::default()))
            .await
            .map_err(|_| McpTestError::Timeout(self.timeout))?
            .map_err(|e| McpTestError::Rmcp(e.to_string()))?;
        Ok(result.tools.into_iter().collect())
    }

    /// Call a tool by name with the given JSON arguments.
    ///
    /// # Errors
    ///
    /// Returns [`McpTestError::Timeout`] or [`McpTestError::Rmcp`] on failure.
    pub async fn call_tool(
        &self,
        name: &str,
        args: serde_json::Value,
    ) -> Result<CallToolResult, McpTestError> {
        use rmcp::model::CallToolRequestParams;
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
}
