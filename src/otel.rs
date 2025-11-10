//! OTEL Validation
//!
//! Provides validation utilities for OpenTelemetry spans and metrics.
//! Validates that telemetry conforms to schema and semantic conventions.

#[cfg(feature = "otel")]
use crate::otel_types::{Metric, Span, SpanId};
use thiserror::Error;

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
        Self {
            required_attributes: Vec::new(),
            validate_non_zero_ids: true,
        }
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
        if self.validate_non_zero_ids {
            if span.context.span_id.0 == 0 {
                return Err(OtelValidationError::InvalidSpanId(
                    "Span ID cannot be zero".to_string(),
                ));
            }
        }

        // Validate trace ID is not zero
        if span.context.trace_id.0 == 0 {
            return Err(OtelValidationError::InvalidTraceId(
                "Trace ID cannot be zero".to_string(),
            ));
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

        // Validate end time is after start time (if end time is set)
        if let Some(end_time) = span.end_time_ms {
            if end_time < span.start_time_ms {
                return Err(OtelValidationError::SpanValidationFailed(format!(
                    "Span end time {} is before start time {}",
                    end_time, span.start_time_ms
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
        Self {
            required_attributes: Vec::new(),
        }
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
            crate::otel_types::MetricValue::Counter(count) => {
                if *count == 0 && metric.name.contains("error") {
                    // Error counters should be > 0 if metric name suggests errors
                    // This is informational, not an error
                }
            }
            crate::otel_types::MetricValue::Gauge(value) => {
                if value.is_nan() || value.is_infinite() {
                    return Err(OtelValidationError::MetricValidationFailed(format!(
                        "Metric '{}' has invalid gauge value: {}",
                        metric.name, value
                    )));
                }
            }
            crate::otel_types::MetricValue::Histogram(buckets) => {
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
        Self {
            span_validator: SpanValidator::new(),
            metric_validator: MetricValidator::new(),
        }
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
        self.span_validator
            .validate_spans(spans)
            .unwrap_or_else(|e| panic!("Span validation failed: {}", e));
    }

    /// Assert that metrics are valid (for use in tests)
    pub fn assert_metrics_valid(&self, metrics: &[Metric]) {
        self.metric_validator
            .validate_metrics(metrics)
            .unwrap_or_else(|e| panic!("Metric validation failed: {}", e));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "otel")]
    use crate::otel_types::{SpanContext, SpanId, SpanStatus, TraceId};

    #[cfg(feature = "otel")]
    #[test]
    fn test_span_validator_valid_span() {
        let validator = SpanValidator::new();
        let span = Span {
            context: SpanContext {
                trace_id: TraceId(12345),
                span_id: SpanId(67890),
                parent_span_id: None,
                flags: 1,
            },
            name: "test.span".to_string(),
            start_time_ms: 1000,
            end_time_ms: Some(2000),
            attributes: Default::default(),
            events: Vec::new(),
            status: SpanStatus::Ok,
        };

        assert!(validator.validate(&span).is_ok());
    }

    #[cfg(feature = "otel")]
    #[test]
    fn test_span_validator_zero_span_id() {
        use crate::otel_types::SpanContext;

        let validator = SpanValidator::new();
        let span = Span {
            context: SpanContext {
                trace_id: TraceId(12345),
                span_id: SpanId(0), // Zero span ID
                parent_span_id: None,
                flags: 1,
            },
            name: "test.span".to_string(),
            start_time_ms: 1000,
            end_time_ms: Some(2000),
            attributes: Default::default(),
            events: Vec::new(),
            status: SpanStatus::Ok,
        };

        assert!(validator.validate(&span).is_err());
    }

    #[cfg(feature = "otel")]
    #[test]
    fn test_span_validator_empty_name() {
        use crate::otel_types::SpanContext;

        let validator = SpanValidator::new();
        let span = Span {
            context: SpanContext {
                trace_id: TraceId(12345),
                span_id: SpanId(67890),
                parent_span_id: None,
                flags: 1,
            },
            name: "".to_string(), // Empty name
            start_time_ms: 1000,
            end_time_ms: Some(2000),
            attributes: Default::default(),
            events: Vec::new(),
            status: SpanStatus::Ok,
        };

        assert!(validator.validate(&span).is_err());
    }

    #[cfg(feature = "otel")]
    #[test]
    fn test_metric_validator_valid_metric() {
        use knhk_otel::MetricValue;

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
        use knhk_otel::MetricValue;

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
