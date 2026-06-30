//! Google A2A (Agent-to-Agent) protocol testing harness.
//!
//! Provides an [`A2aStubAgent`] (axum HTTP stub) and [`A2aTaskHarness`]
//! (reqwest client) for testing A2A agents without a live LLM.
//!
//! Enable via the `a2a` Cargo feature.
//!
//! # Quick start
//!
//! ```ignore
//! use chicago_tdd_mcp::a2a::{A2aStubAgent, A2aTaskHarness, types::*};
//!
//! #[tokio::test]
//! async fn agent_completes_task() {
//!     let card = AgentCard {
//!         name: "test".into(),
//!         description: "stub".into(),
//!         url: String::new(),
//!         skills: vec![],
//!         version: "1.0".into(),
//!     };
//!     let stub = A2aStubAgent::builder(card)
//!         .default_response(Task {
//!             id: "1".into(),
//!             state: TaskState::Completed,
//!             messages: vec![],
//!             error: None,
//!         })
//!         .build()
//!         .unwrap();
//!
//!     let harness = A2aTaskHarness::new(stub.url());
//!     let task = harness.send_task(TaskSendParams {
//!         id: "1".into(),
//!         message: Message {
//!             role: MessageRole::User,
//!             parts: vec![Part::Text(TextPart { text: "hello".into() })],
//!         },
//!         session_id: None,
//!     }).await.unwrap();
//!
//!     chicago_tdd_mcp::a2a::assert::assert_task_completed(&task);
//!     stub.shutdown();
//! }
//! ```

pub mod assert;
pub mod harness;
pub mod stub;
pub mod types;

pub use harness::A2aTaskHarness;
pub use stub::{A2aStubAgent, A2aStubAgentBuilder, A2aUnmatchedPolicy};

/// Errors that can occur during A2A test operations.
#[derive(Debug, thiserror::Error)]
pub enum A2aTestError {
    /// The operation did not complete within the configured deadline.
    #[error("timeout after {0:?}")]
    Timeout(std::time::Duration),
    /// An HTTP transport error occurred.
    #[error("HTTP error: {0}")]
    Http(String),
    /// The response could not be parsed.
    #[error("parse error: {0}")]
    Parse(String),
    /// The agent returned a JSON-RPC error.
    #[error("A2A error code {code}: {message}")]
    A2aError {
        /// Numeric JSON-RPC error code.
        code: i32,
        /// Human-readable error message.
        message: String,
    },
}
