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

#[cfg(test)]
mod tests {
    use chicago_tdd_tools::assert_eq_msg;
    use chicago_tdd_tools::core::builders::TestDataBuilder;
    use chicago_tdd_tools::test;
    use chicago_tdd_tools::{assert_ok, AssertionBuilder, GenericTestDataBuilder, ValidatedTestDataBuilder};
    
    // **Poka-yoke**: Import validators only where used (conditionally compiled)
    #[cfg(feature = "otel")]
    use chicago_tdd_tools::observability::otel::{MetricValidator, OtelTestHelper, SpanValidator};
    
    #[cfg(feature = "otel")]
    use chicago_tdd_tools::assertions::assert_that_with_msg;
    
    #[cfg(feature = "otel")]
    use chicago_tdd_tools::ValidatedAssertion;

    // ========================================================================
    // 1st Idea Tests: Basic implementations
    // ========================================================================

    test!(test_first_idea_builder, {
        // Arrange: Create builder with test data
        let builder = TestDataBuilder::new().with_var("key1", "value1").with_var("key2", "value2");

        // Act: Build test data
        let data = builder.build();

        // Assert: Verify data contains expected values
        assert_eq_msg!(data.get("key1"), Some(&"value1".to_string()), "Key1 should match");
        assert_eq_msg!(data.get("key2"), Some(&"value2".to_string()), "Key2 should match");
    });

    // ========================================================================
    // 2nd Idea Tests: Generic versions with OTEL
    // ========================================================================

    test!(test_second_idea_generic_builder, {
        // Arrange: Create generic builder with test data
        let builder = GenericTestDataBuilder::<String, String>::new()
            .with_var("key1".to_string(), "value1".to_string())
            .with_var("key2", "value2"); // Works with &str too

        // Act: Build test data
        let data = builder.build();

        // Assert: Verify data contains expected values
        assert_eq_msg!(data.get("key1"), Some(&"value1".to_string()), "Key1 should match");
        assert_eq_msg!(data.get("key2"), Some(&"value2".to_string()), "Key2 should match");
    });

    #[cfg(feature = "otel")]
    test!(test_second_idea_builder_with_otel, {
        // Arrange: Create generic builder with test data
        let builder = GenericTestDataBuilder::<String, String>::new()
            .with_var("key1", "value1")
            .with_var("key2", "value2");

        // Act: Build test data with OTEL span
        let (data, span) = builder.build_with_otel("test_build");

        // Assert: Verify data contains expected values
        assert_eq_msg!(data.get("key1"), Some(&"value1".to_string()), "Key1 should match");
        assert_eq_msg!(data.get("key2"), Some(&"value2".to_string()), "Key2 should match");

        // Assert: Validate OTEL span
        let validator = SpanValidator::new();
        assert_ok!(&validator.validate(&span));

        // Assert: Verify span attributes
        assert_eq_msg!(&span.name, &"test_build".to_string(), "Span name should match");
        assert_eq_msg!(
            span.attributes.get("item_count"),
            Some(&"2".to_string()),
            "Item count should match"
        );
        assert_eq_msg!(
            &span.status,
            &chicago_tdd_tools::otel::types::SpanStatus::Ok,
            "Span status should be Ok"
        );
    });

    test!(test_second_idea_assertion_builder, {
        // Arrange: Create assertion builder with test value
        let value = 42;
        let builder = AssertionBuilder::new(value).assert_that(|v| *v > 0).assert_eq(&42);

        // Act & Assert: Verify builder returns correct value
        assert_eq_msg!(&builder.into_value(), &42, "Builder value should match");
    });

    #[cfg(feature = "otel")]
    test!(test_second_idea_assertion_builder_with_otel, {
        // Arrange: Create assertion builder with test value and OTEL span
        let value = 42;
        let builder =
            AssertionBuilder::new(value).with_span("test_assertion").assert_that(|v| *v > 0);

        // Act: Get span from builder
        let span = builder.into_span();
        assert_that_with_msg(&span.is_some(), |v| *v, "Span should be present");

        // Assert: Validate OTEL span
        let span = span.unwrap();
        let validator = SpanValidator::new();
        assert_ok!(&validator.validate(&span));
        assert_eq_msg!(
            &span.status,
            &chicago_tdd_tools::otel::types::SpanStatus::Ok,
            "Span status should be Ok"
        );
    });

    // ========================================================================
    // 3rd Idea Tests: Type-level validation + OTEL + Weaver
    // ========================================================================

    test!(test_third_idea_validated_builder, {
        // Arrange: Create validated builder with test data
        let builder = ValidatedTestDataBuilder::<()>::new()
            .with_var("key1", "value1")
            .with_var("key2", "value2");

        // Act: Build test data
        let data = builder.build();

        // Assert: Verify data contains expected values
        assert_eq_msg!(data.get("key1"), Some(&"value1".to_string()), "Key1 should match");
        assert_eq_msg!(data.get("key2"), Some(&"value2".to_string()), "Key2 should match");
    });

    #[cfg(feature = "otel")]
    test!(test_third_idea_validated_builder_with_otel, {
        // Arrange: Create validated builder with test data and OTEL span
        let builder = ValidatedTestDataBuilder::<()>::new()
            .start_span("test_validated_build")
            .with_var("key1", "value1")
            .with_var("key2", "value2");

        // Act: Build test data with OTEL span
        let (data, span_opt) = builder.build_with_otel();

        // Assert: Verify data contains expected values
        assert_eq_msg!(data.get("key1"), Some(&"value1".to_string()), "Key1 should match");
        assert_eq_msg!(data.get("key2"), Some(&"value2".to_string()), "Key2 should match");

        // Assert: Validate OTEL span exists and is valid
        assert_that_with_msg(&span_opt.is_some(), |v| *v, "Span should be present");
        let span = span_opt.unwrap();
        let validator = SpanValidator::new();
        assert_ok!(&validator.validate(&span));

        // Assert: Verify span attributes
        assert_eq_msg!(
            span.attributes.get("item_count"),
            Some(&"2".to_string()),
            "Item count should match"
        );
        assert_eq_msg!(
            &span.status,
            &chicago_tdd_tools::otel::types::SpanStatus::Ok,
            "Span status should be Ok"
        );
    });

    #[cfg(feature = "otel")]
    test!(test_third_idea_validated_assertion, {
        // Arrange: Create validated assertion with test value
        let value = 42;
        let validated =
            ValidatedAssertion::new(value, "test_validated_assertion").assert_that(|v| *v > 0);

        // Act: Get span and metric from validated assertion (before into_value moves validated)
        let span = validated.span();
        let metric = validated.metric();

        // Assert: Validate OTEL span (before moving validated)
        let span_validator = SpanValidator::new();
        assert_ok!(&span_validator.validate(span));
        assert_eq_msg!(
            &span.status,
            &chicago_tdd_tools::otel::types::SpanStatus::Ok,
            "Span status should be Ok"
        );

        // Assert: Validate OTEL metric (before moving validated)
        assert_that_with_msg(&metric.is_some(), |v| *v, "Metric should be present");
        let metric = metric.unwrap();
        let metric_validator = MetricValidator::new();
        assert_ok!(&metric_validator.validate(metric));
        assert_eq_msg!(
            &metric.name,
            &"chicago_tdd_tools.assertions.total".to_string(),
            "Metric name should match"
        );

        // Now we can move validated to get the value (after all borrows are done)
        let result_value = validated.into_value();

        // Assert: Verify value matches expected
        assert_eq_msg!(&result_value, &42, "Value should match");
    });

    // ========================================================================
    // Weaver Validation Tests
    // ========================================================================

    #[cfg(feature = "weaver")]
    test!(test_weaver_validator_creation, {
        // Arrange: Create registry path
        use std::path::PathBuf;
        // **Poka-yoke**: Import only where used to prevent unused import errors
        use chicago_tdd_tools::observability::weaver::WeaverValidator;
        let registry_path = PathBuf::from("registry/");

        // Act: Create Weaver validator
        let validator = WeaverValidator::new(registry_path);

        // Assert: Verify validator was created correctly
        assert_that_with_msg(
            &!validator.is_running(),
            |v| *v,
            "Validator should not be running initially",
        );
        assert_eq_msg!(
            &validator.otlp_endpoint(),
            &"http://127.0.0.1:4317".to_string(),
            "OTLP endpoint should match"
        );
    });

    #[cfg(all(feature = "otel", feature = "weaver"))]
    test!(test_otel_spans_validated_by_helper, {
        // Arrange: Create test OTEL span
        use chicago_tdd_tools::otel::types::{
            Span, SpanContext, SpanId, SpanRelationship, SpanState, SpanStatus, TraceId,
        };

        let span = Span {
            context: SpanContext {
                trace_id: TraceId(12345),
                span_id: SpanId(67890),
                relationship: SpanRelationship::Root,
                flags: 1,
            },
            name: "test.span".to_string(),
            state: SpanState::Completed { start_time_ms: 1000, end_time_ms: 2000 },
            attributes: std::collections::BTreeMap::new(),
            events: Vec::new(),
            status: SpanStatus::Ok,
        };

        // Act: Validate span using helper
        let helper = OtelTestHelper::new();
        helper.assert_spans_valid(&[span]);

        // Assert: If no panic, span is valid (assert_spans_valid panics on invalid spans)
    });

    #[cfg(all(feature = "otel", feature = "weaver"))]
    test!(test_otel_metrics_validated_by_helper, {
        // Arrange: Create test OTEL metric
        use chicago_tdd_tools::otel::types::{Metric, MetricValue};
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;

        let metric = Metric {
            name: "test.metric".to_string(),
            value: MetricValue::Counter(42),
            timestamp_ms: timestamp,
            attributes: std::collections::BTreeMap::new(),
        };

        // Act: Validate metric using helper
        let helper = OtelTestHelper::new();
        helper.assert_metrics_valid(&[metric]);

        // Assert: If no panic, metric is valid (assert_metrics_valid panics on invalid metrics)
    });
}
