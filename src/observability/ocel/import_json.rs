//! OCEL 2.0 JSON import — fail-closed ingestion into the in-crate [`OcelLog`].
//!
//! Accepts OCEL 2.0 JSON documents of the shape
//! `{"ocel:version": "2.0", "ocel:events": {...}, "ocel:objects": {...}}`
//! and maps them into the in-crate OCEL types ([`OcelLog`], [`TestOcelEvent`]).
//!
//! # Fail-closed contract
//!
//! * missing `ocel:version` → [`ImportError::MissingField`]
//! * `ocel:version` != `"2.0"` → [`ImportError::InvalidVersion`]
//! * missing per-event `ocel:activity` / `ocel:timestamp` (or per-object
//!   `ocel:type`) → [`ImportError::MissingField`] carrying the field name
//! * non-JSON input → [`ImportError::MalformedJson`]
//!
//! The published `wasm4pm-compat` OCEL structs (`OcelLog`/`TestOcelEvent`,
//! re-exported from this module) ARE serde-derived, so the wire format
//! deserializes through them directly; this module hand-maps
//! `serde_json::Value` into them on top of the serde layer, validating
//! version and required fields (the wasm4pm-compat *builder* type is a
//! different type and is not part of this import path). The mapping is NOT
//! lossless: unknown `ocel:` shapes (e.g. `ocel:ovmap`) are dropped and
//! event attributes are not reconstructed — this import is a projection,
//! not a lossless conversion.

#![cfg(feature = "ocel-ingest-json")]

use super::types::{OcelLog, TestObjectType, TestOcelEvent};
use serde_json::Value;
use thiserror::Error;

/// Local, documented error surface for OCEL JSON import (`ConformanceError`
/// style: typed variants, no stringly-typed failure, no panics on input).
#[derive(Debug, Clone, Error)]
pub enum ImportError {
    /// The input was not syntactically valid JSON.
    #[error("malformed OCEL JSON: {message}")]
    MalformedJson {
        /// Human-readable parse failure description.
        message: String,
    },
    /// A required field was absent (top-level, per-event, or per-object).
    #[error("missing required field: {field} (in {subject})")]
    MissingField {
        /// The exact `ocel:`-prefixed field name that was absent.
        field: &'static str,
        /// Where the field was missing (document, event id, or object id).
        subject: String,
    },
    /// `ocel:version` was present but not `"2.0"`.
    #[error("unsupported ocel:version {found:?} (only \"2.0\" accepted)")]
    InvalidVersion {
        /// The version string that was found.
        found: String,
    },
}

/// Required per-event fields in OCEL 2.0 JSON.
const REQUIRED_EVENT_FIELDS: [&str; 2] = ["ocel:activity", "ocel:timestamp"];
/// Required per-object fields in OCEL 2.0 JSON.
const REQUIRED_OBJECT_FIELDS: [&str; 1] = ["ocel:type"];

/// Import an OCEL 2.0 JSON document into the in-crate [`OcelLog`].
///
/// Fails closed: refuses the whole document on any missing required field or
/// a non-`"2.0"` `ocel:version` — never partially imports.
///
/// # Errors
/// Returns [`ImportError`] on malformed JSON, missing required fields, or an
/// unsupported version. See the module-level fail-closed contract.
pub fn import_ocel_json(json: &str) -> Result<OcelLog, ImportError> {
    let doc: Value = serde_json::from_str(json)
        .map_err(|e| ImportError::MalformedJson { message: e.to_string() })?;

    let version = doc.get("ocel:version").ok_or_else(|| ImportError::MissingField {
        field: "ocel:version",
        subject: "<document>".to_string(),
    })?;
    let version = version
        .as_str()
        .ok_or_else(|| ImportError::InvalidVersion { found: version.to_string() })?;
    if version != "2.0" {
        return Err(ImportError::InvalidVersion { found: version.to_string() });
    }

    let mut log = OcelLog::new();

    if let Some(objects) = doc.get("ocel:objects").and_then(Value::as_object) {
        for (id, obj) in objects {
            for required in REQUIRED_OBJECT_FIELDS {
                if obj.get(required).is_none() || obj.get(required) == Some(&Value::Null) {
                    return Err(ImportError::MissingField { field: required, subject: id.clone() });
                }
            }
            let object_type: TestObjectType =
                deserialize_field(obj.get("ocel:type"), "ocel:type", id)?;
            let attributes: std::collections::HashMap<String, Value> = match obj.get("ocel:ovmap") {
                Some(Value::Object(map)) => map.clone().into_iter().collect(),
                Some(_) => {
                    return Err(ImportError::MissingField {
                        field: "ocel:ovmap",
                        subject: id.clone(),
                    })
                }
                None => std::collections::HashMap::new(),
            };
            log.objects
                .insert(id.clone(), super::types::TestObject { object_type, attributes });
        }
    }

    if let Some(events) = doc.get("ocel:events").and_then(Value::as_object) {
        for (id, event) in events {
            for required in REQUIRED_EVENT_FIELDS {
                if event.get(required).is_none() || event.get(required) == Some(&Value::Null) {
                    return Err(ImportError::MissingField { field: required, subject: id.clone() });
                }
            }
            let activity: super::types::TestActivity =
                deserialize_field(event.get("ocel:activity"), "ocel:activity", id)?;
            let timestamp_ns: u64 =
                deserialize_field(event.get("ocel:timestamp"), "ocel:timestamp", id)?;
            let case_id = event
                .get("ocel:caseId")
                .and_then(Value::as_str)
                .map_or_else(|| id.clone(), str::to_owned);
            let objects: Vec<(String, TestObjectType)> = event
                .get("ocel:objectTypes")
                .and_then(Value::as_array)
                .map(|arr| {
                    arr.iter()
                        .filter_map(|entry| {
                            let obj = entry.as_object()?;
                            let oid = obj.get("id")?.as_str()?.to_owned();
                            let oty: TestObjectType =
                                serde_json::from_value(obj.get("type")?.clone()).ok()?;
                            Some((oid, oty))
                        })
                        .collect()
                })
                .unwrap_or_default();
            let attributes: std::collections::HashMap<String, Value> = event
                .get("ocel:attributes")
                .and_then(Value::as_object)
                .map(|map| map.clone().into_iter().collect())
                .unwrap_or_default();
            log.events.insert(
                id.clone(),
                TestOcelEvent {
                    event_id: id.clone(),
                    case_id,
                    activity,
                    timestamp_ns,
                    objects,
                    attributes,
                },
            );
        }
    }

    Ok(log)
}

/// Deserialize a `serde_json` field value into `T` with a typed refusal on
/// absent-but-required or wrong-shape values.
fn deserialize_field<T: serde::de::DeserializeOwned>(
    value: Option<&Value>,
    field: &'static str,
    id: &str,
) -> Result<T, ImportError> {
    let value =
        value.ok_or_else(|| ImportError::MissingField { field, subject: id.to_string() })?;
    serde_json::from_value(value.clone())
        .map_err(|e| ImportError::MissingField { field, subject: format!("{id}: {e}") })
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOLDEN: &str = r#"{
        "ocel:version": "2.0",
        "ocel:events": {
            "e1": {
                "ocel:timestamp": 1700000000000000000,
                "ocel:activity": "TestStarted",
                "ocel:caseId": "run-42",
                "ocel:attributes": {"suite": "core"},
                "ocel:objectTypes": [{"id": "f1", "type": "Fixture"}]
            },
            "e2": {
                "ocel:timestamp": 1700000000000000001,
                "ocel:activity": "ReceiptGenerated",
                "ocel:caseId": "run-42"
            }
        },
        "ocel:objects": {
            "f1": {"ocel:type": "Fixture", "ocel:ovmap": {"name": "db"}}
        }
    }"#;

    #[test]
    fn golden_import_maps_events_and_objects() {
        let log = import_ocel_json(GOLDEN).unwrap();
        assert_eq!(log.events.len(), 2);
        assert_eq!(log.objects.len(), 1);

        let e1 = log.events.get("e1").unwrap();
        assert_eq!(e1.event_id, "e1");
        assert_eq!(e1.case_id, "run-42");
        assert_eq!(e1.timestamp_ns, 1_700_000_000_000_000_000);
        assert_eq!(serde_json::to_string(&e1.activity).unwrap(), r#""TestStarted""#);
        assert_eq!(e1.objects, vec![("f1".to_string(), TestObjectType::Fixture)]);
        assert_eq!(e1.attributes.get("suite").and_then(Value::as_str), Some("core"));

        let obj = log.objects.get("f1").unwrap();
        assert_eq!(obj.object_type, TestObjectType::Fixture);
        assert_eq!(obj.attributes.get("name").and_then(Value::as_str), Some("db"));
    }

    #[test]
    fn case_id_defaults_to_event_id_when_absent() {
        // A document whose only event carries no `ocel:caseId`: the case
        // falls back to the event id (GOLDEN's events all carry a caseId).
        let json = r#"{
            "ocel:version": "2.0",
            "ocel:events": {"e9": {"ocel:timestamp": 7, "ocel:activity": "TestStarted"}}
        }"#;
        let log = import_ocel_json(json).unwrap();
        assert_eq!(log.events.get("e9").unwrap().case_id, "e9");
    }

    #[test]
    fn refuses_malformed_json() {
        let err = import_ocel_json("{not json").unwrap_err();
        assert!(matches!(err, ImportError::MalformedJson { .. }));
    }

    #[test]
    fn refuses_missing_version() {
        let json = r#"{"ocel:events": {}}"#;
        let err = import_ocel_json(json).unwrap_err();
        assert!(matches!(err, ImportError::MissingField { field: "ocel:version", .. }));
    }

    #[test]
    fn refuses_wrong_version() {
        let json = r#"{"ocel:version": "1.0", "ocel:events": {}}"#;
        let err = import_ocel_json(json).unwrap_err();
        assert!(matches!(
            err,
            ImportError::InvalidVersion { found } if found == "1.0"
        ));
    }

    #[test]
    fn refuses_event_missing_activity() {
        let json = r#"{
            "ocel:version": "2.0",
            "ocel:events": {"e1": {"ocel:timestamp": 1}}
        }"#;
        let err = import_ocel_json(json).unwrap_err();
        assert!(matches!(
            err,
            ImportError::MissingField { field: "ocel:activity", subject }
                if subject == "e1"
        ));
    }

    #[test]
    fn refuses_event_missing_timestamp() {
        let json = r#"{
            "ocel:version": "2.0",
            "ocel:events": {"e1": {"ocel:activity": "TestStarted"}}
        }"#;
        let err = import_ocel_json(json).unwrap_err();
        assert!(matches!(
            err,
            ImportError::MissingField { field: "ocel:timestamp", subject }
                if subject == "e1"
        ));
    }

    #[test]
    fn refuses_null_required_field() {
        let json = r#"{
            "ocel:version": "2.0",
            "ocel:events": {"e1": {"ocel:activity": null, "ocel:timestamp": 1}}
        }"#;
        let err = import_ocel_json(json).unwrap_err();
        assert!(matches!(
            err,
            ImportError::MissingField { field: "ocel:activity", subject }
                if subject == "e1"
        ));
    }

    #[test]
    fn refuses_object_missing_type() {
        let json = r#"{
            "ocel:version": "2.0",
            "ocel:events": {},
            "ocel:objects": {"o1": {"ocel:ovmap": {}}}
        }"#;
        let err = import_ocel_json(json).unwrap_err();
        assert!(matches!(
            err,
            ImportError::MissingField { field: "ocel:type", subject }
                if subject == "o1"
        ));
    }

    #[test]
    fn roundtrip_via_serde_json() {
        let log = import_ocel_json(GOLDEN).unwrap();
        let serialized = serde_json::to_string(&log).unwrap();
        let reparsed: OcelLog = serde_json::from_str(&serialized).unwrap();
        assert_eq!(reparsed.events.len(), log.events.len());
        assert_eq!(reparsed.objects.len(), log.objects.len());
        assert_eq!(
            reparsed.events.get("e1").unwrap().timestamp_ns,
            log.events.get("e1").unwrap().timestamp_ns
        );
    }
}
