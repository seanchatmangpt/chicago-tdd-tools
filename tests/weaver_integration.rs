//! Unified Observability Integration Tests
//!
//! **Poka-yoke**: These are INTEGRATION tests, not unit tests. They test real Weaver CLI
//! integration and require Weaver binary to be available. Unit tests should NOT test
//! Weaver integration - they should only test types and validators.
//!
//! **Working Capabilities Tested:**
//! 1. WeaverTestFixture can be created and configured
//! 2. Tracer can be acquired from fixture
//! 3. Spans can be emitted and flushed
//! 4. Fixture cleanup works correctly (via blocking thread pattern)
//! 5. Validation results can be parsed and asserted
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

    /// Working Capability: WeaverTestFixture can be created and used end-to-end
    ///
    /// This test verifies:
    /// 1. WeaverTestFixture can be created (working capability)
    /// 2. Tracer can be acquired from fixture (working capability)
    /// 3. Spans can be emitted and flushed (working capability)
    /// 4. Fixture cleanup works correctly via blocking thread pattern (working capability)
    ///
    /// **Pattern**: Use tokio runtime for force_flush(), then move finish() to blocking thread
    /// to avoid async/blocking conflicts.
    #[tokio::test]
    async fn test_unified_api_weaver_integration() {
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

        // Flush telemetry to ensure it's sent to Weaver (requires tokio runtime)
        tracer
            .force_flush()
            .unwrap_or_else(|err| panic!("Failed to flush tracer: {err}"));

        // Wait for telemetry to be processed
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Act: Finish fixture (flushes telemetry, stops weaver, parses results)
        // **TRIZ Solution**: Use finish_async() which handles async/blocking context switching internally
        // This eliminates the need for manual thread spawning (Principle #13: The Other Way Round)
        drop(tracer); // Drop tracer before finishing fixture
        let results = fixture
            .finish_async()
            .await
            .unwrap_or_else(|err| panic!("Failed to finalise Weaver fixture: {err}"));

        // Assert: Verify telemetry was validated by Weaver (working capability)
        assert_telemetry_valid(&results)
            .unwrap_or_else(|err| panic!("Weaver validation failed: {err}"));
    }

    /// Working Capability: WeaverTestFixture happy path with minimal configuration
    ///
    /// This test verifies:
    /// 1. Default configuration works (working capability)
    /// 2. Basic span emission and validation (working capability)
    /// 3. Automatic cleanup via blocking thread pattern (working capability)
    #[tokio::test]
    async fn test_weaver_fixture_happy_path() {
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

        // Flush telemetry (requires tokio runtime)
        tracer
            .force_flush()
            .unwrap_or_else(|err| panic!("Failed to flush tracer: {err}"));

        // Wait for telemetry to be processed
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Act: Finish fixture using async-aware method
        // **TRIZ Solution**: finish_async() handles async/blocking context switching internally
        drop(tracer); // Drop tracer before finishing fixture
        let results = fixture
            .finish_async()
            .await
            .unwrap_or_else(|err| panic!("Failed to finalise Weaver fixture: {err}"));

        // Assert: Verify validation succeeded (working capability)
        assert_telemetry_valid(&results)
            .unwrap_or_else(|err| panic!("Weaver validation failed: {err}"));
    }

    /// Working Capability: WeaverTestFixture produces validation reports
    ///
    /// This test verifies:
    /// 1. Reports are generated in output directory (working capability)
    /// 2. Reports can be parsed and validated (working capability)
    /// 3. Validation results are accessible (working capability)
    ///
    /// **Pattern**: Use blocking sleep and clone output_dir before moving fixture
    /// to avoid borrow-after-move errors.
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

        // Flush telemetry (requires tokio runtime, but we're in blocking test)
        // **Working Capability Pattern**: Use tokio::runtime::Runtime for blocking context
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            tracer
                .force_flush()
                .unwrap_or_else(|err| panic!("Failed to flush tracer: {err}"));
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        });

        // Wait for telemetry to be processed (blocking sleep for non-async test)
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Act: Finish fixture and get results (uses blocking operations, move to blocking thread)
        drop(tracer); // Drop tracer before moving fixture
                      // **Poka-yoke**: Clone output_dir path before moving fixture (prevent borrow-after-move)
        let output_dir = fixture.output_dir().to_path_buf();
        let results = {
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let result = fixture.finish();
                tx.send(result).unwrap();
            });
            rx.recv().unwrap()
        }
        .unwrap_or_else(|err| panic!("Failed to finalise Weaver fixture: {err}"));

        // Assert: Verify reports were generated and can be validated (working capability)
        assert_telemetry_valid(&results)
            .unwrap_or_else(|err| panic!("Weaver validation failed: {err}"));

        // Assert: Verify output directory exists and contains reports (working capability)
        assert!(output_dir.exists(), "Weaver output directory should exist: {:?}", output_dir);
    }
}
