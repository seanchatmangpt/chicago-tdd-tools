use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use reqwest::Client;
use serde_json::json;

use super::types::{AgentCard, Task, TaskSendParams};
use super::A2aTestError;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

/// A thin `reqwest`-backed client harness for the A2A protocol.
///
/// Use this in tests to drive a real (or stub) A2A agent over HTTP.
pub struct A2aTaskHarness {
    client: Client,
    base_url: String,
    timeout: Duration,
    next_id: AtomicU64,
}

impl A2aTaskHarness {
    /// Create a new harness pointing at `base_url`
    /// (e.g. `"http://127.0.0.1:12345"`).
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
            timeout: DEFAULT_TIMEOUT,
            next_id: AtomicU64::new(1),
        }
    }

    /// Override the per-request timeout (default: 10 s).
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    async fn jsonrpc_call<P, R>(&self, method: &str, params: P) -> Result<R, A2aTestError>
    where
        P: serde::Serialize,
        R: serde::de::DeserializeOwned,
    {
        let req = json!({
            "jsonrpc": "2.0",
            "id": self.next_id(),
            "method": method,
            "params": params,
        });

        let response =
            tokio::time::timeout(self.timeout, self.client.post(&self.base_url).json(&req).send())
                .await
                .map_err(|_| A2aTestError::Timeout(self.timeout))?
                .map_err(|e| A2aTestError::Http(e.to_string()))?;

        let body: serde_json::Value =
            response.json().await.map_err(|e| A2aTestError::Http(e.to_string()))?;

        if let Some(err) = body.get("error") {
            let code = err.get("code").and_then(|c| c.as_i64()).unwrap_or(-1) as i32;
            let message =
                err.get("message").and_then(|m| m.as_str()).unwrap_or("unknown").to_string();
            return Err(A2aTestError::A2aError { code, message });
        }

        let result = body
            .get("result")
            .ok_or_else(|| A2aTestError::Parse("missing result field".into()))?;

        serde_json::from_value(result.clone()).map_err(|e| A2aTestError::Parse(e.to_string()))
    }

    /// Send a task to the A2A agent (`tasks/send`).
    ///
    /// # Errors
    ///
    /// Returns [`A2aTestError`] on transport, timeout, or protocol failure.
    pub async fn send_task(&self, params: TaskSendParams) -> Result<Task, A2aTestError> {
        self.jsonrpc_call("tasks/send", params).await
    }

    /// Retrieve a task by ID (`tasks/get`).
    ///
    /// # Errors
    ///
    /// Returns [`A2aTestError`] on transport, timeout, or protocol failure.
    pub async fn get_task(&self, id: &str) -> Result<Task, A2aTestError> {
        self.jsonrpc_call("tasks/get", json!({"id": id})).await
    }

    /// Cancel a task by ID (`tasks/cancel`).
    ///
    /// # Errors
    ///
    /// Returns [`A2aTestError`] on transport, timeout, or protocol failure.
    pub async fn cancel_task(&self, id: &str) -> Result<serde_json::Value, A2aTestError> {
        self.jsonrpc_call("tasks/cancel", json!({"id": id})).await
    }

    /// Fetch the agent's [`AgentCard`] from `/.well-known/agent.json`.
    ///
    /// # Errors
    ///
    /// Returns [`A2aTestError`] on transport, timeout, or parse failure.
    pub async fn fetch_agent_card(&self) -> Result<AgentCard, A2aTestError> {
        let url = format!("{}/.well-known/agent.json", self.base_url);

        let response = tokio::time::timeout(self.timeout, self.client.get(&url).send())
            .await
            .map_err(|_| A2aTestError::Timeout(self.timeout))?
            .map_err(|e| A2aTestError::Http(e.to_string()))?;

        response
            .json::<AgentCard>()
            .await
            .map_err(|e| A2aTestError::Parse(e.to_string()))
    }
}
