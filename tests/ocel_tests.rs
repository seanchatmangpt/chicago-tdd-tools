//! OCEL 2.0 integration tests.
//! All items here are gated on the `ocel-generation` feature — the OCEL module
//! types are only compiled when that feature is active.
#![cfg(feature = "ocel-generation")]
#![allow(missing_docs)]

use chicago_tdd_tools::core::governance::{
    Diagnostic, DiagnosticCategory, DiagnosticCode, DiagnosticSink, RunSummary, Severity,
};
use chicago_tdd_tools::observability::ocel::OcelCollector;
use std::path::PathBuf;
use std::sync::Arc;

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
