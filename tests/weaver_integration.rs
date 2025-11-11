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
//! **Import pattern**: Use `chicago_tdd_tools::observability::ObservabilityTest` for unified API.

#[cfg(all(feature = "weaver", feature = "otel", test))]
mod weaver_integration_tests {
    use chicago_tdd_tools::assert_ok;
    use chicago_tdd_tools::assertions::assert_that;
    use chicago_tdd_tools::async_test;
    use chicago_tdd_tools::observability::{ObservabilityTest, TestConfig};
    use chicago_tdd_tools::otel::types::{SpanContext, SpanId, SpanStatus, TraceId};
    use std::fs;
    use std::path::PathBuf;
    use std::time::Duration;
    use tokio::time::sleep;

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

    /// Utility: ensure weaver smoke output directory exists before tests run
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

    /// Integration test that exercises the new Weaver fixture end-to-end.
    async_test!(test_unified_api_weaver_integration, {
        if !ensure_weaver_prerequisites() {
            return;
        }
        ensure_weaver_reports_dir();

        let tracer = ObservabilityTest::new(TestConfig::default())
            .map_err(|err| format!("Failed to initialise ObservabilityTest: {err}"))?;

        let mut span = tracer.tracer().start("integration-span");
        span.set_attribute(KeyValue::new("test.case", "weaver_fixture_happy_path"));
        span.end();

        tracer.force_flush().map_err(|err| format!("Failed to flush tracer: {err}"))?;

        let results = tracer
            .finish()
            .map_err(|err| format!("Failed to finalise ObservabilityTest: {err}"))?;

        assert_telemetry_valid(&results).map_err(|err| format!("Weaver validation failed: {err}"))
    });

    async_test_with_timeout!(test_weaver_fixture_happy_path, 30, {
        if !ensure_weaver_prerequisites() {
            return;
        }
        ensure_weaver_reports_dir();

        let mut fixture = WeaverTestFixture::new()
            .map_err(|err| format!("Failed to initialise Weaver fixture: {err}"))?;

        let tracer = fixture
            .tracer("weaver-integration", "chicago-tdd-tools-weaver-tests")
            .map_err(|err| format!("Failed to acquire Weaver tracer: {err}"))?;

        let mut span = tracer.tracer().start("integration-span");
        span.set_attribute(KeyValue::new("test.case", "weaver_fixture_happy_path"));
        span.end();

        tracer.force_flush().map_err(|err| format!("Failed to flush tracer: {err}"))?;

        let results = fixture
            .finish()
            .map_err(|err| format!("Failed to finalise Weaver fixture: {err}"))?;

        assert_telemetry_valid(&results).map_err(|err| format!("Weaver validation failed: {err}"))
    });

    async_test_with_timeout!(test_weaver_fixture_reports_rendered, 30, {
        if !ensure_weaver_prerequisites() {
            return;
        }
        ensure_weaver_reports_dir();

        let tracer = ObservabilityTest::new(TestConfig::default())
            .map_err(|err| format!("Failed to initialise ObservabilityTest: {err}"))?;

        let mut span = tracer.tracer().start("integration-span");
        span.set_attribute(KeyValue::new("test.case", "weaver_fixture_reports_rendered"));
        span.end();

        tracer.force_flush().map_err(|err| format!("Failed to flush tracer: {err}"))?;

        let results = tracer
            .finish()
            .map_err(|err| format!("Failed to finalise ObservabilityTest: {err}"))?;

        assert_telemetry_valid(&results).map_err(|err| format!("Weaver validation failed: {err}"))
    });
}
