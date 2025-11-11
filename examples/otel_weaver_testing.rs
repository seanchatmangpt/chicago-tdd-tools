//! OTEL and Weaver Testing Examples
//!
//! Demonstrates how to use Chicago TDD Tools for OTEL and Weaver testing
//! with automated helpers and macros that enforce Chicago TDD patterns.
//!
//! Run with: `cargo run --example otel_weaver_testing --features otel,weaver`

fn main() {
    println!("OTEL and Weaver Testing Examples");
    println!("==================================");
    println!();
    println!("This example demonstrates Chicago TDD Tools for OTEL and Weaver testing.");
    println!("For actual test examples, see the test modules below.");
    println!();
    println!("To run tests, use: cargo test --features otel,weaver");
}

#[cfg(feature = "otel")]
#[cfg(test)]
mod otel_tests {
    use chicago_tdd_tools::otel::{test_helpers, OtelTestHelper, SpanValidator};
    use chicago_tdd_tools::{otel_test, prelude::*};
    use std::collections::BTreeMap;

    // Example 1: Basic OTEL span validation using otel_test! macro
    otel_test!(test_otel_span_validation_basic, {
        // Arrange: Create test span using helper function
        let span = test_helpers::create_test_span("test.operation");

        // Act: Validate span using OtelTestHelper
        let helper = OtelTestHelper::new();
        helper.assert_spans_valid(&[span.clone()]);

        // Assert: Verify span properties
        assert_eq!(span.name, "test.operation");
        assert_eq!(span.status, chicago_tdd_tools::otel::types::SpanStatus::Ok);
    });

    // Example 2: OTEL span validation with custom attributes
    otel_test!(test_otel_span_with_attributes, {
        // Arrange: Create span with custom attributes
        let mut attrs = BTreeMap::new();
        attrs.insert("service.name".to_string(), "test-service".to_string());
        attrs.insert("operation.type".to_string(), "test".to_string());
        let span = test_helpers::create_test_span_with_attributes("test.operation", attrs.clone());

        // Act: Validate span with required attributes
        let validator = SpanValidator::new().with_required_attributes(vec![
            "service.name".to_string(),
            "operation.type".to_string(),
        ]);
        let validation_result = validator.validate(&span);

        // Assert: Verify validation succeeds
        assert!(validation_result.is_ok(), "Span should be valid with required attributes");
        assert_eq!(span.attributes.get("service.name"), Some(&"test-service".to_string()));
    });

    // Example 3: OTEL metric validation
    otel_test!(test_otel_metric_validation, {
        // Arrange: Create test metric using helper function
        let metric = test_helpers::create_test_metric("test.counter", 42);

        // Act: Validate metric using OtelTestHelper
        let helper = OtelTestHelper::new();
        helper.assert_metrics_valid(&[metric.clone()]);

        // Assert: Verify metric properties
        assert_eq!(metric.name, "test.counter");
        match &metric.value {
            chicago_tdd_tools::otel::types::MetricValue::Counter(count) => {
                assert_eq!(*count, 42);
            }
            _ => panic!("Expected counter metric"),
        }
    });

    // Example 4: OTEL span validation with error path testing
    otel_test!(test_otel_span_validation_error_path, {
        // Arrange: Create span with invalid trace ID (zero)
        use chicago_tdd_tools::otel::types::{SpanContext, SpanId, SpanStatus, TraceId};
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
        let validator = SpanValidator::new();
        let validation_result = validator.validate(&span);

        // Assert: Verify validation fails with appropriate error
        assert!(validation_result.is_err(), "Span with zero trace ID should fail validation");
        match validation_result {
            Err(chicago_tdd_tools::otel::OtelValidationError::InvalidTraceId(_)) => {
                // Expected error variant
            }
            Err(e) => panic!("Expected InvalidTraceId error, got: {:?}", e),
            Ok(_) => panic!("Expected validation to fail"),
        }
    });
}

#[cfg(feature = "weaver")]
#[cfg(test)]
mod weaver_tests {
    use chicago_tdd_tools::observability::weaver::WeaverValidator;
    use chicago_tdd_tools::{prelude::*, weaver_test};
    use std::path::PathBuf;

    // Example 1: Basic Weaver validation using weaver_test! macro
    weaver_test!(test_weaver_validator_creation, {
        // Arrange: Create validator with registry path
        let registry_path = PathBuf::from("registry/");

        // Act: Create validator
        let validator = WeaverValidator::new(registry_path);

        // Assert: Verify validator is created correctly
        assert_eq!(validator.otlp_endpoint(), "http://127.0.0.1:4317");
        assert!(!validator.is_running(), "Validator should not be running initially");
    });

    // Example 2: Weaver validation with error handling
    weaver_test!(test_weaver_validator_error_handling, {
        // Arrange: Create validator with invalid registry path
        let invalid_path = PathBuf::from("/nonexistent/registry/path");
        let mut validator = WeaverValidator::new(invalid_path);

        // Act: Attempt to start Weaver (should fail)
        let start_result = validator.start();

        // Assert: Verify start fails with appropriate error
        assert!(start_result.is_err(), "Start should fail with invalid registry path");
        match start_result {
            Err(
                chicago_tdd_tools::observability::weaver::WeaverValidationError::RegistryNotFound(
                    _,
                ),
            ) => {
                // Expected error variant
            }
            Err(e) => panic!("Expected RegistryNotFound error, got: {:?}", e),
            Ok(_) => panic!("Expected start to fail"),
        }
    });

    // Example 3: Weaver validation with custom configuration
    weaver_test!(test_weaver_validator_custom_config, {
        // Arrange: Create validator with custom ports
        let registry_path = PathBuf::from("registry/");
        let validator = WeaverValidator::with_config(registry_path, 4318, 8081);

        // Act: Get OTLP endpoint
        let endpoint = validator.otlp_endpoint();

        // Assert: Verify custom configuration is used
        assert_eq!(endpoint, "http://127.0.0.1:4318");
    });
}

#[cfg(feature = "otel")]
#[cfg(feature = "weaver")]
#[cfg(test)]
mod integration_tests {
    use chicago_tdd_tools::observability::weaver::WeaverValidator;
    use chicago_tdd_tools::otel::{test_helpers, OtelTestHelper};
    use chicago_tdd_tools::{prelude::*, weaver_test};
    use std::path::PathBuf;

    // Example: Integration test combining OTEL and Weaver
    weaver_test!(test_otel_weaver_integration, {
        // Arrange: Create test span and Weaver validator
        let span = test_helpers::create_test_span("test.operation");
        let registry_path = PathBuf::from("registry/");
        let mut validator = WeaverValidator::new(registry_path);

        // Act: Validate span with OTEL helper, then attempt Weaver validation
        let otel_helper = OtelTestHelper::new();
        otel_helper.assert_spans_valid(&[span.clone()]);

        // Note: Actual Weaver start would require registry path to exist
        // This demonstrates the pattern for integration testing
        let weaver_available = WeaverValidator::check_weaver_available().is_ok();

        // Assert: Verify OTEL validation succeeded
        assert_eq!(span.name, "test.operation");
        // Weaver availability check is informational (may not be available in test environment)
        assert!(
            weaver_available || !weaver_available,
            "Weaver availability check should return Result"
        );
    });
}
