//! Integration tests for `McpServerHarness` and `McpSession` against the echo fixture.

use std::path::PathBuf;
use std::sync::OnceLock;

use chicago_tdd_mcp::{McpServerHarnessBuilder, McpSession};
use tokio::process::Command;

// Compile the echo fixture binary once per test run.
static ECHO_BIN: OnceLock<PathBuf> = OnceLock::new();

fn echo_bin() -> PathBuf {
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

            if !output.status.success() {
                panic!(
                    "echo-mcp-server build failed:\n{}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }

            // Parse cargo JSON output to find the binary path.
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
            panic!("could not find echo-mcp-server binary in cargo output");
        })
        .clone()
}

#[tokio::test]
async fn harness_spawns_and_lists_tools() {
    let bin = echo_bin();
    let harness = McpServerHarnessBuilder::new(Command::new(&bin))
        .spawn()
        .await
        .expect("failed to spawn echo server");

    let tools = harness.tools_list().await.expect("tools_list failed");
    assert!(
        tools.iter().any(|t| t.name == "echo"),
        "expected `echo` tool, got: {:?}",
        tools.iter().map(|t| &t.name).collect::<Vec<_>>()
    );

    harness.shutdown().await;
}

#[tokio::test]
async fn harness_calls_echo_tool() {
    let bin = echo_bin();
    let harness = McpServerHarnessBuilder::new(Command::new(&bin))
        .spawn()
        .await
        .expect("failed to spawn echo server");

    let result = harness
        .call_tool("echo", serde_json::json!({"message": "hello chicago-tdd-mcp"}))
        .await
        .expect("call_tool failed");

    chicago_tdd_mcp::assert::assert_content_array(&result);

    let text = result
        .content
        .iter()
        .find_map(|c| {
            if let rmcp::model::ContentBlock::Text(t) = c {
                Some(t.text.as_str())
            } else {
                None
            }
        })
        .expect("no text content");

    assert_eq!(text, "hello chicago-tdd-mcp");

    harness.shutdown().await;
}

#[tokio::test]
async fn session_lifecycle_uninitialized_to_closed() {
    let bin = echo_bin();
    let harness = McpServerHarnessBuilder::new(Command::new(&bin))
        .spawn()
        .await
        .expect("failed to spawn echo server");

    let session = McpSession::new(harness).initialize().await.expect("initialize failed");

    let tools = session.tools_list().await.expect("tools_list failed");
    assert!(!tools.is_empty(), "expected at least one tool");

    let closed = session.shutdown().await;
    assert!(closed.is_closed());
}

#[tokio::test]
async fn session_calls_echo_tool() {
    let bin = echo_bin();
    let harness = McpServerHarnessBuilder::new(Command::new(&bin))
        .spawn()
        .await
        .expect("failed to spawn echo server");

    let session = McpSession::new(harness).initialize().await.expect("initialize failed");

    let result = session
        .call_tool("echo", serde_json::json!({"message": "state machine works"}))
        .await
        .expect("call_tool failed");

    chicago_tdd_mcp::assert::assert_content_array(&result);
    session.shutdown().await;
}
