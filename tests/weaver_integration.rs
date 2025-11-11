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
    use std::path::Path;

    use chicago_tdd_tools::async_test_with_timeout;
    use chicago_tdd_tools::observability::fixtures::{assert_telemetry_valid, WeaverTestFixture};
    use opentelemetry::trace::Tracer as _;
    use opentelemetry::KeyValue;

    fn require_weaver_prerequisites() {
        if !Path::new("registry").exists() {
            panic!(
                "🚨 Registry path does not exist.\n\
                 ⚠️  STOP: Cannot proceed with Weaver integration test\n\
                 💡 FIX: Clone semantic conventions into ./registry"
            );
        }

        use chicago_tdd_tools::observability::weaver::types::WeaverLiveCheck;
        if WeaverLiveCheck::check_weaver_available().is_err() {
            panic!(
                "🚨 Weaver binary not available\n\
                 ⚠️  STOP: Cannot proceed with Weaver integration test\n\
                 💡 FIX: Install Weaver (cargo install weaver) and ensure it is on PATH"
            );
        }
    }

    /// Integration test that exercises the new Weaver fixture end-to-end.
    async_test_with_timeout!(test_weaver_fixture_happy_path, 30, {
        require_weaver_prerequisites();

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
}
