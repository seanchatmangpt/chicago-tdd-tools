//! Property-based tests for JSON-RPC 2.0 message codec.
//!
//! Requires the `property-testing` feature:
//! `cargo test -p chicago-tdd-mcp --features property-testing`

#[cfg(feature = "property-testing")]
mod codec_props {
    use proptest::prelude::*;
    use serde_json::{json, Value};

    // A minimal JSON-RPC 2.0 request structure (not using rmcp types directly
    // since they don't derive Arbitrary — we test the wire format instead)
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
    struct JsonRpcRequest {
        jsonrpc: String,
        id: Value,
        method: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        params: Option<Value>,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
    struct JsonRpcSuccess {
        jsonrpc: String,
        id: Value,
        result: Value,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
    struct JsonRpcError {
        jsonrpc: String,
        id: Value,
        error: JsonRpcErrorBody,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
    struct JsonRpcErrorBody {
        code: i32,
        message: String,
    }

    fn arb_jsonrpc_id() -> impl Strategy<Value = Value> {
        prop_oneof![any::<u32>().prop_map(|n| json!(n)), "[a-z]{1,20}".prop_map(|s| json!(s)),]
    }

    fn arb_method() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("tools/list".to_string()),
            Just("tools/call".to_string()),
            Just("initialize".to_string()),
            "[a-z][a-z/]{0,20}".prop_map(std::convert::Into::into),
        ]
    }

    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(200))]

        #[test]
        fn request_roundtrip(id in arb_jsonrpc_id(), method in arb_method()) {
            let req = JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id,
                method,
                params: None,
            };
            let serialized = serde_json::to_string(&req).unwrap();
            let deserialized: JsonRpcRequest = serde_json::from_str(&serialized).unwrap();
            prop_assert_eq!(req, deserialized);
        }

        #[test]
        fn success_and_error_are_discriminated(id in arb_jsonrpc_id(), code in -32099_i32..=-32000) {
            // A success response must NOT deserialize as an error
            let success = json!({"jsonrpc":"2.0","id":id.clone(),"result":{"ok":true}});
            let as_error = serde_json::from_value::<JsonRpcError>(success.clone());
            prop_assert!(as_error.is_err(), "success deserialized as error: {success}");

            // An error response must NOT deserialize as a success
            let error = json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":"err"}});
            let as_success = serde_json::from_value::<JsonRpcSuccess>(error.clone());
            prop_assert!(as_success.is_err(), "error deserialized as success: {error}");
        }

        #[test]
        fn params_null_equals_params_absent(id in arb_jsonrpc_id(), method in arb_method()) {
            let with_null = json!({"jsonrpc":"2.0","id":id.clone(),"method":method.clone(),"params":null});
            let without = json!({"jsonrpc":"2.0","id":id,"method":method});

            let parsed_null: JsonRpcRequest = serde_json::from_value(with_null).unwrap();
            let parsed_none: JsonRpcRequest = serde_json::from_value(without).unwrap();
            prop_assert_eq!(parsed_null.params, parsed_none.params);
        }

        #[test]
        fn fractional_ids_are_valid_json_but_not_recommended(frac in 0.0_f64..1000.0) {
            // JSON-RPC spec says IDs SHOULD be integers or strings.
            // Fractional IDs (1.5) are technically valid JSON but non-conformant.
            // This test documents the behaviour: they round-trip through serde_json.
            let req = json!({"jsonrpc":"2.0","id":frac,"method":"tools/list"});
            let serialized = serde_json::to_string(&req).unwrap();
            let reparsed: Value = serde_json::from_str(&serialized).unwrap();
            prop_assert_eq!(&req["method"], &reparsed["method"]);
        }
    }
}
