//! High level Weaver fixture that wires live-check into Chicago TDD tests.

#![cfg(all(feature = "weaver", feature = "otel"))]

use std::path::Path;
use std::time::Duration;

use tempfile::TempDir;

use crate::observability::unified::TestConfig;
use crate::observability::{ObservabilityError, ObservabilityResult, ObservabilityTest};

use super::{TelemetryCapture, TelemetryTracer, ValidationResults};

/// Fixture that manages Weaver live-check lifecycle and telemetry capture.
pub struct WeaverTestFixture {
    observability: ObservabilityTest,
    capture: TelemetryCapture,
    output_dir: TempDir,
}

impl WeaverTestFixture {
    /// Create a new Weaver test fixture with zero configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if Weaver cannot be started or configured.
    pub fn new() -> ObservabilityResult<Self> {
        Self::with_config(TestConfig::default())
    }

    /// Create a new Weaver test fixture with custom configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if Weaver cannot be started or configured.
    pub fn with_config(mut config: TestConfig) -> ObservabilityResult<Self> {
        config.weaver_enabled = true;

        let output_dir = TempDir::new().map_err(|err| {
            ObservabilityError::ValidationFailed(format!(
                "Failed to create temporary Weaver report directory: {err}"
            ))
        })?;

        config.weaver_output_dir = Some(output_dir.path().to_path_buf());

        let observability = ObservabilityTest::with_config(config)?;

        let capture = TelemetryCapture::new(observability.otlp_endpoint());

        Ok(Self { observability, capture, output_dir })
    }

    /// Acquire a tracer that exports spans to the Weaver live-check instance.
    ///
    /// The tracer should be used to instrument the code under test.  When the
    /// fixture is finished all pending telemetry is flushed automatically.
    ///
    /// # Errors
    ///
    /// Returns an error if the tracer cannot be created.
    pub fn tracer(
        &self,
        instrumentation_name: &str,
        service_name: &str,
    ) -> ObservabilityResult<TelemetryTracer> {
        self.capture.tracer(instrumentation_name, service_name)
    }

    /// Access the underlying observability test helper for advanced scenarios.
    #[must_use]
    pub const fn observability(&self) -> &ObservabilityTest {
        &self.observability
    }

    /// Location of the Weaver report directory for this fixture.
    #[must_use]
    pub fn output_dir(&self) -> &Path {
        self.output_dir.path()
    }

    /// Flush captured telemetry, wait for Weaver to produce reports, and parse
    /// the validation output.
    ///
    /// # Errors
    ///
    /// Returns an error if telemetry capture fails, Weaver processing fails,
    /// or validation results cannot be parsed.
    pub fn finish(&self) -> ObservabilityResult<ValidationResults> {
        self.capture.flush()?;

        // Give Weaver a moment to process the final export.
        std::thread::sleep(Duration::from_millis(200));

        ValidationResults::from_report_dir(self.output_dir())
    }
}

impl Default for WeaverTestFixture {
    #[allow(clippy::panic)] // Test fixture - panic is acceptable if initialization fails
    fn default() -> Self {
        // **Kaizen improvement**: Use unwrap_or_else instead of expect for consistency
        Self::new().unwrap_or_else(|err| panic!("Failed to initialise WeaverTestFixture: {err}"))
    }
}
