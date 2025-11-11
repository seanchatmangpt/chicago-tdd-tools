# Pattern 5: Real Collaborators

## Context

You are validating behavior that depends on external systems – telemetry, containers, queues – and want confidence that integrations behave the same way in production.

## Problem

Mock-heavy tests can mask integration gaps, drift from reality, and erode trust in the test suite. When production fails, tests offer little guidance.

## Solution

Use the framework's integration helpers to exercise real collaborators. For containers, enable the `testcontainers` feature and run against Docker. For telemetry, use `otel_test!` and `weaver_test!` to validate spans and semantic conventions. Keep tests categorized so slower integration suites run intentionally.

## Forces

- Fidelity vs. speed: real dependencies cost more time, so isolate them behind feature flags and profiles
- Determinism vs. variability: control randomness via fixtures and builders
- Observability vs. complexity: prefer higher-level validators over low-level asserts

## Examples

```rust
use chicago_tdd_tools::prelude::*;
use chicago_tdd_tools::observability::otel::{OtelTestHelper, SpanValidator};

otel_test!(test_span_follows_conventions, {
    // Arrange
    let helper = OtelTestHelper::new();
    let span = helper.capture(|tracer| tracer.span("checkout"));

    // Act
    let result = helper.assert_spans_valid(&[span.clone()]);

    // Assert
    assert_ok!(&result);
});
```

```rust
use chicago_tdd_tools::prelude::*;
use chicago_tdd_tools::integration::testcontainers::ContainerClient;

fixture_test_with_timeout!(test_postgres_roundtrip, fixture, 30, {
    // Arrange
    let client = ContainerClient::for_image("postgres:16").await?;

    // Act
    let rows = client.query("SELECT 1").await?;

    // Assert
    assert_eq!(rows[0].get::<i32, _>(0), 1);

    Ok::<(), testcontainers::Error>(())
});
```

## Related Patterns

- Pattern 4: Resource Cleanup
- Pattern 16: Fixture Lifecycle Management
- Pattern 18: Timeout Defense in Depth
