//! End-to-end tests against the `oclnr-mcp` binary from osx-clnr.
//!
//! These tests are skipped (not failed) if `OCLNR_MCP_BIN` is not set.
//! Set it to the path of the compiled `oclnr-mcp` binary before running:
//!
//! ```sh
//! OCLNR_MCP_BIN=~/osx-clnr/target/debug/oclnr-mcp \
//!   cargo test -p chicago-tdd-mcp --features testing
//! ```

use std::path::Path;

use chicago_tdd_mcp::{McpServerHarnessBuilder, McpSession};
use rmcp::model::{CallToolResult, ContentBlock};
use tokio::process::Command;

fn oclnr_mcp_bin() -> Option<String> {
    std::env::var("OCLNR_MCP_BIN")
        .ok()
        .or_else(|| which::which("oclnr-mcp").ok().map(|p| p.display().to_string()))
}

/// Parse the first text content block of a tool result as JSON.
fn result_json(result: &CallToolResult) -> serde_json::Value {
    for item in &result.content {
        if let ContentBlock::Text(t) = item {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&t.text) {
                return v;
            }
        }
    }
    panic!("no parseable JSON text content in result: {:?}", result.content);
}

/// Build a scratch project directory containing an obvious, safe-to-delete
/// Rust build artifact (`target/`) so `audit_scan`/`plan_build` have a real
/// candidate to find and delete end-to-end.
///
/// `plan_build`'s MCP schema has no `ignore_recent_hours` override (the CLI
/// it shells out to defaults to 168h), so freshly-written files would be
/// filtered out as "recently modified". Backdate mtimes past that window.
fn seed_junk_project(workspace: &Path) {
    let project = workspace.join("fake-crate");
    let target = project.join("target").join("debug");
    std::fs::create_dir_all(&target).expect("create fake target dir");
    std::fs::write(
        project.join("Cargo.toml"),
        "[package]\nname = \"fake-crate\"\nversion = \"0.1.0\"\n",
    )
    .expect("write Cargo.toml");
    std::fs::write(target.join("junk.bin"), vec![0u8; 4096]).expect("write junk build artifact");

    let old = filetime::FileTime::from_unix_time(
        (std::time::SystemTime::now() - std::time::Duration::from_secs(400 * 3600))
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time before epoch")
            .as_secs() as i64,
        0,
    );
    for dir in [&project, &target, workspace] {
        filetime::set_file_mtime(dir, old).expect("backdate dir mtime");
    }
    filetime::set_file_mtime(target.join("junk.bin"), old).expect("backdate file mtime");
    filetime::set_file_mtime(project.join("Cargo.toml"), old).expect("backdate file mtime");
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

/// Drives all 17 `oclnr-mcp` tools through the real workflow, end to end,
/// against a scratch workspace containing an actual junk build artifact.
///
/// Order follows CLAUDE.md's documented protocol:
/// query_workflow_state -> audit_scan -> audit_parse -> plan_build ->
/// plan_inspect -> plan_validate -> safety_audit -> plan_approve ->
/// delete_dry_run -> delete_execute -> receipt_parse -> receipt_verify ->
/// receipt_certify -> snapshot_audit -> emergency_reclaim -> plan_rollback ->
/// clear_artifacts -> query_workflow_state.
#[tokio::test]
async fn oclnr_mcp_full_workflow_end_to_end() {
    let Some(bin) = oclnr_mcp_bin() else {
        eprintln!("Skipping oclnr_mcp_full_workflow_end_to_end: set OCLNR_MCP_BIN to enable");
        return;
    };

    let tmp = tempfile::tempdir().expect("create temp workspace");
    let workspace = tmp.path().to_path_buf();
    seed_junk_project(&workspace);
    let workspace_str = workspace.display().to_string();

    let harness = McpServerHarnessBuilder::new(Command::new(&bin))
        .timeout(std::time::Duration::from_secs(30))
        .spawn()
        .await
        .expect("failed to spawn oclnr-mcp");
    let session = McpSession::new(harness).initialize().await.expect("initialize failed");

    // 1. query_workflow_state — fresh workspace starts Unstarted.
    let r = session
        .call_tool("query_workflow_state", serde_json::json!({ "workspace": workspace_str }))
        .await
        .expect("query_workflow_state failed");
    chicago_tdd_mcp::assert::assert_content_array(&r);

    // 2. audit_scan — scan the seeded workspace. ignore_recent_hours: 0 so the
    // freshly-created junk files (mtime = now) aren't filtered out.
    let r = session
        .call_tool(
            "audit_scan",
            serde_json::json!({
                "workspace": workspace_str,
                "roots": [workspace_str],
                "ignore_recent_hours": 0
            }),
        )
        .await
        .expect("audit_scan failed");
    chicago_tdd_mcp::assert::assert_content_array(&r);
    let audit_json = result_json(&r);
    let audit_file = audit_json
        .get("audit_file")
        .and_then(|v| v.as_str())
        .expect("audit_scan must return audit_file")
        .to_string();
    assert!(Path::new(&audit_file).exists(), "audit_file {audit_file} should exist on disk");

    // 3. audit_parse — read the produced audit evidence back.
    let r = session
        .call_tool("audit_parse", serde_json::json!({ "audit_file": audit_file }))
        .await
        .expect("audit_parse failed");
    chicago_tdd_mcp::assert::assert_content_array(&r);

    // 4. plan_build — requires AuditComplete state from step 2.
    let r = session
        .call_tool(
            "plan_build",
            serde_json::json!({
                "workspace": workspace_str,
                "audit_file": audit_file,
                "roots": [workspace_str]
            }),
        )
        .await
        .expect("plan_build failed");
    chicago_tdd_mcp::assert::assert_content_array(&r);
    let plan_json = result_json(&r);
    let plan_file = plan_json
        .get("plan_file")
        .and_then(|v| v.as_str())
        .expect("plan_build must return plan_file")
        .to_string();
    assert!(Path::new(&plan_file).exists(), "plan_file {plan_file} should exist on disk");

    // 5. plan_inspect
    let r = session
        .call_tool("plan_inspect", serde_json::json!({ "plan_file": plan_file }))
        .await
        .expect("plan_inspect failed");
    chicago_tdd_mcp::assert::assert_content_array(&r);

    // 6. plan_validate
    let r = session
        .call_tool(
            "plan_validate",
            serde_json::json!({ "workspace": workspace_str, "plan_file": plan_file }),
        )
        .await
        .expect("plan_validate failed");
    chicago_tdd_mcp::assert::assert_content_array(&r);

    // 7. safety_audit
    let r = session
        .call_tool("safety_audit", serde_json::json!({ "plan_file": plan_file }))
        .await
        .expect("safety_audit failed");
    chicago_tdd_mcp::assert::assert_content_array(&r);

    // 8. plan_approve — requires confirm: true.
    let r = session
        .call_tool(
            "plan_approve",
            serde_json::json!({
                "plan_file": plan_file,
                "approver_name": "e2e-test",
                "approval_reason": "automated end-to-end workflow test",
                "confirm": true
            }),
        )
        .await
        .expect("plan_approve failed");
    chicago_tdd_mcp::assert::assert_content_array(&r);

    // 9. delete_dry_run — preview only, no side effects on the plan itself.
    let r = session
        .call_tool(
            "delete_dry_run",
            serde_json::json!({ "workspace": workspace_str, "plan_file": plan_file }),
        )
        .await
        .expect("delete_dry_run failed");
    chicago_tdd_mcp::assert::assert_content_array(&r);

    // 10. delete_execute — requires confirm: true. Actually deletes the junk target/ dir.
    let receipt_file = workspace.join("deletion-receipt.json").display().to_string();
    let r = session
        .call_tool(
            "delete_execute",
            serde_json::json!({
                "workspace": workspace_str,
                "plan_file": plan_file,
                "receipt_file": receipt_file,
                "confirm": true
            }),
        )
        .await
        .expect("delete_execute failed");
    chicago_tdd_mcp::assert::assert_content_array(&r);
    assert!(Path::new(&receipt_file).exists(), "receipt_file {receipt_file} should exist on disk");
    assert!(
        !workspace.join("fake-crate").join("target").exists(),
        "seeded target/ dir should have been deleted by delete_execute"
    );

    // 11. receipt_parse
    let r = session
        .call_tool("receipt_parse", serde_json::json!({ "receipt_file": receipt_file }))
        .await
        .expect("receipt_parse failed");
    chicago_tdd_mcp::assert::assert_content_array(&r);

    // 12. receipt_verify
    let r = session
        .call_tool(
            "receipt_verify",
            serde_json::json!({ "workspace": workspace_str, "receipt_file": receipt_file }),
        )
        .await
        .expect("receipt_verify failed");
    chicago_tdd_mcp::assert::assert_content_array(&r);

    // 13. receipt_certify — requires confirm: true.
    let r = session
        .call_tool(
            "receipt_certify",
            serde_json::json!({ "workspace": workspace_str, "receipt_file": receipt_file, "confirm": true }),
        )
        .await
        .expect("receipt_certify failed");
    chicago_tdd_mcp::assert::assert_content_array(&r);

    // 14. snapshot_audit
    let r = session
        .call_tool("snapshot_audit", serde_json::json!({ "workspace": workspace_str }))
        .await
        .expect("snapshot_audit failed");
    chicago_tdd_mcp::assert::assert_content_array(&r);

    // 15. emergency_reclaim — deliberately NOT executed with confirm:true here.
    // This tool is NOT scoped to `workspace`: it sweeps real home-directory
    // caches and real APFS snapshots on whatever `mount` is given, regardless
    // of the sandboxed workspace under test. Only exercise its safety guards:
    // missing confirmation must refuse, and `mount` must be required input.
    let r = session
        .call_tool(
            "emergency_reclaim",
            serde_json::json!({ "workspace": workspace_str, "mount": "/", "target_free_gb": 1.0 }),
        )
        .await
        .expect("emergency_reclaim (no confirm) transport failed");
    chicago_tdd_mcp::assert::assert_tool_error(&r);

    let r = session
        .call_tool(
            "emergency_reclaim",
            serde_json::json!({ "workspace": workspace_str, "target_free_gb": 1.0, "confirm": true }),
        )
        .await
        .expect("emergency_reclaim (missing mount) transport failed");
    chicago_tdd_mcp::assert::assert_tool_error(&r);

    // 16. plan_rollback — requires confirm: true.
    let r = session
        .call_tool(
            "plan_rollback",
            serde_json::json!({ "workspace": workspace_str, "receipt_file": receipt_file, "confirm": true }),
        )
        .await
        .expect("plan_rollback failed");
    chicago_tdd_mcp::assert::assert_content_array(&r);

    // 17. clear_artifacts — requires confirm: true; resets workflow state to Unstarted.
    let r = session
        .call_tool(
            "clear_artifacts",
            serde_json::json!({ "workspace": workspace_str, "confirm": true }),
        )
        .await
        .expect("clear_artifacts failed");
    chicago_tdd_mcp::assert::assert_content_array(&r);

    // Final query_workflow_state — confirm the workflow completed a full cycle.
    let r = session
        .call_tool("query_workflow_state", serde_json::json!({ "workspace": workspace_str }))
        .await
        .expect("final query_workflow_state failed");
    chicago_tdd_mcp::assert::assert_content_array(&r);
    let final_json = result_json(&r);
    assert_eq!(
        final_json.get("state").and_then(|v| v.as_str()),
        Some("UNSTARTED"),
        "clear_artifacts should reset workflow state back to Unstarted, got: {final_json:?}"
    );

    session.shutdown().await;
}
