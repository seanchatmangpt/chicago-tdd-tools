# OpenTelemetry and Weaver Testing Guide

> Version-agnostic guide: check `Cargo.toml` for the current crate version and
> `src/observability/otel/types.rs` for the current OTEL types.

## Overview

Chicago TDD Tools provides comprehensive observability testing through OTEL (OpenTelemetry) and Weaver integration. This guide covers practical patterns for testing telemetry in your applications.

## Table of Contents

1. [Quick Start](#quick-start)
2. [OTEL Basics](#otel-basics)
3. [Weaver Integration](#weaver-integration)
4. [Testing Patterns](#testing-patterns)
5. [Best Practices](#best-practices)
6. [Troubleshooting](#troubleshooting)
7. [Real-World Examples](#real-world-examples)
8. [Architecture & Technical Details](#architecture--technical-details)

## Quick Start

### Enable Features

Add to `Cargo.toml`:

```toml
[dev-dependencies]
chicago-tdd-tools = { version = "26.10.12", features = ["otel", "weaver"] }
```

### Simple OTEL Span Test

```rust
#[cfg(feature = "otel")]
#[test]
fn test_application_span() {
    use chicago_tdd_tools::observability::ObservabilityTest;

    // Create observability test
    let test = ObservabilityTest::new()
        .expect("failed to initialize observability test");

    // Your application creates spans, we validate them
    // This would be called after your app creates telemetry
    assert!(true); // Validation would happen here
}
```

## OTEL Basics

### Understanding Spans

OTEL spans represent a unit of work in your application:

```
┌─ Trace ID: 1234567890abcdef
│
├─ Root Span: "process_request"
│  ├─ Start: 2024-01-01T12:00:00Z
│  ├─ End: 2024-01-01T12:00:01Z
│  ├─ Span ID: aaaa
│  └─ Attributes:
│     ├─ http.method = "GET"
│     ├─ http.url = "/api/users"
│     └─ http.status_code = 200
│
└─ Child Span: "database_query"
   ├─ Span ID: bbbb
   ├─ Parent: aaaa
   └─ Attributes:
      ├─ db.system = "postgres"
      └─ db.statement = "SELECT * FROM users"
```

### Creating Spans for Testing

```rust
#[cfg(feature = "otel")]
#[test]
fn test_span_creation() {
    use chicago_tdd_tools::observability::otel::types::{
        Attributes, Span, SpanContext, SpanStatus, TraceId, SpanId,
    };

    // Create a completed span
    let span = Span::new_completed(
        SpanContext::root(TraceId(12345), SpanId(67890), 0),
        "test_operation".to_string(),
        0,   // start_time_ms
        100, // end_time_ms
        Attributes::new(),
        Vec::new(), // events
        SpanStatus::Ok,
    )
    .expect("valid span");

    // Validate span structure
    assert!(!span.name.is_empty());
    assert!(span.context.trace_id.0 != 0);
    assert!(span.is_completed());
}
```

### Metrics

Metrics measure quantitative aspects of your application:

```rust
#[cfg(feature = "otel")]
#[test]
fn test_metric_creation() {
    use chicago_tdd_tools::observability::otel::types::{Attributes, Metric, MetricValue};

    // Create a metric
    let metric = Metric {
        name: "http_requests_total".to_string(),
        value: MetricValue::Counter(42),
        timestamp_ms: 1704110400000,
        attributes: Attributes::new(),
    };

    // Validate metric
    assert!(!metric.name.is_empty());
    assert!(matches!(metric.value, MetricValue::Counter(42)));
}
```

## Weaver Integration

### What is Weaver?

Weaver is OTel's schema registry for semantic conventions. It validates that your telemetry conforms to:
- Industry-standard attribute names
- Expected data types
- Semantic meaning

### Enable Weaver Testing

```rust
#[cfg(all(feature = "otel", feature = "weaver"))]
#[test]
fn test_with_weaver_validation() -> Result<(), Box<dyn std::error::Error>> {
    use chicago_tdd_tools::observability::{ObservabilityTest, TestConfig};
    use std::path::PathBuf;

    let config = TestConfig {
        weaver_enabled: true,
        registry_path: Some(PathBuf::from("semantic-conventions")),
        ..Default::default()
    };

    let test = ObservabilityTest::with_config(config)?;
    // Now telemetry is validated against semantic conventions
    Ok(())
}
```

### Weaver Configuration

```rust
let config = TestConfig {
    // Enable Weaver validation
    weaver_enabled: true,

    // Path to semantic conventions registry
    registry_path: Some(PathBuf::from("../weaver-registry")),

    // OTLP gRPC port for Weaver to listen on
    otlp_grpc_port: 4317,

    // Admin port for Weaver API
    admin_port: 4320,

    // Enable compile-time validation
    compile_time_validation: true,

    // Where to write Weaver JSON reports
    weaver_output_dir: Some(PathBuf::from("./test-reports")),
};
```

## Architecture & Technical Details

### Weaver Live-Check Architecture

The Weaver integration uses Weaver's `live-check` command to run an ephemeral OTLP receiver that validates traces against a Semantic Conventions registry in real-time.

1. **Protocol Requirements (gRPC only):**
   The Weaver `live-check` command currently ONLY exposes a gRPC listener (configured via `--otlp-grpc-port`). It **does not** support HTTP trace ingestion. Therefore, test clients *must* use the `grpc-tonic` feature in `opentelemetry-otlp` and configure their exporters with `.with_tonic()`.

2. **Tokio Runtime Requirement:**
   Because `tonic` strictly requires a Tokio runtime to drive its asynchronous I/O, synchronous test suites cannot simply use blocking HTTP clients. The observability fixtures automatically spin up and maintain a dedicated `tokio::runtime::Runtime` internally to keep the gRPC transport and `BatchSpanProcessor` alive throughout the test.

3. **Semantic Conventions Registry Structure:**
   Modern versions of the [OpenTelemetry Semantic Conventions](https://github.com/open-telemetry/semantic-conventions) repository store their schema files in a `model/` subdirectory. The Weaver integration intelligently detects this and automatically traverses into `registry/model` if it exists, ensuring the Weaver binary targets the correct YAML/JSON manifests without failing.

## Testing Patterns

### Pattern 1: Validate Span Attributes

```rust
#[cfg(feature = "otel")]
#[test]
fn test_http_span_attributes() {
    use chicago_tdd_tools::observability::otel::types::{
        Attributes, Span, SpanContext, SpanStatus, TraceId, SpanId,
    };

    let mut attributes = Attributes::new();
    attributes.insert("http.method".to_string(), "GET".to_string());
    attributes.insert("http.url".to_string(), "/api/users".to_string());
    attributes.insert("http.status_code".to_string(), "200".to_string());

    let span = Span::new_completed(
        SpanContext::root(TraceId(1), SpanId(1), 0),
        "http.request".to_string(),
        1000,
        1100,
        attributes,
        Vec::new(),
        SpanStatus::Ok,
    )
    .expect("valid span");

    // Verify required attributes are present
    assert_eq!(span.attributes.get("http.method").map(|s| s.as_str()), Some("GET"));
    assert_eq!(
        span.attributes.get("http.status_code").map(|s| s.as_str()),
        Some("200")
    );
}
```

### Pattern 2: Test Error Spans

```rust
#[cfg(feature = "otel")]
#[test]
fn test_error_span_recording() {
    use chicago_tdd_tools::observability::otel::types::{
        Attributes, Span, SpanContext, SpanStatus, TraceId, SpanId,
    };

    let mut attributes = Attributes::new();
    attributes.insert("error".to_string(), "true".to_string());
    attributes.insert("error.type".to_string(), "ConnectionTimeout".to_string());

    let error_span = Span::new_completed(
        SpanContext::root(TraceId(1), SpanId(2), 0),
        "database.error".to_string(),
        1000,
        1050,
        attributes,
        Vec::new(),
        SpanStatus::Error,
    )
    .expect("valid span");

    // Verify error attributes
    assert_eq!(error_span.attributes.get("error").map(|s| s.as_str()), Some("true"));
    assert_eq!(error_span.status, SpanStatus::Error);
}
```

### Pattern 3: Hierarchical Spans

```rust
#[cfg(feature = "otel")]
#[test]
fn test_parent_child_span_relationship() {
    use chicago_tdd_tools::observability::otel::types::{
        Attributes, Span, SpanContext, SpanStatus, TraceId, SpanId,
    };

    // Parent span
    let parent_span = Span::new_completed(
        SpanContext::root(TraceId(100), SpanId(1), 0),
        "process_order".to_string(),
        0,
        1000,
        Attributes::new(),
        Vec::new(),
        SpanStatus::Ok,
    )
    .expect("valid span");

    // Child span (same trace ID, parent-linked span ID)
    let child_span = Span::new_completed(
        SpanContext::child(TraceId(100), SpanId(2), SpanId(1), 0),
        "validate_payment".to_string(),
        100,
        500,
        Attributes::new(),
        Vec::new(),
        SpanStatus::Ok,
    )
    .expect("valid span");

    // Verify trace relationship
    assert_eq!(parent_span.context.trace_id, child_span.context.trace_id);
    assert_ne!(parent_span.context.span_id, child_span.context.span_id);
    assert!(child_span.context.is_child());
    assert_eq!(child_span.context.parent_span_id(), Some(SpanId(1)));
}
```

## Best Practices

### 1. Use Semantic Convention Attributes

```rust
// ✅ Good: Follows OTEL semantic conventions
vec![
    ("http.method".to_string(), "POST".to_string()),
    ("http.url".to_string(), "/api/users".to_string()),
    ("http.status_code".to_string(), "201".to_string()),
    ("http.response_time_ms".to_string(), "45".to_string()),
]

// ❌ Bad: Non-standard attribute names
vec![
    ("method".to_string(), "POST".to_string()),
    ("url".to_string(), "/api/users".to_string()),
    ("status".to_string(), "201".to_string()),
]
```

### 2. Test Both Happy and Error Paths

```rust
#[cfg(feature = "otel")]
#[test]
fn test_operation_success() {
    // Test successful operation recording
}

#[cfg(feature = "otel")]
#[test]
fn test_operation_failure() {
    // Test error recording
}
```

### 3. Include Timing Information

```rust
let span = Span::new_completed(
    SpanContext::root(TraceId(1), SpanId(1), 0),
    "api_call".to_string(),
    1704110400000, // start_time_ms, milliseconds since epoch
    1704110400100, // end_time_ms: operation took 100ms
    Attributes::new(),
    Vec::new(),
    SpanStatus::Ok,
)
.expect("valid span");

let duration_ms = span.end_time_ms().expect("completed") - span.start_time_ms();
assert!(duration_ms > 0);
```

### 4. Document Span Semantics

```rust
// ✅ Good: Clear what this span represents
/// Span for HTTP request processing
/// Includes all time from request receipt to response send
#[cfg(feature = "otel")]
#[test]
fn test_http_request_span() {
    // ...
}

// ❌ Bad: Unclear purpose
#[cfg(feature = "otel")]
#[test]
fn test_span() {
    // ...
}
```

### 5. Use Trace IDs for Correlation

```rust
#[cfg(feature = "otel")]
#[test]
fn test_request_tracing() {
    use chicago_tdd_tools::observability::otel::types::{SpanContext, SpanId, TraceId};

    let trace_id = TraceId(12345);

    // All spans in this request share the same trace ID
    let span1 = SpanContext::root(trace_id, SpanId(1), 0);
    let span2 = SpanContext::child(trace_id, SpanId(2), SpanId(1), 0);

    // Verify all spans in the trace can be linked
    assert_eq!(span1.trace_id, span2.trace_id);
}
```

## Troubleshooting

### Weaver Binary Not Found

```
Error: WeaverBinaryNotFound
```

**Solution**: Install Weaver
```bash
just weaver-bootstrap
# or
cargo install weaver
```

### Registry Path Not Found

```
Error: RegistryNotFound
```

**Solution**: Provide explicit registry path
```rust
let config = TestConfig {
    registry_path: Some(PathBuf::from("path/to/registry")),
    ..Default::default()
};
```

### OTEL Feature Not Enabled

```
Error: FeatureDisabled("otel")
```

**Solution**: Add to `Cargo.toml`
```toml
chicago-tdd-tools = { features = ["otel"] }
```

## Real-World Examples

### Example 1: Testing HTTP Instrumentation

```rust
#[cfg(feature = "otel")]
#[test]
fn test_http_server_span() {
    use chicago_tdd_tools::observability::otel::types::{
        Attributes, Span, SpanContext, SpanStatus, TraceId, SpanId,
    };

    // Simulate HTTP server span
    let mut attributes = Attributes::new();
    attributes.insert("http.method".to_string(), "GET".to_string());
    attributes
        .insert("http.url".to_string(), "http://localhost:8080/api/v1/users".to_string());
    attributes.insert("http.status_code".to_string(), "200".to_string());
    attributes.insert("http.response_time_ms".to_string(), "150".to_string());

    let request_span = Span::new_completed(
        SpanContext::root(TraceId(0x4bf92f3577b34da6), SpanId(0x00f067aa0ba902b7), 0),
        "http.server.request".to_string(),
        1704110400000,
        1704110400150,
        attributes,
        Vec::new(),
        SpanStatus::Ok,
    )
    .expect("valid span");

    // Validate attributes
    assert_eq!(
        request_span.attributes.get("http.method").map(|s| s.as_str()),
        Some("GET")
    );
    assert_eq!(
        request_span.attributes.get("http.status_code").map(|s| s.as_str()),
        Some("200")
    );
}
```

### Example 2: Testing Database Operations

```rust
#[cfg(feature = "otel")]
#[test]
fn test_database_span() {
    use chicago_tdd_tools::observability::otel::types::{
        Attributes, Span, SpanContext, SpanStatus, TraceId, SpanId,
    };

    let mut attributes = Attributes::new();
    attributes.insert("db.system".to_string(), "postgresql".to_string());
    attributes.insert("db.name".to_string(), "myapp_db".to_string());
    attributes.insert("db.operation".to_string(), "SELECT".to_string());
    attributes
        .insert("db.statement".to_string(), "SELECT * FROM users WHERE id = ?".to_string());
    attributes.insert("db.rows_affected".to_string(), "1".to_string());

    let db_span = Span::new_completed(
        SpanContext::root(TraceId(0x4bf92f3577b34da6), SpanId(0x00f067aa0ba902b8), 0),
        "db.client.operation".to_string(),
        1704110400050,
        1704110400120,
        attributes,
        Vec::new(),
        SpanStatus::Ok,
    )
    .expect("valid span");

    // Validate database attributes
    assert_eq!(
        db_span.attributes.get("db.system").map(|s| s.as_str()),
        Some("postgresql")
    );
    assert_eq!(
        db_span.attributes.get("db.operation").map(|s| s.as_str()),
        Some("SELECT")
    );
}
```

### Example 3: Testing Error Instrumentation

```rust
#[cfg(feature = "otel")]
#[test]
fn test_error_instrumentation() {
    use chicago_tdd_tools::observability::otel::types::{
        Attributes, Span, SpanContext, SpanStatus, TraceId, SpanId,
    };

    let mut attributes = Attributes::new();
    attributes.insert("error".to_string(), "true".to_string());
    attributes.insert("error.type".to_string(), "InvalidToken".to_string());
    attributes.insert("error.message".to_string(), "Token signature invalid".to_string());

    let error_span = Span::new_completed(
        SpanContext::root(TraceId(0x4bf92f3577b34da6), SpanId(0x00f067aa0ba902b9), 0),
        "auth.validate".to_string(),
        1704110400000,
        1704110400020,
        attributes,
        Vec::new(),
        SpanStatus::Error,
    )
    .expect("valid span");

    // Verify error attributes
    assert_eq!(error_span.status, SpanStatus::Error);
    assert_eq!(
        error_span.attributes.get("error").map(|s| s.as_str()),
        Some("true")
    );
    assert_eq!(
        error_span.attributes.get("error.type").map(|s| s.as_str()),
        Some("InvalidToken")
    );
}
```

## See Also

- [OpenTelemetry Documentation](https://opentelemetry.io/)
- [Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/)
- [Weaver Documentation](https://github.com/open-telemetry/weaver)
- [Observability Testing Guide](./observability-testing-guide.md)

## Questions?

Contact the development team if you need help with observability testing.
