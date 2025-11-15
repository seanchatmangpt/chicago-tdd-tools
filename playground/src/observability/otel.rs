//! OTEL Examples
//!
//! Demonstrates OpenTelemetry span and metric validation.

#[cfg(feature = "otel")]
use chicago_tdd_tools::observability::otel::types::*;
#[cfg(feature = "otel")]
use chicago_tdd_tools::observability::otel::*;
#[cfg(feature = "otel")]
use chicago_tdd_tools::prelude::*;
#[cfg(feature = "otel")]
use std::collections::BTreeMap;

#[cfg(feature = "otel")]
/// Example: Basic span validation
pub fn example_otel_span_basic() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange: Create test span
    let span = test_helpers::create_test_span("test.operation");

    // Act: Validate span
    let validator = SpanValidator::new();
    validator.validate(&span)?;

    // Assert: Verify span properties
    assert_eq!(span.name, "test.operation");
    assert_eq!(span.status, SpanStatus::Ok);
    Ok(())
}

#[cfg(feature = "otel")]
/// Example: Span with attributes
pub fn example_otel_span_attributes() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange: Create span with attributes
    let mut attrs = BTreeMap::new();
    attrs.insert("service.name".to_string(), "test-service".to_string());
    attrs.insert("operation.type".to_string(), "test".to_string());
    let span = test_helpers::create_test_span_with_attributes("test.operation", attrs.clone());

    // Act: Validate span with required attributes
    let validator = SpanValidator::new()
        .with_required_attributes(vec!["service.name".to_string(), "operation.type".to_string()]);
    validator.validate(&span)?;

    // Assert: Verify attributes
    assert_eq!(span.attributes.get("service.name"), Some(&"test-service".to_string()));
    Ok(())
}

#[cfg(feature = "otel")]
/// Example: Metric validation
pub fn example_otel_metric() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange: Create test metric
    let metric = test_helpers::create_test_metric("test.counter", 42);

    // Act: Validate metric
    let validator = MetricValidator::new();
    validator.validate(&metric)?;

    // Assert: Verify metric properties
    assert_eq!(metric.name, "test.counter");
    match &metric.value {
        MetricValue::Counter(count) => assert_eq!(*count, 42),
        _ => return Err("Expected counter metric".into()),
    }
    Ok(())
}

#[cfg(feature = "otel")]
/// Example: OTEL test helper
pub fn example_otel_helper() {
    // Arrange: Create spans and metrics
    let span = test_helpers::create_test_span("test.operation");
    let metric = test_helpers::create_test_metric("test.counter", 42);

    // Act: Validate using helper
    let helper = OtelTestHelper::new();
    helper.assert_spans_valid(&[span]);
    helper.assert_metrics_valid(&[metric]);

    // Assert: Validation passed (no panic)
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "otel")]
    use super::*;
    #[cfg(feature = "otel")]
    use chicago_tdd_tools::prelude::*;

    #[cfg(feature = "otel")]
    otel_test!(test_otel_span_basic, {
        // Arrange-Act-Assert: Run example
        assert_ok!(example_otel_span_basic());
    });

    #[cfg(feature = "otel")]
    otel_test!(test_otel_span_attributes, {
        // Arrange-Act-Assert: Run example
        assert_ok!(example_otel_span_attributes());
    });

    #[cfg(feature = "otel")]
    otel_test!(test_otel_metric, {
        // Arrange-Act-Assert: Run example
        assert_ok!(example_otel_metric());
    });

    #[cfg(feature = "otel")]
    otel_test!(test_otel_helper, {
        // Arrange-Act-Assert: Run example
        example_otel_helper();
    });
}
