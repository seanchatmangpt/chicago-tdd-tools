//! OCEL 2.0 integration tests.
//! All items here are gated on the `ocel-generation` feature — the OCEL module
//! types are only compiled when that feature is active.
#![cfg(feature = "ocel-generation")]
#![allow(missing_docs)]

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
