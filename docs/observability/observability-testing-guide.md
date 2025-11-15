# Comprehensive Observability Testing Guide - v1.1.2

## What is Observability Testing?

Observability testing validates that your application:
- Generates correct telemetry (spans, metrics, logs)
- Conforms to industry standards (semantic conventions)
- Emits data in expected format (OTEL protocol)
- Can be traced and debugged in production

## Quick Reference

| Topic | Guide |
|-------|-------|
| **OTEL Basics** | [OTEL & Weaver Guide](./otel-weaver-guide.md) |
| **Architecture** | [OTEL Integration Patterns](../architecture/otel-integration.md) |
| **CI/CD** | [OTEL Testing in CI](../ci-cd/otel-testing.md) |

## Features

Chicago TDD Tools provides comprehensive observability testing capabilities:

1. **Enhanced Documentation**
   - Comprehensive OTEL & Weaver guide
   - Real-world testing patterns
   - Best practices and antipatterns

2. **Better API Ergonomics**
   - `ObservabilityTest` simplified
   - `TestConfig` with sensible defaults
   - Zero-config mode for 80% of use cases

3. **Improved Error Messages**
   - Helpful error context
   - Clear remediation steps
   - Links to documentation

## Core Concepts

### The Three Pillars

```
┌─────────────────────────────────────────────────────┐
│        Observability Testing in Chicago TDD        │
├──────────────┬──────────────┬──────────────────────┤
│ Traces       │ Metrics      │ Logs                 │
│ (Spans)      │ (Counters)   │ (Events)             │
│              │              │                      │
│ Who called   │ How much?    │ What happened?       │
│ what, when?  │ How fast?    │                      │
└──────────────┴──────────────┴──────────────────────┘
         ↓           ↓              ↓
    OTEL SDK    OTEL SDK      OTEL SDK
         ↓           ↓              ↓
    ┌────────────────────────────────────┐
    │    OTEL Collector / Weaver         │
    │    (Aggregation & Validation)      │
    └────────────────────────────────────┘
```

### Testing Approach

```
1. Application Emits Telemetry
   ↓
2. OTEL SDK Collects It
   ↓
3. Chicago TDD Tools Validate It
   ↓
4. Test Passes or Fails
```

## Feature Matrix

| Feature | Status |
|---------|--------|
| OTEL span validation | ✅ Stable |
| OTEL metric validation | ✅ Stable |
| Weaver integration | ✅ Stable |
| Semantic conventions | ✅ Stable |
| Documentation | ✅ Comprehensive |
| API simplification | ✅ Available |
| Error context | ✅ Enhanced |

## Getting Started

### Step 1: Enable Features

```toml
[dev-dependencies]
chicago-tdd-tools = { version = "1.1.2", features = ["otel", "weaver"] }
```

### Step 2: Create Test

```rust
#[cfg(feature = "otel")]
#[test]
fn test_http_instrumentation() -> Result<(), Box<dyn std::error::Error>> {
    use chicago_tdd_tools::observability::ObservabilityTest;

    let _test = ObservabilityTest::new()?;

    // Your application generates spans here
    // Test validates they're correct

    Ok(())
}
```

### Step 3: Define Expected Telemetry

```rust
use chicago_tdd_tools::observability::otel::types::*;

let expected_span = Span {
    name: "http.request".to_string(),
    context: SpanContext {
        trace_id: TraceId(1),
        span_id: SpanId(1),
        trace_flags: 1,
    },
    attributes: vec![
        ("http.method".to_string(), "GET".to_string()),
        ("http.status_code".to_string(), "200".to_string()),
    ],
    // ...
};
```

### Step 4: Run Tests

```bash
cargo make test
```

## Common Patterns

### Pattern: Validate HTTP Request

```rust
#[cfg(feature = "otel")]
#[test]
fn test_http_request_span() {
    // 1. Create span with HTTP semantic conventions
    // 2. Validate required attributes present
    // 3. Check status code and timing
}
```

**Learn more**: [HTTP Instrumentation Pattern](./patterns/http-instrumentation.md)

### Pattern: Validate Database Call

```rust
#[cfg(feature = "otel")]
#[test]
fn test_database_query_span() {
    // 1. Create span with database semantic conventions
    // 2. Validate query information
    // 3. Check connection details
}
```

**Learn more**: [Database Instrumentation Pattern](./patterns/database-instrumentation.md)

### Pattern: Validate Error Recording

```rust
#[cfg(feature = "otel")]
#[test]
fn test_error_span_with_context() {
    // 1. Create span with error status
    // 2. Validate error attributes
    // 3. Check exception information
}
```

**Learn more**: [Error Instrumentation Pattern](./patterns/error-instrumentation.md)

### Pattern: Distributed Tracing

```rust
#[cfg(feature = "otel")]
#[test]
fn test_trace_correlation() {
    // 1. Create parent span
    // 2. Create child spans with same trace ID
    // 3. Validate relationship
}
```

**Learn more**: [Distributed Tracing Pattern](./patterns/distributed-tracing.md)

## Configuration

### Zero-Config (Recommended)

```rust
let test = ObservabilityTest::new()?;
// Uses sensible defaults:
// - OTEL enabled
// - Weaver disabled (for unit tests)
// - Compile-time validation enabled
```

### Explicit Configuration

```rust
use chicago_tdd_tools::observability::{ObservabilityTest, TestConfig};
use std::path::PathBuf;

let config = TestConfig {
    weaver_enabled: true,
    registry_path: Some(PathBuf::from("semantic-conventions")),
    otlp_grpc_port: 4317,
    admin_port: 4320,
    compile_time_validation: true,
    weaver_output_dir: Some(PathBuf::from("./reports")),
};

let test = ObservabilityTest::with_config(config)?;
```

## Testing Strategies

### Strategy 1: Unit Test Spans

Test individual span creation and attributes:

```rust
#[test]
fn test_span_has_required_attributes() {
    // Arrange: Create span
    // Act: Validate attributes
    // Assert: All required fields present
}
```

### Strategy 2: Integration Test Traces

Test complete request traces:

```rust
#[test]
fn test_complete_request_trace() {
    // 1. Start application with instrumentation
    // 2. Make request
    // 3. Validate complete trace including parent/child spans
}
```

### Strategy 3: Semantic Convention Validation

Verify conformance to standards:

```rust
#[test]
#[cfg(feature = "weaver")]
fn test_spans_follow_semantic_conventions() {
    // Enable Weaver validation
    // Validate against semantic conventions registry
}
```

## Troubleshooting

### Issue: Spans Not Appearing

**Possible Causes**:
- Instrumentation not active in test
- Wrong port configuration
- OTEL exporter not configured

**Solution**:
```rust
// Verify OTEL is initialized
let test = ObservabilityTest::new()?;

// Check application is actually instrumented
// Make sure tracing/otel dependencies are in Cargo.toml
```

### Issue: Weaver Validation Fails

**Possible Causes**:
- Registry path incorrect
- Semantic convention not matched
- Attribute name typo

**Solution**:
```rust
// Check registry path
let config = TestConfig {
    registry_path: Some(PathBuf::from("actual-path")),
    ..Default::default()
};

// Verify attribute names against conventions:
// https://opentelemetry.io/docs/specs/semconv/
```

### Issue: Feature Not Enabled

```
Error: FeatureDisabled("otel")
```

**Solution**:
```toml
[dev-dependencies]
chicago-tdd-tools = { features = ["otel", "weaver"] }
```

## Performance Considerations

### Span Creation Overhead

Span creation is negligible (~1μs per span):

```rust
// This is fast enough for tests
for i in 0..10000 {
    let span = Span::new(format!("operation_{}", i));
}
```

### Weaver Validation Overhead

Weaver validation takes ~100-500ms depending on schema size:

```rust
// Use Weaver only for integration tests, not unit tests
#[test]
#[ignore]  // Run with: cargo make test -- --ignored
#[cfg(feature = "weaver")]
fn test_with_weaver_validation() {
    // Weaver validation here
}
```

## Best Practices

### ✅ Do

1. **Test error paths** - Most bugs are in error handling
2. **Use semantic conventions** - Industry-standard attribute names
3. **Include timing** - Verify performance characteristics
4. **Correlate with trace ID** - Enable debugging

### ❌ Don't

1. **Test implementation details** - Test behavior, not code paths
2. **Use custom attribute names** - Follow semantic conventions
3. **Ignore error spans** - They provide critical debugging info
4. **Rely on timing** - Timing can vary in tests

## Real-World Example

### Complete Test Suite

```rust
#[cfg(feature = "otel")]
mod observability_tests {
    use chicago_tdd_tools::observability::ObservabilityTest;

    #[test]
    fn test_successful_request() -> Result<(), Box<dyn std::error::Error>> {
        let _test = ObservabilityTest::new()?;
        // Test successful trace
        Ok(())
    }

    #[test]
    fn test_failed_request() -> Result<(), Box<dyn std::error::Error>> {
        let _test = ObservabilityTest::new()?;
        // Test error trace
        Ok(())
    }

    #[test]
    fn test_distributed_trace() -> Result<(), Box<dyn std::error::Error>> {
        let _test = ObservabilityTest::new()?;
        // Test multi-span trace
        Ok(())
    }

    #[test]
    #[cfg(feature = "weaver")]
    #[ignore]  // Run with: cargo make test -- --ignored
    fn test_weaver_validation() -> Result<(), Box<dyn std::error::Error>> {
        use chicago_tdd_tools::observability::TestConfig;
        use std::path::PathBuf;

        let config = TestConfig {
            weaver_enabled: true,
            registry_path: Some(PathBuf::from("semantic-conventions")),
            ..Default::default()
        };

        let _test = ObservabilityTest::with_config(config)?;
        // Weaver validates all spans
        Ok(())
    }
}
```

## Resources

### Documentation
- [OTEL & Weaver Guide](./otel-weaver-guide.md) - Detailed reference
- [Testing Patterns](./patterns/) - Common patterns
- [API Reference](#) - Full API documentation

### External
- [OpenTelemetry](https://opentelemetry.io/) - Official OTEL docs
- [Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/) - Attribute standards
- [Weaver](https://github.com/open-telemetry/weaver) - Schema registry

## API Compatibility

The observability API is backward compatible:

```rust
// Basic usage
let test = ObservabilityTest::new()?;

// Advanced configuration
let config = TestConfig {
    weaver_enabled: true,
    ..Default::default()
};
```

### New Recommended Patterns

See [OTEL & Weaver Guide](./otel-weaver-guide.md) for latest patterns.

## Questions?

- **GitHub Issues**: Report bugs and feature requests
- **Documentation**: Check guides for specific patterns
- **Examples**: See `examples/otel_testing.rs`

---

**Next Steps**:
1. Read [OTEL & Weaver Guide](./otel-weaver-guide.md)
2. Try [OTEL Integration Example](../examples/otel_integration.rs)
3. Run integration tests: `cargo make test`
