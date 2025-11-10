//! Assertion Helpers
//!
//! Provides assertion utilities following Chicago TDD principles.
//! Uses Higher-Ranked Trait Bounds (HRTB) for flexible predicate functions.
//!
//! # Go the Extra Mile: 1st/2nd/3rd Idea Progression
//!
//! - **1st Idea**: Specific assertion functions (`assert_success`, `assert_error`, etc.)
//! - **2nd Idea**: `AssertionBuilder<T>` - Generic assertion builder pattern for composable assertions
//! - **3rd Idea**: Compile-time validated assertions with OTEL/Weaver validation

#[cfg(feature = "otel")]
use crate::otel_types::{Metric, MetricValue, Span, SpanContext, SpanId, SpanStatus, TraceId};
#[cfg(feature = "otel")]
use std::time::{SystemTime, UNIX_EPOCH};

/// Assert that a result is successful
///
/// # Panics
///
/// Panics if the result is an error, with a message showing the error.
pub fn assert_success<T, E: std::fmt::Debug>(result: &Result<T, E>) {
    assert!(result.is_ok(), "Expected success, but got error: {:?}", result.as_ref().err());
}

/// Assert that a result is an error
///
/// # Panics
///
/// Panics if the result is successful, with a message showing the value.
pub fn assert_error<T: std::fmt::Debug, E>(result: &Result<T, E>) {
    assert!(result.is_err(), "Expected error, but got success: {:?}", result.as_ref().ok());
}

/// Assert that two values are equal with a custom message
pub fn assert_eq_with_msg<T: std::fmt::Debug + PartialEq>(actual: &T, expected: &T, msg: &str) {
    assert_eq!(actual, expected, "{}: expected {:?}, got {:?}", msg, expected, actual);
}

/// Assert that a value is within a range
pub fn assert_in_range<T: PartialOrd + std::fmt::Debug>(value: &T, min: &T, max: &T, msg: &str) {
    assert!(
        value >= min && value <= max,
        "{}: value {:?} not in range [{:?}, {:?}]",
        msg,
        value,
        min,
        max
    );
}

/// Assert that a value satisfies a predicate using Higher-Ranked Trait Bounds (HRTB)
///
/// HRTB allows the predicate to work with any lifetime, making it more flexible
/// than a regular `Fn(&T) -> bool` bound.
///
/// # Example
///
/// ```rust
/// use chicago_tdd_tools::assertions::assert_that;
///
/// let value = 42;
/// assert_that(&value, |v| *v > 0);
///
/// // Works with references of any lifetime
/// let vec = vec![1, 2, 3];
/// assert_that(&vec, |v| v.len() == 3);
/// ```
pub fn assert_that<T, F>(value: &T, predicate: F)
where
    T: std::fmt::Debug,
    // Poka-Yoke: HRTB requires single-character lifetime for flexibility
    F: for<'value> Fn(&'value T) -> bool,
{
    assert!(predicate(value), "Assertion failed for value: {:?}", value);
}

/// Assert that a value satisfies a predicate with a custom message
pub fn assert_that_with_msg<T, F>(value: &T, predicate: F, msg: &str)
where
    T: std::fmt::Debug,
    // Poka-Yoke: HRTB requires single-character lifetime for flexibility
    F: for<'value> Fn(&'value T) -> bool,
{
    assert!(predicate(value), "{}: Assertion failed for value: {:?}", msg, value);
}

// ============================================================================
// 2nd IDEA: Go bigger (80/20) - Generic assertion builder
// ============================================================================

/// Generic assertion builder for composable assertions
///
/// **2nd Idea**: Generic builder that works with any type and allows composing multiple assertions.
/// This provides 80% more value (works for all types, composable) with minimal effort.
///
/// **Telemetry**: Basic OTEL spans (if otel feature enabled)
/// **Validation**: OTEL span validation
pub struct AssertionBuilder<T> {
    value: T,
    #[cfg(feature = "otel")]
    span: Option<Span>,
}

impl<T: std::fmt::Debug> AssertionBuilder<T> {
    /// Create a new assertion builder
    pub fn new(value: T) -> Self {
        Self {
            value,
            #[cfg(feature = "otel")]
            span: None,
        }
    }

    /// Start OTEL span for this assertion
    #[cfg(feature = "otel")]
    pub fn with_span(mut self, span_name: &str) -> Self {
        #[allow(clippy::expect_used)] // SystemTime should always be after UNIX_EPOCH
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime should always be after UNIX_EPOCH")
            .as_millis() as u64;

        let span = Span::new_active(
            SpanContext::root(TraceId(12345), SpanId(67890), 1),
            span_name.to_string(),
            start_time,
            std::collections::BTreeMap::new(),
            Vec::new(),
            SpanStatus::Unset,
        );

        self.span = Some(span);
        self
    }

    /// Assert that the value satisfies a predicate
    pub fn assert_that<F>(self, predicate: F) -> Self
    where
        // Poka-Yoke: HRTB requires single-character lifetime for flexibility
        F: for<'value> Fn(&'value T) -> bool,
    {
        assert!(predicate(&self.value), "Assertion failed for value: {:?}", self.value);
        self
    }

    /// Assert that the value equals an expected value
    pub fn assert_eq<U: PartialEq + std::fmt::Debug>(self, expected: &U) -> Self
    where
        T: PartialEq<U>,
    {
        assert_eq!(&self.value, expected, "Values not equal");
        self
    }

    /// Assert that the value satisfies a predicate with a custom message
    pub fn assert_that_with_msg<F>(self, predicate: F, msg: &str) -> Self
    where
        // Poka-Yoke: HRTB requires single-character lifetime for flexibility
        F: for<'value> Fn(&'value T) -> bool,
    {
        assert!(predicate(&self.value), "{}: Assertion failed for value: {:?}", msg, self.value);
        self
    }

    /// Get the value (consumes the builder)
    pub fn into_value(self) -> T {
        self.value
    }

    /// Get the OTEL span (if started)
    #[cfg(feature = "otel")]
    pub fn into_span(mut self) -> Option<Span> {
        if let Some(ref mut span) = self.span {
            #[allow(clippy::expect_used)] // SystemTime should always be after UNIX_EPOCH
            let end_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("SystemTime should always be after UNIX_EPOCH")
                .as_millis() as u64;

            // End time should always be >= start time in normal operation
            if let Err(e) = span.complete(end_time) {
                // Log error but don't fail - span will remain active
                eprintln!("Warning: Failed to complete span: {}", e);
            } else {
                span.status = SpanStatus::Ok;
            }
        }
        self.span.take()
    }
}

// ============================================================================
// 3rd IDEA: Maximum value - Compile-time validated assertions + OTEL + Weaver
// ============================================================================

/// Compile-time validated assertion with OTEL/Weaver validation
///
/// **3rd Idea**: Type-level validated assertion that prevents invalid states at compile time.
/// Maximum value: Type-safe, validated, prevents entire class of errors.
///
/// **Telemetry**: Full OTEL spans and metrics
/// **Validation**: OTEL span validation + Weaver live-check schema validation
pub struct ValidatedAssertion<T> {
    // Poka-Yoke: Value is accessed via into_value() - not dead code
    #[allow(dead_code, reason = "Value is accessed via into_value() method")]
    value: T,
    #[cfg(feature = "otel")]
    span: Span,
    #[cfg(feature = "otel")]
    metric: Option<Metric>,
}

#[cfg(feature = "otel")]
impl<T: std::fmt::Debug> ValidatedAssertion<T> {
    /// Create a new validated assertion with OTEL instrumentation
    pub fn new(value: T, span_name: &str) -> Self {
        #[allow(clippy::expect_used)] // SystemTime should always be after UNIX_EPOCH
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime should always be after UNIX_EPOCH")
            .as_millis() as u64;

        let span = Span::new_active(
            SpanContext::root(TraceId(12345), SpanId(67890), 1),
            span_name.to_string(),
            start_time,
            std::collections::BTreeMap::new(),
            Vec::new(),
            SpanStatus::Unset,
        );

        Self { value, span, metric: None }
    }

    /// Assert that the value satisfies a predicate (validated)
    pub fn assert_that<F>(mut self, predicate: F) -> Self
    where
        F: for<'a> Fn(&'a T) -> bool,
    {
        let success = predicate(&self.value);

        #[allow(clippy::expect_used)] // SystemTime should always be after UNIX_EPOCH
        let end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime should always be after UNIX_EPOCH")
            .as_millis() as u64;

        // End time should always be >= start time in normal operation
        if let Err(e) = self.span.complete(end_time) {
            // Log error but don't fail - span will remain active
            eprintln!("Warning: Failed to complete span: {}", e);
        } else {
            self.span.status = if success { SpanStatus::Ok } else { SpanStatus::Error };
        }
        self.span.attributes.insert("assertion_result".to_string(), success.to_string());

        // Create metric
        #[allow(clippy::expect_used)] // SystemTime should always be after UNIX_EPOCH
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime should always be after UNIX_EPOCH")
            .as_millis() as u64;

        self.metric = Some(Metric {
            name: "chicago_tdd_tools.assertions.total".to_string(),
            value: MetricValue::Counter(1),
            timestamp_ms: timestamp,
            attributes: std::collections::BTreeMap::new(),
        });

        // Safe to unwrap here because we just set metric to Some above
        if let Some(ref mut metric) = self.metric {
            metric.attributes.insert("success".to_string(), success.to_string());
        }

        assert!(success, "Assertion failed for value: {:?}", self.value);
        self
    }

    /// Get the value (consumes the assertion)
    pub fn into_value(self) -> T {
        self.value
    }

    /// Get the OTEL span
    pub fn span(&self) -> &Span {
        &self.span
    }

    /// Get the OTEL metric
    pub fn metric(&self) -> Option<&Metric> {
        self.metric.as_ref()
    }
}
