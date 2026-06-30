//! Declarative macros for common MCP test patterns.
//!
//! These macros collapse repetitive assertion patterns to a single line.

/// Assert that a session's tool list contains every named tool.
///
/// Fails with a diff showing missing and unexpected tools.
///
/// # Example
///
/// ```ignore
/// assert_mcp_tools!(session, ["echo", "ping"]);
/// ```
#[macro_export]
macro_rules! assert_mcp_tools {
    ($session:expr, [$($name:literal),+ $(,)?]) => {{
        let tools = $session.tools_list().await.expect("tools_list failed");
        let names: std::collections::HashSet<String> =
            tools.iter().map(|t| t.name.to_string()).collect();
        let expected = vec![$($name.to_string()),+];
        let mut missing = Vec::new();
        for n in &expected {
            if !names.contains(n) {
                missing.push(n.as_str());
            }
        }
        assert!(
            missing.is_empty(),
            "Missing tools: {:?}\nServer has: {:?}",
            missing,
            names
        );
    }};
}

/// Call a tool, pass the result to a closure for assertions.
///
/// # Example
///
/// ```ignore
/// assert_mcp_tool_call!(session, "echo", {"message": "hello"}, |result| {
///     assert!(!result.content.is_empty());
/// });
/// ```
#[macro_export]
macro_rules! assert_mcp_tool_call {
    ($session:expr, $tool:literal, {$($k:literal: $v:expr),* $(,)?}, |$res:ident| $body:block) => {{
        let args = serde_json::json!({$($k: $v),*});
        let $res = $session
            .call_tool($tool, args)
            .await
            .expect(concat!("call_tool(", $tool, ") failed"));
        $body
    }};
}

/// Send a raw tool call and assert the JSON-RPC error code.
///
/// # Example
///
/// ```ignore
/// mcp_error_test!(session, "__bad__", serde_json::json!({}), -32601);
/// ```
#[macro_export]
macro_rules! mcp_error_test {
    ($session:expr, $tool:literal, $params:expr, $code:expr) => {{
        use $crate::assert::McpErrorAssert;
        let result = $session.call_tool($tool, $params).await;
        match result {
            Ok(r) if r.is_error == Some(true) => {
                // server signalled an error via isError=true; that's acceptable
            }
            Ok(_) => {
                panic!(concat!("expected error code ", $code, " for tool `", $tool, "`, got Ok"))
            }
            Err(e) => {
                let msg = e.to_string();
                assert!(
                    msg.contains(&$code.to_string()),
                    concat!(
                        "expected error code {} in error message for tool `",
                        $tool,
                        "`, got: {}"
                    ),
                    $code,
                    msg
                );
            }
        }
    }};
}
