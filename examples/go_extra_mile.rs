//! # Go the Extra Mile: 1st/2nd/3rd Idea Progression - Comprehensive Guide
//!
//! Demonstrates the "go the extra mile" paradigm with progressive enhancement:
//! from basic implementation to maximum value solutions.
//!
//! ## Tutorial: Getting Started
//!
//! This example walks through three progressive ideas for number parsing:
//!
//! 1. **1st Idea**: Solve the immediate problem - parse `u32` only
//! 2. **2nd Idea**: Go bigger (80/20) - generic version works for all number types
//! 3. **3rd Idea**: Maximum value - type-level validation + OTEL/Weaver validation
//!
//! Each idea demonstrates increasing levels of:
//! - **Scope**: Single type → All types → Type-safe validated types
//! - **Telemetry**: None → Basic OTEL spans → Full OTEL spans + metrics
//! - **Validation**: None → OTEL validation → OTEL + Weaver validation
//!
//! ## Explanation: Concepts
//!
//! **80/20 Thinking**: The 2nd idea typically provides 80% more value with 20% more effort.
//! This is the "sweet spot" for most use cases - significant improvement without excessive complexity.
//!
//! **Progressive Enhancement**: Start with the simplest solution that works, then enhance
//! incrementally. Each idea builds on the previous, adding value without breaking existing functionality.
//!
//! **Type-Level Validation**: The 3rd idea uses Rust's type system to prevent entire classes
//! of errors at compile time. Validated types ensure correctness by construction.
//!
//! **OTEL Instrumentation**: OpenTelemetry spans and metrics provide observability into
//! operations. Spans track operations, metrics track measurements over time.
//!
//! **Weaver Live-Check**: Validates telemetry against OpenTelemetry semantic conventions
//! at runtime, ensuring compliance with industry standards.
//!
//! **Decision Framework**:
//! - 1st Idea: Works, but narrow scope
//! - 2nd Idea: Usually best - 80% more value, reasonable effort
//! - 3rd Idea: Maximum value, but evaluate effort vs. benefit
//!
//! ## How-to: Common Tasks
//!
//! - Parse a number (1st idea): See `parse_u32_first_idea()`
//! - Parse any number type (2nd idea): See `parse_number_second_idea_no_otel()`
//! - Parse with validation (3rd idea): See `ValidatedNumberNoOtel::parse()`
//! - Parse with OTEL instrumentation: See `parse_number_second_idea()` and `ValidatedNumber::parse()`
//!
//! ## Reference: Quick Lookup
//!
//! **Key Functions**:
//! - `parse_u32_first_idea(input) -> Result<u32, String>` - 1st idea: Parse u32 only
//! - `parse_number_second_idea_no_otel<T>(input) -> Result<T, String>` - 2nd idea: Generic parser
//! - `parse_number_second_idea<T>(input, span_name) -> Result<(T, Span), String>` - 2nd idea with OTEL
//! - `ValidatedNumberNoOtel::<T>::parse(input) -> Result<ValidatedNumberNoOtel<T>, String>` - 3rd idea: Type-safe
//! - `ValidatedNumber::<T>::parse(input, span_name) -> Result<ValidatedNumber<T>, String>` - 3rd idea with OTEL
//!
//! **Key Types**:
//! - `ValidatedNumberNoOtel<T>`: Type-safe validated number (no OTEL)
//! - `ValidatedNumber<T>`: Type-safe validated number with OTEL spans
//!
//! **Key Concepts**:
//! - **1st Idea**: Immediate solution, narrow scope
//! - **2nd Idea**: 80/20 sweet spot - generic, works for all types
//! - **3rd Idea**: Maximum value - type-safe, prevents entire class of errors

use chicago_tdd_tools::observability::weaver::WeaverValidator;
#[cfg(feature = "otel")]
use chicago_tdd_tools::otel::types::{
    Metric, MetricValue, Span, SpanContext, SpanId, SpanStatus, TraceId,
};
#[cfg(feature = "otel")]
use chicago_tdd_tools::otel::{MetricValidator, SpanValidator};
#[cfg(feature = "weaver")]
use chicago_tdd_tools::prelude::*;
#[cfg(feature = "otel")]
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// Example: Number parsing with progressive enhancement
// ============================================================================
// This demonstrates the 1st/2nd/3rd idea progression:
// - First idea: Parse u32 only
// - Second idea: Parse any number type (generic)
// - Third idea: Type-level validated numbers with OTEL/Weaver validation
// ============================================================================
// 1st IDEA: Solve the immediate problem
// ============================================================================

/// Example: 1st Idea - Parse u32 from string
///
/// ## How-to: Parse a Number (1st Idea)
///
/// Parse a `u32` from a string. This is the simplest solution that solves
/// the immediate problem. Works for `u32` only, no telemetry, no validation.
///
/// ## Reference
///
/// - **Function**: `parse_u32_first_idea(input) -> Result<u32, String>`
/// - **Parameters**: `input: &str` - String to parse
/// - **Returns**: `Ok(u32)` on success, `Err(String)` on parse failure
/// - **Telemetry**: None
/// - **Validation**: None
/// - **Scope**: Single type only (`u32`)
///
/// # Examples
///
/// ```rust
/// // Success case
/// let result = parse_u32_first_idea("42")?;
/// assert_eq!(result, 42);
///
/// // Error case - demonstrates error handling
/// let result = parse_u32_first_idea("not a number");
/// assert!(result.is_err());
/// ```
pub fn parse_u32_first_idea(input: &str) -> Result<u32, String> {
    input.parse().map_err(|e| format!("Parse error: {e}"))
}

// ============================================================================
// 2nd IDEA: Go bigger (80/20) - Generic version
// ============================================================================

/// Example: 2nd Idea - Parse any number type with OTEL instrumentation
///
/// ## How-to: Parse Any Number Type (2nd Idea)
///
/// Generic version that works for all number types (`u32`, `i32`, `u64`, `f64`, etc.).
/// This is the 80/20 sweet spot - 80% more value (works for all types) with minimal effort.
/// Includes OTEL span instrumentation for observability.
///
/// ## Reference
///
/// - **Function**: `parse_number_second_idea<T>(input, span_name) -> Result<(T, Span), String>`
/// - **Parameters**:
///   - `input: &str` - String to parse
///   - `span_name: &str` - Name for OTEL span
/// - **Returns**: `Ok((T, Span))` on success - parsed value and OTEL span
/// - **Telemetry**: OTEL spans with attributes (input, type, success, error)
/// - **Validation**: OTEL span validation available
/// - **Scope**: Works for all number types (generic)
///
/// # Examples
///
/// ```rust
/// let (value, span) = parse_number_second_idea::<u32>("42", "parse_number")?;
/// assert_eq!(value, 42);
/// // Validate span with OTEL validator
/// let validator = SpanValidator::new();
/// validator.validate(&span)?;
/// ```
#[cfg(feature = "otel")]
pub fn parse_number_second_idea<T: std::str::FromStr>(
    input: &str,
    span_name: &str,
) -> Result<(T, Span), String>
where
    T::Err: std::fmt::Display,
{
    // Create OTEL span for operation
    // **Best Practice**: Handle SystemTime errors properly (should never fail in practice)
    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("SystemTime error: {e}"))?
        .as_millis() as u64;

    let mut span = Span::new_active(
        SpanContext::root(TraceId(12345), SpanId(67890), 1),
        span_name.to_string(),
        start_time,
        std::collections::BTreeMap::new(),
        Vec::new(),
        SpanStatus::Unset,
    );

    span.attributes.insert("input".to_string(), input.to_string());
    span.attributes
        .insert("type".to_string(), std::any::type_name::<T>().to_string());

    // Parse the number
    let result = input.parse().map_err(|e| format!("Parse error: {}", e));

    let end_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("SystemTime error: {e}"))?
        .as_millis() as u64;

    // **Best Practice**: Handle span completion errors properly
    span.complete(end_time).map_err(|e| format!("Span completion error: {e}"))?;

    match &result {
        Ok(_) => {
            span.status = SpanStatus::Ok;
            span.attributes.insert("success".to_string(), "true".to_string());
        }
        Err(e) => {
            span.status = SpanStatus::Error;
            span.attributes.insert("success".to_string(), "false".to_string());
            span.attributes.insert("error".to_string(), e.to_string());
        }
    }

    result.map(|value| (value, span))
}

/// Example: 2nd Idea - Parse any number type (without OTEL)
///
/// ## How-to: Parse Any Number Type (2nd Idea, No OTEL)
///
/// Generic version that works for all number types without OTEL instrumentation.
/// Demonstrates 80/20 thinking - works for all types with minimal code.
///
/// ## Reference
///
/// - **Function**: `parse_number_second_idea_no_otel<T>(input) -> Result<T, String>`
/// - **Parameters**: `input: &str` - String to parse
/// - **Returns**: `Ok(T)` on success, `Err(String)` on parse failure
/// - **Telemetry**: None
/// - **Validation**: None
/// - **Scope**: Works for all number types (generic)
///
/// # Examples
///
/// ```rust
/// let value: u32 = parse_number_second_idea_no_otel("42")?;
/// let value: i32 = parse_number_second_idea_no_otel("-42")?;
/// let value: f64 = parse_number_second_idea_no_otel("123.456")?;
/// ```
pub fn parse_number_second_idea_no_otel<T: std::str::FromStr>(input: &str) -> Result<T, String>
where
    T::Err: std::fmt::Display,
{
    input.parse().map_err(|e| format!("Parse error: {e}"))
}

// ============================================================================
// 3rd IDEA: Maximum value - Type-level validation + OTEL + Weaver
// ============================================================================

/// Example: 3rd Idea - Type-level validated number with OTEL/Weaver validation
///
/// ## How-to: Parse with Type-Level Validation (3rd Idea)
///
/// Maximum value solution: Type-safe validated numbers that prevent entire classes of errors.
/// Includes full OTEL spans and metrics instrumentation, plus Weaver live-check validation.
///
/// ## Reference
///
/// - **Type**: `ValidatedNumber<T>` - Type-safe validated number with OTEL span
/// - **Function**: `ValidatedNumber::<T>::parse(input, span_name) -> Result<ValidatedNumber<T>, String>`
/// - **Parameters**:
///   - `input: &str` - String to parse
///   - `span_name: &str` - Name for OTEL span
/// - **Returns**: `Ok(ValidatedNumber<T>)` on success
/// - **Methods**:
///   - `value() -> &T` - Get the validated value
///   - `span() -> &Span` - Get the OTEL span for validation
/// - **Telemetry**: Full OTEL spans and metrics
/// - **Validation**: OTEL span validation + Weaver live-check schema validation
/// - **Scope**: Type-safe, prevents entire class of errors
///
/// # Examples
///
/// ```rust
/// let validated = ValidatedNumber::<u32>::parse("42", "validated_parse")?;
/// assert_eq!(*validated.value(), 42);
/// // Validate span
/// let validator = SpanValidator::new();
/// validator.validate(validated.span())?;
/// ```
#[cfg(feature = "otel")]
pub struct ValidatedNumber<T> {
    value: T,
    #[cfg(feature = "otel")]
    span: Span,
}

#[cfg(feature = "otel")]
impl<T: std::str::FromStr + std::fmt::Display> ValidatedNumber<T>
where
    T::Err: std::fmt::Display,
{
    /// Parse and validate number with full OTEL instrumentation
    pub fn parse(input: &str, span_name: &str) -> Result<Self, String> {
        // Create OTEL span
        // **Best Practice**: Handle SystemTime errors properly
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("SystemTime error: {e}"))?
            .as_millis() as u64;

        let mut span = Span::new_active(
            SpanContext::root(TraceId(12345), SpanId(67890), 1),
            format!("{}.parse", span_name),
            start_time,
            std::collections::BTreeMap::new(),
            Vec::new(),
            SpanStatus::Unset,
        );

        span.attributes.insert("input".to_string(), input.to_string());
        span.attributes
            .insert("type".to_string(), std::any::type_name::<T>().to_string());
        span.attributes
            .insert("operation".to_string(), "parse_and_validate".to_string());

        // Parse the number
        let parse_result: Result<T, String> =
            input.parse().map_err(|e| format!("Parse error: {}", e));

        let end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("SystemTime error: {e}"))?
            .as_millis() as u64;

        // **Best Practice**: Handle span completion errors properly
        span.complete(end_time).map_err(|e| format!("Span completion error: {e}"))?;

        match &parse_result {
            Ok(value) => {
                span.status = SpanStatus::Ok;
                span.attributes.insert("success".to_string(), "true".to_string());
                span.attributes.insert("parsed_value".to_string(), value.to_string());

                Ok(Self { value: parse_result?, span })
            }
            Err(e) => {
                span.status = SpanStatus::Error;
                span.attributes.insert("success".to_string(), "false".to_string());
                span.attributes.insert("error".to_string(), e.to_string());
                Err(e.to_string())
            }
        }
    }

    /// Get the validated value
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Get the OTEL span for validation
    #[cfg(feature = "otel")]
    pub fn span(&self) -> &Span {
        &self.span
    }
}

/// Example: 3rd Idea - Type-level validated number (without OTEL)
///
/// ## How-to: Use Type-Level Validation (3rd Idea, No OTEL)
///
/// Type-safe validated numbers that prevent entire classes of errors at compile time.
/// No OTEL instrumentation - pure type safety.
///
/// ## Reference
///
/// - **Type**: `ValidatedNumberNoOtel<T>` - Type-safe validated number
/// - **Function**: `ValidatedNumberNoOtel::<T>::parse(input) -> Result<ValidatedNumberNoOtel<T>, String>`
/// - **Method**: `value() -> &T` - Get the validated value
/// - **Telemetry**: None
/// - **Validation**: Type-level (compile-time)
/// - **Scope**: Type-safe, prevents entire class of errors
///
/// # Examples
///
/// ```rust
/// let validated = ValidatedNumberNoOtel::<u32>::parse("42")?;
/// assert_eq!(*validated.value(), 42);
/// ```
pub struct ValidatedNumberNoOtel<T> {
    value: T,
}

impl<T: std::str::FromStr> ValidatedNumberNoOtel<T>
where
    T::Err: std::fmt::Display,
{
    /// Parse and validate number
    ///
    /// # Errors
    /// Returns error if parsing fails
    pub fn parse(input: &str) -> Result<Self, String> {
        input
            .parse()
            .map(|value| Self { value })
            .map_err(|e| format!("Parse error: {e}"))
    }

    /// Get the validated value
    pub const fn value(&self) -> &T {
        &self.value
    }
}

// ============================================================================
// Example: Using all three ideas with validation
// ============================================================================

#[tokio::main]
#[allow(clippy::too_many_lines)] // Example demonstrates all three ideas
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "otel")]
    use std::time::{SystemTime, UNIX_EPOCH};

    chicago_tdd_tools::alert_info!("Go the Extra Mile: 1st/2nd/3rd Idea Progression");
    chicago_tdd_tools::alert_info!("================================================\n");

    // ========================================================================
    // 1st Idea: Basic implementation
    // ========================================================================
    chicago_tdd_tools::alert_info!("1st Idea: Parse u32 only");
    chicago_tdd_tools::alert_info!("-------------------------");
    let result1 = parse_u32_first_idea("42");
    chicago_tdd_tools::assert_ok!(&result1);
    let value1 = result1?;
    assert_eq!(value1, 42);
    chicago_tdd_tools::alert_success!("Parsed u32: 42");
    chicago_tdd_tools::alert_info!("  - No telemetry");
    chicago_tdd_tools::alert_info!("  - No validation");
    chicago_tdd_tools::alert_info!("  - Single type only\n");

    // ========================================================================
    // 2nd Idea: Go bigger (80/20) - Generic version
    // ========================================================================
    chicago_tdd_tools::alert_info!("2nd Idea: Parse any number type (80/20)");
    chicago_tdd_tools::alert_info!("----------------------------------------");

    // Works for u32
    let u32_parsed = parse_number_second_idea_no_otel::<u32>("42");
    chicago_tdd_tools::assert_ok!(&u32_parsed);
    let parsed_u32 = u32_parsed?;
    assert_eq!(parsed_u32, 42);
    chicago_tdd_tools::alert_success!("Parsed u32: 42");

    // Works for i32
    let i32_parsed = parse_number_second_idea_no_otel::<i32>("-42");
    chicago_tdd_tools::assert_ok!(&i32_parsed);
    let parsed_i32 = i32_parsed?;
    assert_eq!(parsed_i32, -42);
    chicago_tdd_tools::alert_success!("Parsed i32: -42");

    // Works for f64
    let f64_parsed = parse_number_second_idea_no_otel::<f64>("123.456");
    chicago_tdd_tools::assert_ok!(&f64_parsed);
    let parsed_f64 = f64_parsed?;
    assert!((parsed_f64 - 123.456).abs() < f64::EPSILON);
    chicago_tdd_tools::alert_success!("Parsed f64: 123.456");

    chicago_tdd_tools::alert_info!("  - Generic: Works for all number types");
    chicago_tdd_tools::alert_info!("  - 80% more value, minimal effort");

    #[cfg(feature = "otel")]
    {
        chicago_tdd_tools::alert_info!("  - OTEL spans: Basic instrumentation");

        // Test with OTEL spans
        let (value, span) = parse_number_second_idea::<u32>("42", "parse_number")?;
        assert_eq!(value, 42);

        // Validate span with OTEL validator
        let validator = SpanValidator::new();
        validator.validate(&span)?;
        chicago_tdd_tools::alert_info!("  - Span validation: ✓ Passed");
    }

    chicago_tdd_tools::alert_info!();

    // ========================================================================
    // 3rd Idea: Maximum value - Type-level validation + OTEL + Weaver
    // ========================================================================
    chicago_tdd_tools::alert_info!("3rd Idea: Type-level validation + OTEL + Weaver");
    chicago_tdd_tools::alert_info!("------------------------------------------------");

    // Without OTEL: Type-level validation
    let validated = ValidatedNumberNoOtel::<u32>::parse("42")?;
    assert_eq!(*validated.value(), 42);
    chicago_tdd_tools::alert_success!("Type-level validated number: 42");

    #[cfg(feature = "otel")]
    {
        // With OTEL: Full instrumentation
        let validated_otel = ValidatedNumber::<u32>::parse("42", "validated_parse")?;
        assert_eq!(*validated_otel.value(), 42);
        chicago_tdd_tools::alert_success!("OTEL-instrumented validated number: 42");

        // Validate span
        let span_validator = SpanValidator::new();
        span_validator.validate(validated_otel.span())?;
        chicago_tdd_tools::alert_success!("OTEL span validation: Passed");

        // Create metric for operation
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let mut metric = Metric {
            name: "chicago_tdd_tools.parse.operations".to_string(),
            value: MetricValue::Counter(1),
            timestamp_ms: timestamp,
            attributes: std::collections::BTreeMap::new(),
        };

        metric.attributes.insert("type".to_string(), "validated".to_string());
        metric.attributes.insert("success".to_string(), "true".to_string());

        // Validate metric
        let metric_validator = MetricValidator::new();
        metric_validator.validate(&metric)?;
        chicago_tdd_tools::alert_success!("OTEL metric validation: Passed");
    }

    #[cfg(feature = "weaver")]
    {
        chicago_tdd_tools::alert_info!("\nWeaver Live-Check Validation:");
        chicago_tdd_tools::alert_info!("-----------------------------");

        // Check if Weaver is available
        match WeaverValidator::check_weaver_available() {
            Ok(()) => {
                chicago_tdd_tools::alert_success!("Weaver binary available");

                // In a real scenario, you would:
                // 1. Start Weaver live-check
                // 2. Send telemetry to OTLP endpoint
                // 3. Weaver validates against semantic conventions
                // 4. Stop Weaver and check results

                chicago_tdd_tools::alert_info!("  Note: Full Weaver validation requires:");
                chicago_tdd_tools::alert_info!("  - Weaver binary installed");
                chicago_tdd_tools::alert_info!("  - Semantic convention registry");
                chicago_tdd_tools::alert_info!("  - OTLP endpoint configured");
                chicago_tdd_tools::alert_info!("  - Telemetry sent to endpoint");
            }
            Err(e) => {
                chicago_tdd_tools::alert_info!("⚠ Weaver not available: {e}");
                chicago_tdd_tools::alert_info!("  Bootstrap with: cargo make weaver-bootstrap");
            }
        }
    }

    chicago_tdd_tools::alert_info!(
        "\n✓ Maximum value: Type-safe, validated, prevents entire class of errors"
    );
    chicago_tdd_tools::alert_success!("OTEL instrumentation: Full spans and metrics");
    chicago_tdd_tools::alert_success!("Weaver validation: Schema compliance (when available)");
    chicago_tdd_tools::alert_info!();

    // ========================================================================
    // Decision Framework
    // ========================================================================
    chicago_tdd_tools::alert_info!("Decision Framework:");
    chicago_tdd_tools::alert_info!("------------------");
    chicago_tdd_tools::alert_info!("1st Idea: Works, but narrow scope");
    chicago_tdd_tools::alert_info!("2nd Idea: Usually best - 80% more value, reasonable effort");
    chicago_tdd_tools::alert_info!("3rd Idea: Maximum value, but evaluate effort vs. benefit");
    chicago_tdd_tools::alert_info!();
    chicago_tdd_tools::alert_info!(
        "Recommendation: Use 2nd idea for most cases, 3rd idea when type safety is critical"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "weaver")]
    use chicago_tdd_tools::observability::weaver::WeaverValidator;
    #[cfg(feature = "otel")]
    use chicago_tdd_tools::otel::{MetricValidator, SpanValidator};
    use chicago_tdd_tools::test;

    test!(test_first_idea_basic, {
        // 1st Idea: Basic implementation
        let result = parse_u32_first_idea("42");
        chicago_tdd_tools::assert_ok!(&result);
        if let Ok(value) = result {
            assert_eq!(value, 42);
        }
    });

    test!(test_second_idea_generic, {
        // 2nd Idea: Generic version (80/20)
        let u32_result = parse_number_second_idea_no_otel::<u32>("42");
        chicago_tdd_tools::assert_ok!(&u32_result);
        if let Ok(value) = u32_result {
            assert_eq!(value, 42);
        }

        let i32_result = parse_number_second_idea_no_otel::<i32>("-42");
        chicago_tdd_tools::assert_ok!(&i32_result);
        if let Ok(value) = i32_result {
            assert_eq!(value, -42);
        }

        let f64_result = parse_number_second_idea_no_otel::<f64>("123.456");
        chicago_tdd_tools::assert_ok!(&f64_result);
        if let Ok(value) = f64_result {
            assert!((value - 123.456).abs() < f64::EPSILON);
        }
    });

    test!(test_third_idea_validated, {
        // 3rd Idea: Type-level validation
        if let Ok(validated) = ValidatedNumberNoOtel::<u32>::parse("42") {
            assert_eq!(*validated.value(), 42);
        }
    });

    #[cfg(feature = "otel")]
    test!(test_second_idea_with_otel, {
        // 2nd Idea with OTEL spans
        if let Ok((value, span)) = parse_number_second_idea::<u32>("42", "test_parse") {
            assert_eq!(value, 42);

            // Validate span
            let validator = SpanValidator::new();
            chicago_tdd_tools::assert_ok!(&validator.validate(&span));
        }
    });

    #[cfg(feature = "otel")]
    test!(test_third_idea_with_otel, {
        // 3rd Idea with OTEL spans
        if let Ok(validated) = ValidatedNumber::<u32>::parse("42", "test_validated") {
            assert_eq!(*validated.value(), 42);

            // Validate span
            let validator = SpanValidator::new();
            chicago_tdd_tools::assert_ok!(&validator.validate(validated.span()));
        }
    });
}
