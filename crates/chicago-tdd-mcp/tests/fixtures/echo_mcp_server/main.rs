//! Minimal rmcp echo server for chicago-tdd-mcp harness tests.
//!
//! Exposes one tool: `echo` — returns its `message` argument as text content.
//! Built as a binary in the test suite; compiled once via `OnceLock`.

use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    schemars, tool, tool_handler, tool_router, ErrorData, ServerHandler, ServiceExt,
};

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct EchoParams {
    /// The text to echo back.
    message: String,
}

#[derive(Clone)]
struct EchoServer {
    #[expect(dead_code, reason = "tool_handler macro accesses this router field")]
    tool_router: ToolRouter<EchoServer>,
}

#[tool_router]
impl EchoServer {
    pub fn new() -> Self {
        Self { tool_router: Self::tool_router() }
    }

    #[tool(description = "Echo a message back as text content")]
    fn echo(
        &self,
        Parameters(EchoParams { message }): Parameters<EchoParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Ok(CallToolResult::success(vec![ContentBlock::text(message)]))
    }
}

#[tool_handler]
impl ServerHandler for EchoServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    EchoServer::new().serve(rmcp::transport::io::stdio()).await?.waiting().await?;
    Ok(())
}
