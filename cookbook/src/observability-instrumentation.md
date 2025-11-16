# Observability & Instrumentation in Tests

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
chicago-tdd-tools = { version = "1.3.0", features = ["observability-full"] }
```

This enables:
- `otel` feature: OpenTelemetry span/metric validation
- `weaver` feature: Live Weaver semantic convention validation
- Automatically downloads Weaver binary + registry

### 2. Initialize Observability in Tests

```rust
use chicago_tdd_tools::observability::weaver::WeaverTestFixture;
use chicago_tdd_tools::observability::otel::OtelTestValidator;

#[test]
fn test_with_observability() {
    // Arrange: Create fixture with observability
    let fixture = WeaverTestFixture::new()
        .expect("Failed to initialize Weaver fixture");

    // Act: Your code under test runs (emits spans)
    your_function_that_emits_spans();

    // Assert: Validate the spans
    let spans = fixture.collect_spans();
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].name, "your.operation");
}
```

---

## Pattern Integration: OTEL Validation (No Weaver)

Use when: You want to validate span structure without Weaver semantic conventions.

### Example: Validating Span Attributes

```rust
use chicago_tdd_tools::observability::otel::OtelTestValidator;

#[test]
fn test_span_has_correct_attributes() {
    // Arrange
    let validator = OtelTestValidator::new();

    // Act: Function under test emits span
    let result = my_function_that_creates_span();

    // Assert: Span has expected attributes
    validator.assert_span_exists("db.query", |span| {
        assert_eq!(span.get_attribute("db.system"), Some("postgres"));
        assert_eq!(span.get_attribute("db.operation"), Some("SELECT"));
        // Verify no .unwrap() calls in code (no panics recorded)
        assert!(!span.has_exception());
    });

    // Verify function returned correct result
    assert_ok!(result);
}
```

**What this validates:**
- ✅ Span was created with correct name
- ✅ Required attributes present
- ✅ Attribute values correct
- ✅ No exceptions in span

---

## Pattern Integration: Weaver Live Validation

Use when: You want to validate spans against semantic conventions in real-time.

### Example: Validating Against Semantic Conventions

```rust
use chicago_tdd_tools::observability::weaver::WeaverTestFixture;

#[test]
fn test_http_request_span_complies_with_conventions() {
    // Arrange: Create Weaver fixture
    let fixture = WeaverTestFixture::new()
        .expect("Weaver not available (needs Docker)");

    // Act: Code under test
    let response = make_http_request("GET", "https://api.example.com/users/123");

    // Assert: Span complies with semantic conventions
    let result = fixture.validate_span("http.client", |span| {
        // Weaver checks these automatically:
        // - http.method is one of GET, POST, etc (not "download")
        // - http.url is valid URL (not just "example")
        // - http.status_code is 100-599 (not 999)
        // - http.response_content_length is positive (not negative)
        span
    });

    assert_ok!(result);
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
use chicago_tdd_tools::integration::testcontainers::GenericContainer;
use chicago_tdd_tools::observability::weaver::WeaverTestFixture;

#[test]
fn test_database_query_with_instrumentation() {
    // Arrange: Real database container + Weaver
    let db_container = GenericContainer::new(client, "postgres", "latest")
        .expect("Docker not available");

    let weaver = WeaverTestFixture::new()
        .expect("Weaver not available");

    let db_url = db_container.get_connection_string();

    // Act: Execute query (which emits spans)
    let result = execute_query_with_logging(db_url, "SELECT * FROM users");

    // Assert: Verify both behavior and observability
    assert_ok!(result);

    // Validate span structure
    let spans = weaver.collect_spans();
    assert!(spans.iter().any(|s| s.name == "db.client.operation"));

    // Validate semantic conventions
    weaver.validate_span("db.client.operation", |span| {
        // Weaver checks: db.system = "postgres", db.operation = "SELECT", etc
        assert_eq!(span.get_attribute("db.system"), Some("postgres"));
    }).expect("Span violates semantic conventions");
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
// Ensure spans are flushed before collecting
let spans = fixture.collect_spans_with_timeout(Duration::from_secs(1))?;
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
    integration::testcontainers::GenericContainer,
    observability::weaver::WeaverTestFixture,
    test,
};

// Pattern 5 (Real Collaborators) + Observability
test!(test_user_service_with_full_observability, {
    // Arrange: Database + Weaver
    let db = GenericContainer::new(client, "postgres", "latest")?;
    let weaver = WeaverTestFixture::new()?;
    let db_url = db.get_connection_string();

    let user_service = UserService::new(&db_url);

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
    // Validate structure
    let spans = weaver.collect_spans();
    assert!(spans.iter().any(|s| s.name == "db.client.operation"));
    assert!(spans.iter().any(|s| s.name == "user.service.create"));

    // Validate semantic conventions (Weaver)
    weaver.validate_span("db.client.operation", |span| {
        assert_eq!(span.get_attribute("db.system"), Some("postgres"));
        assert_eq!(span.get_attribute("db.operation"), Some("INSERT"));
    })?;

    // Validate trace shows proper nesting
    // user.service.create → db.client.operation (parent-child)
    let service_span = spans.iter().find(|s| s.name == "user.service.create").unwrap();
    let db_span = spans.iter().find(|s| s.name == "db.client.operation").unwrap();
    assert_eq!(db_span.parent_span_id, service_span.span_id);
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
