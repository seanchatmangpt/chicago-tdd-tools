# Getting Started - SPR

Get up and running with Chicago TDD Tools in 5 minutes. Verified, runnable examples.

## Prerequisites

**Rust**: Edition 2021 (Rust 1.70+). **Cargo**: Latest stable. **Tokio**: Required for async tests. **Docker**: Optional, for `testcontainers` feature.

**Verify**: `rustc --version` (1.70+), `cargo --version` (latest stable).

## Installation

**Step 1**: Add dependency to `Cargo.toml`:
```toml
[dev-dependencies]
chicago-tdd-tools = { path = "../chicago-tdd-tools" }
tokio = { version = "1.0", features = ["rt", "macros"] }
```

**Step 2**: Verify installation: `cargo make check`. If errors: verify path, Rust edition 2021, Tokio in dev-dependencies.

## Your First Test

**Create** `tests/my_first_test.rs`:
```rust
use chicago_tdd_tools::prelude::*;

test!(test_basic_example, {
    // Arrange: Set up test data
    let input = 5;
    // Act: Execute feature
    let result = input * 2;
    // Assert: Verify behavior
    assert_eq!(result, 10);
});
```

**Run**: `cargo make test test_basic_example`. **Verify**: Test passes. AAA pattern: Arrange (input), Act (result), Assert (verify).

## Common Patterns

**Async Test**: `async_test!(name, { /* AAA */ })`. **Fixture Test**: `fixture_test!(name, fixture, { /* AAA */ })`. **Data Builder**: `TestDataBuilder::new().with_var(key, value).build_json()`. **Result Assertions**: `assert_ok!(&result)`, `assert_err!(&result)`. **Performance Test**: `performance_test!(name, { let (result, ticks) = measure_ticks(|| operation()); assert_within_tick_budget!(ticks); })`.

## Optional Features

### Individual Features

Enable individual features as needed:
- **`property-testing`**: Property-based testing with proptest (random test generation)
- **`snapshot-testing`**: Snapshot testing with insta (output comparison)
- **`fake-data`**: Fake data generation for realistic test data
- **`mutation-testing`**: Mutation testing for test quality validation
- **`concurrency-testing`**: Concurrency testing with loom (thread model checking)
- **`parameterized-testing`**: Parameterized tests with rstest
- **`cli-testing`**: CLI testing with trycmd (command execution testing)
- **`testcontainers`**: Docker container support (requires Docker running)
- **`otel`**: OpenTelemetry span/metric validation
- **`weaver`**: Weaver live validation (requires `otel`, auto-downloads Weaver CLI)
- **`async`**: Async fixture providers (async traits, Rust 1.75+)

### Feature Groups (Recommended)

For better DX, use feature groups for common combinations:

**`testing-extras`**: Most common advanced testing features
```toml
[dev-dependencies]
chicago-tdd-tools = { 
    path = "../chicago-tdd-tools",
    features = ["testing-extras"]  # Enables property-testing, snapshot-testing, fake-data
}
```

**`testing-full`**: All testing features
```toml
[dev-dependencies]
chicago-tdd-tools = { 
    path = "../chicago-tdd-tools",
    features = ["testing-full"]  # All testing features
}
```

**`observability-full`**: Complete observability stack
```toml
[dev-dependencies]
chicago-tdd-tools = { 
    path = "../chicago-tdd-tools",
    features = ["observability-full"]  # otel + weaver
}
```

**`integration-full`**: Full integration testing
```toml
[dev-dependencies]
chicago-tdd-tools = { 
    path = "../chicago-tdd-tools",
    features = ["integration-full"]  # testcontainers + weaver
}
```

**See**: [README Features](../README.md#features) for complete feature documentation.

## Verify Installation

**Run**: `cargo make test`. Runs all unit tests, integration tests, examples.

## Next Steps

- **[Quick Guide](QUICK_GUIDE.md)** - Essential patterns (80% of use cases)
- **[User Guide](USER_GUIDE.md)** - Comprehensive guide with all features
- **[API Reference](API_REFERENCE.md)** - Complete API documentation
- **[Architecture](ARCHITECTURE.md)** - Design principles and patterns
- **[Examples](../examples/)** - Working code examples (`cargo make test-examples`)

## Troubleshooting

### Common Compilation Errors

**Error**: `cannot find crate 'chicago_tdd_tools'`
- **Fix**: Verify path in `Cargo.toml` is correct
- **Check**: `[dev-dependencies]` section includes `chicago-tdd-tools = { path = "..." }`

**Error**: `edition 2021 required`
- **Fix**: Add `edition = "2021"` to `Cargo.toml`:
  ```toml
  [package]
  edition = "2021"
  ```

**Error**: `cannot find macro 'test!' in this scope`
- **Fix**: Add `use chicago_tdd_tools::prelude::*;` at top of test file
- **Alternative**: Use explicit import: `use chicago_tdd_tools::test;`

**Error**: `cannot find module 'observability'` or `cannot find module 'integration'`
- **Fix**: Enable required feature in `Cargo.toml`:
  ```toml
  chicago-tdd-tools = { features = ["otel", "weaver", "testcontainers"] }
  ```
- **Note**: Feature-gated modules require explicit feature flags

**Error**: `feature 'X' is required for module Y`
- **Fix**: Add feature to `Cargo.toml`:
  ```toml
  chicago-tdd-tools = { features = ["feature-name"] }
  ```

### Common Runtime Errors

**Error**: `TestFixture::new()` panics
- **Fix**: Use `fixture_test!` macro for async tests, or ensure tokio runtime
- **Note**: `TestFixture::new()` returns `Result` - handle errors properly

**Error**: Property tests don't compile
- **Fix**: Enable `property-testing` feature in `Cargo.toml`

**Error**: Testcontainers tests fail
- **Fix**: Ensure Docker is running
- **Check**: Run `docker ps` to verify Docker is available

**Error**: Performance tests fail on non-x86_64
- **Fix**: RDTSC is x86_64-specific, automatically falls back to `std::time::Instant`
- **Note**: Tick budget still applies, fallback is transparent

## Platform-Specific

**Linux**: Works out of box. RDTSC on x86_64. Docker for testcontainers. **macOS**: Works out of box. RDTSC on x86_64 (Intel). ARM uses `std::time::Instant` fallback. Docker Desktop for testcontainers. **Windows**: Works out of box. RDTSC on x86_64. Docker Desktop for testcontainers.

## Summary

**Key Associations**: Installation = Dependency + Verify. First Test = AAA Pattern = Success. Common Patterns = Macros + Builders + Assertions. Optional Features = Feature Flags = Docker.

**Pattern**: Add dependency → Create test → Run test → Verify success → Explore features. All tests follow AAA pattern. Use macros for all tests. Verify observable outputs.
