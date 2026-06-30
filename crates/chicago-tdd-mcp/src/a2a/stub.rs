use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};

use super::types::{AgentCard, Message, Part, Task, TaskSendParams, TaskState};

/// Determines what happens when the stub receives a task whose input text
/// has no registered response.
#[derive(Debug, Clone, Copy)]
pub enum A2aUnmatchedPolicy {
    /// Panic immediately — makes the test fail loudly.
    FailFast,
    /// Return a JSON-RPC error with the given error code.
    ReturnError(i32),
}

impl Default for A2aUnmatchedPolicy {
    fn default() -> Self {
        Self::FailFast
    }
}

struct StubState {
    agent_card: AgentCard,
    /// Canned responses keyed by the first text part of the incoming message.
    task_responses: HashMap<String, Task>,
    default_response: Option<Task>,
    policy: A2aUnmatchedPolicy,
    recorded_tasks: Vec<TaskSendParams>,
}

/// Builder for [`A2aStubAgent`].
pub struct A2aStubAgentBuilder {
    agent_card: AgentCard,
    task_responses: HashMap<String, Task>,
    default_response: Option<Task>,
    policy: A2aUnmatchedPolicy,
}

impl A2aStubAgentBuilder {
    /// Create a new builder with the given [`AgentCard`].
    #[must_use]
    pub fn new(card: AgentCard) -> Self {
        Self {
            agent_card: card,
            task_responses: HashMap::new(),
            default_response: None,
            policy: A2aUnmatchedPolicy::default(),
        }
    }

    /// Register a canned [`Task`] response for tasks whose first text part
    /// equals `input_text`.
    #[must_use]
    pub fn on_task_send(mut self, input_text: impl Into<String>, response: Task) -> Self {
        self.task_responses.insert(input_text.into(), response);
        self
    }

    /// Fallback response when no `input_text` matches.
    #[must_use]
    pub fn default_response(mut self, task: Task) -> Self {
        self.default_response = Some(task);
        self
    }

    /// Set the policy for unmatched task inputs.
    #[must_use]
    pub fn unmatched_policy(mut self, policy: A2aUnmatchedPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Bind to a random port and start the axum server.
    ///
    /// # Errors
    ///
    /// Returns an IO error if the port cannot be bound.
    pub async fn build(self) -> Result<A2aStubAgent, std::io::Error> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;

        let state = Arc::new(Mutex::new(StubState {
            agent_card: self.agent_card,
            task_responses: self.task_responses,
            default_response: self.default_response,
            policy: self.policy,
            recorded_tasks: Vec::new(),
        }));

        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let app = Router::new()
            .route("/", post(handle_jsonrpc))
            .route("/.well-known/agent.json", get(handle_agent_card))
            .with_state(state.clone());

        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    let _ = shutdown_rx.await;
                })
                .await
                .ok();
        });

        Ok(A2aStubAgent { url: format!("http://{addr}"), state, shutdown_tx: Some(shutdown_tx) })
    }
}

async fn handle_agent_card(State(state): State<Arc<Mutex<StubState>>>) -> Json<AgentCard> {
    let s = state.lock().await;
    Json(s.agent_card.clone())
}

/// Extract the first text part from a message, if present.
fn first_text_part(message: &Message) -> String {
    message
        .parts
        .iter()
        .find_map(|p| if let Part::Text(t) = p { Some(t.text.clone()) } else { None })
        .unwrap_or_default()
}

async fn handle_jsonrpc(
    State(state): State<Arc<Mutex<StubState>>>,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    let method = body.get("method").and_then(Value::as_str).unwrap_or("");
    let id = body.get("id").cloned().unwrap_or(Value::Null);

    match method {
        "tasks/send" => handle_tasks_send(state, id, &body).await,
        "tasks/get" => handle_tasks_get(state, id, &body).await,
        "tasks/cancel" => {
            let resp = serde_json::json!({
                "jsonrpc": "2.0", "id": id, "result": {"canceled": true}
            });
            (StatusCode::OK, Json(resp))
        }
        other => {
            let resp = serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": {"code": -32601, "message": format!("method not found: {other}")}
            });
            (StatusCode::OK, Json(resp))
        }
    }
}

async fn handle_tasks_send(
    state: Arc<Mutex<StubState>>,
    id: Value,
    body: &Value,
) -> (StatusCode, Json<Value>) {
    let Some(params_val) = body.get("params") else {
        let resp = serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "error": {"code": -32602, "message": "invalid params"}
        });
        return (StatusCode::BAD_REQUEST, Json(resp));
    };

    let send_params = match serde_json::from_value::<TaskSendParams>(params_val.clone()) {
        Ok(p) => p,
        Err(e) => {
            let resp = serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": {"code": -32602, "message": format!("invalid params: {e}")}
            });
            return (StatusCode::BAD_REQUEST, Json(resp));
        }
    };

    let input_text = first_text_part(&send_params.message);

    let mut s = state.lock().await;
    s.recorded_tasks.push(send_params.clone());

    if let Some(task) = s
        .task_responses
        .get(&input_text)
        .cloned()
        .or_else(|| s.default_response.clone())
    {
        let resp = serde_json::json!({
            "jsonrpc": "2.0", "id": id, "result": task
        });
        return (StatusCode::OK, Json(resp));
    }

    let policy = s.policy;
    drop(s);

    match policy {
        // Intentional: FailFast is test infrastructure — panicking here causes the
        // test to fail with a clear message rather than silently returning wrong data.
        A2aUnmatchedPolicy::FailFast => {
            panic!("A2aStubAgent: unmatched task input: {input_text:?}")
        }
        A2aUnmatchedPolicy::ReturnError(code) => {
            let resp = serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": {"code": code, "message": format!("no response for input: {input_text}")}
            });
            (StatusCode::OK, Json(resp))
        }
    }
}

async fn handle_tasks_get(
    state: Arc<Mutex<StubState>>,
    id: Value,
    body: &Value,
) -> (StatusCode, Json<Value>) {
    let task_id = body
        .get("params")
        .and_then(|p| p.get("id"))
        .and_then(Value::as_str)
        .unwrap_or("");

    let s = state.lock().await;
    let found = s.recorded_tasks.iter().find(|t| t.id == task_id).map(|t| Task {
        id: t.id.clone(),
        state: TaskState::Completed,
        messages: vec![],
        error: None,
    });
    drop(s);

    if let Some(task) = found {
        let resp = serde_json::json!({"jsonrpc": "2.0", "id": id, "result": task});
        (StatusCode::OK, Json(resp))
    } else {
        let resp = serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "error": {"code": -32001, "message": "task not found"}
        });
        (StatusCode::OK, Json(resp))
    }
}

/// An axum-based HTTP stub that implements the A2A protocol for testing.
///
/// Create via [`A2aStubAgent::builder`]. The server binds to a random port on
/// `127.0.0.1` and shuts down when the stub is dropped or [`shutdown`](Self::shutdown)
/// is called.
pub struct A2aStubAgent {
    url: String,
    state: Arc<Mutex<StubState>>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl A2aStubAgent {
    /// Returns a [`A2aStubAgentBuilder`] for the given [`AgentCard`].
    #[must_use]
    pub fn builder(card: AgentCard) -> A2aStubAgentBuilder {
        A2aStubAgentBuilder::new(card)
    }

    /// The base URL of this stub (e.g. `"http://127.0.0.1:54321"`).
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// All tasks received by this stub so far, in arrival order.
    pub async fn recorded_tasks(&self) -> Vec<TaskSendParams> {
        self.state.lock().await.recorded_tasks.clone()
    }

    /// Assert that the stub received at least one task whose first text part
    /// contains `text` as a substring.
    ///
    /// # Panics
    ///
    /// Panics if no matching task is found.
    pub async fn assert_received_task_with_text(&self, text: &str) {
        let tasks = self.recorded_tasks().await;
        let found = tasks.iter().any(|t| {
            t.message
                .parts
                .iter()
                .any(|p| matches!(p, Part::Text(tp) if tp.text.contains(text)))
        });
        assert!(found, "expected task with text {text:?}, got: {tasks:?}");
    }

    /// Gracefully shut down the axum server.
    pub fn shutdown(mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

impl Drop for A2aStubAgent {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}
