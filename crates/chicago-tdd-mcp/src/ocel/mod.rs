//! OCEL 2.0 event log integration for MCP test runs.
//!
//! Emits structured evidence events for each MCP request/response pair so that
//! test runs produce a machine-verifiable process log.
//!
//! Gated behind the `ocel` feature. Uses the `wasm4pm_compat::ocel::OCEL` format
//! (OCEL 2.0 JSON standard).

#[cfg(feature = "ocel")]
use chrono::Utc;
#[cfg(feature = "ocel")]
use uuid::Uuid;
#[cfg(feature = "ocel")]
use wasm4pm_compat::ocel::{
    OCELEvent, OCELObject, OCELObjectAttribute, OCELRelationship, OCELType, OCEL,
};

/// An MCP session event for inclusion in an OCEL log.
#[cfg(feature = "ocel")]
#[derive(Debug, Clone)]
pub struct McpOcelEvent {
    /// The JSON-RPC method name (e.g. `"tools/call"`).
    pub method: String,
    /// Stable request identifier.
    pub request_id: String,
    /// BLAKE3 digest of the canonical JSON serialization of request params.
    pub params_digest: String,
    /// BLAKE3 digest of the response result (or `None` for notifications).
    pub result_digest: Option<String>,
    /// Whether the response was an error.
    pub is_error: bool,
    /// Whether all assertions on this event passed.
    pub assertions_passed: bool,
}

#[cfg(feature = "ocel")]
impl McpOcelEvent {
    /// Create a new event, computing BLAKE3 digests of params and result.
    #[must_use]
    pub fn new(
        method: impl Into<String>,
        request_id: impl Into<String>,
        params: &serde_json::Value,
        result: Option<&serde_json::Value>,
        is_error: bool,
        assertions_passed: bool,
    ) -> Self {
        let params_digest =
            blake3::hash(serde_json::to_string(params).unwrap_or_default().as_bytes())
                .to_hex()
                .to_string();

        let result_digest = result.map(|r| {
            blake3::hash(serde_json::to_string(r).unwrap_or_default().as_bytes())
                .to_hex()
                .to_string()
        });

        Self {
            method: method.into(),
            request_id: request_id.into(),
            params_digest,
            result_digest,
            is_error,
            assertions_passed,
        }
    }
}

/// Build an OCEL 2.0 log for an MCP test session.
///
/// Produces a `wasm4pm_compat::ocel::OCEL` document containing one event per
/// MCP request/response pair plus assertion outcome events.
///
/// # Panics
///
/// Never panics under normal conditions.
#[cfg(feature = "ocel")]
#[must_use]
pub fn build_mcp_session_ocel(session_id: Uuid, events: Vec<McpOcelEvent>) -> OCEL {
    use wasm4pm_compat::ocel::OCELAttributeValue;

    let now_rfc3339 = Utc::now().to_rfc3339();

    // Object types
    let session_type = OCELType { name: "McpSession".into(), attributes: vec![] };
    let request_type = OCELType { name: "McpRequest".into(), attributes: vec![] };
    let response_type = OCELType { name: "McpResponse".into(), attributes: vec![] };

    // Event types
    let evt_req = OCELType { name: "mcp_request_sent".into(), attributes: vec![] };
    let evt_resp = OCELType { name: "mcp_response_received".into(), attributes: vec![] };
    let evt_pass = OCELType { name: "mcp_assertion_passed".into(), attributes: vec![] };
    let evt_fail = OCELType { name: "mcp_assertion_failed".into(), attributes: vec![] };

    // Session object
    let session_obj = OCELObject {
        id: session_id.to_string(),
        object_type: "McpSession".into(),
        attributes: vec![],
        relationships: vec![],
    };

    let mut objects: Vec<OCELObject> = vec![session_obj];
    let mut ocel_events: Vec<OCELEvent> = Vec::with_capacity(events.len() * 3);

    for event in &events {
        let req_id = format!("req-{}", event.request_id);
        let resp_id = format!("resp-{}", event.request_id);

        let ts: chrono::DateTime<chrono::FixedOffset> = now_rfc3339.parse().unwrap_or_default();

        // Request object
        objects.push(OCELObject {
            id: req_id.clone(),
            object_type: "McpRequest".into(),
            attributes: vec![
                OCELObjectAttribute {
                    name: "method".into(),
                    value: OCELAttributeValue::String(event.method.clone()),
                    time: ts,
                },
                OCELObjectAttribute {
                    name: "params_digest".into(),
                    value: OCELAttributeValue::String(event.params_digest.clone()),
                    time: ts,
                },
            ],
            relationships: vec![OCELRelationship {
                object_id: session_id.to_string(),
                qualifier: "session".into(),
            }],
        });

        // Response object
        objects.push(OCELObject {
            id: resp_id.clone(),
            object_type: "McpResponse".into(),
            attributes: vec![
                OCELObjectAttribute {
                    name: "is_error".into(),
                    value: OCELAttributeValue::Boolean(event.is_error),
                    time: ts,
                },
                OCELObjectAttribute {
                    name: "result_digest".into(),
                    value: event.result_digest.as_deref().map_or(OCELAttributeValue::Null, |d| {
                        OCELAttributeValue::String(d.to_owned())
                    }),
                    time: ts,
                },
            ],
            relationships: vec![OCELRelationship {
                object_id: req_id.clone(),
                qualifier: "request".into(),
            }],
        });

        // Request sent event
        ocel_events.push(OCELEvent {
            id: format!("ev-req-{}", event.request_id),
            event_type: "mcp_request_sent".into(),
            time: ts,
            attributes: vec![],
            relationships: vec![OCELRelationship {
                object_id: req_id.clone(),
                qualifier: "request".into(),
            }],
        });

        // Response received event
        ocel_events.push(OCELEvent {
            id: format!("ev-resp-{}", event.request_id),
            event_type: "mcp_response_received".into(),
            time: ts,
            attributes: vec![],
            relationships: vec![
                OCELRelationship { object_id: resp_id.clone(), qualifier: "response".into() },
                OCELRelationship { object_id: req_id.clone(), qualifier: "request".into() },
            ],
        });

        // Assertion event
        let assertion_type =
            if event.assertions_passed { "mcp_assertion_passed" } else { "mcp_assertion_failed" };
        ocel_events.push(OCELEvent {
            id: format!("ev-assert-{}", event.request_id),
            event_type: assertion_type.into(),
            time: ts,
            attributes: vec![],
            relationships: vec![OCELRelationship {
                object_id: resp_id,
                qualifier: "response".into(),
            }],
        });
    }

    OCEL {
        event_types: vec![evt_req, evt_resp, evt_pass, evt_fail],
        object_types: vec![session_type, request_type, response_type],
        events: ocel_events,
        objects,
    }
}
