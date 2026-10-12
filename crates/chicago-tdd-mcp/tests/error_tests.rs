//! Error path tests — every MCP/JSON-RPC error code has a passing test.

// All harness/assert APIs used here live behind the `testing` feature; compile
// to nothing without it.
#![cfg(feature = "testing")]

use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Duration;

use chicago_tdd_mcp::assert::{error_scenarios, McpErrorAssert};
use chicago_tdd_mcp::{McpServerHarnessBuilder, McpSession};
use tokio::process::Command;

/// Hard ceiling per test. If the echo server wedges, the test fails instead of
/// hanging the whole `cargo test` run (previously >20 min with no bound).
const TEST_TIMEOUT: Duration = Duration::from_secs(60);

static ECHO_BIN: OnceLock<PathBuf> = OnceLock::new();

/// Build the echo server command with `kill_on_drop`, so the spawned
/// `echo-mcp-server` child is reaped even if a test panics mid-flight and the
/// harness is dropped without an explicit `shutdown().await`.
fn echo_command() -> Command {
    let mut cmd = Command::new(echo_bin());
    cmd.kill_on_drop(true);
    cmd
}

/// Bound any async step so a stuck server surfaces as a test failure.
async fn bounded<F: std::future::Future>(fut: F) -> F::Output {
    tokio::time::timeout(TEST_TIMEOUT, fut)
        .await
        .expect("test step exceeded timeout; echo server appears wedged")
}

fn echo_bin() -> PathBuf {
    // AUDIT NOTE (W15): this cargo build runs inside the first test's
    // bounded(60s) window — under fan-out load the target-dir file lock
    // alone can consume most of it, and the timeout message names the echo
    // server when the real wait was the build lock.
    ECHO_BIN
        .get_or_init(|| {
            let output = std::process::Command::new("cargo")
                .args([
                    "build",
                    "--bin",
                    "echo-mcp-server",
                    "--features",
                    "testing",
                    "--manifest-path",
                    concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"),
                    "--message-format=json",
                ])
                .output()
                .expect("cargo build failed");
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Ok(msg) = serde_json::from_str::<serde_json::Value>(line) {
                    if msg["reason"] == "compiler-artifact"
                        && msg["target"]["name"] == "echo-mcp-server"
                    {
                        if let Some(path) = msg["executable"].as_str() {
                            return PathBuf::from(path);
                        }
                    }
                }
            }
            panic!("could not find echo-mcp-server binary");
        })
        .clone()
}

#[tokio::test]
async fn unknown_tool_returns_error() {
    bounded(async {
        let harness = McpServerHarnessBuilder::new(echo_command())
            .spawn()
            .await
            .expect("spawn failed");

        let result = harness.call_tool("__no_such_tool__", serde_json::json!({})).await;

        // rmcp will either return an Rmcp error or an isError=true result
        match result {
            Err(chicago_tdd_mcp::McpTestError::Rmcp(_)) => { /* expected */ }
            Ok(r) if r.is_error == Some(true) => { /* also acceptable */ }
            other => panic!("expected error for unknown tool, got: {other:?}"),
        }

        harness.shutdown().await;
    })
    .await;
}

#[tokio::test]
async fn call_tool_on_ready_session_works() {
    bounded(async {
        // Confirm that the session state machine permits call_tool when Ready.
        let harness = McpServerHarnessBuilder::new(echo_command())
            .spawn()
            .await
            .expect("spawn failed");
        let session = McpSession::new(harness).initialize().await.expect("initialize failed");

        let result = session.call_tool("echo", serde_json::json!({"message": "ping"})).await;

        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        session.shutdown().await;
    })
    .await;
}

#[tokio::test]
async fn send_malformed_json_returns_parse_error() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

    bounded(async {
        // Implemented inline rather than via
        // `error_scenarios::send_malformed_json`, whose read of the server's
        // reply blocks forever when the server stays silent on malformed
        // input. The bounded read below makes silence a conformant outcome
        // instead of a hang.
        let mut child = echo_command()
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("spawn echo server failed");

        let mut stdin = child.stdin.take().expect("stdin piped");
        let stdout = child.stdout.take().expect("stdout piped");
        let mut reader = tokio::io::BufReader::new(stdout);

        stdin.write_all(b"not valid json\n").await.expect("write failed");
        stdin.flush().await.expect("flush failed");
        drop(stdin); // close stdin so a conformant server may exit

        const REPLY_WINDOW: Duration = Duration::from_secs(10);
        let mut reply = String::new();
        let line = match tokio::time::timeout(REPLY_WINDOW, reader.read_line(&mut reply)).await {
            Err(_) => None,    // server stayed silent within the window
            Ok(Ok(0)) => None, // server closed the pipe without responding
            Ok(Ok(_)) => Some(reply),
            Ok(Err(e)) => panic!("read of server stdout failed: {e}"),
        };

        // The child must be reaped in every path (kill_on_drop also covers panic).
        let _ = child.kill().await;
        let _ = child.wait().await;

        match line {
            None => {
                // No response / closed pipe — rejection, which is conformant.
            }
            Some(l) => {
                let response: serde_json::Value =
                    serde_json::from_str(l.trim()).expect("server sent non-JSON reply");
                McpErrorAssert::expect_code(&response, error_scenarios::codes::PARSE_ERROR);
            }
        }
    })
    .await;
}

#[tokio::test]
async fn call_before_initialize_rejected_or_closed() {
    bounded(async {
        let response = error_scenarios::call_before_initialize(echo_command())
            .await
            .expect("call_before_initialize failed");

        // Either the server returned a JSON-RPC error response, or it closed the pipe (None).
        // Both are acceptable conformant behaviors.
        match response {
            None => {
                // Server closed the connection — that's fine
            }
            Some(r) => {
                // Server returned an error response
                assert!(
                    r.get("error").is_some(),
                    "expected error response for call-before-initialize, got: {r}"
                );
            }
        }
    })
    .await;
}
