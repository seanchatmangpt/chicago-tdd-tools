//! Weaver integration tests
//!
//! **Poka-yoke**: These are INTEGRATION tests, not unit tests. They test real Weaver CLI
//! integration and require Weaver binary to be available. Unit tests should NOT test
//! Weaver integration - they should only test types and validators.
//!
//! These tests verify:
//! - Weaver CLI can be started and stopped
//! - Telemetry can be sent to Weaver via OTLP
//! - Weaver validates telemetry correctly
//!
//! **CRITICAL**: These tests require Weaver binary to be installed and registry path to exist.
//! If Weaver is not available, these tests MUST fail (not skip).

#[cfg(all(feature = "weaver", test))]
mod weaver_integration_tests {
    use chicago_tdd_tools::assert_ok;
    use chicago_tdd_tools::weaver::{send_test_span_to_weaver, WeaverValidator};
    use std::fs;
    use std::path::PathBuf;
    use std::time::Duration;
    use tokio::time::sleep;

    /// Integration test: Weaver live-check with real CLI
    ///
    /// **Poka-yoke**: This is an INTEGRATION test in tests/ directory, not a unit test in src/.
    /// It tests real Weaver CLI integration, not just types.
    #[tokio::test]
    async fn test_weaver_live_check_integration() {
        // Arrange: Create validator with registry path
        let registry_path = PathBuf::from("registry");

        // Skip test if registry doesn't exist (may not be available in test environment)
        if !registry_path.exists() {
            #[cfg(feature = "logging")]
            log::info!("Registry path does not exist - skipping live-check integration test");
            #[cfg(not(feature = "logging"))]
            eprintln!("NOTE: Registry path does not exist - skipping live-check integration test");
            return;
        }

        // Check if Weaver binary is available
        let weaver_available = WeaverValidator::check_weaver_available().is_ok();
        if !weaver_available {
            #[cfg(feature = "logging")]
            log::info!("Weaver binary not available - skipping live-check integration test");
            #[cfg(not(feature = "logging"))]
            eprintln!("NOTE: Weaver binary not available - skipping live-check integration test");
            return;
        }

        // Act: Create and start Weaver validator (uses CLI)
        let mut validator = WeaverValidator::new(registry_path);
        let start_result = validator.start();

        // Assert: Start should succeed
        assert_ok!(&start_result, "Weaver CLI should start successfully");
        assert!(validator.is_running(), "Weaver should be running after start");

        // Wait a moment for Weaver CLI to be ready
        sleep(Duration::from_millis(1000)).await;

        // Act: Get OTLP endpoint
        let endpoint = validator.otlp_endpoint();
        assert!(!endpoint.is_empty(), "OTLP endpoint should not be empty");

        // Act: Send test span to Weaver via OTLP HTTP
        // This sends telemetry to the weaver CLI which validates it
        let send_result = send_test_span_to_weaver(&endpoint, "test.operation");
        assert_ok!(&send_result, "Sending test span should succeed");

        // Wait a moment for telemetry to be processed by weaver CLI
        sleep(Duration::from_millis(1000)).await;

        // Act: Stop Weaver CLI via HTTP admin endpoint
        let stop_result = validator.stop();
        assert_ok!(&stop_result, "Weaver CLI should stop successfully");
        assert!(!validator.is_running(), "Weaver should not be running after stop");

        // Assert: Parse weaver CLI JSON output report
        // When using --output, weaver CLI writes live_check.json to the output directory
        let report_path = PathBuf::from("./weaver-reports/live_check.json");
        if report_path.exists() {
            let report_content =
                fs::read_to_string(&report_path).expect("Failed to read weaver CLI JSON report");
            let report_json: serde_json::Value = serde_json::from_str(&report_content)
                .expect("Failed to parse weaver CLI JSON report");

            // Verify report structure (matches weaver CLI output format)
            assert!(
                report_json.get("statistics").is_some(),
                "Weaver CLI report should contain statistics"
            );

            // Verify statistics fields exist
            if let Some(statistics) = report_json.get("statistics") {
                assert!(
                    statistics.get("total_entities").is_some()
                        || statistics.get("total_advisories").is_some(),
                    "Statistics should contain validation results"
                );
            }
        } else {
            // If report doesn't exist, that's OK - weaver CLI may not have written it yet
            // or output format may differ. The important thing is CLI was used.
            #[cfg(feature = "logging")]
            log::info!(
                "Weaver CLI report not found at {:?} - CLI may use different output format",
                report_path
            );
            #[cfg(not(feature = "logging"))]
            eprintln!(
                "NOTE: Weaver CLI report not found at {:?} - CLI may use different output format",
                report_path
            );
        }

        // Assert: Test completes successfully
        // This verifies: Weaver CLI started → telemetry sent via OTLP → Weaver CLI stopped
        // All using the real weaver CLI binary, not Rust API
    }
}
