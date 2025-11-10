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

chicago_test!(test_basic_example, {
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

**Async Test**: `chicago_async_test!(name, { /* AAA */ })`. **Fixture Test**: `chicago_fixture_test!(name, fixture, { /* AAA */ })`. **Data Builder**: `TestDataBuilder::new().with_var(key, value).build_json()`. **Result Assertions**: `assert_ok!(&result)`, `assert_err!(&result)`. **Performance Test**: `chicago_performance_test!(name, { let (result, ticks) = measure_ticks(|| operation()); assert_within_tick_budget!(ticks); })`.

## Optional Features

**Weaver**: Enable `weaver` feature. Weaver CLI auto-downloaded. See WEAVER_LIVE_CHECK.md. **Property-Based**: Enable `property-testing` feature. **Mutation**: Enable `mutation-testing` feature. **Testcontainers**: Enable `testcontainers` feature. Requires Docker running.

## Verify Installation

**Run**: `cargo make test`. Runs all unit tests, integration tests, examples.

## Next Steps

- **[Quick Guide](QUICK_GUIDE.md)** - Essential patterns (80% of use cases)
- **[User Guide](USER_GUIDE.md)** - Comprehensive guide with all features
- **[API Reference](API_REFERENCE.md)** - Complete API documentation
- **[Architecture](ARCHITECTURE.md)** - Design principles and patterns
- **[Examples](../examples/)** - Working code examples (`cargo make test-examples`)

## Troubleshooting

**Compilation Errors**: `cannot find crate` → verify path in Cargo.toml. `edition 2021 required` → add `edition = "2021"`. `cannot find macro` → ensure `use chicago_tdd_tools::prelude::*;`.

**Runtime Errors**: `TestFixture::new()` panics → ensure tokio runtime for async tests, use `chicago_fixture_test!`. Property tests don't compile → enable `property-testing` feature. Testcontainers tests fail → ensure Docker running.

**Performance Tests**: Fail on non-x86_64 → RDTSC is x86_64-specific, falls back to `std::time::Instant`. Tick budget still applies.

## Platform-Specific

**Linux**: Works out of box. RDTSC on x86_64. Docker for testcontainers. **macOS**: Works out of box. RDTSC on x86_64 (Intel). ARM uses `std::time::Instant` fallback. Docker Desktop for testcontainers. **Windows**: Works out of box. RDTSC on x86_64. Docker Desktop for testcontainers.

## Summary

**Key Associations**: Installation = Dependency + Verify. First Test = AAA Pattern = Success. Common Patterns = Macros + Builders + Assertions. Optional Features = Feature Flags = Docker.

**Pattern**: Add dependency → Create test → Run test → Verify success → Explore features. All tests follow AAA pattern. Use macros for all tests. Verify observable outputs.
