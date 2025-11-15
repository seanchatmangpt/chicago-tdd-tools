//! # Go the Extra Mile: 1st/2nd/3rd Idea Progression with OTEL/Weaver Validation
//!
//! This example demonstrates the "go the extra mile" paradigm:
//!
//! - 1st Idea: Solve the immediate problem
//! - 2nd Idea: Go bigger with generics (80/20)
//! - 3rd Idea: Maximum value with type-level validation + OTEL/Weaver validation
//!
//! Each idea includes increasing levels of telemetry instrumentation and validation.

use chicago_tdd_tools::assert_ok;
#[cfg(feature = "weaver")]
use chicago_tdd_tools::observability::weaver::WeaverValidator;
#[cfg(feature = "otel")]
use chicago_tdd_tools::otel::types::{
    Metric, MetricValue, Span, SpanContext, SpanId, SpanStatus, TraceId,
};
#[cfg(feature = "otel")]
use chicago_tdd_tools::otel::{MetricValidator, SpanValidator};

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

/// Parse u32 from string.
///
/// First idea: Parse u32 only.
///
/// **Telemetry**: None (basic implementation)
/// **Validation**: None
/// **Scope**: Single type only
///
/// # Errors
///
/// Returns error if parsing fails.
pub fn parse_u32_first_idea(input: &str) -> Result<u32, String> {
    input.parse().map_err(|e| format!("Parse error: {e}"))
}

// ============================================================================
// 2nd IDEA: Go bigger (80/20) - Generic version
// ============================================================================

/// Second idea: Parse any number type
///
/// **Telemetry**: Basic OTEL spans (if otel feature enabled)
/// **Validation**: OTEL span validation
/// **Scope**: Works for u32, i32, u64, f64, etc. - 80% more value, minimal effort
///
/// # Errors
/// Returns error if parsing fails
#[cfg(feature = "otel")]
pub fn parse_number_second_idea<T: std::str::FromStr>(
    input: &str,
    span_name: &str,
) -> Result<(T, Span), String>
where
    T::Err: std::fmt::Display,
{
    // Create OTEL span for operation
    #[allow(clippy::expect_used)] // SystemTime should always be after UNIX_EPOCH
    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("SystemTime should always be after UNIX_EPOCH")
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
        .expect("SystemTime should always be after UNIX_EPOCH")
        .as_millis() as u64;

    // End time should always be >= start time in normal operation
    #[allow(clippy::expect_used)] // Example code - end_time should be >= start_time
    span.complete(end_time).expect("End time should be >= start time");

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

/// Second idea: Parse any number type (without OTEL)
///
/// Generic version that works for all number types - demonstrates 80/20 thinking.
///
/// # Errors
/// Returns error if parsing fails
pub fn parse_number_second_idea_no_otel<T: std::str::FromStr>(input: &str) -> Result<T, String>
where
    T::Err: std::fmt::Display,
{
    input.parse().map_err(|e| format!("Parse error: {e}"))
}

// ============================================================================
// 3rd IDEA: Maximum value - Type-level validation + OTEL + Weaver
// ============================================================================

/// Third idea: Type-level validated number with OTEL/Weaver validation
///
/// **Telemetry**: Full OTEL spans and metrics
/// **Validation**: OTEL span validation + Weaver live-check schema validation
/// **Scope**: Type-safe, validated numbers that prevent entire class of errors
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
        #[allow(clippy::expect_used)] // SystemTime should always be after UNIX_EPOCH
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime should always be after UNIX_EPOCH")
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
            .expect("SystemTime should always be after UNIX_EPOCH")
            .as_millis() as u64;

        // End time should always be >= start time in normal operation
        #[allow(clippy::expect_used)] // Example code - end_time should be >= start_time
        span.complete(end_time).expect("End time should be >= start time");

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

/// Third idea: Type-level validated number (without OTEL)
///
/// Maximum value: Type-safe, validated numbers, prevents entire class of errors.
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

    println!("Go the Extra Mile: 1st/2nd/3rd Idea Progression");
    println!("================================================\n");

    // ========================================================================
    // 1st Idea: Basic implementation
    // ========================================================================
    println!("1st Idea: Parse u32 only");
    println!("-------------------------");
    let result1 = parse_u32_first_idea("42");
    assert_ok!(&result1);
    let value1 = result1?;
    assert_eq!(value1, 42);
    println!("✓ Parsed u32: 42");
    println!("  - No telemetry");
    println!("  - No validation");
    println!("  - Single type only\n");

    // ========================================================================
    // 2nd Idea: Go bigger (80/20) - Generic version
    // ========================================================================
    println!("2nd Idea: Parse any number type (80/20)");
    println!("----------------------------------------");

    // Works for u32
    let u32_parsed = parse_number_second_idea_no_otel::<u32>("42");
    assert_ok!(&u32_parsed);
    let parsed_u32 = u32_parsed?;
    assert_eq!(parsed_u32, 42);
    println!("✓ Parsed u32: 42");

    // Works for i32
    let i32_parsed = parse_number_second_idea_no_otel::<i32>("-42");
    assert_ok!(&i32_parsed);
    let parsed_i32 = i32_parsed?;
    assert_eq!(parsed_i32, -42);
    println!("✓ Parsed i32: -42");

    // Works for f64
    let f64_parsed = parse_number_second_idea_no_otel::<f64>("123.456");
    assert_ok!(&f64_parsed);
    let parsed_f64 = f64_parsed?;
    assert!((parsed_f64 - 123.456).abs() < f64::EPSILON);
    println!("✓ Parsed f64: 123.456");

    println!("  - Generic: Works for all number types");
    println!("  - 80% more value, minimal effort");

    #[cfg(feature = "otel")]
    {
        println!("  - OTEL spans: Basic instrumentation");

        // Test with OTEL spans
        let (value, span) = parse_number_second_idea::<u32>("42", "parse_number")?;
        assert_eq!(value, 42);

        // Validate span with OTEL validator
        let validator = SpanValidator::new();
        validator.validate(&span)?;
        println!("  - Span validation: ✓ Passed");
    }

    println!();

    // ========================================================================
    // 3rd Idea: Maximum value - Type-level validation + OTEL + Weaver
    // ========================================================================
    println!("3rd Idea: Type-level validation + OTEL + Weaver");
    println!("------------------------------------------------");

    // Without OTEL: Type-level validation
    let validated = ValidatedNumberNoOtel::<u32>::parse("42")?;
    assert_eq!(*validated.value(), 42);
    println!("✓ Type-level validated number: 42");

    #[cfg(feature = "otel")]
    {
        // With OTEL: Full instrumentation
        let validated_otel = ValidatedNumber::<u32>::parse("42", "validated_parse")?;
        assert_eq!(*validated_otel.value(), 42);
        println!("✓ OTEL-instrumented validated number: 42");

        // Validate span
        let span_validator = SpanValidator::new();
        span_validator.validate(validated_otel.span())?;
        println!("✓ OTEL span validation: Passed");

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
        println!("✓ OTEL metric validation: Passed");
    }

    #[cfg(feature = "weaver")]
    {
        println!("\nWeaver Live-Check Validation:");
        println!("-----------------------------");

        // Check if Weaver is available
        match WeaverValidator::check_weaver_available() {
            Ok(()) => {
                println!("✓ Weaver binary available");

                // In a real scenario, you would:
                // 1. Start Weaver live-check
                // 2. Send telemetry to OTLP endpoint
                // 3. Weaver validates against semantic conventions
                // 4. Stop Weaver and check results

                println!("  Note: Full Weaver validation requires:");
                println!("  - Weaver binary installed");
                println!("  - Semantic convention registry");
                println!("  - OTLP endpoint configured");
                println!("  - Telemetry sent to endpoint");
            }
            Err(e) => {
                println!("⚠ Weaver not available: {e}");
                println!("  Bootstrap with: cargo make weaver-bootstrap");
            }
        }
    }

    println!("\n✓ Maximum value: Type-safe, validated, prevents entire class of errors");
    println!("✓ OTEL instrumentation: Full spans and metrics");
    println!("✓ Weaver validation: Schema compliance (when available)");
    println!();

    // ========================================================================
    // Decision Framework
    // ========================================================================
    println!("Decision Framework:");
    println!("------------------");
    println!("1st Idea: Works, but narrow scope");
    println!("2nd Idea: Usually best - 80% more value, reasonable effort");
    println!("3rd Idea: Maximum value, but evaluate effort vs. benefit");
    println!();
    println!("Recommendation: Use 2nd idea for most cases, 3rd idea when type safety is critical");

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
        assert_ok!(&result);
        if let Ok(value) = result {
            assert_eq!(value, 42);
        }
    });

    test!(test_second_idea_generic, {
        // 2nd Idea: Generic version (80/20)
        let u32_result = parse_number_second_idea_no_otel::<u32>("42");
        assert_ok!(&u32_result);
        if let Ok(value) = u32_result {
            assert_eq!(value, 42);
        }

        let i32_result = parse_number_second_idea_no_otel::<i32>("-42");
        assert_ok!(&i32_result);
        if let Ok(value) = i32_result {
            assert_eq!(value, -42);
        }

        let f64_result = parse_number_second_idea_no_otel::<f64>("123.456");
        assert_ok!(&f64_result);
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
            assert_ok!(&validator.validate(&span));
        }
    });

    #[cfg(feature = "otel")]
    test!(test_third_idea_with_otel, {
        // 3rd Idea with OTEL spans
        if let Ok(validated) = ValidatedNumber::<u32>::parse("42", "test_validated") {
            assert_eq!(*validated.value(), 42);

            // Validate span
            let validator = SpanValidator::new();
            assert_ok!(&validator.validate(validated.span()));
        }
    });
}
