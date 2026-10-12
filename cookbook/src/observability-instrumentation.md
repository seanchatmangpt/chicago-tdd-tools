# Observability & Instrumentation in Tests

> 🔧 **OBSERVABILITY & INSTRUMENTATION IN TESTS** | **How-to** | Solve this problem

> 🔍 **REFERENCE** | Instrument tests with observability to validate telemetry in real-time

This guide shows how to use Chicago TDD Tools' observability features to validate that your code generates correct telemetry (OpenTelemetry spans, metrics, Weaver validation).

---

## Quick Answer: What Observability Testing Solves

| Problem | Solution | Pattern |
|---------|----------|---------|
| "Do my spans have the right attributes?" | OTEL span validation | Pattern 5 + Observability |
| "Is my instrumentation correct?" | Live Weaver validation | Pattern 5 + Weaver |
| "Are metric names and units correct?" | OTEL metric validation | Pattern 5 + Observability |
| "Did this code emit a span?" | Span existence check | Pattern 5 + Observability |

---

## Core Principle: Observability as Behavior

In Chicago TDD, observability is part of **observable behavior**. Pattern 5 (Real Collaborators) extends to real observability systems:

- ✅ **Real spans** (not mock log lines)
- ✅ **Real metrics** (not assertions on method calls)
- ✅ **Real Weaver validation** (not just "did we call the API")
- ✅ **Semantic convention compliance** (not freestyle attributes)

---

## Setup: Enable Observability Features

### 1. Add Features to Cargo.toml

```toml
[dev-dependencies]
chicago-tdd-tools = { version = "26.10.12", features = ["observability-full"] }
```

This enables:
- `otel` feature: OpenTelemetry span/metric validation
- `weaver` feature: Live Weaver semantic convention validation
- Automatically downloads Weaver binary + registry

### 2. Initialize Observability in Tests

```rust
use chicago_tdd_tools::observability::fixtures::WeaverTestFixture;

#[test]
fn test_with_observability() {
    // Arrange: Create fixture with observability
    let mut fixture = WeaverTestFixture::new()
        .expect("Failed to initialize Weaver fixture");

    // Act: Your code under test runs (emits spans through the fixture tracer)
    let tracer = fixture
        .tracer("test-component", "your-service")
        .expect("Failed to create tracer");
    your_function_that_emits_spans(&tracer);

    // Assert: Flush telemetry and inspect the Weaver validation results
    let results = fixture
        .finish()
        .expect("Failed to collect Weaver validation results");
    assert!(!results.has_violations());
}
```

---

## Pattern Integration: OTEL Validation (No Weaver)

Use when: You want to validate span structure without Weaver semantic conventions.

### Example: Validating Span Attributes

```rust
use chicago_tdd_tools::observability::otel::test_helpers::create_test_span_with_attributes;
use chicago_tdd_tools::observability::otel::SpanValidator;
use std::collections::BTreeMap;

#[test]
fn test_span_has_correct_attributes() {
    // Arrange
    let validator = SpanValidator::new()
        .with_required_attributes(vec![
            "db.system".to_string(),
            "db.operation".to_string(),
        ]);

    // Act: Function under test emits span
    let result = my_function_that_creates_span();
    let span = create_test_span_with_attributes("db.query", {
        let mut attrs = BTreeMap::new();
        attrs.insert("db.system".to_string(), "postgres".to_string());
        attrs.insert("db.operation".to_string(), "SELECT".to_string());
        attrs
    });

    // Assert: Span satisfies the validator
    validator
        .validate(&span)
        .expect("Span failed validation");
    // Verify function returned correct result
    assert_ok!(result);
}
```

**What this validates:**
- ✅ Span was created with correct name
- ✅ Required attributes present (`SpanValidator::with_required_attributes`)
- ✅ Span ID / trace ID valid (non-zero validation is on by default)
- ✅ Function behavior correct

---

## Pattern Integration: Weaver Live Validation

Use when: You want to validate spans against semantic conventions in real-time.

### Example: Validating Against Semantic Conventions

```rust
use chicago_tdd_tools::observability::fixtures::WeaverTestFixture;

#[test]
fn test_http_request_span_complies_with_conventions() {
    // Arrange: Create Weaver fixture
    let mut fixture = WeaverTestFixture::new()
        .expect("Weaver not available (needs Docker)");

    // Act: Code under test, instrumented through the fixture tracer
    let tracer = fixture
        .tracer("http-client", "your-service")
        .expect("Failed to create tracer");
    let response = make_http_request(&tracer, "GET", "https://api.example.com/users/123");

    // Assert: Flush telemetry and check Weaver's verdict
    let results = fixture
        .finish()
        .expect("Failed to collect Weaver validation results");

    // Weaver checks these automatically:
    // - http.method is one of GET, POST, etc (not "download")
    // - http.url is valid URL (not just "example")
    // - http.status_code is 100-599 (not 999)
    // - http.response_content_length is positive (not negative)
    assert!(!results.has_violations(), "{}", results.violations_summary());
    assert_ok!(response);
}
```

**What Weaver validates:**
- ✅ Attribute names match semantic conventions
- ✅ Attribute types correct (string, int, array, etc)
- ✅ Attribute values valid (http.status_code is 1-599, not 999)
- ✅ Required attributes present
- ✅ No extra non-standard attributes

---

## Advanced: Combining Pattern 5 (Real Collaborators) + Observability

### Full Integration Test: Database Query + Observability

```rust
use chicago_tdd_tools::integration::testcontainers::{ContainerClient, GenericContainer};
use chicago_tdd_tools::observability::fixtures::WeaverTestFixture;

#[test]
fn test_database_query_with_instrumentation() {
    // Arrange: Real database container + Weaver
    let client = ContainerClient::new();
    let db_container = GenericContainer::new(&client, "postgres", "latest")
        .expect("Docker not available");

    let mut weaver = WeaverTestFixture::new()
        .expect("Weaver not available");

    let port = db_container.get_host_port(5432)?;
    let db_url = format!("postgres://localhost:{port}/db");

    // Act: Execute query (which emits spans through the fixture tracer)
    let tracer = weaver
        .tracer("db-client", "your-service")
        .expect("Failed to create tracer");
    let result = execute_query_with_logging(&tracer, db_url, "SELECT * FROM users");

    // Assert: Verify both behavior and observability
    assert_ok!(result);

    // Flush telemetry and validate semantic conventions via Weaver
    let results = weaver
        .finish()
        .expect("Failed to collect Weaver validation results");
    // Weaver checks: db.system = "postgres", db.operation = "SELECT", etc
    assert!(!results.has_violations(), "{}", results.violations_summary());
}
```

**This pattern validates:**
- ✅ Code behavior (query succeeded)
- ✅ Real database interaction (not mock)
- ✅ Correct instrumentation (spans emitted)
- ✅ Semantic convention compliance (Weaver validation)

---

## Common Patterns: When to Use What

| Scenario | Use | Why |
|----------|-----|-----|
| **Validating span structure (names, attributes)** | OTEL | Lightweight, no Weaver |
| **Validating semantic conventions** | Weaver | Automatic validation against standards |
| **Testing database instrumentation** | Pattern 5 + OTEL/Weaver | Real database + real spans |
| **Testing HTTP client instrumentation** | Pattern 5 + OTEL/Weaver | Real HTTP calls + real spans |
| **Testing async code instrumentation** | OTEL + async_test! | Async spans + async tests |
| **Validating multi-span traces** | Weaver | Validates entire trace structure |

---

## Troubleshooting: Common Observability Issues

### Issue 1: "Weaver Not Available"

```
thread 'test_with_weaver' panicked at 'Failed to initialize Weaver: Docker not running'
```

**Solution:**
```bash
# Start Docker
docker daemon

# Or skip Weaver in tests
export WEAVER_ALLOW_SKIP=1  # Test runs without Weaver validation
```

### Issue 2: "Span Not Found"

```
thread 'test_http_span' panicked at 'Expected span "http.client" not found'
```

**Causes & Solutions:**
- ❌ Code didn't emit span → Check instrumentation code
- ❌ Span name wrong → Check span name in code vs test
- ❌ Span emitted after test collected → Flush spans before assert

**Fix:**
```rust
// Ensure spans are flushed before validating
let results = fixture.finish()?;
assert!(!results.has_violations(), "{}", results.violations_summary());
```

### Issue 3: "Semantic Convention Violation"

```
thread 'test_http_span' panicked at 'Attribute "http.status_code": expected int, got string "200"'
```

**Solution:** Check attribute types in instrumentation code
```rust
// ❌ WRONG: String status code
span.set_attribute("http.status_code", "200");

// ✅ RIGHT: Integer status code
span.set_attribute("http.status_code", 200i32);
```

---

## The Checklist: Before Shipping Observability

Before shipping code that emits telemetry, verify:

**Span Existence:**
- ✅ Code emits expected spans
- ✅ Span names match semantic conventions
- ✅ Spans appear in correct order
- ✅ Spans have correct parent-child relationships

**Span Attributes:**
- ✅ All required attributes present
- ✅ Attribute names match conventions
- ✅ Attribute types correct (int, string, array)
- ✅ Attribute values valid and sensible

**Semantic Conventions (Weaver):**
- ✅ All attributes validated against registry
- ✅ No typos in attribute names
- ✅ Required attributes always present
- ✅ Attribute values within valid ranges

**Integration:**
- ✅ Spans emitted during real operations (Pattern 5)
- ✅ Test passes with real database/service
- ✅ Test passes with real Weaver validation
- ✅ No `.unwrap()` in instrumentation code

---

## Real-World Example: Complete Observability Test

```rust
use chicago_tdd_tools::{
    integration::testcontainers::{ContainerClient, GenericContainer},
    observability::fixtures::WeaverTestFixture,
    test,
};

// Pattern 5 (Real Collaborators) + Observability
test!(test_user_service_with_full_observability, {
    // Arrange: Database + Weaver
    let client = ContainerClient::new();
    let db = GenericContainer::new(&client, "postgres", "latest")?;
    let mut weaver = WeaverTestFixture::new()?;
    let port = db.get_host_port(5432)?;
    let db_url = format!("postgres://localhost:{port}/db");

    let tracer = weaver
        .tracer("user-service", "your-service")?;
    let user_service = UserService::new(&db_url, tracer);

    // Act: Create user (emits spans, writes to DB)
    let result = user_service.create_user(User {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    });

    // Assert: Behavior correct
    assert_ok!(result);
    let user_id = result.unwrap();

    // Assert: Database state correct (Pattern 5 = real DB)
    let stored_user = db.query("SELECT * FROM users WHERE id = ?", &[user_id])?;
    assert_eq!(stored_user.name, "Alice");

    // Assert: Observability correct
    // Flush telemetry and validate semantic conventions (Weaver)
    let results = weaver.finish()?;
    assert!(!results.has_violations(), "{}", results.violations_summary());
});
```

This test validates:
- ✅ Behavior (user created, stored in DB)
- ✅ Integration (real database)
- ✅ Observability (spans emitted)
- ✅ Semantic compliance (Weaver validation)
- ✅ Trace structure (spans properly linked)

---

## Related Patterns

This guide integrates with:
- **Pattern 5: Real Collaborators** - Use real observability systems, not mocks
- **Pattern 1: AAA Pattern** - Arrange (setup observability), Act (emit spans), Assert (validate)
- **Pattern 4: Resource Cleanup** - Weaver/OTEL resources cleaned up automatically
- **Pattern 16: Fixture Lifecycle** - WeaverTestFixture guarantees cleanup via Drop

---

## Next Steps

1. **Start simple:** Use OTEL validation without Weaver
2. **Add Weaver:** Enable Docker and validate semantic conventions
3. **Integrate with Pattern 5:** Use real services + observability together
4. **Automate:** Add observability checks to shipping checklist

See [Pattern Dependencies](pattern-dependencies.md) for how observability fits with other patterns.


## Quick Glance

| Aspect | Details |
|--------|---------|
| Problem | Addressed by this pattern |
| Solution | Core idea in 1-2 sentences |
| When To Use | Typical scenarios |
| When NOT To Use | Anti-patterns |
| Trade-offs | What you gain/lose |
| Complexity | Low/Medium/High |
| Real-World Example | Link to actual code |
