use serde::{Deserialize, Serialize};

/// The lifecycle state of an A2A task.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TaskState {
    /// Task has been submitted and is queued.
    Submitted,
    /// Task is actively being processed.
    Working,
    /// Task requires additional input from the caller.
    InputRequired,
    /// Task finished successfully.
    Completed,
    /// Task ended with a failure.
    Failed,
    /// Task was canceled.
    Canceled,
}

/// Describes an A2A agent and its capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCard {
    /// Human-readable name of the agent.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Base URL where the agent is reachable.
    pub url: String,
    /// Skills this agent exposes.
    #[serde(default)]
    pub skills: Vec<AgentSkill>,
    /// Semantic version of the agent.
    #[serde(default)]
    pub version: String,
}

/// A single capability offered by an A2A agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSkill {
    /// Stable identifier for the skill.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Human-readable description.
    pub description: String,
}

/// A task in the A2A protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    /// Unique task identifier.
    pub id: String,
    /// Current lifecycle state.
    pub state: TaskState,
    /// Messages exchanged during the task.
    #[serde(default)]
    pub messages: Vec<Message>,
    /// Populated when `state` is `Failed`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<TaskError>,
}

/// Error detail attached to a failed task.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskError {
    /// Numeric error code.
    pub code: i32,
    /// Human-readable error message.
    pub message: String,
}

/// Identifies which party produced a message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MessageRole {
    /// Message from the calling user/system.
    User,
    /// Message from the A2A agent.
    Agent,
}

/// A single message in an A2A task conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    /// Who sent this message.
    pub role: MessageRole,
    /// Ordered content parts.
    pub parts: Vec<Part>,
}

/// A content part within a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Part {
    /// Plain text content.
    Text(TextPart),
    /// Binary file content (base64-encoded).
    File(FilePart),
    /// Arbitrary structured data.
    Data(DataPart),
}

/// A plain-text message part.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextPart {
    /// The text content.
    pub text: String,
}

/// A binary file message part.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilePart {
    /// MIME type of the file.
    pub mime_type: String,
    /// Base64-encoded file bytes.
    pub data: String,
}

/// A structured-data message part.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataPart {
    /// Arbitrary JSON payload.
    pub data: serde_json::Value,
}

/// Parameters for the `tasks/send` JSON-RPC method.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSendParams {
    /// Client-assigned task ID.
    pub id: String,
    /// Initial message from the user.
    pub message: Message,
    /// Optional session ID for multi-turn continuity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

/// Parameters for the `tasks/get` JSON-RPC method.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskGetParams {
    /// ID of the task to retrieve.
    pub id: String,
}

/// Parameters for the `tasks/cancel` JSON-RPC method.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskCancelParams {
    /// ID of the task to cancel.
    pub id: String,
}

/// A JSON-RPC 2.0 request envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest<P> {
    /// Must be `"2.0"`.
    pub jsonrpc: String,
    /// Caller-assigned request ID.
    pub id: serde_json::Value,
    /// Method name (e.g. `"tasks/send"`).
    pub method: String,
    /// Method-specific parameters.
    pub params: P,
}

/// A JSON-RPC 2.0 response envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse<R> {
    /// Must be `"2.0"`.
    pub jsonrpc: String,
    /// Mirrors the request `id`.
    pub id: serde_json::Value,
    /// Present on success.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<R>,
    /// Present on failure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// A JSON-RPC 2.0 error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Numeric error code.
    pub code: i32,
    /// Human-readable error message.
    pub message: String,
}
