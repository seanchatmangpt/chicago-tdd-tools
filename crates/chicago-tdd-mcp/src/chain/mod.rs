//! Multi-hop MCP chain testing.
//!
//! `McpAgentBridge` lets you test chains of MCP servers: an orchestrator that
//! calls one MCP server which in turn calls another, all in-process.
//!
//! # Quick start
//!
//! ```ignore
//! use chicago_tdd_mcp::chain::McpAgentBridge;
//! use chicago_tdd_mcp::transport::pipe::McpPipe;
//!
//! // EchoServer is defined in tests with #[tool_router] + #[tool_handler]
//! #[tokio::test]
//! async fn two_hop_chain() {
//!     // downstream: real EchoServer
//!     // bridge: forwards any unknown tool to the downstream
//!     let bridge = McpAgentBridge::builder()
//!         .with_server(EchoServer::new())
//!         .await
//!         .unwrap()
//!         .build();
//!
//!     // orchestrator connects to the bridge via McpPipe
//!     let mut pipe = McpPipe::new_in_process(bridge).await.unwrap();
//!     let result = pipe.call_tool("echo", serde_json::json!({"message": "chain"})).await.unwrap();
//!     assert!(!result.content.is_empty());
//! }
//! ```

pub mod bridge;

pub use bridge::{ForwardedCall, McpAgentBridge, McpAgentBridgeBuilder};
