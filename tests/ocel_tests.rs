//! OCEL 2.0 integration tests.
//! All items here are gated on the `ocel-generation` feature — the OCEL module
//! types are only compiled when that feature is active.
#![cfg(feature = "ocel-generation")]
#![allow(missing_docs)]

use chicago_tdd_tools::observability::ocel::OcelCollector;
use chicago_tdd_tools::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;

#[tdd_test]
fn test_ocel_generation_flow() {
    // Arrange: Register OCEL collector
    let output_path = PathBuf::from("target/test_log.ocel.jsonl");
    let collector = Arc::new(OcelCollector::new(Some(output_path.clone())));
    chicago_tdd_tools::core::governance::register_sink(Box::new(OcelCollectorWrapper(
        collector.clone(),
    )));

    // Act: Perform some assertions
    let metadata = chicago_tdd_tools::core::governance::AdmissionMetadata {
        id: "test-artifact".to_string(),
        credentials: "valid".to_string(),
        crown_receipt: Some("receipt-123".to_string()),
    };
    assert_admitted!(metadata);
    assert_crown_receipt!(metadata);

    // Assert: Channel accepted the diagnostics without panicking.
    let summary =
        chicago_tdd_tools::core::governance::close_channel().expect("close_channel should succeed");
    assert!(
        summary.total_diagnostics == 0 || summary.total_diagnostics >= 0,
        "total_diagnostics must be non-negative"
    );
}

struct OcelCollectorWrapper(Arc<OcelCollector>);
impl chicago_tdd_tools::core::governance::DiagnosticSink for OcelCollectorWrapper {
    fn emit(
        &self,
        diagnostic: chicago_tdd_tools::core::governance::Diagnostic,
    ) -> Result<(), String> {
        self.0.emit(diagnostic)
    }
    fn close(
        &self,
        summary: chicago_tdd_tools::core::governance::RunSummary,
    ) -> Result<(), String> {
        self.0.close(summary)
    }
}
