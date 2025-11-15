# OTEL/Weaver Testing Example

**Category:** How-To Guide
**Level:** Advanced
**Prerequisites:** Understanding of observability and OpenTelemetry
**Features Required:** `otel`, `weaver`

---

## Overview

This example demonstrates how to use Chicago TDD Tools unified observability testing API with automatic resource management and zero-cost abstractions for validating OpenTelemetry spans/metrics and Weaver semantic conventions.

**What you'll learn:**
- OTEL span validation
- OTEL metric validation
- Weaver live-check integration
- Unified observability API

---

## Quick Start

```bash
cargo test --features otel,weaver --example otel_weaver_testing
```

---

## Prerequisites

- Rust 1.70+ (Edition 2021)
- Chicago TDD Tools with `otel` and `weaver` features
- Understanding of OpenTelemetry concepts

**Add to Cargo.toml:**
```toml
[dev-dependencies]
chicago-tdd-tools = { path = "../chicago-tdd-tools", features = ["otel", "weaver"] }
```

**For Weaver:**
```bash
cargo make weaver-bootstrap  # Install Weaver CLI and registry
```

---

## Key Concepts

### OpenTelemetry (OTEL)

**Spans:** Represent operations in distributed traces
- Context: Trace ID, Span ID, flags
- Name: Operation name
- Attributes: Key-value pairs
- Events: Timestamped events
- Status: Ok, Error, or Unset

**Metrics:** Represent measurements over time
- Name: Metric name
- Value: Counter, Gauge, or Histogram
- Timestamp: Measurement time
- Attributes: Key-value pairs

### Weaver Live-Check

Validates telemetry against OpenTelemetry semantic conventions at runtime:
- Intercepts telemetry sent to OTLP endpoints
- Validates against semantic convention schemas
- Ensures compliance with industry standards

### Unified API

`ObservabilityTest` provides single interface for both OTEL and Weaver validation.

---

## Code Examples

### Example 1: OTEL Span Validation

```rust
use chicago_tdd_tools::observability::{ObservabilityTest, TestConfig};
use chicago_tdd_tools::otel::types::{SpanContext, SpanId, SpanStatus, TraceId};
use std::collections::BTreeMap;

// Arrange: Create test span
let context = SpanContext::root(TraceId(12345), SpanId(67890), 1);
let span = Span::new_active(
    context,
    "test.operation".to_string(),
    1000,
    BTreeMap::new(),
    Vec::new(),
    SpanStatus::Ok,
);

// Act: Validate span using unified API
let config = TestConfig {
    weaver_enabled: false,  // Disable Weaver for simple unit test
    ..Default::default()
};
let test = ObservabilityTest::with_config(config)?;
let result = test.validate_span(&span);

// Assert: Verify validation succeeds
assert!(result.is_ok(), "Span should be valid");
```

### Example 2: OTEL Span with Attributes

```rust
// Arrange: Create span with custom attributes
let mut attrs = BTreeMap::new();
attrs.insert("service.name".to_string(), "test-service".to_string());
attrs.insert("operation.type".to_string(), "test".to_string());

let context = SpanContext::root(TraceId(12345), SpanId(67890), 1);
let span = Span::new_active(
    context,
    "test.operation".to_string(),
    1000,
    attrs,
    Vec::new(),
    SpanStatus::Ok,
);

// Act: Validate span
let config = TestConfig { weaver_enabled: false, ..Default::default() };
let test = ObservabilityTest::with_config(config)?;
let result = test.validate_span(&span);

// Assert: Verify validation succeeds
assert!(result.is_ok());
assert_eq!(span.attributes.get("service.name"), Some(&"test-service".to_string()));
```

### Example 3: OTEL Metric Validation

```rust
use chicago_tdd_tools::otel::types::MetricValue;

// Arrange: Create test metric
let metric = Metric {
    name: "test.counter".to_string(),
    value: MetricValue::Counter(42),
    timestamp_ms: 1000,
    attributes: BTreeMap::new(),
};

// Act: Validate metric
let config = TestConfig { weaver_enabled: false, ..Default::default() };
let test = ObservabilityTest::with_config(config)?;
let result = test.validate_metric(&metric);

// Assert: Verify validation succeeds
assert!(result.is_ok());
assert_eq!(metric.name, "test.counter");
match &metric.value {
    MetricValue::Counter(count) => assert_eq!(*count, 42),
    _ => panic!("Expected counter metric"),
}
```

### Example 4: Weaver Validator Creation

```rust
use std::path::PathBuf;

// Arrange: Create test with registry path
let registry_path = PathBuf::from("registry/");
let config = TestConfig {
    registry_path: Some(registry_path),
    weaver_enabled: true,
    ..Default::default()
};

// Act: Create test with unified API
let test = ObservabilityTest::with_config(config)?;

// Assert: Verify OTLP endpoint
assert_eq!(test.otlp_endpoint(), "http://127.0.0.1:4317");
```

### Example 5: Custom Ports

```rust
// Arrange: Create test with custom ports
let config = TestConfig {
    registry_path: Some(PathBuf::from("registry/")),
    otlp_grpc_port: 4318,
    admin_port: 8081,
    ..Default::default()
};

// Act: Create test and get endpoint
let test = ObservabilityTest::with_config(config)?;
let endpoint = test.otlp_endpoint();

// Assert: Verify custom configuration
assert_eq!(endpoint, "http://127.0.0.1:4318");
```

---

## Configuration

### TestConfig Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `weaver_enabled` | `bool` | `true` | Enable Weaver validation |
| `registry_path` | `Option<PathBuf>` | `None` | Path to Weaver registry |
| `otlp_grpc_port` | `u16` | `4317` | OTLP gRPC port |
| `admin_port` | `u16` | `8080` | Weaver admin port |

### Zero Config for 80% Cases

```rust
let config = TestConfig::default();
// Uses default ports, Weaver enabled
```

---

## Common Patterns

### Pattern 1: Unit Testing (OTEL Only)

```rust
let config = TestConfig {
    weaver_enabled: false,  // Disable Weaver for fast unit tests
    ..Default::default()
};
let test = ObservabilityTest::with_config(config)?;
test.validate_span(&span)?;
```

### Pattern 2: Integration Testing (OTEL + Weaver)

```rust
let config = TestConfig {
    registry_path: Some(PathBuf::from("registry/")),
    weaver_enabled: true,
    ..Default::default()
};
let test = ObservabilityTest::with_config(config)?;
test.validate_span(&span)?;  // Both OTEL and Weaver validation
```

### Pattern 3: Error Path Testing

```rust
let invalid_context = SpanContext::root(TraceId(0), SpanId(67890), 1);  // Invalid
let span_result = Span::new_completed(
    invalid_context,
    "test".to_string(),
    1000,
    2000,
    BTreeMap::new(),
    Vec::new(),
    SpanStatus::Ok,
);

match span_result {
    Ok(span) => {
        let result = test.validate_span(&span);
        assert!(result.is_err(), "Should fail validation");
    }
    Err(e) => println!("Expected error for invalid trace ID: {e}"),
}
```

---

## Best Practices

### 1. Use Appropriate Config for Test Type

```rust
// Unit tests: Disable Weaver
let config = TestConfig { weaver_enabled: false, ..Default::default() };

// Integration tests: Enable Weaver
let config = TestConfig { weaver_enabled: true, ..Default::default() };
```

### 2. Test Both Success and Error Paths

```rust
// Success path
test.validate_span(&valid_span)?;

// Error path
assert!(test.validate_span(&invalid_span).is_err());
```

### 3. Use Semantic Convention Attributes

```rust
let mut attrs = BTreeMap::new();
attrs.insert("service.name".to_string(), "my-service".to_string());
attrs.insert("http.method".to_string(), "GET".to_string());
attrs.insert("http.status_code".to_string(), "200".to_string());
```

---

## Troubleshooting

### Error: "otel feature required"

**Cause:** Feature not enabled

**Fix:**
```toml
[dev-dependencies]
chicago-tdd-tools = { path = "../chicago-tdd-tools", features = ["otel"] }
```

### Error: "Weaver not available"

**Cause:** Weaver CLI not installed

**Fix:**
```bash
cargo make weaver-bootstrap
```

### Validation Fails

**Cause:** Span/metric doesn't conform to specification

**Fix:**
- Check trace ID is non-zero
- Verify timestamp ordering
- Ensure required attributes present

---

## Next Steps

After mastering OTEL/Weaver testing, explore:

1. **[Testcontainers](testcontainers_example.md)** - Container-based testing
2. **[Go the Extra Mile](go_extra_mile.md)** - Progressive enhancement
3. **[Advanced Features](advanced_features.md)** - Type-level guarantees

---

## Related Documentation

- [Examples README](README.md) - All examples overview
- [Weaver Documentation](../docs/features/WEAVER_LIVE_CHECK.md) - Complete Weaver guide
- [API Reference](../docs/reference/API_REFERENCE.md) - Complete API documentation

---

## Reference

### Key Types

- `ObservabilityTest` - Unified API for observability validation
- `TestConfig` - Configuration for observability testing
- `Span` - OpenTelemetry span representation
- `Metric` - OpenTelemetry metric representation
- `SpanContext` - Span context (trace ID, span ID, flags)

### Key Functions

- `ObservabilityTest::with_config(config) -> Result<ObservabilityTest, Error>`
- `ObservabilityTest::validate_span(span) -> Result<(), ValidationError>`
- `ObservabilityTest::validate_metric(metric) -> Result<(), ValidationError>`
- `ObservabilityTest::otlp_endpoint() -> String`

### Default Ports

- OTLP gRPC: `4317`
- Weaver Admin: `8080`

---

**Quality is the default. Prevention beats detection.**

*Example: otel_weaver_testing.rs | Version: 1.2.0 | Updated: 2025-11-15*
