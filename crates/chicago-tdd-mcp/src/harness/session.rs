//! `McpSession<S>` — phantom-type lifecycle state machine.
//!
//! Illegal MCP message orderings (e.g. calling `tools_list` before `initialize`)
//! are compile errors, not runtime panics.
//!
//! # States
//!
//! ```text
//! Uninitialized ──initialize()──▶ Ready ──shutdown()──▶ Closed
//! ```

use std::marker::PhantomData;

use rmcp::model::{CallToolResult, Prompt, Resource, Tool};

use crate::error::McpTestError;
use crate::harness::server::McpServerHarness;
use crate::transport::pipe::McpPipe;

/// Marker: the session has not yet performed the MCP `initialize` handshake.
pub struct Uninitialized;
/// Marker: the session is initialized and ready for tool calls.
pub struct Ready;
/// Marker: the session has been shut down.
pub struct Closed;

/// The backing connection, either an out-of-process harness or an in-process pipe.
enum SessionInner {
    Harness(McpServerHarness),
    Pipe(McpPipe),
}

impl SessionInner {
    async fn tools_list(&self) -> Result<Vec<Tool>, McpTestError> {
        match self {
            Self::Harness(h) => h.tools_list().await,
            Self::Pipe(p) => p.list_tools().await,
        }
    }

    async fn call_tool(
        &self,
        name: &str,
        args: serde_json::Value,
    ) -> Result<CallToolResult, McpTestError> {
        match self {
            Self::Harness(h) => h.call_tool(name, args).await,
            Self::Pipe(p) => p.call_tool(name, args).await,
        }
    }

    async fn prompts_list(&self) -> Result<Vec<Prompt>, McpTestError> {
        match self {
            Self::Harness(h) => h.prompts_list().await,
            // Prompts are rare in test scenarios; return an empty list for pipe sessions.
            Self::Pipe(_) => Ok(Vec::new()),
        }
    }

    async fn resources_list(&self) -> Result<Vec<Resource>, McpTestError> {
        match self {
            Self::Harness(h) => h.resources_list().await,
            // Resources are rare in test scenarios; return an empty list for pipe sessions.
            Self::Pipe(_) => Ok(Vec::new()),
        }
    }

    async fn shutdown(self) {
        match self {
            Self::Harness(h) => h.shutdown().await,
            // Dropping the pipe client signals the server task to stop.
            Self::Pipe(_) => {}
        }
    }
}

/// A typed MCP session. `None` only in the `Closed` state.
pub struct McpSession<S> {
    inner: Option<SessionInner>,
    _state: PhantomData<S>,
}

impl McpSession<Uninitialized> {
    /// Wrap an out-of-process server harness in an uninitialized session.
    #[must_use]
    pub fn new(harness: McpServerHarness) -> Self {
        Self { inner: Some(SessionInner::Harness(harness)), _state: PhantomData }
    }

    /// Create a session wrapping an in-process server (no subprocess needed).
    ///
    /// # Errors
    ///
    /// Returns [`McpTestError`] if the pipe cannot be established.
    pub async fn new_in_process<S>(server: S) -> Result<McpSession<Uninitialized>, McpTestError>
    where
        S: rmcp::ServerHandler + Send + 'static,
    {
        let pipe = McpPipe::new_in_process(server).await?;
        Ok(Self { inner: Some(SessionInner::Pipe(pipe)), _state: PhantomData })
    }

    /// Probe the server to confirm it is alive, then transition to [`Ready`].
    ///
    /// # Errors
    ///
    /// Returns [`McpTestError`] if the server is unreachable or the probe fails.
    pub async fn initialize(self) -> Result<McpSession<Ready>, McpTestError> {
        // SAFETY: always Some in Uninitialized state.
        let inner = self.inner.expect("inner is always Some in Uninitialized");
        // Probe with a tools/list to confirm the server is alive.
        inner.tools_list().await?;
        Ok(McpSession { inner: Some(inner), _state: PhantomData })
    }
}

impl McpSession<Ready> {
    fn inner(&self) -> &SessionInner {
        // SAFETY: always Some in Ready state.
        self.inner.as_ref().expect("inner is always Some in Ready")
    }

    /// List the tools the server exposes.
    ///
    /// # Errors
    ///
    /// Returns [`McpTestError`] on transport failure.
    pub async fn tools_list(&self) -> Result<Vec<Tool>, McpTestError> {
        self.inner().tools_list().await
    }

    /// Call a tool by name with JSON arguments.
    ///
    /// # Errors
    ///
    /// Returns [`McpTestError`] on transport or server error.
    pub async fn call_tool(
        &self,
        name: &str,
        args: serde_json::Value,
    ) -> Result<CallToolResult, McpTestError> {
        self.inner().call_tool(name, args).await
    }

    /// List the prompts the server exposes.
    ///
    /// For in-process pipe sessions, an empty `Vec` is returned (prompts are rare
    /// in test scenarios and `McpPipe` does not yet expose this endpoint).
    ///
    /// # Errors
    ///
    /// Returns [`McpTestError`] on transport failure.
    pub async fn prompts_list(&self) -> Result<Vec<Prompt>, McpTestError> {
        self.inner().prompts_list().await
    }

    /// List the resources the server exposes.
    ///
    /// For in-process pipe sessions, an empty `Vec` is returned (resources are rare
    /// in test scenarios and `McpPipe` does not yet expose this endpoint).
    ///
    /// # Errors
    ///
    /// Returns [`McpTestError`] on transport failure.
    pub async fn resources_list(&self) -> Result<Vec<Resource>, McpTestError> {
        self.inner().resources_list().await
    }

    /// Shut down the server and return a [`Closed`] session as proof.
    pub async fn shutdown(mut self) -> McpSession<Closed> {
        if let Some(inner) = self.inner.take() {
            inner.shutdown().await;
        }
        McpSession { inner: None, _state: PhantomData }
    }
}

impl McpSession<Closed> {
    /// Always returns `true`; exists as a compile-time proof that shutdown ran.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        true
    }
}
