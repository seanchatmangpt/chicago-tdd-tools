//! Clean integration tests for Milestone 1 Governance functionality.
#![cfg(feature = "governance-tests")]
#![allow(missing_docs)]
#![allow(clippy::pedantic, clippy::nursery, clippy::cargo)]

use chicago_tdd_tools::core::governance::{
    close_channel, emit_diagnostic, get_domain, register_sink, set_channel_capacity, Diagnostic,
    DiagnosticCategory, DiagnosticCode, DiagnosticSink, RunSummary, Severity, SourceLocation,
};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::Duration;

// Test lock to serialize execution of tests using the global state
static TEST_LOCK: Mutex<()> = Mutex::new(());

#[derive(Clone, Default)]
struct MockSink {
    emitted: Arc<Mutex<Vec<Diagnostic>>>,
    closed_summary: Arc<Mutex<Option<RunSummary>>>,
    closed: Arc<Mutex<bool>>,
}

impl MockSink {
    fn new() -> Self {
        Self::default()
    }
}

impl DiagnosticSink for MockSink {
    fn emit(&self, diagnostic: Diagnostic) -> Result<(), String> {
        if *self.closed.lock().unwrap_or_else(|e| e.into_inner()) {
            return Err("Sink is closed".to_string());
        }
        self.emitted.lock().unwrap_or_else(|e| e.into_inner()).push(diagnostic);
        Ok(())
    }

    fn close(&self, summary: RunSummary) -> Result<(), String> {
        let mut closed = self.closed.lock().unwrap_or_else(|e| e.into_inner());
        if *closed {
            return Err("Sink already closed".to_string());
        }
        *closed = true;
        *self.closed_summary.lock().unwrap_or_else(|e| e.into_inner()) = Some(summary);
        Ok(())
    }
}

fn create_test_diagnostic(
    domain: &str,
    category: DiagnosticCategory,
    ordinal: u16,
    msg: &str,
    severity: Severity,
) -> Diagnostic {
    Diagnostic {
        code: DiagnosticCode::new(domain.to_string(), category, ordinal),
        category,
        severity,
        location: Some(SourceLocation { file: "test.rs".to_string(), line: 1, column: 1 }),
        message: msg.to_string(),
        context: std::collections::HashMap::new(),
        run_id: "test-run".to_string(),
        agent_id: None,
        source_module: "test_module",
        elapsed_ns: 0,
    }
}

// ============================================================================
// 1. Concurrency (Stress Test)
// ============================================================================

#[test]
fn test_governance_concurrency_stress() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _ = close_channel();

    let sink = MockSink::new();
    register_sink(Box::new(sink.clone()));

    let num_threads = 16;
    let emissions_per_thread = 100;
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = Vec::new();

    for i in 0..num_threads {
        let barrier_clone = barrier.clone();
        let handle = thread::spawn(move || {
            barrier_clone.wait();
            for j in 0..emissions_per_thread {
                let msg = format!("Diag from thread {i} number {j}");
                let diag = create_test_diagnostic(
                    "CORE",
                    DiagnosticCategory::Conformance,
                    1,
                    &msg,
                    Severity::Warning,
                );
                emit_diagnostic(&diag);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let summary = close_channel().unwrap();

    assert_eq!(summary.total_diagnostics, num_threads * emissions_per_thread);
    assert_eq!(sink.emitted.lock().unwrap().len(), num_threads * emissions_per_thread);
    assert!(*sink.closed.lock().unwrap());
}

// ============================================================================
// 2. Mock Sink Emission and Close
// ============================================================================

#[test]
fn test_governance_mock_sink_emission_and_close() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _ = close_channel();

    let sink = MockSink::new();
    register_sink(Box::new(sink.clone()));

    let diag = create_test_diagnostic(
        "CORE",
        DiagnosticCategory::Admission,
        1,
        "test emission",
        Severity::Andon,
    );
    emit_diagnostic(&diag);

    let summary = close_channel().unwrap();

    assert_eq!(summary.total_diagnostics, 1);
    assert!(*sink.closed.lock().unwrap());
    assert_eq!(sink.emitted.lock().unwrap().len(), 1);
    assert_eq!(sink.emitted.lock().unwrap()[0].message, "test emission");

    let closed_sum = sink.closed_summary.lock().unwrap().clone().unwrap();
    assert_eq!(closed_sum.total_diagnostics, 1);
    assert_eq!(closed_sum.andon_count, 1);
}

// ============================================================================
// 3. RunSummary Statistics Compilation
// ============================================================================

#[test]
fn test_governance_run_summary_statistics() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _ = close_channel();

    let sink = MockSink::new();
    register_sink(Box::new(sink.clone()));

    // 2 Andons for Admission
    emit_diagnostic(&create_test_diagnostic(
        "CORE",
        DiagnosticCategory::Admission,
        1,
        "admission error 1",
        Severity::Andon,
    ));
    emit_diagnostic(&create_test_diagnostic(
        "CORE",
        DiagnosticCategory::Admission,
        2,
        "admission error 2",
        Severity::Andon,
    ));

    // 1 Warning for Drift
    emit_diagnostic(&create_test_diagnostic(
        "CORE",
        DiagnosticCategory::Drift,
        1,
        "drift warning",
        Severity::Warning,
    ));

    // 1 Info for Lineage
    emit_diagnostic(&create_test_diagnostic(
        "CORE",
        DiagnosticCategory::Lineage,
        1,
        "lineage info",
        Severity::Info,
    ));

    let summary = close_channel().unwrap();

    assert_eq!(summary.evaluated, 4);
    assert_eq!(summary.andon_count, 2);
    assert_eq!(summary.warning_count, 1);
    assert_eq!(summary.admitted, 2);
    assert_eq!(summary.p_admitted, 0.5);
    assert_eq!(summary.dominant_category, Some(DiagnosticCategory::Admission));
    assert_eq!(summary.total_diagnostics, 4);

    assert_eq!(summary.category_counts.get(&DiagnosticCategory::Admission), Some(&2));
    assert_eq!(summary.category_counts.get(&DiagnosticCategory::Drift), Some(&1));
    assert_eq!(summary.category_counts.get(&DiagnosticCategory::Lineage), Some(&1));
}

// ============================================================================
// 4. Custom Serde for DiagnosticCode
// ============================================================================

#[test]
fn test_governance_diagnostic_code_serde() {
    // 1. Serialization
    let code = DiagnosticCode::new("MYAPP", DiagnosticCategory::Bypass, 5);
    let serialized = serde_json::to_string(&code).unwrap();
    assert_eq!(serialized, "\"MYAPP-BYP-005\"");

    // 2. Deserialization Success
    let deserialized: DiagnosticCode = serde_json::from_str("\"MYAPP-BYP-005\"").unwrap();
    assert_eq!(deserialized, code);

    // 3. Deserialization of 2-part format (defaults domain to "CORE")
    let two_part: DiagnosticCode = serde_json::from_str("\"ADM-001\"").unwrap();
    assert_eq!(two_part.domain, "CORE");
    assert_eq!(two_part.category, DiagnosticCategory::Admission);
    assert_eq!(two_part.ordinal, 1);

    // 4. Parsing success/failures
    assert!(DiagnosticCode::parse("CORE-ADM-001").is_ok());
    assert!(DiagnosticCode::parse("MYAPP-BYP-010").is_ok());
    assert!(DiagnosticCode::parse("ADM-001").is_ok());

    // Parse error paths
    assert!(DiagnosticCode::parse("").is_err());
    assert!(DiagnosticCode::parse("ADM").is_err());
    assert!(DiagnosticCode::parse("CORE-XXX-001").is_err());
    assert!(DiagnosticCode::parse("CORE-ADM-abc").is_err());
    assert!(DiagnosticCode::parse("-ADM-001").is_err());
    assert!(DiagnosticCode::parse("MY-APP-ADM-001").is_err());
}

// ============================================================================
// 5. Deadlock-Free Validations
// ============================================================================

struct DeadlockingSink;

impl DiagnosticSink for DeadlockingSink {
    fn emit(&self, _diagnostic: Diagnostic) -> Result<(), String> {
        // Calls get_domain() which locks get_state()
        let _ = get_domain();
        Ok(())
    }

    fn close(&self, _summary: RunSummary) -> Result<(), String> {
        Ok(())
    }
}

#[test]
fn test_governance_deadlock_free_sync_write_emission() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _ = close_channel();

    // Set capacity to Some(0) to trigger sync write behavior (direct call to sink.emit under the lock)
    set_channel_capacity(Some(0));
    register_sink(Box::new(DeadlockingSink));

    // Spawn the diagnostic emission in a separate thread so we can time it out
    let (tx, rx) = std::sync::mpsc::channel();
    thread::spawn(move || {
        let diag = create_test_diagnostic(
            "CORE",
            DiagnosticCategory::Conformance,
            1,
            "trigger deadlock check",
            Severity::Warning,
        );
        emit_diagnostic(&diag);
        tx.send(()).ok();
    });

    // Wait for completion with a timeout of 500ms
    let result = rx.recv_timeout(Duration::from_millis(500));

    // Clean up channel capacity
    set_channel_capacity(None);

    assert!(
        result.is_ok(),
        "Expected emit_diagnostic to NOT deadlock when capacity is Some(0) and sink calls get_domain(), but it timed out/deadlocked!"
    );
}
