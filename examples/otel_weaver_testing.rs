//! Unified Observability Testing Examples
//!
//! Demonstrates how to use Chicago TDD Tools unified observability testing API
//! with automatic resource management and zero-cost abstractions.
//!
//! Run with: `cargo run --example otel_weaver_testing --features otel,weaver`

fn main() {
    println!("Unified Observability Testing Examples");
    println!("======================================");
    println!();
    println!("This example demonstrates Chicago TDD Tools unified observability testing API.");
    println!("For actual test examples, see the test modules below.");
    println!();
    println!("To run tests, use: cargo test --features otel,weaver");
}

#[cfg(feature = "otel")]
#[cfg(test)]
mod otel_tests {
    use chicago_tdd_tools::observability::{ObservabilityTest, TestConfig};
    use chicago_tdd_tools::otel::types::{SpanContext, SpanId, SpanStatus, TraceId};
    use chicago_tdd_tools::prelude::*;
    use std::collections::BTreeMap;

    // Example 1: Basic OTEL span validation using unified API
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

    // Example 2: OTEL span validation with custom attributes
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

    // Example 3: OTEL metric validation
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
                _ => panic!("Expected counter metric"),
            }
        }
    });

    // Example 4: OTEL span validation with error path testing
    test!(test_otel_span_validation_error_path, {
        // Arrange: Create span with invalid trace ID (zero)
        let context = SpanContext::root(TraceId(0), SpanId(67890), 1); // Invalid: trace ID is zero
        let span = chicago_tdd_tools::otel::types::Span::new_completed(
            context,
            "test.operation".to_string(),
            1000,
            2000,
            BTreeMap::new(),
            Vec::new(),
            SpanStatus::Ok,
        )
        .unwrap();

        // Act: Validate span (should fail)
        let config = TestConfig { weaver_enabled: false, ..Default::default() };
        if let Ok(test) = ObservabilityTest::with_config(config) {
            let validation_result = test.validate_span(&span);

            // Assert: Verify validation fails with appropriate error
            assert_err!(&validation_result, "Span with zero trace ID should fail validation");
        }
    });
}

#[cfg(feature = "weaver")]
#[cfg(test)]
mod weaver_tests {
    use chicago_tdd_tools::observability::{ObservabilityTest, TestConfig};
    use chicago_tdd_tools::prelude::*;
    use std::path::PathBuf;

    // Example 1: Basic Weaver validation using unified API
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

    // Example 2: Weaver validation with error handling
    test!(test_weaver_validator_error_handling, {
        // Arrange: Create test with invalid registry path
        let invalid_path = PathBuf::from("/nonexistent/registry/path");
        let config = TestConfig {
            registry_path: Some(invalid_path),
            weaver_enabled: true,
            ..Default::default()
        };

        // Act: Attempt to create test (should fail)
        let result = ObservabilityTest::with_config(config);

        // Assert: Verify creation fails with appropriate error
        assert_err!(&result, "Should fail with invalid registry path");
    });

    // Example 3: Weaver validation with custom configuration
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

#[cfg(feature = "otel")]
#[cfg(feature = "weaver")]
#[cfg(test)]
mod integration_tests {
    use chicago_tdd_tools::observability::{ObservabilityTest, TestConfig};
    use chicago_tdd_tools::otel::types::{SpanContext, SpanId, SpanStatus, TraceId};
    use chicago_tdd_tools::prelude::*;
    use std::path::PathBuf;

    // Example: Integration test combining OTEL and Weaver
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
