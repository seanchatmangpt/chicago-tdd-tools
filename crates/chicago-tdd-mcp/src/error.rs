//! Error types for chicago-tdd-mcp test harnesses.

/// All errors that can occur in chicago-tdd-mcp harnesses and assertions.
#[derive(Debug, thiserror::Error)]
pub enum McpTestError {
    /// A request or response did not arrive within the deadline.
    #[error("timeout after {0:?}")]
    Timeout(std::time::Duration),

    /// JSON serialization or deserialization failed.
    #[error("JSON error: {0}")]
    ParseError(#[from] serde_json::Error),

    /// The underlying rmcp transport or service returned an error.
    #[error("rmcp error: {0}")]
    Rmcp(String),

    /// The server returned a response that did not match expectations.
    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),

    /// A tool's `inputSchema` rejected the provided arguments.
    #[error("schema violation: {0}")]
    SchemaViolation(String),

    /// An I/O error occurred on the transport.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// The requested tool was not found in the server's tool list.
    #[error("tool not found: {0}")]
    ToolNotFound(String),
}
