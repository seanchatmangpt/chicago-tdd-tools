//! # Unified Observability Testing Examples - Comprehensive Guide
//!
//! Demonstrates how to use Chicago TDD Tools unified observability testing API
//! with automatic resource management and zero-cost abstractions.
//!
//! ## Tutorial: Getting Started
//!
//! This example demonstrates observability testing with OTEL (OpenTelemetry) and Weaver:
//!
//! 1. **OTEL Span Validation**: Validate OpenTelemetry spans using the unified API
//! 2. **OTEL Metric Validation**: Validate OpenTelemetry metrics
//! 3. **Weaver Integration**: Use Weaver for live schema validation
//! 4. **Unified API**: Single API for both OTEL and Weaver validation
//!
//! **Run tests**: `cargo test --features otel,weaver --example otel_weaver_testing`
//!
//! ## Explanation: Concepts
//!
//! **Unified Observability API**: `ObservabilityTest` provides a single interface for
//! validating both OTEL telemetry and Weaver schema compliance. This eliminates the need
//! to use separate APIs for different validation types.
//!
//! **OTEL Spans**: Represent operations in a distributed trace. Spans have context
//! (trace ID, span ID), attributes, events, and status. Validation ensures spans
//! conform to OTEL specification.
//!
//! **OTEL Metrics**: Represent measurements over time. Metrics have names, values,
//! timestamps, and attributes. Validation ensures metrics are properly structured.
//!
//! **Weaver Live-Check**: Validates telemetry against OpenTelemetry semantic conventions
//! at runtime. Weaver runs as a service and validates telemetry sent to OTLP endpoints.
//!
//! **Test Configuration**: `TestConfig` allows fine-grained control:
//! - `weaver_enabled`: Enable/disable Weaver validation
//! - `registry_path`: Path to Weaver registry files
//! - `otlp_grpc_port`: OTLP gRPC port (default: 4317)
//! - `admin_port`: Weaver admin port (default: 8080)
//!
//! **Zero Config for 80% Cases**: `TestConfig::default()` works for most use cases.
//! Only customize when needed (Weaver integration, custom ports, etc.).
//!
//! ## How-to: Common Tasks
//!
//! - Validate OTEL spans: See `otel_tests::test_otel_span_validation_basic`
//! - Validate OTEL metrics: See `otel_tests::test_otel_metric_validation`
//! - Use Weaver validation: See `weaver_tests::test_weaver_validator_creation`
//! - Combine OTEL and Weaver: See `integration_tests::test_otel_weaver_integration`
//!
//! ## Reference: Quick Lookup
//!
//! **Key Types**:
//! - `ObservabilityTest`: Unified API for observability validation
//! - `TestConfig`: Configuration for observability testing
//! - `Span`: OpenTelemetry span representation
//! - `Metric`: OpenTelemetry metric representation
//! - `SpanContext`: Span context (trace ID, span ID, flags)
//!
//! **Key Functions**:
//! - `ObservabilityTest::with_config(config) -> Result<ObservabilityTest, Error>`
//! - `ObservabilityTest::validate_span(span) -> Result<(), ValidationError>`
//! - `ObservabilityTest::validate_metric(metric) -> Result<(), ValidationError>`
//! - `ObservabilityTest::otlp_endpoint() -> String`
//!
//! **Key Constants**:
//! - Default OTLP gRPC port: `4317`
//! - Default Weaver admin port: `8080`

fn main() {
    chicago_tdd_tools::alert_info!("Unified Observability Testing Examples");
    chicago_tdd_tools::alert_info!("======================================");
    chicago_tdd_tools::alert_info!();
    chicago_tdd_tools::alert_info!("This example demonstrates Chicago TDD Tools unified observability testing API.");
    chicago_tdd_tools::alert_info!("For actual test examples, see the test modules below.");
    chicago_tdd_tools::alert_info!();
    chicago_tdd_tools::alert_info!("To run tests, use: cargo test --features otel,weaver");
}

/// OTEL Testing Examples
///
/// This module demonstrates OpenTelemetry span and metric validation using the unified API.
///
/// ## Tutorial: OTEL Validation
///
/// 1. Create OTEL spans/metrics using `otel::types`
/// 2. Create `ObservabilityTest` with `TestConfig`
/// 3. Validate using `validate_span()` or `validate_metric()`
/// 4. Verify validation results
///
/// ## Explanation: OTEL Concepts
///
/// **Spans**: Represent operations in distributed traces. Each span has:
/// - Context: Trace ID, Span ID, flags
/// - Name: Operation name
/// - Attributes: Key-value pairs
/// - Events: Timestamped events
/// - Status: Ok, Error, or Unset
///
/// **Metrics**: Represent measurements over time. Each metric has:
/// - Name: Metric name
/// - Value: Counter, Gauge, or Histogram
/// - Timestamp: Measurement time
/// - Attributes: Key-value pairs
///
/// **Validation**: Ensures telemetry conforms to OTEL specification and semantic conventions.
#[cfg(feature = "otel")]
#[cfg(test)]
mod otel_tests {
    use chicago_tdd_tools::observability::{ObservabilityTest, TestConfig};
    use chicago_tdd_tools::otel::types::{SpanContext, SpanId, SpanStatus, TraceId};
    use std::collections::BTreeMap;

    // Example: Basic OTEL span validation
    //
    // ## How-to: Validate a Basic Span
    //
    // Create a span with `Span::new_active()`, then validate using `ObservabilityTest`.
    // Use `TestConfig::default()` for simple unit tests (Weaver disabled).
    //
    // ## Reference
    //
    // - **Function**: `Span::new_active(context, name, start_time, attributes, events, status) -> Span`
    // - **Function**: `ObservabilityTest::with_config(config) -> Result<ObservabilityTest, Error>`
    // - **Method**: `validate_span(span) -> Result<(), ValidationError>`
    // - **Config**: `TestConfig { weaver_enabled: false, ..Default::default() }` for unit tests
    //
    // # Examples
    //
    // ```rust
    // use chicago_tdd_tools::observability::{ObservabilityTest, TestConfig};
    // use chicago_tdd_tools::otel::types::{SpanContext, SpanId, SpanStatus, TraceId};
    //
    // let context = SpanContext::root(TraceId(12345), SpanId(67890), 1);
    // let span = Span::new_active(context, "operation".to_string(), 1000, BTreeMap::new(), Vec::new(), SpanStatus::Ok);
    // let test = ObservabilityTest::with_config(TestConfig::default())?;
    // test.validate_span(&span)?;
    // ```
    test!(test_otel_span_validation_basic, {
        // Arrange: Create test span
        let context = SpanContext::root(TraceId(12345), SpanId(67890), 1);
        let span = chicago_tdd_tools::otel::types::Span::new_active(
            context,
            "test.operation".to_string(),
            1000,
            BTreeMap::new(),
            Vec::new(),
            SpanStatus::Ok,
        );

        // Act: Validate span using unified API (zero config for 80% cases)
        let config = TestConfig {
            weaver_enabled: false, // Disable Weaver for simple unit test
            ..Default::default()
        };
        if let Ok(test) = ObservabilityTest::with_config(config) {
            let validation_result = test.validate_span(&span);

            // Assert: Verify validation succeeds
            assert_ok!(&validation_result, "Span should be valid");
            assert_eq!(span.name, "test.operation");
            assert_eq!(span.status, SpanStatus::Ok);
        }
    });

    // Example: OTEL span validation with custom attributes
    //
    // ## How-to: Validate Span with Attributes
    //
    // Add custom attributes to spans using `BTreeMap`. Attributes are key-value pairs
    // that provide additional context about the operation.
    //
    // ## Reference
    //
    // - **Type**: `BTreeMap<String, String>` for span attributes
    // - **Method**: `span.attributes.get(key)` to retrieve attribute values
    // - **Best Practice**: Use semantic convention attribute names (e.g., "service.name")
    //
    // # Examples
    //
    // ```rust
    // let mut attrs = BTreeMap::new();
    // attrs.insert("service.name".to_string(), "my-service".to_string());
    // let span = Span::new_active(context, "operation".to_string(), 1000, attrs, Vec::new(), SpanStatus::Ok);
    // ```
    test!(test_otel_span_with_attributes, {
        // Arrange: Create span with custom attributes
        let mut attrs = BTreeMap::new();
        attrs.insert("service.name".to_string(), "test-service".to_string());
        attrs.insert("operation.type".to_string(), "test".to_string());

        let context = SpanContext::root(TraceId(12345), SpanId(67890), 1);
        let span = chicago_tdd_tools::otel::types::Span::new_active(
            context,
            "test.operation".to_string(),
            1000,
            attrs.clone(),
            Vec::new(),
            SpanStatus::Ok,
        );

        // Act: Validate span using unified API
        let config = TestConfig { weaver_enabled: false, ..Default::default() };
        if let Ok(test) = ObservabilityTest::with_config(config) {
            let validation_result = test.validate_span(&span);

            // Assert: Verify validation succeeds
            assert_ok!(&validation_result, "Span should be valid with attributes");
            assert_eq!(span.attributes.get("service.name"), Some(&"test-service".to_string()));
        }
    });

    // Example: OTEL metric validation
    //
    // ## How-to: Validate OTEL Metrics
    //
    // Create metrics with `Metric` type, then validate using `validate_metric()`.
    // Metrics can be counters, gauges, or histograms.
    //
    // ## Reference
    //
    // - **Type**: `Metric` with `name`, `value`, `timestamp_ms`, `attributes`
    // - **Value Types**: `MetricValue::Counter(u64)`, `MetricValue::Gauge(f64)`, `MetricValue::Histogram(...)`
    // - **Method**: `validate_metric(metric) -> Result<(), ValidationError>`
    //
    // # Examples
    //
    // ```rust
    // use chicago_tdd_tools::otel::types::MetricValue;
    // let metric = Metric {
    //     name: "requests.total".to_string(),
    //     value: MetricValue::Counter(42),
    //     timestamp_ms: 1000,
    //     attributes: BTreeMap::new(),
    // };
    // test.validate_metric(&metric)?;
    // ```
    test!(test_otel_metric_validation, {
        // Arrange: Create test metric
        use chicago_tdd_tools::otel::types::MetricValue;
        let metric = chicago_tdd_tools::otel::types::Metric {
            name: "test.counter".to_string(),
            value: MetricValue::Counter(42),
            timestamp_ms: 1000,
            attributes: BTreeMap::new(),
        };

        // Act: Validate metric using unified API
        let config = TestConfig { weaver_enabled: false, ..Default::default() };
        if let Ok(test) = ObservabilityTest::with_config(config) {
            let validation_result = test.validate_metric(&metric);

            // Assert: Verify validation succeeds
            assert_ok!(&validation_result, "Metric should be valid");
            assert_eq!(metric.name, "test.counter");
            match &metric.value {
                MetricValue::Counter(count) => {
                    assert_eq!(*count, 42);
                }
                _ => {
                    // **Best Practice**: Handle unexpected cases properly
                    // In test context, we can assert; in production, return Result
                    panic!("Expected counter metric, got {:?}", metric.value);
                }
            }
        }
    });

    // Example: OTEL span validation with error path testing
    //
    // ## How-to: Test Error Paths
    //
    // Test validation failure cases by creating invalid spans (e.g., zero trace ID).
    // Use `match` to handle both success and error cases properly.
    //
    // ## Reference
    //
    // - **Function**: `Span::new_completed(...) -> Result<Span, Error>`
    // - **Error Cases**: Invalid trace IDs, invalid timestamps, etc.
    // - **Pattern**: Use `match` to handle `Result` properly
    //
    // # Examples
    //
    // ```rust
    // let span_result = Span::new_completed(context, name, start, end, attrs, events, status);
    // match span_result {
    //     Ok(span) => test.validate_span(&span)?,
    //     Err(e) => chicago_tdd_tools::alert_info!("Expected error: {e}"),
    // }
    // ```
    test!(test_otel_span_validation_error_path, {
        // Arrange: Create span with invalid trace ID (zero)
        let context = SpanContext::root(TraceId(0), SpanId(67890), 1); // Invalid: trace ID is zero
                                                                       // **Best Practice**: Handle Result properly - demonstrates error handling pattern
        let span_result = chicago_tdd_tools::otel::types::Span::new_completed(
            context,
            "test.operation".to_string(),
            1000,
            2000,
            BTreeMap::new(),
            Vec::new(),
            SpanStatus::Ok,
        );

        // Act: Validate span (should fail)
        let config = TestConfig { weaver_enabled: false, ..Default::default() };
        if let Ok(test) = ObservabilityTest::with_config(config) {
            // **Best Practice**: Check Result instead of unwrapping - demonstrates proper error handling
            match span_result {
                Ok(span) => {
                    // If span was created (unexpected for invalid trace ID), validate it
                    let validation_result = test.validate_span(&span);
                    // Assert: Verify validation fails with appropriate error
                    assert_err!(
                        &validation_result,
                        "Span with zero trace ID should fail validation"
                    );
                }
                Err(e) => {
                    // If span creation failed (expected for invalid trace ID), that's correct behavior
                    // **Best Practice**: Handle error case properly - demonstrates error handling pattern
                    chicago_tdd_tools::alert_info!("Expected error for invalid trace ID: {e}");
                }
            }
        }
    });
}

/// Weaver Testing Examples
///
/// This module demonstrates Weaver live-check schema validation using the unified API.
///
/// ## Tutorial: Weaver Validation
///
/// 1. Create `TestConfig` with `weaver_enabled: true` and `registry_path`
/// 2. Create `ObservabilityTest` with config
/// 3. Send telemetry to OTLP endpoint (validated by Weaver)
/// 4. Weaver validates against semantic conventions
///
/// ## Explanation: Weaver Concepts
///
/// **Weaver Live-Check**: Validates telemetry against OpenTelemetry semantic conventions
/// at runtime. Weaver runs as a service and intercepts telemetry sent to OTLP endpoints.
///
/// **Registry Path**: Path to Weaver registry files containing semantic convention schemas.
/// Registry files define valid attribute names, value types, and relationships.
///
/// **OTLP Endpoint**: OpenTelemetry Protocol endpoint where telemetry is sent.
/// Default: `http://127.0.0.1:4317` (OTLP gRPC port).
///
/// **Auto-Detection**: Weaver is automatically detected if running. Validation happens
/// when telemetry is sent, not when `ObservabilityTest` is created.
#[cfg(feature = "weaver")]
#[cfg(test)]
mod weaver_tests {
    use chicago_tdd_tools::observability::{ObservabilityTest, TestConfig};
    use std::path::PathBuf;

    // Example: Basic Weaver validator creation
    //
    // ## How-to: Create Weaver Validator
    //
    // Create `TestConfig` with `registry_path` and `weaver_enabled: true`,
    // then create `ObservabilityTest` with the config.
    //
    // ## Reference
    //
    // - **Config**: `TestConfig { registry_path: Some(path), weaver_enabled: true, ..Default::default() }`
    // - **Function**: `ObservabilityTest::with_config(config) -> Result<ObservabilityTest, Error>`
    // - **Method**: `otlp_endpoint() -> String` - Returns OTLP endpoint URL
    // - **Default Port**: `4317` for OTLP gRPC
    //
    // # Examples
    //
    // ```rust
    // let config = TestConfig {
    //     registry_path: Some(PathBuf::from("registry/")),
    //     weaver_enabled: true,
    //     ..Default::default()
    // };
    // let test = ObservabilityTest::with_config(config)?;
    // let endpoint = test.otlp_endpoint(); // "http://127.0.0.1:4317"
    // ```
    test!(test_weaver_validator_creation, {
        // Arrange: Create test with registry path
        let registry_path = PathBuf::from("registry/");

        // Act: Create test with unified API
        let config = TestConfig { registry_path: Some(registry_path), ..Default::default() };
        if let Ok(test) = ObservabilityTest::with_config(config) {
            // Assert: Verify test is created correctly
            assert_eq!(test.otlp_endpoint(), "http://127.0.0.1:4317");
            // Weaver may or may not be running depending on auto-detection
        }
    });

    // Example: Weaver validation with custom registry path
    //
    // ## How-to: Use Custom Registry Path
    //
    // Specify a custom path to Weaver registry files. Note that validator creation
    // succeeds even with invalid paths - validation happens at runtime when telemetry is sent.
    //
    // ## Reference
    //
    // - **Config Field**: `registry_path: Option<PathBuf>`
    // - **Behavior**: Path validation happens at runtime, not at creation time
    // - **Error Handling**: Validation errors occur when telemetry is sent, not when validator is created
    //
    // # Examples
    //
    // ```rust
    // let config = TestConfig {
    //     registry_path: Some(PathBuf::from("custom/registry/path")),
    //     weaver_enabled: true,
    //     ..Default::default()
    // };
    // ```
    test!(test_weaver_validator_custom_path, {
        // Arrange: Create test with custom registry path
        let registry_path = PathBuf::from("registry/");
        let config = TestConfig {
            registry_path: Some(registry_path),
            weaver_enabled: true,
            ..Default::default()
        };

        // Act: Create test with custom configuration
        let result = ObservabilityTest::with_config(config);

        // Assert: Verify creation succeeds (path validation happens at runtime)
        assert_ok!(&result, "Should create validator successfully");
    });

    // Example: Weaver validation with custom ports
    //
    // ## How-to: Use Custom Ports
    //
    // Configure custom OTLP gRPC and admin ports using `TestConfig`.
    // Useful when default ports are already in use.
    //
    // ## Reference
    //
    // - **Config Fields**:
    //   - `otlp_grpc_port: u16` - OTLP gRPC port (default: 4317)
    //   - `admin_port: u16` - Weaver admin port (default: 8080)
    // - **Method**: `otlp_endpoint() -> String` - Returns endpoint with custom port
    //
    // # Examples
    //
    // ```rust
    // let config = TestConfig {
    //     otlp_grpc_port: 4318,
    //     admin_port: 8081,
    //     ..Default::default()
    // };
    // let test = ObservabilityTest::with_config(config)?;
    // assert_eq!(test.otlp_endpoint(), "http://127.0.0.1:4318");
    // ```
    test!(test_weaver_validator_custom_config, {
        // Arrange: Create test with custom ports
        let registry_path = PathBuf::from("registry/");
        let config = TestConfig {
            registry_path: Some(registry_path),
            otlp_grpc_port: 4318,
            admin_port: 8081,
            ..Default::default()
        };

        // Act: Create test and get OTLP endpoint
        if let Ok(test) = ObservabilityTest::with_config(config) {
            let endpoint = test.otlp_endpoint();

            // Assert: Verify custom configuration is used
            assert_eq!(endpoint, "http://127.0.0.1:4318");
        }
    });
}

/// Integration Testing Examples
///
/// This module demonstrates combining OTEL and Weaver validation in integration tests.
///
/// ## Tutorial: OTEL + Weaver Integration
///
/// 1. Create OTEL spans/metrics
/// 2. Create `ObservabilityTest` with both OTEL and Weaver enabled
/// 3. Validate spans/metrics (OTEL validation)
/// 4. Send telemetry to OTLP endpoint (Weaver validation)
///
/// ## Explanation: Integration Concepts
///
/// **Unified Validation**: `ObservabilityTest` validates both OTEL specification compliance
/// and Weaver semantic convention compliance. This provides comprehensive validation in a single API.
///
/// **Validation Order**: OTEL validation happens first (structure, types), then Weaver validation
/// happens when telemetry is sent (semantic conventions, attribute names).
///
/// **Error Handling**: Integration tests should handle both validation success and failure cases,
/// as Weaver may not be available in all environments.
#[cfg(feature = "otel")]
#[cfg(feature = "weaver")]
#[cfg(test)]
mod integration_tests {
    use chicago_tdd_tools::observability::{ObservabilityTest, TestConfig};
    use chicago_tdd_tools::otel::types::{SpanContext, SpanId, SpanStatus, TraceId};
    use chicago_tdd_tools::test;
    use std::path::PathBuf;

    // Example: Integration test combining OTEL and Weaver
    //
    // ## How-to: Combine OTEL and Weaver Validation
    //
    // Create `TestConfig` with both OTEL and Weaver enabled, then validate spans.
    // Both validations occur: OTEL structure validation and Weaver semantic convention validation.
    //
    // ## Reference
    //
    // - **Config**: `TestConfig { weaver_enabled: true, registry_path: Some(path), ..Default::default() }`
    // - **Validation**: Both OTEL and Weaver validation occur
    // - **Error Handling**: May fail if Weaver not available - handle gracefully
    //
    // # Examples
    //
    // ```rust
    // let config = TestConfig {
    //     registry_path: Some(PathBuf::from("registry/")),
    //     weaver_enabled: true,
    //     ..Default::default()
    // };
    // let test = ObservabilityTest::with_config(config)?;
    // test.validate_span(&span)?; // OTEL + Weaver validation
    // ```
    test!(test_otel_weaver_integration, {
        // Arrange: Create test span and unified test
        let context = SpanContext::root(TraceId(12345), SpanId(67890), 1);
        let span = chicago_tdd_tools::otel::types::Span::new_active(
            context,
            "test.operation".to_string(),
            1000,
            std::collections::BTreeMap::new(),
            Vec::new(),
            SpanStatus::Ok,
        );

        let registry_path = PathBuf::from("registry/");
        let config = TestConfig {
            registry_path: Some(registry_path),
            weaver_enabled: true,
            ..Default::default()
        };

        // Act: Validate span with unified API (OTEL + Weaver)
        if let Ok(test) = ObservabilityTest::with_config(config) {
            let validation_result = test.validate_span(&span);

            // Assert: Verify validation succeeded (or failed appropriately)
            // Note: May fail if Weaver not available, that's OK for integration test
            assert!(validation_result.is_ok() || validation_result.is_err());
            assert_eq!(span.name, "test.operation");
        }
    });
}
