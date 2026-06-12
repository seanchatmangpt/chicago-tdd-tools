use chicago_tdd_tools::observability::ocel::{OcelCollector, TestActivity};
use chicago_tdd_tools::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(feature = "ocel-generation")]
#[tdd_test]
fn test_ocel_generation_flow() {
    // Arrange: Register OCEL collector
    let output_path = PathBuf::from("target/test_log.ocel.jsonl");
    let collector = Arc::new(OcelCollector::new(Some(output_path.clone())));
    chicago_tdd_tools::core::governance::register_sink(Box::new(OcelCollectorWrapper(
        collector.clone(),
    )));

    // Act: Perform some assertions
    let metadata = AdmissionMetadata {
        id: "test-artifact".to_string(),
        credentials: "valid".to_string(),
        crown_receipt: Some("receipt-123".to_string()),
    };
    assert_admitted!(metadata);
    assert_crown_receipt!(metadata);

    // Assert: Check if events were collected
    // (In a real test, we would wait for close or check internal state)
    // For now, just ensure it compiles and runs without panic
}

struct OcelCollectorWrapper(Arc<OcelCollector>);
impl chicago_tdd_tools::core::governance::DiagnosticSink for OcelCollectorWrapper {
    fn emit(&self, diagnostic: Diagnostic) -> Result<(), String> {
        self.0.emit(diagnostic)
    }
    fn close(&self, summary: RunSummary) -> Result<(), String> {
        self.0.close(summary)
    }
}
