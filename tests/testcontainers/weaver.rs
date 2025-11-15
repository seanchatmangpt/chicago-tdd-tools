/// Testcontainers verification tests for Weaver integration
///
/// These tests verify that Weaver integration works correctly using testcontainers.
/// This demonstrates Chicago TDD principles:
/// - Real Collaborators: Actual Weaver container, not mocks
/// - State Verification: Verify Weaver binary works in container
/// - Behavior Verification: Verify Weaver can execute commands
///
/// Note: These tests require Docker to be running and both testcontainers and weaver features enabled.

#[cfg(all(feature = "testcontainers", feature = "weaver", test))]
mod weaver_tests {
    mod common {
        include!("../test_common.inc");
    }
    use chicago_tdd_tools::{async_test, test};
    use chicago_tdd_tools::assertions::assert_that_with_msg;
    use chicago_tdd_tools::{assert_eq_msg, assert_ok};
    use chicago_tdd_tools::testcontainers::*;
    use common::require_docker;
    use std::collections::HashMap;
    
    // Macros exported via #[macro_export] need to be used with full path in nested modules
    #[allow(unused_macros)] // Macro may be used in future tests
    macro_rules! assert_err {
        ($($args:tt)*) => {
            chicago_tdd_tools::assert_err!($($args)*)
        };
    }

    fn allow_weaver_skip() -> bool {
        matches!(
            std::env::var("WEAVER_ALLOW_SKIP"),
            Ok(value) if matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES")
        )
    }

    fn handle_prereq_failure(message: &str) -> bool {
        if allow_weaver_skip() {
            eprintln!("⏭️  Skipping Weaver test: {message} (WEAVER_ALLOW_SKIP set)");
            true
        } else {
            panic!("{message}\nSet WEAVER_ALLOW_SKIP=1 to bypass Weaver tests explicitly.");
        }
    }

    // Kaizen improvement: Extract repeated Docker image names to constants
    // Pattern: Use named constants for repeated string literals to improve maintainability
    const WEAVER_IMAGE: &str = "otel/weaver";
    const WEAVER_TAG: &str = "latest";
    const PYTHON_IMAGE: &str = "python";
    const PYTHON_TAG: &str = "3-slim";

    // Test that Weaver Docker image is available and can execute commands
    //
    // This test verifies:
    // 1. Weaver Docker image can be pulled and run
    // 2. Weaver binary exists in the container
    // 3. Weaver can execute basic commands (--version)
    //
    // This is Chicago TDD: Real Collaborators (actual Docker container),
    // State Verification (weaver binary exists), Behavior Verification (weaver works)
    test!(weaver_container_available, {
        require_docker();

        // Arrange: Create Weaver container with entrypoint override to keep it running
        // **Root Cause Fix**: otel/weaver image has entrypoint [/weaver/weaver] that interferes with custom commands.
        // When entrypoint is /bin/sh, the command should be -c and args should be the script.
        // Docker will execute: /bin/sh -c "sleep infinity"
        let client = ContainerClient::new();
        let container = GenericContainer::with_command(
            client.client(),
            WEAVER_IMAGE,
            WEAVER_TAG,
            "-c",
            &["sleep infinity"],
            Some(&["/bin/sh"]),
        )
        .unwrap_or_else(|e| panic!("Failed to create Weaver container: {}", e));

        // Wait for container to be ready
        std::thread::sleep(std::time::Duration::from_millis(1000));

        // Act: Execute weaver --version in container
        let result = container.exec("weaver", &["--version"]);

        // Assert: Verify Weaver works (state verification)
        assert_ok!(&result, "Weaver should execute --version successfully");
        let exec_result = result.expect("Exec should succeed after assert_ok");
        assert_eq_msg!(&exec_result.exit_code, &0, "Weaver --version should succeed");
        assert_that_with_msg(
            &(exec_result.stdout.contains("weaver") || exec_result.stdout.contains("Weaver")),
            |v| *v,
            "Weaver version output should contain 'weaver' or 'Weaver'"
        );
    });

    // Test that Weaver can execute registry check command in container
    //
    // This test verifies:
    // 1. Weaver can execute registry commands
    // 2. Weaver provides helpful error messages when registry not found
    //
    // This is Chicago TDD: Behavior Verification (weaver command behavior),
    // Error Path Testing (80% of bugs)
    test!(weaver_container_registry_check, {
        require_docker();

        // Arrange: Create Weaver container with entrypoint override
        let client = ContainerClient::new();
        let container = GenericContainer::with_command(
            client.client(),
            WEAVER_IMAGE,
            WEAVER_TAG,
            "-c",
            &["sleep infinity"],
            Some(&["/bin/sh"]),
        )
        .unwrap_or_else(|e| panic!("Failed to create Weaver container: {}", e));

        // Wait for container to be ready
        std::thread::sleep(std::time::Duration::from_millis(1000));

        // Act: Execute weaver registry check with non-existent registry
        let result = container.exec("weaver", &["registry", "check", "-r", "/nonexistent/registry"]);

        // Assert: Verify Weaver provides helpful error (behavior verification)
        assert_ok!(&result, "Weaver should execute command (even if it fails)");
        let exec_result = result.expect("Exec should succeed after assert_ok");
        // Weaver should return non-zero exit code for invalid registry
        assert_that_with_msg(
            &exec_result.exit_code,
            |v| *v != 0,
            "Weaver should fail with invalid registry"
        );
        // Verify error message is helpful (behavior verification)
        // Accept any error output - weaver may format errors differently
        assert_that_with_msg(
            &(!exec_result.stderr.is_empty() || !exec_result.stdout.is_empty()),
            |v| *v,
            "Weaver should provide error output"
        );
    });

    // Test that Weaver container can be used for live-check verification
    //
    // This test verifies:
    // 1. Weaver container has weaver binary available
    // 2. Weaver can be used for integration testing
    //
    // This is Chicago TDD: Real Collaborators (actual Weaver container),
    // Working Capability Testing (verify Weaver works in containerized environment)
    test!(weaver_container_live_check_capability, {
        require_docker();

        // Arrange: Create Weaver container with entrypoint override
        let client = ContainerClient::new();
        let container = GenericContainer::with_command(
            client.client(),
            WEAVER_IMAGE,
            WEAVER_TAG,
            "-c",
            &["sleep infinity"],
            Some(&["/bin/sh"]),
        )
        .unwrap_or_else(|e| panic!("Failed to create Weaver container: {}", e));

        // Wait for container to be ready
        std::thread::sleep(std::time::Duration::from_millis(1000));

        // Act: Check if weaver binary exists and can show help
        let result = container.exec("weaver", &["--help"]);

        // Assert: Verify Weaver is functional (working capability)
        assert_ok!(&result, "Weaver should execute --help successfully");
        let exec_result = result.expect("Exec should succeed after assert_ok");
        // Weaver --help may return 0 or 1 depending on version, so just check it executed
        assert_that_with_msg(
            &(!exec_result.stdout.is_empty() || !exec_result.stderr.is_empty()),
            |v| *v,
            "Weaver should produce output (help text or error)"
        );
    });

    // Test that Weaver live-check validates OTEL telemetry emitted from within a testcontainer
    //
    // This test verifies the complete integration workflow:
    // 1. Weaver live-check runs on host using WeaverTestFixture
    // 2. Container emits OTEL telemetry
    // 3. Container sends telemetry to host's weaver OTLP endpoint
    // 4. Weaver receives and validates the telemetry
    //
    // This is Chicago TDD: Real Collaborators (actual containers, real weaver),
    // Behavior Verification (verify telemetry validation workflow),
    // Integration Testing (end-to-end validation)
    async_test!(test_weaver_live_check_otel_from_container, {
        require_docker();

        // Arrange: Check prerequisites
        let registry_path = std::path::PathBuf::from("registry");
        if !registry_path.exists() && handle_prereq_failure("Registry path does not exist (run cargo make weaver-bootstrap)") {
            return;
        }

        use chicago_tdd_tools::observability::weaver::types::WeaverLiveCheck;
        if WeaverLiveCheck::check_weaver_available().is_err()
            && handle_prereq_failure("Weaver binary not available (run cargo make weaver-bootstrap)")
        {
            return;
        }

        // Arrange: Start Weaver live-check using WeaverTestFixture (modern API)
        // **Root Cause Fix**: Use WeaverTestFixture instead of WeaverValidator to avoid async/blocking conflicts
        // WeaverTestFixture uses ObservabilityTest which handles lifecycle properly
        use chicago_tdd_tools::observability::fixtures::WeaverTestFixture;
        let mut fixture = WeaverTestFixture::new()
            .unwrap_or_else(|err| panic!("Failed to create WeaverTestFixture: {err}"));

        // Get weaver OTLP endpoint from fixture
        let weaver_endpoint = fixture.observability().otlp_endpoint();
        // Extract port from endpoint (format: "http://127.0.0.1:4317")
        let weaver_port = weaver_endpoint
            .trim_start_matches("http://")
            .trim_start_matches("https://")
            .split(':')
            .nth(1)
            .unwrap_or("4317");

        // Determine host address for container to reach host
        // Use host.docker.internal (works on Docker Desktop and modern Linux)
        let host_address = "host.docker.internal";

        // Wait for Weaver to be ready
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        // Arrange: Create Python container with OTEL SDK
        let client = ContainerClient::new();
        let mut env_vars = HashMap::new();
        env_vars.insert(
            "OTEL_EXPORTER_OTLP_ENDPOINT".to_string(),
            format!("http://{}:{}/v1/traces", host_address, weaver_port),
        );
        env_vars.insert("OTEL_SERVICE_NAME".to_string(), "testcontainer-app".to_string());

        let container = GenericContainer::with_env_and_command(
            client.client(),
            PYTHON_IMAGE,
            PYTHON_TAG,
            env_vars,
            Some(("sleep", &["infinity"])),
        )
        .unwrap_or_else(|e| panic!("Failed to create Python container: {}", e));

        // Act: Install OTEL SDK and emit telemetry from container
        // Python script that installs OTEL SDK and emits a span
        let python_script = r#"
import subprocess
import sys

# Install OTEL SDK
subprocess.check_call([
    sys.executable, "-m", "pip", "install", "--quiet",
    "opentelemetry-api", "opentelemetry-sdk", "opentelemetry-exporter-otlp-proto-http"
])

# Emit OTEL telemetry
from opentelemetry import trace
from opentelemetry.exporter.otlp.proto.http.trace_exporter import OTLPSpanExporter
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor
import os

# Get endpoint from environment
endpoint = os.environ.get("OTEL_EXPORTER_OTLP_ENDPOINT", "http://host.docker.internal:4317/v1/traces")

# Create exporter
exporter = OTLPSpanExporter(endpoint=endpoint)

# Create tracer provider
provider = TracerProvider()
processor = BatchSpanProcessor(exporter)
provider.add_span_processor(processor)
trace.set_tracer_provider(provider)

# Create tracer and emit span
tracer = trace.get_tracer(__name__)
with tracer.start_as_current_span("test.operation.from.container") as span:
    span.set_attribute("test.source", "testcontainer")
    span.set_attribute("test.type", "integration")
    span.set_attribute("test.name", "weaver_live_check_otel_from_container")

# Force flush to ensure spans are sent
provider.force_flush()

print("OTEL telemetry emitted successfully")
"#;

        // Execute Python script in container
        let exec_result = container.exec("python", &["-c", python_script]);
        assert_ok!(&exec_result, "Python script should execute successfully");
        let exec_output = exec_result.expect("Exec should succeed after assert_ok");
        assert_eq_msg!(
            &exec_output.exit_code,
            &0,
            "Python script should succeed (exit code 0)"
        );
        assert_that_with_msg(
            &exec_output.stdout.contains("OTEL telemetry emitted successfully"),
            |v| *v,
            "Python script should confirm telemetry emission"
        );

        // Wait for telemetry to be processed by weaver
        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

        // Act: Finish fixture (flushes telemetry and stops weaver)
        // **Root Cause Fix**: WeaverTestFixture handles cleanup automatically via finish()
        // This avoids async/blocking conflicts because ObservabilityTest manages lifecycle properly
        let results = fixture
            .finish()
            .unwrap_or_else(|err| panic!("Failed to finish WeaverTestFixture: {err}"));

        // Assert: Verify weaver received and validated telemetry
        use chicago_tdd_tools::observability::fixtures::assert_telemetry_valid;
        assert_telemetry_valid(&results)
            .unwrap_or_else(|err| panic!("Weaver validation failed: {err}"));

        // Assert: Test completes successfully
        // This verifies: Weaver started → Container emitted OTEL → Container sent to weaver → Weaver stopped
        // All using real containers and real weaver CLI
    });
}
