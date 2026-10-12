//! OCEL 2.0 integration tests.
//! All items here are gated on the `ocel-generation` feature — the OCEL module
//! types are only compiled when that feature is active.
#![cfg(feature = "ocel-generation")]
#![allow(missing_docs)]
#![allow(clippy::pedantic, clippy::nursery, clippy::cargo)]
// JUSTIFICATION: test code only — assertions on real collected state.
#![allow(clippy::unwrap_used)]

use chicago_tdd_tools::core::governance::{
    Diagnostic, DiagnosticCategory, DiagnosticCode, DiagnosticSink, RunSummary, Severity,
};
use chicago_tdd_tools::observability::ocel::OcelCollector;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Unique-per-call string id. Uses `uuid::Uuid::new_v4()` when the optional
/// `uuid-v4` feature is on; falls back to a deterministic monotonic counter
/// (process id + atomic tick) otherwise.
fn unique_suffix() -> String {
    #[cfg(feature = "uuid-v4")]
    {
        uuid::Uuid::new_v4().to_string()
    }
    #[cfg(not(feature = "uuid-v4"))]
    {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        format!(
            "id-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        )
    }
}

#[test]
fn test_ocel_collector_creation() {
    let output_path = PathBuf::from("target/test_log.ocel.jsonl");
    let collector = Arc::new(OcelCollector::new(Some(output_path)));
    // Verify collector is created without panicking
    let _ = collector;
}

#[test]
fn test_ocel_collector_full_lifecycle() {
    let output_path = PathBuf::from("target/test_lifecycle_log.ocel.json");
    if output_path.exists() {
        let _ = std::fs::remove_file(&output_path);
    }

    let collector = OcelCollector::new(Some(output_path.clone()));

    // Create a diagnostic with context representing an object reference
    let mut context = std::collections::HashMap::new();
    let _ = context.insert("fixture_id", serde_json::Value::String("my-fixture-id".to_string()));

    let diag = Diagnostic {
        code: DiagnosticCode::new("test_domain".to_string(), DiagnosticCategory::Conformance, 1),
        category: DiagnosticCategory::Conformance,
        severity: Severity::Info,
        location: None,
        message: "Test diagnostic message".to_string(),
        context,
        run_id: "test-run-123".to_string(),
        agent_id: None,
        source_module: "test_module",
        elapsed_ns: 1000,
    };

    // Emit the diagnostic
    assert!(collector.emit(diag).is_ok());

    // Close the collector (which seals the run and writes the file)
    let summary = RunSummary {
        run_id: "test-run-123".to_string(),
        total_diagnostics: 1,
        ..Default::default()
    };

    assert!(collector.close(summary).is_ok());

    // Verify output file exists and is not empty
    assert!(output_path.exists());
    let file_content = std::fs::read_to_string(&output_path).unwrap();
    assert!(file_content.contains("test-run-123"));
    assert!(file_content.contains("my-fixture-id"));
}

// ============================================================================
// Collector: real diagnostics -> collected log state
// ============================================================================

use chicago_tdd_tools::observability::ocel::projections;
use chicago_tdd_tools::observability::ocel::types::{
    OcelLog, TestActivity, TestObjectType, TestOcelEvent,
};
use chicago_tdd_tools::observability::ocel::wasm4pm::{seal_run, TestSuiteWitness};
use wasm4pm_compat::admission::Admission;

fn make_diagnostic(
    run_id: &str,
    elapsed_ns: u64,
    context: HashMap<&'static str, serde_json::Value>,
) -> Diagnostic {
    Diagnostic {
        code: DiagnosticCode::new("ocel_test".to_string(), DiagnosticCategory::Conformance, 1),
        category: DiagnosticCategory::Conformance,
        severity: Severity::Warning,
        location: None,
        message: "collector test diagnostic".to_string(),
        context,
        run_id: run_id.to_string(),
        agent_id: None,
        source_module: "ocel_tests",
        elapsed_ns,
    }
}

fn fixture_context(fixture_id: &str, schema_id: &str) -> HashMap<&'static str, serde_json::Value> {
    let mut ctx = HashMap::new();
    ctx.insert("fixture_id", serde_json::Value::String(fixture_id.to_string()));
    ctx.insert("schema_id", serde_json::Value::String(schema_id.to_string()));
    ctx
}

#[test]
fn test_collector_emits_events_with_objects_and_attributes() {
    let collector = OcelCollector::new(None);

    let ctx = fixture_context("fix-1", "schema-1");
    // Also exercise the Artifact fallback mapping for other *_id keys.
    let mut ctx2 = ctx.clone();
    ctx2.insert("widget_id", serde_json::Value::String("artifact-9".to_string()));
    ctx2.insert("note", serde_json::Value::String("plain attribute".to_string()));

    assert!(collector.emit(make_diagnostic("run-a", 100, ctx)).is_ok());
    assert!(collector.emit(make_diagnostic("run-a", 200, ctx2)).is_ok());

    let (receipted, digest_hex) = seal_run(&collector, "run-a".to_string())
        .unwrap_or_else(|e| panic!("seal_run failed: {e}"));
    assert_eq!(digest_hex.len(), 64, "digest must be hex-encoded blake3 (32 bytes)");

    let log: &OcelLog = receipted.inner();
    assert_eq!(log.events.len(), 2, "both diagnostics must be admitted");
    assert!(log.global_log.get("ocel:version").is_some_and(|v| v == "2.0"));

    // Deterministic event ids by admission order.
    let ev0: &TestOcelEvent = log.events.get("evt_000").unwrap();
    let ev1: &TestOcelEvent = log.events.get("evt_001").unwrap();

    assert_eq!(ev0.case_id, "run-a");
    assert_eq!(ev1.case_id, "run-a");
    assert_eq!(ev0.timestamp_ns, 100);
    assert_eq!(ev1.timestamp_ns, 200);
    // With `uuid-v4` the collector mints unique v4 ids; without it ids are the
    // documented empty-string zero-ambient fallback (host assigns ids).
    #[cfg(feature = "uuid-v4")]
    assert_ne!(ev0.event_id, ev1.event_id, "event ids must be unique (uuid)");
    #[cfg(not(feature = "uuid-v4"))]
    {
        assert!(ev0.event_id.is_empty(), "no uuid-v4: event_id is empty fallback");
        assert!(ev1.event_id.is_empty(), "no uuid-v4: event_id is empty fallback");
    }

    // Activity reflects the emitted diagnostic category/severity.
    assert!(matches!(
        ev0.activity,
        TestActivity::DiagnosticEmitted {
            category: DiagnosticCategory::Conformance,
            severity: Severity::Warning
        }
    ));

    // Object typing: fixture -> Fixture, schema -> Schema, other *_id -> Artifact.
    let type_of = |ev: &TestOcelEvent, id: &str| {
        ev.objects.iter().find(|(oid, _)| oid == id).map(|(_, ty)| *ty)
    };
    assert_eq!(type_of(ev0, "fix-1"), Some(TestObjectType::Fixture));
    assert_eq!(type_of(ev0, "schema-1"), Some(TestObjectType::Schema));
    assert_eq!(type_of(ev1, "artifact-9"), Some(TestObjectType::Artifact));

    // Attributes carry the full context map.
    assert_eq!(ev1.attributes.get("note").and_then(|v| v.as_str()), Some("plain attribute"));
    assert_eq!(ev1.attributes.get("fixture_id").and_then(|v| v.as_str()), Some("fix-1"));
}

#[test]
fn test_collector_refuses_non_monotonic_timestamps() {
    let collector = OcelCollector::new(None);

    assert!(collector
        .emit(make_diagnostic("run-b", 100, fixture_context("fix-b", "schema-b")))
        .is_ok());
    // Same timestamp as the last admitted event for this case -> refused (not added).
    assert!(collector
        .emit(make_diagnostic("run-b", 100, fixture_context("fix-b", "schema-b")))
        .is_ok());
    // Strictly lower timestamp -> refused too.
    assert!(collector
        .emit(make_diagnostic("run-b", 99, fixture_context("fix-b", "schema-b")))
        .is_ok());

    let (receipted, _) = seal_run(&collector, "run-b".to_string()).unwrap();
    let log = receipted.inner();
    assert_eq!(log.events.len(), 1, "only the first (monotonic) event may be admitted");
    assert_eq!(log.events.get("evt_000").unwrap().timestamp_ns, 100);
}

#[test]
fn test_collector_timestamps_monotonic_per_case() {
    let collector = OcelCollector::new(None);

    // run-x and run-y are independent cases: each starts its own clock.
    assert!(collector
        .emit(make_diagnostic("run-x", 100, fixture_context("fx", "sx")))
        .is_ok());
    assert!(collector
        .emit(make_diagnostic("run-y", 100, fixture_context("fx", "sx")))
        .is_ok());
    assert!(collector
        .emit(make_diagnostic("run-x", 150, fixture_context("fx", "sx")))
        .is_ok());

    let (receipted, _) = seal_run(&collector, "run-x".to_string()).unwrap();
    let log = receipted.inner();
    assert_eq!(log.events.len(), 3);
    let cases: Vec<&str> = ["evt_000", "evt_001", "evt_002"]
        .iter()
        .map(|id| log.events.get(*id).unwrap().case_id.as_str())
        .collect();
    assert_eq!(cases, vec!["run-x", "run-y", "run-x"], "events keep admission order");
}

#[test]
fn test_collector_refuses_empty_case_id() {
    let collector = OcelCollector::new(None);
    assert!(collector.emit(make_diagnostic("", 100, fixture_context("fx", "sx"))).is_ok());

    let (receipted, _) = seal_run(&collector, "default".to_string()).unwrap();
    assert!(receipted.inner().events.is_empty(), "empty case_id must be refused");
}

#[test]
fn test_collector_close_writes_sealed_log_file() {
    let dir = std::env::temp_dir().join(format!(
        "ocel-collector-close-{}-{}",
        std::process::id(),
        unique_suffix()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let output_path = dir.join("sealed_log.ocel.json");

    let collector = OcelCollector::new(Some(output_path.clone()));
    assert!(collector
        .emit(make_diagnostic("run-close", 10, fixture_context("fx", "sx")))
        .is_ok());

    let summary =
        RunSummary { run_id: "run-close".to_string(), total_diagnostics: 1, ..Default::default() };
    assert!(collector.close(summary).is_ok());

    let written = std::fs::read_to_string(&output_path).unwrap();
    let parsed: OcelLog = serde_json::from_str(&written).unwrap();
    assert_eq!(parsed.events.len(), 1);
    assert_eq!(parsed.events.get("evt_000").unwrap().case_id, "run-close");

    // Clean up the temp dir in the same test.
    std::fs::remove_dir_all(&dir).unwrap();
}

// ============================================================================
// Projections: admitted log -> projected evidence
// ============================================================================

#[test]
fn test_project_admission_events_projects_structure() {
    let collector = OcelCollector::new(None);
    assert!(collector
        .emit(make_diagnostic("run-p", 100, fixture_context("fx", "sx")))
        .is_ok());
    assert!(collector
        .emit(make_diagnostic("run-p", 200, fixture_context("fx", "sx")))
        .is_ok());
    let (receipted, _) = seal_run(&collector, "run-p".to_string()).unwrap();

    // Climb back to the Admitted stage so the projection has a lawful input.
    let admitted =
        Admission::<OcelLog, TestSuiteWitness>::new(receipted.inner().clone()).into_evidence();
    let projected = projections::project_admission_events(admitted);

    let log: &OcelLog = &projected.value;
    assert_eq!(log.events.len(), 2);
    assert!(log.global_log.get("ocel:version").is_some_and(|v| v == "2.0"));
    // Event ids survive the projection unchanged.
    for i in 0..2 {
        assert!(log.events.contains_key(&format!("evt_{i:03}")));
    }
    assert_eq!(projected.value.events.get("evt_000").unwrap().case_id, "run-p");
}

// ============================================================================
// Discovery (feature-gated: ocel-generation-discovery)
// ============================================================================

#[cfg(feature = "ocel-generation-discovery")]
mod discovery_tests {
    use super::*;
    use chicago_tdd_tools::observability::ocel::discovery::{
        graduate_for_discovery, ProcessModelStore,
    };
    use wasm4pm_compat::engine_bridge::GraduationReason;

    #[test]
    fn graduate_for_discovery_grounds_evidence() {
        let collector = OcelCollector::new(None);
        assert!(collector
            .emit(make_diagnostic("run-d", 100, fixture_context("fx", "sx")))
            .is_ok());
        let (receipted, _) = seal_run(&collector, "run-d".to_string()).unwrap();

        // Graduation takes the Receipted evidence directly (discovery.rs:37).
        let candidate = graduate_for_discovery(&receipted);

        assert_eq!(candidate.reason, GraduationReason::NeedsDiscovery);
        assert_eq!(candidate.subject, "test-execution-process-model");
        // evidence_ref is derived from the receipted log's event count.
        assert_eq!(candidate.evidence_ref, receipted.inner().events.len().to_string());
        assert!(!candidate.evidence_ref.is_empty(), "non-empty evidence_ref = grounded");
        assert!(candidate.is_grounded());
    }

    #[test]
    fn process_model_store_overwrite_smoke() {
        let store = ProcessModelStore::default();
        // ProcessModelStore exposes only `new`/`store` (no read API); the smoke
        // contract is that repeated inserts with the same name are accepted
        // (HashMap::insert overwrite semantics) and never panic.
        store.store("model-a", vec![1, 2, 3]);
        store.store("model-a", vec![4, 5]);
        store.store("model-b", vec![9]);
    }
}
