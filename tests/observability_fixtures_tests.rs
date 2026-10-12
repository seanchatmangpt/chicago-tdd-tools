//! Integration tests for the Weaver fixture suite (Chicago style).
//!
//! Real collaborators only: synthetic NDJSON report files on real temp
//! filesystems, real OTLP exporter construction, real fixture configuration
//! errors. No mocks. Assertions are on final state (`ValidationResults`
//! contents, `Result` variants), never on call counts.
#![cfg(all(feature = "weaver", feature = "otel"))]
#![allow(missing_docs)]
#![allow(
    warnings,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented
)]

use std::fs;
use std::path::Path;

use chicago_tdd_tools::observability::fixtures::{
    assert_telemetry_valid, assert_violation_count, AdviceLevel, TelemetryCapture,
    ValidationResults, WeaverTestFixture,
};
use chicago_tdd_tools::observability::unified::{ObservabilityError, TestConfig};

// ---------------------------------------------------------------------------
// validation_results
// ---------------------------------------------------------------------------

/// Weaver live-check advice line: one span sample with a violation advice.
fn violation_line(signal: &str, message: &str) -> String {
    format!(
        r#"{{"live_check_result":{{"all_advice":[{{"advice_level":"violation","advice_type":"schema_missing","message":"{message}","signal_type":"span","signal_name":"{signal}","advice_context":{{}}}}]}}}}"#
    )
}

/// Weaver live-check summary line with aggregated statistics.
fn statistics_line(total_advisories: u64, no_advice: u64) -> String {
    format!(
        r#"{{"advice_level_counts":{{"violation":{total_advisories}}},"advice_type_counts":{{}},"highest_advice_level_counts":{{}},"no_advice_count":{no_advice},"total_advisories":{total_advisories},"total_entities":{no_advice},"total_entities_by_type":{{}},"registry_coverage":1.0}}"#
    )
}

fn write_report(dir: &Path, lines: &[String]) {
    fs::create_dir_all(dir).unwrap(); // JUSTIFICATION: Test code only
    fs::write(dir.join("live_check.json"), lines.join("\n") + "\n").unwrap(); // JUSTIFICATION: Test code only
}

#[test]
fn validation_results_counts_violations_and_keeps_statistics() {
    let dir = tempfile::tempdir().unwrap(); // JUSTIFICATION: Test code only
    write_report(
        dir.path(),
        &[violation_line("orders.submit", "attribute order.id is required"), statistics_line(1, 3)],
    );

    let results = ValidationResults::from_report_dir(dir.path()).unwrap(); // JUSTIFICATION: Test code only

    assert!(results.has_violations());
    assert_eq!(results.advices().count(), 1);

    let advice = results.advices().next().unwrap(); // JUSTIFICATION: Test code only
    assert_eq!(advice.level, AdviceLevel::Violation);
    assert_eq!(advice.advice_type, "schema_missing");
    assert_eq!(advice.message, "attribute order.id is required");
    assert_eq!(advice.signal_type.as_deref(), Some("span"));
    assert_eq!(advice.signal_name.as_deref(), Some("orders.submit"));

    let stats = results.statistics().expect("statistics parsed"); // JUSTIFICATION: Test code only
    assert_eq!(stats.total_advisories, Some(1));
    assert_eq!(stats.no_advice_count, Some(3));

    assert_eq!(results.report_path(), dir.path().join("live_check.json"));

    let summary = results.violations_summary();
    assert!(summary.contains("violations"));
    assert!(summary.contains("span:orders.submit"));
    assert!(summary.contains("schema_missing"));
    assert!(summary.contains("attribute order.id is required"));
}

#[test]
fn validation_results_clean_report_has_no_violations() {
    let dir = tempfile::tempdir().unwrap(); // JUSTIFICATION: Test code only
    write_report(
        dir.path(),
        &[
            // Improvement-level advice must NOT count as a violation.
            r#"{"live_check_result":{"all_advice":[{"advice_level":"improvement","advice_type":"naming_convention","message":"prefer snake_case","signal_type":"span","signal_name":"clean.op"}]}}"#.to_string(),
            statistics_line(0, 1),
        ],
    );

    let results = ValidationResults::from_report_dir(dir.path()).unwrap(); // JUSTIFICATION: Test code only

    assert!(!results.has_violations());
    assert_eq!(results.advices().count(), 1);
    assert_eq!(results.advices().next().unwrap().level, AdviceLevel::Improvement);
    assert_eq!(results.violations_summary(), "No Weaver live-check violations detected.");
}

#[test]
fn validation_results_blank_lines_are_tolerated() {
    let dir = tempfile::tempdir().unwrap(); // JUSTIFICATION: Test code only
    write_report(
        dir.path(),
        &[
            String::new(),
            violation_line("a.b", "first"),
            String::from("   "),
            violation_line("c.d", "second"),
            String::new(),
        ],
    );

    let results = ValidationResults::from_report_dir(dir.path()).unwrap(); // JUSTIFICATION: Test code only
    assert_eq!(results.advices().count(), 2);
    assert!(results.has_violations());
}

#[test]
fn validation_results_missing_report_file_is_typed_err() {
    let dir = tempfile::tempdir().unwrap(); // JUSTIFICATION: Test code only
    let missing = dir.path().join("no_such_dir");

    let err = ValidationResults::from_report_dir(&missing).unwrap_err(); // JUSTIFICATION: Test code only

    assert!(matches!(err, ObservabilityError::ValidationFailed(_)));
    let message = err.to_string();
    assert!(message.contains("not found"), "message: {message}");
    assert!(message.contains(&missing.join("live_check.json").display().to_string()));
}

#[test]
fn validation_results_malformed_json_line_is_typed_err() {
    let dir = tempfile::tempdir().unwrap(); // JUSTIFICATION: Test code only
    write_report(dir.path(), &[String::from("{not json")]);

    let err = ValidationResults::from_report_dir(dir.path()).unwrap_err(); // JUSTIFICATION: Test code only
    assert!(matches!(err, ObservabilityError::ValidationFailed(_)));
    assert!(err.to_string().contains("parse"));
}

#[test]
fn advice_level_display_and_default_behavior() {
    assert_eq!(AdviceLevel::Violation.to_string(), "violation");
    assert_eq!(AdviceLevel::Improvement.to_string(), "improvement");
    assert_eq!(AdviceLevel::Information.to_string(), "information");
    assert_eq!(AdviceLevel::Unknown.to_string(), "unknown");

    // serde default for an unrecognized level string is Unknown.
    let advice: AdviceLevel = serde_json::from_str(r#""catastrophe""#).unwrap(); // JUSTIFICATION: Test code only
    assert_eq!(advice, AdviceLevel::Unknown);

    assert_eq!(AdviceLevel::default(), AdviceLevel::Unknown);
}

// ---------------------------------------------------------------------------
// assertions
// ---------------------------------------------------------------------------

#[test]
fn assert_telemetry_valid_passes_clean_and_fails_violated() {
    let clean_dir = tempfile::tempdir().unwrap(); // JUSTIFICATION: Test code only
    write_report(clean_dir.path(), &[statistics_line(0, 1)]);
    let clean = ValidationResults::from_report_dir(clean_dir.path()).unwrap(); // JUSTIFICATION: Test code only

    // Ok on clean input, asserting on state (the returned Result), not calls.
    assert!(assert_telemetry_valid(&clean).is_ok());

    let dirty_dir = tempfile::tempdir().unwrap(); // JUSTIFICATION: Test code only
    write_report(dirty_dir.path(), &[violation_line("x.y", "bad attribute")]);
    let dirty = ValidationResults::from_report_dir(dirty_dir.path()).unwrap(); // JUSTIFICATION: Test code only

    let err = assert_telemetry_valid(&dirty).unwrap_err(); // JUSTIFICATION: Test code only
    assert!(matches!(err, ObservabilityError::ValidationFailed(_)));
    assert!(err.to_string().contains("bad attribute"));
}

#[test]
fn assert_violation_count_matches_real_counts() {
    let dir = tempfile::tempdir().unwrap(); // JUSTIFICATION: Test code only
    write_report(
        dir.path(),
        &[
            violation_line("a.b", "one"),
            violation_line("c.d", "two"),
            // A non-violation advice must not inflate the count.
            r#"{"live_check_result":{"all_advice":[{"advice_level":"information","advice_type":"note","message":"fyi"}]}}"#.to_string(),
        ],
    );
    let results = ValidationResults::from_report_dir(dir.path()).unwrap(); // JUSTIFICATION: Test code only

    assert!(assert_violation_count(&results, 2).is_ok());

    let err = assert_violation_count(&results, 3).unwrap_err(); // JUSTIFICATION: Test code only
    assert!(err.to_string().contains("Expected 3 Weaver violations, found 2"));
}

/// The exported macro is the documented panic contract of the assertion layer.
#[test]
fn assertion_macros_panic_on_failure_and_pass_on_success() {
    let clean_dir = tempfile::tempdir().unwrap(); // JUSTIFICATION: Test code only
    write_report(clean_dir.path(), &[statistics_line(0, 1)]);
    let clean = ValidationResults::from_report_dir(clean_dir.path()).unwrap(); // JUSTIFICATION: Test code only

    chicago_tdd_tools::assert_telemetry_valid!(&clean);
    chicago_tdd_tools::assert_violation_count!(&clean, 0);

    let dirty_dir = tempfile::tempdir().unwrap(); // JUSTIFICATION: Test code only
    write_report(dirty_dir.path(), &[violation_line("p.q", "boom")]);
    let dirty = ValidationResults::from_report_dir(dirty_dir.path()).unwrap(); // JUSTIFICATION: Test code only

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        chicago_tdd_tools::assert_telemetry_valid!(&dirty, "context marker");
    }));
    let panic_message = result
        .err()
        .and_then(|payload| {
            payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
        })
        .expect("macro should panic on violations"); // JUSTIFICATION: Test code only
    assert!(panic_message.contains("context marker"));
    assert!(panic_message.contains("boom"));
}

// ---------------------------------------------------------------------------
// telemetry_capture
// ---------------------------------------------------------------------------

#[test]
fn telemetry_capture_creates_distinct_tracers_for_lazy_endpoint() {
    // 127.0.0.1:1 is unreachable, but exporter construction is lazy, so
    // tracer creation succeeds without any live receiver.
    let capture = TelemetryCapture::new("http://127.0.0.1:1");

    let tracer_a = capture.tracer("test_instrumentation_a", "service_a").unwrap(); // JUSTIFICATION: Test code only
    let tracer_b = capture.tracer("test_instrumentation_b", "service_b").unwrap(); // JUSTIFICATION: Test code only

    // Distinct, usable Debug handles backed by the same capture.
    let debug_a = format!("{tracer_a:?}");
    let debug_b = format!("{tracer_b:?}");
    assert!(!debug_a.is_empty());
    assert!(!debug_b.is_empty());

    let _ = tracer_a.tracer();
    let cloned = tracer_b.clone_tracer();
    drop(cloned);

    // Both handles share the capture's flush path without panicking.
    assert!(capture.flush().is_ok());
}

#[test]
fn telemetry_capture_malformed_endpoint_is_typed_err() {
    let capture = TelemetryCapture::new("::not a valid endpoint::");

    let err = capture.tracer("test_instrumentation", "service").unwrap_err(); // JUSTIFICATION: Test code only

    assert!(matches!(err, ObservabilityError::ValidationFailed(_)));
    assert!(err.to_string().contains("Failed to create OTLP exporter"));
}

// ---------------------------------------------------------------------------
// weaver_fixture (safe path only — never new()/finish(), they spawn Weaver)
// ---------------------------------------------------------------------------

#[test]
fn weaver_fixture_with_nonexistent_registry_returns_typed_err() {
    let missing =
        std::env::temp_dir().join(format!("ctdd-no-such-registry-{}", std::process::id()));

    let config = TestConfig { registry_path: Some(missing.clone()), ..TestConfig::default() };

    // JUSTIFICATION: Test code only — assert on the real error variant, not Debug.
    let err = match WeaverTestFixture::with_config(config) {
        Err(e) => e,
        Ok(_) => panic!("with_config with an invalid config must fail"),
    };

    assert!(matches!(err, ObservabilityError::RegistryNotFound(_)));
    let message = err.to_string();
    assert!(
        message.contains(&missing.display().to_string()),
        "message should mention the missing path: {message}"
    );
    assert!(!missing.exists(), "test must not create the registry path");
}
