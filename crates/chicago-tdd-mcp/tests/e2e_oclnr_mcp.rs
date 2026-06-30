//! End-to-end tests against the `oclnr-mcp` binary from osx-clnr.
//!
//! These tests are skipped (not failed) if `OCLNR_MCP_BIN` is not set.
//! Set it to the path of the compiled `oclnr-mcp` binary before running:
//!
//! ```sh
//! OCLNR_MCP_BIN=~/osx-clnr/target/debug/oclnr-mcp \
//!   cargo test -p chicago-tdd-mcp --features testing
//! ```

use chicago_tdd_mcp::{McpServerHarnessBuilder, McpSession};
use tokio::process::Command;

fn oclnr_mcp_bin() -> Option<String> {
    std::env::var("OCLNR_MCP_BIN")
        .ok()
        .or_else(|| which::which("oclnr-mcp").ok().map(|p| p.display().to_string()))
}

#[tokio::test]
async fn oclnr_mcp_full_session() {
    let Some(bin) = oclnr_mcp_bin() else {
        eprintln!("Skipping oclnr_mcp_full_session: set OCLNR_MCP_BIN to enable");
        return;
    };

    let harness = McpServerHarnessBuilder::new(Command::new(&bin))
        .spawn()
        .await
        .expect("failed to spawn oclnr-mcp");

    let session = McpSession::new(harness).initialize().await.expect("initialize failed");

    let tools = session.tools_list().await.expect("tools_list failed");
    assert!(!tools.is_empty(), "oclnr-mcp must expose at least one tool");

    // Verify core workflow tools are present.
    let names: std::collections::HashSet<String> =
        tools.iter().map(|t| t.name.to_string()).collect();

    for expected in &["audit_scan", "plan_build", "delete_dry_run", "receipt_verify"] {
        assert!(
            names.contains(*expected),
            "expected tool `{expected}` in oclnr-mcp tool list, got: {names:?}"
        );
    }

    let closed = session.shutdown().await;
    assert!(closed.is_closed());
}

#[tokio::test]
async fn oclnr_mcp_query_workflow_state() {
    let Some(bin) = oclnr_mcp_bin() else {
        eprintln!("Skipping oclnr_mcp_query_workflow_state: set OCLNR_MCP_BIN to enable");
        return;
    };

    let harness = McpServerHarnessBuilder::new(Command::new(&bin))
        .spawn()
        .await
        .expect("failed to spawn oclnr-mcp");

    let session = McpSession::new(harness).initialize().await.expect("initialize failed");

    let result = session
        .call_tool("query_workflow_state", serde_json::json!({}))
        .await
        .expect("query_workflow_state failed");

    chicago_tdd_mcp::assert::assert_content_array(&result);

    session.shutdown().await;
}
