use rmcp::{
    model::{
        CallToolRequestParams, CallToolResult, GetPromptRequestParams, GetPromptResult,
        ListPromptsResult, ListResourceTemplatesResult, ListResourcesResult, ListToolsResult,
        PaginatedRequestParams, ReadResourceRequestParams, ReadResourceResult, ServerCapabilities,
        ServerInfo,
    },
    service::RequestContext,
    ServerHandler,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::error::McpTestError;
use crate::transport::pipe::McpPipe;

/// A recorded forwarded call (for assertion in tests).
#[derive(Debug, Clone)]
pub struct ForwardedCall {
    /// The name of the tool that was called.
    pub tool_name: String,
    /// The arguments passed to the tool.
    pub arguments: serde_json::Value,
}

struct BridgeState {
    downstream: McpPipe,
    recorded: Vec<ForwardedCall>,
}

/// Builder for [`McpAgentBridge`].
pub struct McpAgentBridgeBuilder {
    downstream: Option<McpPipe>,
}

impl McpAgentBridgeBuilder {
    /// Set a pre-built [`McpPipe`] as the downstream.
    #[must_use]
    pub fn with_pipe(mut self, pipe: McpPipe) -> Self {
        self.downstream = Some(pipe);
        self
    }

    /// Build the bridge with an in-process downstream server.
    ///
    /// # Errors
    ///
    /// Returns [`McpTestError`] if the downstream pipe cannot be established.
    pub async fn with_server<S: rmcp::ServerHandler + Send + 'static>(
        mut self,
        server: S,
    ) -> Result<Self, McpTestError> {
        self.downstream = Some(McpPipe::new_in_process(server).await?);
        Ok(self)
    }

    /// Build the [`McpAgentBridge`].
    ///
    /// # Panics
    ///
    /// Panics if no downstream was set via [`with_pipe`](Self::with_pipe) or
    /// [`with_server`](Self::with_server).
    #[must_use]
    pub fn build(self) -> McpAgentBridge {
        McpAgentBridge {
            state: Arc::new(Mutex::new(BridgeState {
                downstream: self.downstream.expect("downstream must be set before build()"),
                recorded: Vec::new(),
            })),
        }
    }
}

/// An MCP server that forwards tool calls to a downstream MCP server.
///
/// Used to test multi-hop MCP chains: orchestrator → bridge → downstream.
pub struct McpAgentBridge {
    state: Arc<Mutex<BridgeState>>,
}

impl McpAgentBridge {
    /// Create a new builder.
    #[must_use]
    pub fn builder() -> McpAgentBridgeBuilder {
        McpAgentBridgeBuilder { downstream: None }
    }

    /// Return all calls forwarded to the downstream.
    pub async fn recorded_calls(&self) -> Vec<ForwardedCall> {
        self.state.lock().await.recorded.clone()
    }
}

impl ServerHandler for McpAgentBridge {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<rmcp::RoleServer>,
    ) -> Result<ListToolsResult, rmcp::ErrorData> {
        let state = self.state.lock().await;
        let tools = state.downstream.list_tools().await.map_err(|e| rmcp::ErrorData {
            code: rmcp::model::ErrorCode(-32_603),
            message: e.to_string().into(),
            data: None,
        })?;
        Ok(ListToolsResult { tools, next_cursor: None, meta: None })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _ctx: RequestContext<rmcp::RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let name = request.name.as_ref().to_string();
        let args = request
            .arguments
            .as_ref()
            .map(|m| {
                serde_json::Value::Object(m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            })
            .unwrap_or(serde_json::Value::Null);

        let mut state = self.state.lock().await;
        state
            .recorded
            .push(ForwardedCall { tool_name: name.clone(), arguments: args.clone() });

        state.downstream.call_tool(&name, args).await.map_err(|e| rmcp::ErrorData {
            code: rmcp::model::ErrorCode(-32_603),
            message: e.to_string().into(),
            data: None,
        })
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<rmcp::RoleServer>,
    ) -> Result<ListResourcesResult, rmcp::ErrorData> {
        Ok(ListResourcesResult { resources: vec![], next_cursor: None, meta: None })
    }

    async fn read_resource(
        &self,
        _request: ReadResourceRequestParams,
        _ctx: RequestContext<rmcp::RoleServer>,
    ) -> Result<ReadResourceResult, rmcp::ErrorData> {
        Err(rmcp::ErrorData::resource_not_found("bridge has no resources", None))
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<rmcp::RoleServer>,
    ) -> Result<ListResourceTemplatesResult, rmcp::ErrorData> {
        Ok(ListResourceTemplatesResult {
            resource_templates: vec![],
            next_cursor: None,
            meta: None,
        })
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<rmcp::RoleServer>,
    ) -> Result<ListPromptsResult, rmcp::ErrorData> {
        Ok(ListPromptsResult { prompts: vec![], next_cursor: None, meta: None })
    }

    async fn get_prompt(
        &self,
        _request: GetPromptRequestParams,
        _ctx: RequestContext<rmcp::RoleServer>,
    ) -> Result<GetPromptResult, rmcp::ErrorData> {
        Err(rmcp::ErrorData::resource_not_found("bridge has no prompts", None))
    }
}
