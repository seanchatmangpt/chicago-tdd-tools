//! Snapshot tests — pins the exact tool list schema of the echo fixture.
//!
//! Run `INSTA_UPDATE=always cargo test -p chicago-tdd-mcp --features snapshot-testing`
//! to regenerate snapshots after intentional changes.

use std::path::PathBuf;
use std::sync::OnceLock;

use chicago_tdd_mcp::McpServerHarnessBuilder;
use tokio::process::Command;

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

#[cfg(feature = "snapshot-testing")]
#[tokio::test]
async fn tools_list_snapshot() {
    let bin = echo_bin();
    let harness = McpServerHarnessBuilder::new(Command::new(&bin))
        .spawn()
        .await
        .expect("failed to spawn echo server");

    chicago_tdd_mcp::assert::schema::assert_tool_list_snapshot(&harness, "echo_tools")
        .await
        .expect("snapshot assertion failed");

    harness.shutdown().await;
}

#[cfg(feature = "snapshot-testing")]
#[tokio::test]
async fn tool_input_schema_snapshot() {
    use chicago_tdd_mcp::assert::schema::canonicalize_tools;

    let bin = echo_bin();
    let harness = McpServerHarnessBuilder::new(Command::new(&bin))
        .spawn()
        .await
        .expect("failed to spawn echo server");

    let tools = harness.tools_list().await.expect("tools_list failed");
    let echo_tool = tools.iter().find(|t| t.name == "echo").expect("echo tool missing");
    let schema = serde_json::to_value(echo_tool.input_schema.as_ref()).expect("serialize schema");
    insta::assert_json_snapshot!("echo_input_schema", schema);

    // Silence unused import warning when snapshot-testing feature gates the usage
    let _ = canonicalize_tools;

    harness.shutdown().await;
}
