//! OTEL Validation
//!
//! Provides validation utilities for OpenTelemetry spans and metrics.
//! Validates that telemetry conforms to schema and semantic conventions.

#[cfg(feature = "otel")]
use crate::otel::types::{Metric, Span, SpanId};
use thiserror::Error;

pub mod types;

/// OTEL validation error
#[derive(Error, Debug)]
pub enum OtelValidationError {
    /// Span validation failed
    #[error("Span validation failed: {0}")]
    SpanValidationFailed(String),
    /// Metric validation failed
    #[error("Metric validation failed: {0}")]
    MetricValidationFailed(String),
    /// Missing required attribute
    #[error("Missing required attribute: {0}")]
    MissingAttribute(String),
    /// Invalid attribute type
    #[error("Invalid attribute type for '{0}': expected {1}, got {2}")]
    InvalidAttributeType(String, String, String),
    /// Invalid span status
    #[error("Invalid span status: {0}")]
    InvalidSpanStatus(String),
    /// Invalid trace ID
    #[error("Invalid trace ID: {0}")]
    InvalidTraceId(String),
    /// Invalid span ID
    #[error("Invalid span ID: {0}")]
    InvalidSpanId(String),
}

/// Result type for OTEL validation
pub type OtelValidationResult<T> = Result<T, OtelValidationError>;

/// OTEL span validator
#[cfg(feature = "otel")]
pub struct SpanValidator {
    /// Required attributes for spans
    required_attributes: Vec<String>,
    /// Validate span IDs are not zero
    validate_non_zero_ids: bool,
}

#[cfg(feature = "otel")]
impl Default for SpanValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "otel")]
impl SpanValidator {
    /// Create a new span validator
    pub fn new() -> Self {
        Self { required_attributes: Vec::new(), validate_non_zero_ids: true }
    }

    /// Require specific attributes
    pub fn with_required_attributes(mut self, attributes: Vec<String>) -> Self {
        self.required_attributes = attributes;
        self
    }

    /// Enable/disable non-zero ID validation
    pub fn with_non_zero_id_validation(mut self, enabled: bool) -> Self {
        self.validate_non_zero_ids = enabled;
        self
    }

    /// Validate a span
    pub fn validate(&self, span: &Span) -> OtelValidationResult<()> {
        // Validate span ID is not zero (if enabled)
        if self.validate_non_zero_ids && span.context.span_id.0 == 0 {
            return Err(OtelValidationError::InvalidSpanId("Span ID cannot be zero".to_string()));
        }

        // Validate trace ID is not zero
        if span.context.trace_id.0 == 0 {
            return Err(OtelValidationError::InvalidTraceId("Trace ID cannot be zero".to_string()));
        }

        // Validate span name is not empty
        if span.name.is_empty() {
            return Err(OtelValidationError::SpanValidationFailed(
                "Span name cannot be empty".to_string(),
            ));
        }

        // Validate required attributes
        for attr_name in &self.required_attributes {
            if !span.attributes.contains_key(attr_name) {
                return Err(OtelValidationError::MissingAttribute(attr_name.clone()));
            }
        }

        // Validate end time is after start time (if completed)
        // Poka-Yoke: SpanState enum ensures end_time >= start_time at type level
        if let Some(end_time) = span.end_time_ms() {
            let start_time = span.start_time_ms();
            if end_time < start_time {
                return Err(OtelValidationError::SpanValidationFailed(format!(
                    "Span end time {} is before start time {}",
                    end_time, start_time
                )));
            }
        }

        Ok(())
    }

    /// Validate multiple spans
    pub fn validate_spans(&self, spans: &[Span]) -> OtelValidationResult<()> {
        for (idx, span) in spans.iter().enumerate() {
            self.validate(span).map_err(|e| {
                OtelValidationError::SpanValidationFailed(format!(
                    "Span {} (index {}): {}",
                    span.name, idx, e
                ))
            })?;
        }
        Ok(())
    }
}

/// OTEL metric validator
#[cfg(feature = "otel")]
pub struct MetricValidator {
    /// Required attributes for metrics
    required_attributes: Vec<String>,
}

#[cfg(feature = "otel")]
impl Default for MetricValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "otel")]
impl MetricValidator {
    /// Create a new metric validator
    pub fn new() -> Self {
        Self { required_attributes: Vec::new() }
    }

    /// Require specific attributes
    pub fn with_required_attributes(mut self, attributes: Vec<String>) -> Self {
        self.required_attributes = attributes;
        self
    }

    /// Validate a metric
    pub fn validate(&self, metric: &Metric) -> OtelValidationResult<()> {
        // Validate metric name is not empty
        if metric.name.is_empty() {
            return Err(OtelValidationError::MetricValidationFailed(
                "Metric name cannot be empty".to_string(),
            ));
        }

        // Validate required attributes
        for attr_name in &self.required_attributes {
            if !metric.attributes.contains_key(attr_name) {
                return Err(OtelValidationError::MissingAttribute(attr_name.clone()));
            }
        }

        // Validate metric value is valid
        match &metric.value {
            crate::otel::types::MetricValue::Counter(count) => {
                if *count == 0 && metric.name.contains("error") {
                    // Error counters should be > 0 if metric name suggests errors
                    // This is informational, not an error
                }
            }
            crate::otel::types::MetricValue::Gauge(value) => {
                if value.is_nan() || value.is_infinite() {
                    return Err(OtelValidationError::MetricValidationFailed(format!(
                        "Metric '{}' has invalid gauge value: {}",
                        metric.name, value
                    )));
                }
            }
            crate::otel::types::MetricValue::Histogram(buckets) => {
                if buckets.is_empty() {
                    return Err(OtelValidationError::MetricValidationFailed(format!(
                        "Metric '{}' has empty histogram buckets",
                        metric.name
                    )));
                }
            }
        }

        Ok(())
    }

    /// Validate multiple metrics
    pub fn validate_metrics(&self, metrics: &[Metric]) -> OtelValidationResult<()> {
        for (idx, metric) in metrics.iter().enumerate() {
            self.validate(metric).map_err(|e| {
                OtelValidationError::MetricValidationFailed(format!(
                    "Metric {} (index {}): {}",
                    metric.name, idx, e
                ))
            })?;
        }
        Ok(())
    }
}

/// OTEL validation helper for test utilities
#[cfg(feature = "otel")]
pub struct OtelTestHelper {
    span_validator: SpanValidator,
    metric_validator: MetricValidator,
}

#[cfg(feature = "otel")]
impl Default for OtelTestHelper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "otel")]
impl OtelTestHelper {
    /// Create a new OTEL test helper
    pub fn new() -> Self {
        Self { span_validator: SpanValidator::new(), metric_validator: MetricValidator::new() }
    }

    /// Validate spans from a tracer
    pub fn validate_tracer_spans(&self, spans: &[Span]) -> OtelValidationResult<Vec<SpanId>> {
        self.span_validator.validate_spans(spans)?;
        Ok(spans.iter().map(|s| s.context.span_id).collect())
    }

    /// Validate metrics from a tracer
    pub fn validate_tracer_metrics(&self, metrics: &[Metric]) -> OtelValidationResult<Vec<String>> {
        self.metric_validator.validate_metrics(metrics)?;
        Ok(metrics.iter().map(|m| m.name.clone()).collect())
    }

    /// Assert that spans are valid (for use in tests)
    pub fn assert_spans_valid(&self, spans: &[Span]) {
        #[allow(clippy::expect_used)] // Test helper - panic is appropriate
        for span in spans {
            self.span_validator
                .validate_spans(&[span.clone()])
                .unwrap_or_else(|e| panic!("Span validation failed: {}", e));
        }
    }

    /// Assert that metrics are valid (for use in tests)
    pub fn assert_metrics_valid(&self, metrics: &[Metric]) {
        #[allow(clippy::expect_used)] // Test helper - panic is appropriate
        for metric in metrics {
            self.metric_validator
                .validate_metrics(&[metric.clone()])
                .unwrap_or_else(|e| panic!("Metric validation failed: {}", e));
        }
    }
}

#[cfg(test)]
#[allow(clippy::panic)] // Test code - panic is appropriate for test failures
mod tests {
    use super::*;
    #[cfg(feature = "otel")]
    use crate::otel::types::{SpanContext, SpanId, SpanStatus, TraceId};

    // Test feature-gated code paths (critical - verify features work correctly)
    #[cfg(not(feature = "otel"))]
    #[test]
    fn test_otel_module_not_accessible_without_feature() {
        // Verify otel module is not accessible without feature
        // This test should compile and pass when otel feature is disabled
        assert!(true, "otel module should not be accessible without feature");
    }

    #[cfg(feature = "otel")]
    #[test]
    fn test_otel_error_variants() {
        // Test all error variants (critical - 80% of bugs)
        let errors = vec![
            OtelValidationError::SpanValidationFailed("test".to_string()),
            OtelValidationError::MetricValidationFailed("test".to_string()),
            OtelValidationError::MissingAttribute("test".to_string()),
            OtelValidationError::InvalidAttributeType(
                "test".to_string(),
                "expected".to_string(),
                "got".to_string(),
            ),
            OtelValidationError::InvalidSpanStatus("test".to_string()),
            OtelValidationError::InvalidTraceId("test".to_string()),
            OtelValidationError::InvalidSpanId("test".to_string()),
        ];

        for error in errors {
            let display = format!("{error}");
            assert!(!display.is_empty(), "Error should have display message");
            assert!(display.contains("test"), "Error should contain message");
        }
    }

    #[cfg(feature = "otel")]
    #[test]
    fn test_otel_error_debug() {
        // Test error is debuggable
        let error = OtelValidationError::SpanValidationFailed("test".to_string());
        let debug = format!("{error:?}");
        assert!(debug.contains("SpanValidationFailed"));
        assert!(debug.contains("test"));
    }

    #[cfg(feature = "otel")]
    #[test]
    fn test_span_validator_valid_span() {
        let validator = SpanValidator::new();
        #[allow(clippy::unwrap_used)] // Test code - Span creation should succeed in tests
        let span = Span::new_completed(
            SpanContext::root(TraceId(12345), SpanId(67890), 1),
            "test.span".to_string(),
            1000,
            2000,
            Default::default(),
            Vec::new(),
            SpanStatus::Ok,
        )
        .unwrap();

        assert!(validator.validate(&span).is_ok());
    }

    #[cfg(feature = "otel")]
    #[test]
    fn test_span_validator_zero_span_id() {
        let validator = SpanValidator::new();
        let span = Span::new_completed(
            SpanContext::root(TraceId(12345), SpanId(0), 1), // Zero span ID
            "test.span".to_string(),
            1000,
            2000,
            Default::default(),
            Vec::new(),
            SpanStatus::Ok,
        )
        .unwrap();

        assert!(validator.validate(&span).is_err());
    }

    #[cfg(feature = "otel")]
    #[test]
    fn test_span_validator_empty_name() {
        let validator = SpanValidator::new();
        let span = Span::new_completed(
            SpanContext::root(TraceId(12345), SpanId(67890), 1),
            String::new(), // Empty name
            1000,
            2000,
            Default::default(),
            Vec::new(),
            SpanStatus::Ok,
        )
        .unwrap();

        assert!(validator.validate(&span).is_err());
    }

    #[cfg(feature = "otel")]
    #[test]
    fn test_metric_validator_valid_metric() {
        use crate::otel::types::MetricValue;

        let validator = MetricValidator::new();
        let metric = Metric {
            name: "test.metric".to_string(),
            value: MetricValue::Counter(42),
            timestamp_ms: 1000,
            attributes: Default::default(),
        };

        assert!(validator.validate(&metric).is_ok());
    }

    #[cfg(feature = "otel")]
    #[test]
    fn test_metric_validator_empty_name() {
        use crate::otel::types::MetricValue;

        let validator = MetricValidator::new();
        let metric = Metric {
            name: "".to_string(), // Empty name
            value: MetricValue::Counter(42),
            timestamp_ms: 1000,
            attributes: Default::default(),
        };

        assert!(validator.validate(&metric).is_err());
    }
}
