# OTEL Instrumentation

> 🔧 How-to | Instrument and verify OpenTelemetry spans and metrics

OpenTelemetry instrumentation provides observability into your operations.

## Quick Reference: OTEL API

| Component | Method / Field | Parameters | Returns | Purpose |
|-----------|----------------|------------|---------|---------|
| `Span` | `new_active()` | `context, name, start_time, attributes, events, status` | `Span` | Create an active span |
| `Span` | `complete()` | `end_time` | `Result<(), Error>` | Complete the span |
| `Span` | `status` | `SpanStatus` value | N/A | Status of the span |
| `Metric` | `name` | `String` | N/A | Name of the metric |
| `Metric` | `value` | `MetricValue` value | N/A | Value of the metric |
| `SpanValidator` | `new()` | None | `SpanValidator` | Create a new validator |
| `SpanValidator` | `validate()` | `&Span` | `Result<(), ValidationError>` | Validate span attributes |

## Creating Spans

```rust
use chicago_tdd_tools::otel::*;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::BTreeMap;

test!(test_span_creation, {
    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_millis() as u64;

    let mut span = Span::new_active(
        SpanContext::root(TraceId(12345), SpanId(67890), 1),
        "parse_user_data",
        start_time,
        BTreeMap::new(),
        Vec::new(),
        SpanStatus::Unset,
    );

    // Add attributes
    span.attributes.insert("user_id".to_string(), "123".to_string());
    span.attributes.insert("operation".to_string(), "parse".to_string());

    // Complete span
    let end_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_millis() as u64;

    span.complete(end_time)?;
    span.status = SpanStatus::Ok;

    assert_eq!(span.status, SpanStatus::Ok);
});
```

## Span Status

Mark success or error:

```rust
// Success
span.status = SpanStatus::Ok;

// Error
span.status = SpanStatus::Error;

// Unset
span.status = SpanStatus::Unset;
```

## Creating Metrics

```rust
test!(test_metric_creation, {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_millis() as u64;

    let mut metric = Metric {
        name: "parsing_operations_total".to_string(),
        value: MetricValue::Counter(1),
        timestamp_ms: timestamp,
        attributes: BTreeMap::new(),
    };

    metric.attributes.insert("type".to_string(), "user".to_string());
    metric.attributes.insert("success".to_string(), "true".to_string());

    // Metric is ready to send to observability backend
    assert_eq!(metric.name, "parsing_operations_total");
});
```

## Span Validation

```rust
test!(test_span_validation, {
    let span = create_valid_span()?;
    let validator = SpanValidator::new();
    
    assert_ok!(&validator.validate(&span));
});
```

## Best Practices

✅ **Do:**
- Use consistent span names
- Add meaningful attributes
- Track operation timing
- Mark errors explicitly

❌ **Don't:**
- Include sensitive data in attributes
- Create excessive spans
- Forget timing information

## Next Steps

**Learn more:**
- [Weaver Validation](weaver.md) - Validate telemetry against semantic conventions
- [Observability & Quality](observability.md) - Full observability framework
- [Best Practices](best-practices.md) - Observability patterns

**Ready to instrument?**
- Add spans to your critical code paths
- Include meaningful attributes for debugging
- Validate spans with `SpanValidator`
- Combine with Weaver for compliance checking