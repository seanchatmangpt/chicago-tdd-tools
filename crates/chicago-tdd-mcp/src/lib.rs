//! `chicago-tdd-mcp` — LLM-free MCP server/client testing infrastructure.
//!
//! # Features
//!
//! | Feature | What it enables |
//! |---|---|
//! | `testing` | [`McpServerHarness`], [`McpStubServer`], [`McpPipe`], [`McpSession`] |
//! | `snapshot-testing` | [`assert::schema::assert_tool_list_snapshot`] via `insta` |
//! | `property-testing` | JSON-RPC codec `proptest` roundtrips |
//! | `ocel` | [`ocel::build_mcp_session_ocel`] OCEL 2.0 evidence log |
//! | `macros` | [`assert_mcp_tools!`], [`assert_mcp_tool_call!`], [`mcp_error_test!`] |
//!
//! # Quick start
//!
//! ```toml
//! # In your project's [dev-dependencies]:
//! chicago-tdd-mcp = { path = "path/to/chicago-tdd-mcp", features = ["testing"] }
//! ```
//!
//! ```ignore
//! use chicago_tdd_mcp::{McpServerHarnessBuilder, McpSession};
//!
//! #[tokio::test]
//! async fn my_mcp_server_has_an_echo_tool() {
//!     let harness = McpServerHarnessBuilder::new(
//!         tokio::process::Command::new("my-mcp-server")
//!     )
//!     .spawn()
//!     .await
//!     .unwrap();
//!
//!     let mut session = McpSession::new(harness).initialize().await.unwrap();
//!     let tools = session.tools_list().await.unwrap();
//!     assert!(tools.iter().any(|t| t.name == "echo"));
//!     session.shutdown().await;
//! }
//! ```

pub mod error;

#[cfg(feature = "testing")]
pub mod assert;
#[cfg(feature = "testing")]
pub mod harness;
#[cfg(feature = "testing")]
pub mod macros;
#[cfg(feature = "testing")]
pub mod transport;

#[cfg(feature = "ocel")]
pub mod ocel;

#[cfg(feature = "chain")]
pub mod chain;

#[cfg(feature = "a2a")]
pub mod a2a;

// Flat re-exports for ergonomic imports.
#[cfg(feature = "testing")]
pub use error::McpTestError;
#[cfg(feature = "testing")]
pub use harness::client::{McpStubServer, StubBuilder, UnmatchedPolicy};
#[cfg(feature = "testing")]
pub use harness::server::{McpServerHarness, McpServerHarnessBuilder};
#[cfg(feature = "testing")]
pub use harness::session::{Closed, McpSession, Ready, Uninitialized};
#[cfg(feature = "testing")]
pub use transport::pipe::McpPipe;

// Re-export rmcp proc-macro attributes for server implementors.
#[cfg(feature = "testing")]
pub use rmcp::{tool, tool_handler, tool_router, ServerHandler};

// Re-export chicago-tdd-mcp-macros for convenience.
#[cfg(feature = "macros")]
pub use chicago_tdd_mcp_macros::mcp_server_test;

// Re-export chain types.
#[cfg(feature = "chain")]
pub use chain::{ForwardedCall, McpAgentBridge, McpAgentBridgeBuilder};
