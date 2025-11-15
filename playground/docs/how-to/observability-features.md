# How to Run Observability Feature Examples

**Quick reference** for observability: OpenTelemetry (OTEL) validation and Weaver semantic convention checking.

## Quick Commands

```bash
# Show all observability features
cargo run -- obs stat

# List available observability examples
cargo run -- obs list

# Run OTEL validation
cargo run -- obs otel

# Run Weaver validation (requires weaver feature + Docker)
cargo run -- obs weav

# Run with all observability features
cargo run --features otel,weaver -- obs stat
```

## Observability Features Overview

| Feature | Purpose | Requirements |
|---------|---------|--------------|
| **OTEL** | OpenTelemetry span/metric validation | Feature: `otel` |
| **Weaver** | Semantic convention live-check | Features: `weaver` + Docker (optional) |

## Prerequisites

### For OTEL Testing

Enable the feature:

```bash
cargo run --features otel -- obs stat
```

Or add to `Cargo.toml`:

```toml
[dev-dependencies]
chicago-tdd-tools = { version = "1.1", features = ["otel"] }
```

### For Weaver Testing

Enable the feature and bootstrap Weaver:

```bash
cargo run --features weaver -- obs stat

# First time: Download Weaver CLI + registry
cargo run --features weaver -- obs bootstrap

# Verify Weaver works
cargo run --features weaver -- obs weav
```

## OTEL Validation

```bash
cargo run --features otel -- obs otel
```

**What it does:**
Validates OpenTelemetry spans and metrics against OTEL specifications.

**Use when:**
- Testing instrumentation code
- Validating span attributes
- Checking metric definitions
- Ensuring OTEL compliance

**Example:**
```rust
use chicago_tdd_tools::observability::otel::*;

test!(test_otel_span, {
    // Create span
    let span = Span::new("http.request", context)
        .with_attribute("http.method", "GET")
        .with_attribute("http.url", "https://example.com");

    // Verify span structure
    assert_eq!(span.name(), "http.request");
    assert!(span.has_attribute("http.method"));
    assert_eq!(span.attribute("http.method"), Some("GET"));
});
```

**Common attributes to validate:**

HTTP spans:
- `http.method` - Request method (GET, POST, etc.)
- `http.url` - Request URL
- `http.status_code` - Response status
- `http.response_content_length` - Response size

Database spans:
- `db.system` - Database type (postgres, mysql, etc.)
- `db.name` - Database name
- `db.statement` - Query statement

## Weaver Validation

```bash
cargo run --features weaver -- obs weav
```

**What it does:**
Validates OTEL spans/metrics against semantic conventions using Weaver.

**Use when:**
- Need semantic convention compliance
- Testing against official OTEL standards
- Validating instrumentation in real-time
- Ensuring consistency across teams

### First Time Setup

```bash
# Bootstrap Weaver (download CLI + registry)
cargo run --features weaver -- obs bootstrap

# This creates:
# - target/<profile>/weaver (executable)
# - registry/ (semantic conventions)

# Verify with smoke test
cargo run --features weaver -- obs smoke
```

### Running Weaver Validation

```bash
cargo run --features weaver -- obs weav
```

**Example fixture-based Weaver test:**
```rust
fixture_test!(test_weaver_validation, fixture, {
    use chicago_tdd_tools::observability::weaver::*;

    // Get Weaver instance
    let weaver = fixture.weaver_instance()?;

    // Create OTEL span
    let span = create_http_span("GET", "/api/users");
    send_otel_span(span.clone());

    // Validate against semantic conventions
    let result = weaver.validate_span("http.request", &span)?;
    assert_ok!(result);
});
```

## Feature Enablement

### OTEL Only

```bash
cargo run --features otel -- obs stat
```

### Weaver Only (includes OTEL)

```bash
cargo run --features weaver -- obs stat
```

### Both OTEL and Weaver

```bash
cargo run --features otel,weaver -- obs stat
```

### Add to Cargo.toml

```toml
[dev-dependencies]
chicago-tdd-tools = { version = "1.1", features = ["otel", "weaver"] }
```

## Testing Observability in Your Code

### Test OTEL Instrumentation

```rust
use chicago_tdd_tools::observability::otel::*;

test!(test_instrumentation, {
    // Create trace
    let trace = create_trace("my.operation");

    // Create spans
    let span = trace.create_span("step1");
    span.with_attribute("user_id", "123");

    // Verify
    assert_eq!(span.trace_id(), trace.id());
    assert!(span.has_attribute("user_id"));
});
```

### Test Metrics

```rust
test!(test_metrics, {
    // Create metric
    let counter = Counter::new("requests");

    // Increment
    counter.add(1);

    // Verify
    assert_eq!(counter.value(), 1);
});
```

### Test Weaver Compliance

```rust
fixture_test!(test_semantic_compliance, fixture, {
    let weaver = fixture.weaver_instance()?;

    // Create compliant span
    let span = Span::new("http.request", context)
        .with_attribute("http.method", "POST")
        .with_attribute("http.url", "https://api.example.com/users")
        .with_attribute("http.status_code", 201);

    // Validate against semantic conventions
    assert_ok!(weaver.validate_span("http.request", &span));
});
```

## Docker Considerations

Weaver can run with or without Docker:

### With Docker (Full Validation)

```bash
# Ensure Docker is running
docker ps

# Run Weaver tests
cargo run --features weaver -- obs weav
```

### Without Docker (Schema Only)

```bash
# Skip Docker requirement
export WEAVER_ALLOW_SKIP=1
cargo run --features weaver -- obs weav
```

## Recommended Observability Path

### Step 1: Learn OTEL Basics

```bash
cargo run --features otel -- obs otel
```

Understand spans, metrics, and attributes.

### Step 2: Test Instrumentation

```rust
use chicago_tdd_tools::observability::otel::*;

test!(test_basic_span, {
    let span = create_test_span("my.operation");
    assert_eq!(span.name(), "my.operation");
});
```

### Step 3: Add Weaver Validation

```bash
cargo run --features weaver -- obs bootstrap
cargo run --features weaver -- obs weav
```

Validate against semantic conventions.

### Step 4: Test in Your Application

```rust
fixture_test!(test_app_observability, fixture, {
    let weaver = fixture.weaver_instance()?;

    // Your app code here
    let result = my_operation();

    // Verify spans were emitted
    assert_ok!(weaver.validate_recent_spans());
});
```

## Best Practices

1. **Start with OTEL** - Understand basic concepts
2. **Add Weaver** - Validate semantic compliance
3. **Test fixtures** - Use fixture-based OTEL tests
4. **Document spans** - Clear attribute meanings
5. **Validate early** - Catch issues before production

## Troubleshooting

**Q: "Feature 'otel' is required"**
A: Enable feature:
```bash
cargo run --features otel -- obs stat
```

**Q: "Weaver not found"**
A: Bootstrap first:
```bash
cargo run --features weaver -- obs bootstrap
```

**Q: "Docker is not running"**
A: Either start Docker or skip:
```bash
export WEAVER_ALLOW_SKIP=1
cargo run --features weaver -- obs weav
```

**Q: "Span validation failed"**
A: Check semantic conventions:
```bash
# Review expected attributes
cargo run --features weaver -- obs weav --verbose
```

## Integration with Other Features

Observability works alongside testing:

```
Core Testing
    ↓
Testing Features (property, mutation, snapshot)
    ↓
Validation Features (coverage, guards, JTBD)
    ↓
Observability (OTEL, Weaver)
    ↓
Integration Testing (Docker, containers)
```

## Next Steps

- **Copy to your project** → [Copying Examples](../tutorials/copying-examples.md)
- **Integration features** → [Integration Features](integration-features.md)
- **Testing features** → [Testing Features Guide](testing-features.md)
- **See all examples** → [Example Inventory](../reference/example-inventory.md)

---

Monitor and validate your instrumentation with OTEL and Weaver.
