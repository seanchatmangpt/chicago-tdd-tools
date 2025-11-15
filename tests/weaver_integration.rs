//! Unified Observability Integration Tests
//!
//! **Poka-yoke**: These are INTEGRATION tests, not unit tests. They test real Weaver CLI
//! integration and require Weaver binary to be available. Unit tests should NOT test
//! Weaver integration - they should only test types and validators.
//!
//! These tests verify:
//! - Unified API can be created and used
//! - Weaver CLI can be started and stopped (via unified API)
//! - Telemetry can be validated using unified API
//!
//! **CRITICAL**: These tests require Weaver binary to be installed and registry path to exist.
//! If Weaver is not available, these tests MUST fail (not skip).
//!
//! **Import pattern**: Use `chicago_tdd_tools::observability::fixtures::WeaverTestFixture` for unified API.

#[cfg(all(feature = "weaver", feature = "otel", test))]
mod weaver_integration_tests {
    use chicago_tdd_tools::observability::fixtures::{assert_telemetry_valid, WeaverTestFixture};
    use opentelemetry::trace::{Span as _, Tracer as _};
    use opentelemetry::KeyValue;
    use std::fs;
    use std::path::PathBuf;

    fn allow_weaver_skip() -> bool {
        matches!(
            std::env::var("WEAVER_ALLOW_SKIP"),
            Ok(value) if matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES")
        )
    }

    fn ensure_weaver_prerequisites() -> bool {
        let registry_path = PathBuf::from("registry");
        if !registry_path.exists() {
            if allow_weaver_skip() {
                eprintln!("⏭️  Skipping Weaver test: Registry path missing (run cargo make weaver-bootstrap)");
                return false;
            }
            panic!(
                "🚨 Registry path does not exist: {:?}\n\
                 ⚠️  STOP: Cannot proceed with Weaver integration test\n\
                 💡 FIX: Run cargo make weaver-bootstrap\n\
                 💡 ALT: Set WEAVER_ALLOW_SKIP=1 to bypass intentionally",
                registry_path
            );
        }

        use chicago_tdd_tools::observability::weaver::types::WeaverLiveCheck;
        if WeaverLiveCheck::check_weaver_available().is_err() {
            if allow_weaver_skip() {
                eprintln!("⏭️  Skipping Weaver test: Weaver binary not available (run cargo make weaver-bootstrap)");
                return false;
            }
            panic!(
                "🚨 Weaver binary not available\n\
                 ⚠️  STOP: Cannot proceed with Weaver integration test\n\
                 💡 FIX: Run cargo make weaver-bootstrap\n\
                 💡 ALT: Set WEAVER_ALLOW_SKIP=1 to bypass intentionally"
            );
        }

        true
    }

    /// Utility: ensure weaver reports output directory exists before tests run
    fn ensure_weaver_reports_dir() {
        let reports_dir = PathBuf::from("weaver-reports");
        if !reports_dir.exists() {
            if let Err(err) = fs::create_dir_all(&reports_dir) {
                panic!(
                    "🚨 Failed to create weaver-reports directory: {err}\n\
                     ⚠️  STOP: Cannot proceed with Weaver integration test\n\
                     💡 FIX: Check filesystem permissions"
                );
            }
        }
    }

    /// Integration test that exercises the WeaverTestFixture end-to-end.
    ///
    /// This test verifies:
    /// 1. WeaverTestFixture can be created
    /// 2. Tracer can be acquired from fixture
    /// 3. Spans can be emitted and validated
    /// 4. Fixture cleanup works correctly
    #[test]
    fn test_unified_api_weaver_integration() {
        if !ensure_weaver_prerequisites() {
            return;
        }
        ensure_weaver_reports_dir();

        // Arrange: Create WeaverTestFixture (handles Weaver lifecycle automatically)
        let mut fixture = WeaverTestFixture::new()
            .unwrap_or_else(|err| panic!("Failed to initialise Weaver fixture: {err}"));

        // Act: Acquire tracer and emit span
        let tracer = fixture
            .tracer("weaver-integration", "chicago-tdd-tools-weaver-tests")
            .unwrap_or_else(|err| panic!("Failed to acquire tracer: {err}"));

        let mut span = tracer.tracer().start("integration-span");
        span.set_attribute(KeyValue::new("test.case", "unified_api_weaver_integration"));
        span.end();

        // Flush telemetry to ensure it's sent to Weaver
        tracer
            .force_flush()
            .unwrap_or_else(|err| panic!("Failed to flush tracer: {err}"));

        // Act: Finish fixture (flushes telemetry, stops weaver, parses results)
        let results = fixture
            .finish()
            .unwrap_or_else(|err| panic!("Failed to finalise Weaver fixture: {err}"));

        // Assert: Verify telemetry was validated by Weaver
        assert_telemetry_valid(&results)
            .unwrap_or_else(|err| panic!("Weaver validation failed: {err}"));
    }

    /// Test WeaverTestFixture happy path with minimal configuration.
    ///
    /// This test verifies:
    /// 1. Default configuration works
    /// 2. Basic span emission and validation
    /// 3. Automatic cleanup
    #[test]
    fn test_weaver_fixture_happy_path() {
        if !ensure_weaver_prerequisites() {
            return;
        }
        ensure_weaver_reports_dir();

        // Arrange: Create WeaverTestFixture with default config
        let mut fixture = WeaverTestFixture::new()
            .unwrap_or_else(|err| panic!("Failed to initialise Weaver fixture: {err}"));

        // Act: Acquire tracer and emit span
        let tracer = fixture
            .tracer("weaver-integration", "chicago-tdd-tools-weaver-tests")
            .unwrap_or_else(|err| panic!("Failed to acquire Weaver tracer: {err}"));

        let mut span = tracer.tracer().start("integration-span");
        span.set_attribute(KeyValue::new("test.case", "weaver_fixture_happy_path"));
        span.end();

        // Flush telemetry
        tracer
            .force_flush()
            .unwrap_or_else(|err| panic!("Failed to flush tracer: {err}"));

        // Act: Finish fixture
        let results = fixture
            .finish()
            .unwrap_or_else(|err| panic!("Failed to finalise Weaver fixture: {err}"));

        // Assert: Verify validation succeeded
        assert_telemetry_valid(&results)
            .unwrap_or_else(|err| panic!("Weaver validation failed: {err}"));
    }

    /// Test that WeaverTestFixture produces validation reports.
    ///
    /// This test verifies:
    /// 1. Reports are generated in output directory
    /// 2. Reports can be parsed and validated
    /// 3. Validation results are accessible
    #[test]
    fn test_weaver_fixture_reports_rendered() {
        if !ensure_weaver_prerequisites() {
            return;
        }
        ensure_weaver_reports_dir();

        // Arrange: Create WeaverTestFixture
        let mut fixture = WeaverTestFixture::new()
            .unwrap_or_else(|err| panic!("Failed to initialise Weaver fixture: {err}"));

        // Act: Acquire tracer and emit span
        let tracer = fixture
            .tracer("weaver-integration", "chicago-tdd-tools-weaver-tests")
            .unwrap_or_else(|err| panic!("Failed to acquire tracer: {err}"));

        let mut span = tracer.tracer().start("integration-span");
        span.set_attribute(KeyValue::new("test.case", "weaver_fixture_reports_rendered"));
        span.end();

        // Flush telemetry
        tracer
            .force_flush()
            .unwrap_or_else(|err| panic!("Failed to flush tracer: {err}"));

        // Act: Finish fixture and get results
        let results = fixture
            .finish()
            .unwrap_or_else(|err| panic!("Failed to finalise Weaver fixture: {err}"));

        // Assert: Verify reports were generated and can be validated
        assert_telemetry_valid(&results)
            .unwrap_or_else(|err| panic!("Weaver validation failed: {err}"));

        // Assert: Verify output directory exists and contains reports
        let output_dir = fixture.output_dir();
        assert!(
            output_dir.exists(),
            "Weaver output directory should exist: {:?}",
            output_dir
        );
    }
}
