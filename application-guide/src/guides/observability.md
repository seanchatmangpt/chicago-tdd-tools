# Observability & Quality

> 🔧 **HOW-TO** | 📚 **REFERENCE** | Add observability and measure quality

Chicago TDD Tools provides comprehensive observability and quality measurement capabilities.

## Quick Reference: Observability API

| Component | Purpose | Key Methods/Fields |
|-----------|---------|-------------------|
| `Span::new_active()` | Create an OTEL span | Parameters: context, name, start_time, attributes, events, status |
| `Span.attributes` | Store span metadata | `.insert(key, value)` |
| `Span.complete()` | Finish span timing | Parameter: end_time |
| `Span.status` | Set result status | Variants: Ok, Error, Unknown |
| `Span.validate()` | Verify span correctness | Returns: `Result<(), Error>` |
| `Metric` struct | Track measurements | Fields: name, value, timestamp_ms, attributes |
| `MetricValue` enum | Metric types | Variants: Counter, Gauge, Histogram |
| `MetricValidator::new()` | Validate metrics | `.validate(&metric)` |
| `WeaverValidator::check_weaver_available()` | Check Weaver CLI | Returns: `Result<(), Error>` |
| `SpanValidator::new()` | Validate span format | `.validate(&span)` |

## Overview

Observability helps you understand what your code is doing:

- **OTEL Instrumentation**: Track operations with spans and metrics
- **Weaver Validation**: Ensure telemetry matches semantic conventions
- **Coverage Measurement**: Verify test coverage
- **Performance Tracking**: Measure operation timing

## OTEL Spans

OTEL (OpenTelemetry) spans track operations:

```rust
use chicago_tdd_tools::otel::*;

test!(test_with_span, {
    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_millis() as u64;

    // Create a span
    let mut span = Span::new_active(
        SpanContext::root(TraceId(123), SpanId(456), 1),
        "parse_operation",
        start_time,
        BTreeMap::new(),
        Vec::new(),
        SpanStatus::Unset,
    );

    // Add attributes
    span.attributes.insert("input".to_string(), "42".to_string());

    // Do work
    let result = "42".parse::<u32>()?;

    // Complete span
    let end_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_millis() as u64;

    span.complete(end_time)?;
    span.status = SpanStatus::Ok;

    // Span is now complete with timing
    assert_ok!(&span.validate());
});
```

## OTEL Metrics

Track measurements over time:

```rust
use chicago_tdd_tools::otel::*;

test!(test_with_metric, {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_millis() as u64;

    let mut metric = Metric {
        name: "requests_total".to_string(),
        value: MetricValue::Counter(42),
        timestamp_ms: timestamp,
        attributes: BTreeMap::new(),
    };

    metric.attributes.insert("endpoint".to_string(), "/api/users".to_string());
    metric.attributes.insert("status".to_string(), "success".to_string());

    // Validate metric
    let validator = MetricValidator::new();
    assert_ok!(&validator.validate(&metric));
});
```

## Weaver Live-Check

Validate telemetry against semantic conventions:

```rust
test!(test_weaver_validation, {
    // Check if Weaver is available
    match WeaverValidator::check_weaver_available() {
        Ok(()) => {
            // Weaver is available
            // Can validate OTEL spans against semantic conventions
            alert_success!("Weaver available");
        }
        Err(e) => {
            alert_info!("Weaver not available: {}", e);
            alert_info!("Install with: cargo make weaver-bootstrap");
        }
    }
});
```

## Coverage Measurement

Measure test coverage:

```bash
# Run coverage
cargo make coverage

# Generate report
cargo make coverage-report
```

Coverage shows:
- Code coverage percentage
- Covered lines
- Uncovered lines
- Branch coverage

**Target**: 80%+ coverage for critical code

## Best Practices

### OTEL Spans

✅ **Do:**
- Add meaningful attributes
- Track timing
- Mark errors with SpanStatus
- Propagate context between services

❌ **Don't:**
- Create spans for every operation (too noisy)
- Include sensitive data in attributes
- Forget to complete spans

### Weaver Validation

✅ **Do:**
- Use semantic conventions
- Validate telemetry early
- Document telemetry schema
- Keep conventions up-to-date

❌ **Don't:**
- Use custom attribute names
- Skip validation
- Ignore schema mismatches

### Coverage

✅ **Do:**
- Aim for 80%+ coverage
- Focus on critical paths
- Test error paths (often uncovered)
- Review coverage reports

❌ **Don't:**
- Obsess over 100% coverage
- Ignore untested lines
- Only focus on coverage number

## Combining Observability with Testing

### OTEL + Unit Tests

```rust
test!(test_with_otel, {
    let span = create_test_span("my_operation");

    // Do work
    let result = my_function()?;

    // Verify behavior AND telemetry
    assert_ok!(&result);
    span.validate()?;
    assert_eq!(span.status, SpanStatus::Ok);
});
```

### Metrics + Property-Based Testing

```rust
test!(test_with_metrics, {
    let strategy = ProptestStrategy::new().with_cases(100);

    strategy.test(any::<u32>(), |num| {
        let timestamp = SystemTime::now()...;
        let mut metric = Metric {
            name: "parsing_attempts".to_string(),
            value: MetricValue::Counter(1),
            timestamp_ms: timestamp,
            attributes: BTreeMap::new(),
        };

        // Validate metric
        let validator = MetricValidator::new();
        validator.validate(&metric).is_ok()
    });
});
```

## Real-World Example

```rust
test!(test_api_with_observability, {
    // Span for entire operation
    let mut operation_span = Span::new_active(
        SpanContext::root(TraceId(1), SpanId(1), 1),
        "api_request",
        start_time,
        BTreeMap::new(),
        Vec::new(),
        SpanStatus::Unset,
    );

    // Make API call
    let result = api_client.get_user(123)?;

    // Record metric
    let mut metric = Metric {
        name: "api_requests_total".to_string(),
        value: MetricValue::Counter(1),
        timestamp_ms: current_time,
        attributes: {
            let mut m = BTreeMap::new();
            m.insert("endpoint".to_string(), "/users".to_string());
            m.insert("status".to_string(), "success".to_string());
            m
        },
    };

    // Validate everything
    let span_validator = SpanValidator::new();
    assert_ok!(&span_validator.validate(&operation_span));

    let metric_validator = MetricValidator::new();
    assert_ok!(&metric_validator.validate(&metric));
});
```

## Observability Checklist

For production code:

- [ ] Operations tracked with OTEL spans
- [ ] Meaningful attributes on spans
- [ ] Metrics for important measurements
- [ ] Error cases marked in telemetry
- [ ] Telemetry validates against conventions
- [ ] 80%+ test coverage
- [ ] Error paths covered
- [ ] Boundary conditions tested

## Next Steps

See how to combine observability with real applications: [Real-World Applications](real-world.md)

---

## Summary

Observability provides:
- ✅ OTEL spans for operation tracking
- ✅ Metrics for measurements
- ✅ Weaver validation for compliance
- ✅ Coverage for test quality

Combined with testing for complete confidence.

