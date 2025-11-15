# OpenTelemetry and Weaver Testing Guide - v1.1.2

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

## Quick Start

### Enable Features

Add to `Cargo.toml`:

```toml
[dev-dependencies]
chicago-tdd-tools = { version = "1.1.2", features = ["otel", "weaver"] }
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
    use chicago_tdd_tools::observability::otel::types::{Span, SpanContext, TraceId, SpanId};

    // Create a span
    let span = Span {
        name: "test_operation".to_string(),
        context: SpanContext {
            trace_id: TraceId(12345),
            span_id: SpanId(67890),
            trace_flags: 1,
        },
        start_time: 0,
        end_time: 100,
        attributes: vec![],
        events: vec![],
        status: "Ok".to_string(),
    };

    // Validate span structure
    assert!(!span.name.is_empty());
    assert!(span.context.trace_id.0 != 0);
}
```

### Metrics

Metrics measure quantitative aspects of your application:

```rust
#[cfg(feature = "otel")]
#[test]
fn test_metric_creation() {
    use chicago_tdd_tools::observability::otel::types::Metric;

    // Create a metric
    let metric = Metric {
        name: "http_requests_total".to_string(),
        description: "Total HTTP requests".to_string(),
        value: 42,
        unit: "requests".to_string(),
        timestamp: 1704110400000,
    };

    // Validate metric
    assert!(!metric.name.is_empty());
    assert!(metric.value >= 0);
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

## Testing Patterns

### Pattern 1: Validate Span Attributes

```rust
#[cfg(feature = "otel")]
#[test]
fn test_http_span_attributes() {
    use chicago_tdd_tools::observability::otel::types::{Span, SpanContext, TraceId, SpanId};

    let span = Span {
        name: "http.request".to_string(),
        context: SpanContext {
            trace_id: TraceId(1),
            span_id: SpanId(1),
            trace_flags: 1,
        },
        start_time: 1000,
        end_time: 1100,
        attributes: vec![
            ("http.method".to_string(), "GET".to_string()),
            ("http.url".to_string(), "/api/users".to_string()),
            ("http.status_code".to_string(), "200".to_string()),
        ],
        events: vec![],
        status: "Ok".to_string(),
    };

    // Verify required attributes are present
    let attrs: std::collections::HashMap<_, _> =
        span.attributes.iter().cloned().collect();

    assert_eq!(attrs.get("http.method").map(|s| s.as_str()), Some("GET"));
    assert_eq!(attrs.get("http.status_code").map(|s| s.as_str()), Some("200"));
}
```

### Pattern 2: Test Error Spans

```rust
#[cfg(feature = "otel")]
#[test]
fn test_error_span_recording() {
    use chicago_tdd_tools::observability::otel::types::{Span, SpanContext, TraceId, SpanId};

    let error_span = Span {
        name: "database.error".to_string(),
        context: SpanContext {
            trace_id: TraceId(1),
            span_id: SpanId(2),
            trace_flags: 1,
        },
        start_time: 1000,
        end_time: 1050,
        attributes: vec![
            ("error".to_string(), "true".to_string()),
            ("error.type".to_string(), "ConnectionTimeout".to_string()),
        ],
        events: vec![],
        status: "Error".to_string(),
    };

    // Verify error attributes
    assert!(error_span.attributes
        .iter()
        .any(|(k, v)| k == "error" && v == "true"));

    assert_eq!(error_span.status, "Error");
}
```

### Pattern 3: Hierarchical Spans

```rust
#[cfg(feature = "otel")]
#[test]
fn test_parent_child_span_relationship() {
    use chicago_tdd_tools::observability::otel::types::{Span, SpanContext, TraceId, SpanId};

    // Parent span
    let parent_span = Span {
        name: "process_order".to_string(),
        context: SpanContext {
            trace_id: TraceId(100),
            span_id: SpanId(1),
            trace_flags: 1,
        },
        start_time: 0,
        end_time: 1000,
        attributes: vec![],
        events: vec![],
        status: "Ok".to_string(),
    };

    // Child span (same trace ID, different span ID)
    let child_span = Span {
        name: "validate_payment".to_string(),
        context: SpanContext {
            trace_id: TraceId(100), // Same trace
            span_id: SpanId(2),     // Different span
            trace_flags: 1,
        },
        start_time: 100,
        end_time: 500,
        attributes: vec![],
        events: vec![],
        status: "Ok".to_string(),
    };

    // Verify trace relationship
    assert_eq!(parent_span.context.trace_id, child_span.context.trace_id);
    assert_ne!(parent_span.context.span_id, child_span.context.span_id);
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
let span = Span {
    name: "api_call".to_string(),
    start_time: 1704110400000,     // milliseconds since epoch
    end_time: 1704110400100,       // operation took 100ms
    // ...
};

let duration_ms = span.end_time - span.start_time;
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
    use chicago_tdd_tools::observability::otel::types::{Span, SpanContext, TraceId};

    let trace_id = TraceId(12345);

    // All spans in this request share the same trace ID
    let span1 = Span {
        context: SpanContext {
            trace_id,
            span_id: SpanId(1),
            trace_flags: 1,
        },
        // ...
    };

    let span2 = Span {
        context: SpanContext {
            trace_id,  // Same trace
            span_id: SpanId(2),
            trace_flags: 1,
        },
        // ...
    };

    // Verify all spans in the trace can be linked
    assert_eq!(span1.context.trace_id, span2.context.trace_id);
}
```

## Troubleshooting

### Weaver Binary Not Found

```
Error: WeaverBinaryNotFound
```

**Solution**: Install Weaver
```bash
cargo make weaver-bootstrap
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
    use chicago_tdd_tools::observability::otel::types::{Span, SpanContext, TraceId, SpanId};

    // Simulate HTTP server span
    let request_span = Span {
        name: "http.server.request".to_string(),
        context: SpanContext {
            trace_id: TraceId(0x4bf92f3577b34da6),
            span_id: SpanId(0x00f067aa0ba902b7),
            trace_flags: 1,
        },
        start_time: 1704110400000,
        end_time: 1704110400150,
        attributes: vec![
            ("http.method".to_string(), "GET".to_string()),
            ("http.url".to_string(), "http://localhost:8080/api/v1/users".to_string()),
            ("http.status_code".to_string(), "200".to_string()),
            ("http.response_time_ms".to_string(), "150".to_string()),
        ],
        events: vec![],
        status: "Ok".to_string(),
    };

    // Validate attributes
    let attrs: std::collections::HashMap<_, _> =
        request_span.attributes.iter().cloned().collect();

    assert_eq!(attrs.get("http.method").map(|s| s.as_str()), Some("GET"));
    assert_eq!(attrs.get("http.status_code").map(|s| s.as_str()), Some("200"));
}
```

### Example 2: Testing Database Operations

```rust
#[cfg(feature = "otel")]
#[test]
fn test_database_span() {
    use chicago_tdd_tools::observability::otel::types::{Span, SpanContext, TraceId, SpanId};

    let db_span = Span {
        name: "db.client.operation".to_string(),
        context: SpanContext {
            trace_id: TraceId(0x4bf92f3577b34da6),
            span_id: SpanId(0x00f067aa0ba902b8),
            trace_flags: 1,
        },
        start_time: 1704110400050,
        end_time: 1704110400120,
        attributes: vec![
            ("db.system".to_string(), "postgresql".to_string()),
            ("db.name".to_string(), "myapp_db".to_string()),
            ("db.operation".to_string(), "SELECT".to_string()),
            ("db.statement".to_string(), "SELECT * FROM users WHERE id = ?".to_string()),
            ("db.rows_affected".to_string(), "1".to_string()),
        ],
        events: vec![],
        status: "Ok".to_string(),
    };

    // Validate database attributes
    let attrs: std::collections::HashMap<_, _> =
        db_span.attributes.iter().cloned().collect();

    assert_eq!(attrs.get("db.system").map(|s| s.as_str()), Some("postgresql"));
    assert_eq!(attrs.get("db.operation").map(|s| s.as_str()), Some("SELECT"));
}
```

### Example 3: Testing Error Instrumentation

```rust
#[cfg(feature = "otel")]
#[test]
fn test_error_instrumentation() {
    use chicago_tdd_tools::observability::otel::types::{Span, SpanContext, TraceId, SpanId};

    let error_span = Span {
        name: "auth.validate".to_string(),
        context: SpanContext {
            trace_id: TraceId(0x4bf92f3577b34da6),
            span_id: SpanId(0x00f067aa0ba902b9),
            trace_flags: 1,
        },
        start_time: 1704110400000,
        end_time: 1704110400020,
        attributes: vec![
            ("error".to_string(), "true".to_string()),
            ("error.type".to_string(), "InvalidToken".to_string()),
            ("error.message".to_string(), "Token signature invalid".to_string()),
        ],
        events: vec![],
        status: "Error".to_string(),
    };

    // Verify error attributes
    assert_eq!(error_span.status, "Error");
    let attrs: std::collections::HashMap<_, _> =
        error_span.attributes.iter().cloned().collect();

    assert_eq!(attrs.get("error").map(|s| s.as_str()), Some("true"));
    assert_eq!(attrs.get("error.type").map(|s| s.as_str()), Some("InvalidToken"));
}
```

## See Also

- [OpenTelemetry Documentation](https://opentelemetry.io/)
- [Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/)
- [Weaver Documentation](https://github.com/open-telemetry/weaver)
- [Observability Testing Guide](../observability/observability-testing-guide.md)

## Questions?

Contact the development team if you need help with observability testing.
