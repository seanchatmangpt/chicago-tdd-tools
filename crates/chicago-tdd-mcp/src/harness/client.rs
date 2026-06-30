//! `McpStubServer` — programmable stub for testing MCP client code.
//!
//! The stub implements `rmcp::ServerHandler` and records every request it receives.
//! Pre-program responses with the builder, then assert on what the client sent.

use std::sync::Arc;

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
use tokio::sync::Mutex;

/// What the stub does when it receives a request it has no canned response for.
#[derive(Debug, Clone, Copy)]
pub enum UnmatchedPolicy {
    /// Panic immediately — the test is wrong.
    FailFast,
    /// Return a JSON-RPC error with the given code.
    ReturnError(i32),
}

impl Default for UnmatchedPolicy {
    fn default() -> Self {
        Self::FailFast
    }
}

/// A recorded request received by the stub.
#[derive(Debug, Clone)]
pub struct RecordedRequest {
    /// The tool name (for `tools/call` requests).
    pub tool_name: Option<String>,
    /// The raw arguments.
    pub arguments: serde_json::Value,
}

/// Builder for [`McpStubServer`].
#[derive(Default)]
pub struct StubBuilder {
    tools_response: Option<Vec<rmcp::model::Tool>>,
    tool_calls: std::collections::HashMap<String, CallToolResult>,
    policy: UnmatchedPolicy,
}

impl StubBuilder {
    /// Set the response for `tools/list`.
    #[must_use]
    pub fn on_tools_list(mut self, tools: Vec<rmcp::model::Tool>) -> Self {
        self.tools_response = Some(tools);
        self
    }

    /// Set a canned response for a specific `tools/call` invocation.
    #[must_use]
    pub fn on_tool_call(mut self, name: impl Into<String>, result: CallToolResult) -> Self {
        self.tool_calls.insert(name.into(), result);
        self
    }

    /// Control behaviour for unregistered requests.
    #[must_use]
    pub fn unmatched_request_policy(mut self, policy: UnmatchedPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Build the stub.
    #[must_use]
    pub fn build(self) -> McpStubServer {
        McpStubServer {
            tools: self.tools_response.unwrap_or_default(),
            tool_calls: self.tool_calls,
            policy: self.policy,
            recorded: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

/// A stub MCP server that records requests and returns pre-programmed responses.
pub struct McpStubServer {
    tools: Vec<rmcp::model::Tool>,
    tool_calls: std::collections::HashMap<String, CallToolResult>,
    policy: UnmatchedPolicy,
    recorded: Arc<Mutex<Vec<RecordedRequest>>>,
}

impl McpStubServer {
    /// Return a new builder.
    #[must_use]
    pub fn builder() -> StubBuilder {
        StubBuilder::default()
    }

    /// Return all requests received so far.
    pub async fn recorded_requests(&self) -> Vec<RecordedRequest> {
        self.recorded.lock().await.clone()
    }

    /// Assert that the stub received a `tools/call` for `name` matching `matcher`.
    ///
    /// # Panics
    ///
    /// Panics if no matching request is found.
    pub async fn assert_received_tool_call(
        &self,
        name: &str,
        matcher: impl Fn(&serde_json::Value) -> bool,
    ) {
        let recorded = self.recorded.lock().await;
        let found = recorded
            .iter()
            .any(|r| r.tool_name.as_deref() == Some(name) && matcher(&r.arguments));
        assert!(
            found,
            "expected tool call `{name}` matching predicate, but none found in {recorded:?}"
        );
    }
}

impl ServerHandler for McpStubServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<rmcp::RoleServer>,
    ) -> Result<ListToolsResult, rmcp::ErrorData> {
        Ok(ListToolsResult { tools: self.tools.clone(), next_cursor: None, meta: None })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _ctx: RequestContext<rmcp::RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let name = request.name.as_ref();
        let args = request
            .arguments
            .as_ref()
            .map(|m| {
                serde_json::Value::Object(m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            })
            .unwrap_or(serde_json::Value::Null);

        self.recorded
            .lock()
            .await
            .push(RecordedRequest { tool_name: Some(name.to_string()), arguments: args });

        if let Some(result) = self.tool_calls.get(name) {
            return Ok(result.clone());
        }

        match self.policy {
            UnmatchedPolicy::FailFast => {
                panic!("McpStubServer: unregistered tool call `{name}`");
            }
            UnmatchedPolicy::ReturnError(code) => Err(rmcp::ErrorData {
                code: rmcp::model::ErrorCode(code),
                message: format!("unregistered tool: {name}").into(),
                data: None,
            }),
        }
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
        Err(rmcp::ErrorData::resource_not_found("no resources", None))
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
        Err(rmcp::ErrorData::resource_not_found("no prompts", None))
    }
}
