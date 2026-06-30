//! Integration tests for multi-hop MCP chains.
//!
//! Requires: `cargo test -p chicago-tdd-mcp --features chain`

#[cfg(feature = "chain")]
mod chain {
    use chicago_tdd_mcp::chain::McpAgentBridge;
    use chicago_tdd_mcp::transport::pipe::McpPipe;
    use chicago_tdd_mcp::McpStubServer;
    use rmcp::model::{CallToolResult, ContentBlock, Tool};

    fn echo_schema() -> std::sync::Arc<serde_json::Map<String, serde_json::Value>> {
        let schema = serde_json::json!({
            "type": "object",
            "properties": { "message": { "type": "string" } },
            "required": ["message"]
        });
        let map = schema.as_object().expect("schema is object").clone();
        std::sync::Arc::new(map)
    }

    fn echo_stub() -> McpStubServer {
        McpStubServer::builder()
            .on_tools_list(vec![Tool::new("echo", "Echo a message back", echo_schema())])
            .on_tool_call("echo", CallToolResult::success(vec![ContentBlock::text("echoed")]))
            .build()
    }

    fn echo_stub_with_text(text: &str) -> McpStubServer {
        McpStubServer::builder()
            .on_tools_list(vec![Tool::new("echo", "Echo a message back", echo_schema())])
            .on_tool_call(
                "echo",
                CallToolResult::success(vec![ContentBlock::text(text.to_owned())]),
            )
            .build()
    }

    #[tokio::test]
    async fn two_hop_chain_routes_tool_call() {
        // downstream: stub that handles "echo"
        let downstream = McpPipe::new_in_process(echo_stub_with_text("two hops"))
            .await
            .expect("downstream pipe failed");

        let bridge = McpAgentBridge::builder().with_pipe(downstream).build();

        let orchestrator = McpPipe::new_in_process(bridge).await.expect("orchestrator pipe failed");
        let result = orchestrator
            .call_tool("echo", serde_json::json!({"message": "two hops"}))
            .await
            .expect("call_tool failed");

        assert!(!result.content.is_empty(), "expected non-empty content");
        let text = result.content.iter().find_map(|c| {
            if let rmcp::model::ContentBlock::Text(t) = c {
                Some(t.text.as_str().to_owned())
            } else {
                None
            }
        });
        assert_eq!(text.as_deref(), Some("two hops"), "text mismatch");
    }

    #[tokio::test]
    async fn bridge_lists_downstream_tools() {
        let downstream =
            McpPipe::new_in_process(echo_stub()).await.expect("downstream pipe failed");
        let bridge = McpAgentBridge::builder().with_pipe(downstream).build();

        let orchestrator = McpPipe::new_in_process(bridge).await.expect("orchestrator pipe failed");
        let tools = orchestrator.list_tools().await.expect("list_tools failed");

        assert!(
            tools.iter().any(|t| t.name == "echo"),
            "expected echo tool in bridge tool list, got: {:?}",
            tools.iter().map(|t| &t.name).collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn bridge_forwards_tool_call_successfully() {
        let downstream =
            McpPipe::new_in_process(echo_stub()).await.expect("downstream pipe failed");
        let bridge = McpAgentBridge::builder().with_pipe(downstream).build();

        let orchestrator = McpPipe::new_in_process(bridge).await.expect("orchestrator pipe failed");
        let result = orchestrator
            .call_tool("echo", serde_json::json!({"message": "forwarded"}))
            .await
            .expect("call_tool failed");

        assert_ne!(result.is_error, Some(true), "expected no error from forwarded call");
        assert!(!result.content.is_empty(), "expected non-empty content");
    }
}
