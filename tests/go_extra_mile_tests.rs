//! Unit tests for "Go the Extra Mile" implementations with OTEL/Weaver validation
//!
//! **Poka-yoke**: These are UNIT tests, not integration tests. They test types,
//! validators, and builders without requiring external services (Docker, Weaver CLI, etc.).
//!
//! These tests validate that the 1st/2nd/3rd idea progression works correctly
//! and that OTEL spans/metrics are properly validated using type-level validators.
//!
//! **CRITICAL**: These tests do NOT use testcontainers, do NOT require Docker,
//! and do NOT start Weaver CLI. They only test Rust types and validators.

use chicago_tdd_tools::chicago_test;
use chicago_tdd_tools::prelude::*;

#[cfg(feature = "otel")]
use chicago_tdd_tools::otel::{MetricValidator, OtelTestHelper, SpanValidator};
#[cfg(feature = "weaver")]
use chicago_tdd_tools::weaver::{validate_schema_static, WeaverValidator};
use chicago_tdd_tools::{AssertionBuilder, GenericTestDataBuilder, ValidatedTestDataBuilder};

#[cfg(feature = "otel")]
use chicago_tdd_tools::ValidatedAssertion;

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // 1st Idea Tests: Basic implementations
    // ========================================================================

    chicago_test!(test_first_idea_builder, {
        // 1st Idea: Basic TestDataBuilder
        let builder = TestDataBuilder::new().with_var("key1", "value1").with_var("key2", "value2");

        let data = builder.build();
        assert_eq!(data.get("key1"), Some(&"value1".to_string()));
        assert_eq!(data.get("key2"), Some(&"value2".to_string()));
    });

    // ========================================================================
    // 2nd Idea Tests: Generic versions with OTEL
    // ========================================================================

    chicago_test!(test_second_idea_generic_builder, {
        // 2nd Idea: Generic builder works with any Into<String> types
        let builder = GenericTestDataBuilder::<String, String>::new()
            .with_var("key1".to_string(), "value1".to_string())
            .with_var("key2", "value2"); // Works with &str too

        let data = builder.build();
        assert_eq!(data.get("key1"), Some(&"value1".to_string()));
        assert_eq!(data.get("key2"), Some(&"value2".to_string()));
    });

    #[cfg(feature = "otel")]
    chicago_test!(test_second_idea_builder_with_otel, {
        // 2nd Idea: Generic builder with OTEL spans
        let builder = GenericTestDataBuilder::<String, String>::new()
            .with_var("key1", "value1")
            .with_var("key2", "value2");

        let (data, span) = builder.build_with_otel("test_build");

        // Verify data
        assert_eq!(data.get("key1"), Some(&"value1".to_string()));
        assert_eq!(data.get("key2"), Some(&"value2".to_string()));

        // Validate OTEL span
        let validator = SpanValidator::new();
        assert_ok!(&validator.validate(&span));

        // Verify span attributes
        assert_eq!(span.name, "test_build");
        assert_eq!(span.attributes.get("item_count"), Some(&"2".to_string()));
        assert_eq!(span.status, chicago_tdd_tools::otel::types::SpanStatus::Ok);
    });

    chicago_test!(test_second_idea_assertion_builder, {
        // 2nd Idea: Assertion builder for composable assertions
        let value = 42;
        let builder = AssertionBuilder::new(value).assert_that(|v| *v > 0).assert_eq(&42);

        assert_eq!(builder.into_value(), 42);
    });

    #[cfg(feature = "otel")]
    chicago_test!(test_second_idea_assertion_builder_with_otel, {
        // 2nd Idea: Assertion builder with OTEL spans
        let value = 42;
        let builder =
            AssertionBuilder::new(value).with_span("test_assertion").assert_that(|v| *v > 0);

        let span = builder.into_span();
        assert!(span.is_some());

        let span = span.unwrap();
        let validator = SpanValidator::new();
        assert_ok!(&validator.validate(&span));
        assert_eq!(span.status, chicago_tdd_tools::otel::types::SpanStatus::Ok);
    });

    // ========================================================================
    // 3rd Idea Tests: Type-level validation + OTEL + Weaver
    // ========================================================================

    chicago_test!(test_third_idea_validated_builder, {
        // 3rd Idea: Validated builder with type-level validation
        let builder = ValidatedTestDataBuilder::<()>::new()
            .with_var("key1", "value1")
            .with_var("key2", "value2");

        let data = builder.build();
        assert_eq!(data.get("key1"), Some(&"value1".to_string()));
        assert_eq!(data.get("key2"), Some(&"value2".to_string()));
    });

    #[cfg(feature = "otel")]
    chicago_test!(test_third_idea_validated_builder_with_otel, {
        // 3rd Idea: Validated builder with OTEL spans
        let builder = ValidatedTestDataBuilder::<()>::new()
            .start_span("test_validated_build")
            .with_var("key1", "value1")
            .with_var("key2", "value2");

        let (data, span_opt) = builder.build_with_otel();

        // Verify data
        assert_eq!(data.get("key1"), Some(&"value1".to_string()));
        assert_eq!(data.get("key2"), Some(&"value2".to_string()));

        // Validate OTEL span
        assert!(span_opt.is_some());
        let span = span_opt.unwrap();
        let validator = SpanValidator::new();
        assert_ok!(&validator.validate(&span));

        // Verify span attributes
        assert_eq!(span.attributes.get("item_count"), Some(&"2".to_string()));
        assert_eq!(span.status, chicago_tdd_tools::otel::types::SpanStatus::Ok);
    });

    #[cfg(feature = "otel")]
    chicago_test!(test_third_idea_validated_assertion, {
        // 3rd Idea: Validated assertion with OTEL spans and metrics
        let value = 42;
        let validated =
            ValidatedAssertion::new(value, "test_validated_assertion").assert_that(|v| *v > 0);

        // Verify value
        assert_eq!(*validated.into_value(), 42);

        // Validate OTEL span
        let span = validated.span();
        let span_validator = SpanValidator::new();
        assert_ok!(&span_validator.validate(span));
        assert_eq!(span.status, chicago_tdd_tools::otel::types::SpanStatus::Ok);

        // Validate OTEL metric
        let metric = validated.metric();
        assert!(metric.is_some());
        let metric = metric.unwrap();
        let metric_validator = MetricValidator::new();
        assert_ok!(&metric_validator.validate(metric));
        assert_eq!(metric.name, "chicago_tdd_tools.assertions.total");
    });

    // ========================================================================
    // Weaver Validation Tests
    // ========================================================================

    #[cfg(feature = "weaver")]
    chicago_test!(test_weaver_validator_creation, {
        // Test Weaver validator can be created
        use std::path::PathBuf;

        let registry_path = PathBuf::from("registry/");
        let validator = WeaverValidator::new(registry_path);

        // Verify validator was created
        assert!(!validator.is_running());
        assert_eq!(validator.otlp_endpoint(), "http://127.0.0.1:4317");
    });

    #[cfg(all(feature = "otel", feature = "weaver"))]
    chicago_test!(test_otel_spans_validated_by_helper, {
        // Test that OTEL spans can be validated using OtelTestHelper
        use chicago_tdd_tools::otel::types::{Span, SpanContext, SpanId, SpanStatus, TraceId};

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
            attributes: std::collections::BTreeMap::new(),
            events: Vec::new(),
            status: SpanStatus::Ok,
        };

        let helper = OtelTestHelper::new();
        helper.assert_spans_valid(&[span]);
    });

    #[cfg(all(feature = "otel", feature = "weaver"))]
    chicago_test!(test_otel_metrics_validated_by_helper, {
        // Test that OTEL metrics can be validated using OtelTestHelper
        use chicago_tdd_tools::otel::types::{Metric, MetricValue};
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;

        let metric = Metric {
            name: "test.metric".to_string(),
            value: MetricValue::Counter(42),
            timestamp_ms: timestamp,
            attributes: std::collections::BTreeMap::new(),
        };

        let helper = OtelTestHelper::new();
        helper.assert_metrics_valid(&[metric]);
    });
}
