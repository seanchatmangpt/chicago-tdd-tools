# Getting Started with Chicago TDD Tools

Get up and running with Chicago TDD Tools in 5 minutes. This guide provides verified, runnable examples that you can copy and paste directly into your project.

## Prerequisites

Before you begin, ensure you have:

- **Rust**: Edition 2021 (Rust 1.70 or later)
- **Cargo**: Latest stable version
- **Tokio**: Required for async tests (will be added as dependency)
- **Docker**: Optional, required only for `testcontainers` feature

Verify your Rust version:

```bash
rustc --version  # Should show 1.70 or later
cargo --version  # Should show latest stable
```

## Installation

### Step 1: Add Dependency

Add `chicago-tdd-tools` to your `Cargo.toml`:

```toml
[dev-dependencies]
chicago-tdd-tools = { path = "../chicago-tdd-tools" }
tokio = { version = "1.0", features = ["rt", "macros"] }
```

**Note**: Adjust the path based on your project structure. If `chicago-tdd-tools` is in a parent directory, use `path = "../chicago-tdd-tools"`.

### Step 2: Verify Installation

Check that the crate compiles:

```bash
cargo make check
```

If you see compilation errors, verify:
- The path to `chicago-tdd-tools` is correct
- Rust edition is set to 2021 in your `Cargo.toml`
- Tokio is included in dev-dependencies

## Your First Test

### Step 1: Create Test File

Create `tests/my_first_test.rs`:

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

### Step 2: Run the Test

```bash
cargo make test test_basic_example
```

**Expected output**:
```
running 1 test
test test_basic_example ... ok

test result: ok. 1 passed; 0 failed; 0 ignored
```

### Step 3: Verify It Works

If the test passes, congratulations! You've written your first Chicago TDD test. The test follows the AAA pattern:
- **Arrange**: Set up test data (`let input = 5`)
- **Act**: Execute the feature (`let result = input * 2`)
- **Assert**: Verify behavior (`assert_eq!(result, 10)`)

## Common Patterns

### Async Test Example

For async operations, use `chicago_async_test!`:

```rust
use chicago_tdd_tools::prelude::*;

chicago_async_test!(test_async_example, {
    // Arrange: Create fixture
    let fixture = TestFixture::new().unwrap();
    
    // Act: Execute async operation
    let counter = fixture.test_counter();
    
    // Assert: Verify state
    assert!(counter >= 0);
});
```

**Run it**:
```bash
cargo make test test_async_example
```

### Test with Automatic Fixture

Use `chicago_fixture_test!` for automatic fixture setup/teardown:

```rust
use chicago_tdd_tools::prelude::*;

chicago_fixture_test!(test_with_fixture, fixture, {
    // Arrange: Fixture automatically created
    let counter = fixture.test_counter();
    
    // Act: Execute test
    let result = counter + 1;
    
    // Assert: Verify behavior
    assert!(result > 0);
    // Fixture automatically cleaned up on drop
});
```

**Run it**:
```bash
cargo make test test_with_fixture
```

### Test Data Builder

Create test data with fluent builders:

```rust
use chicago_tdd_tools::prelude::*;

chicago_test!(test_data_builder, {
    // Arrange: Create test data
    let data = TestDataBuilder::new()
        .with_var("key1", "value1")
        .with_order_data("ORD-001", "100.00")
        .build_json();
    
    // Assert: Verify data
    assert_eq!(data["key1"], "value1");
    assert_eq!(data["order_id"], "ORD-001");
    assert_eq!(data["total_amount"], "100.00");
});
```

**Run it**:
```bash
cargo make test test_data_builder
```

### Asserting Results

Use assertion macros for better error messages:

```rust
use chicago_tdd_tools::prelude::*;

chicago_test!(test_result_assertions, {
    // Arrange: Create results
    let ok_result: Result<u32, String> = Ok(42);
    let err_result: Result<u32, String> = Err("error".to_string());
    
    // Assert: Use assertion macros
    assert_ok!(&ok_result, "Operation should succeed");
    assert_err!(&err_result, "Operation should fail");
});
```

**Run it**:
```bash
cargo make test test_result_assertions
```

### Performance Testing

Validate hot path performance with tick measurement:

```rust
use chicago_tdd_tools::prelude::*;

chicago_performance_test!(test_hot_path, {
    // Arrange: Set up test data
    let input = vec![1, 2, 3];
    
    // Act: Execute hot path and measure ticks
    let (result, ticks) = measure_ticks(|| {
        input.iter().sum::<i32>()
    });
    
    // Assert: Verify performance constraint (≤8 ticks)
    assert_within_tick_budget!(ticks, "Hot path operation");
    assert_eq!(result, 6);
});
```

**Run it**:
```bash
cargo make test test_hot_path
```

**Note**: On non-x86_64 platforms, RDTSC falls back to `std::time::Instant`. The tick budget still applies.

## Optional Features

### Property-Based Testing

Enable property-based testing:

```toml
[dev-dependencies]
chicago-tdd-tools = { 
    path = "../chicago-tdd-tools",
    features = ["property-testing"]
}
```

Example:

```rust
use chicago_tdd_tools::prelude::*;

#[cfg(feature = "property-testing")]
chicago_test!(test_property, {
    // Arrange: Create generator
    let mut generator = PropertyTestGenerator::<10, 3>::new()
        .with_seed(42);
    
    // Act & Assert: Test property
    assert!(
        property_all_data_valid(&mut generator, 100),
        "Property: All generated data is valid"
    );
});
```

### Mutation Testing

Enable mutation testing:

```toml
[dev-dependencies]
chicago-tdd-tools = { 
    path = "../chicago-tdd-tools",
    features = ["mutation-testing"]
}
```

### Testcontainers

Enable Docker container support:

```toml
[dev-dependencies]
chicago-tdd-tools = { 
    path = "../chicago-tdd-tools",
    features = ["testcontainers"]
}
```

**Note**: Requires Docker to be running.

## Verify Installation

Run the full test suite to verify everything works:

```bash
cargo make test
```

This runs:
- All unit tests
- All integration tests
- All examples

## What's Next?

Now that you're set up, explore:

1. **[User Guide](USER_GUIDE.md)** - Comprehensive guide with all features
   - Test fixtures and builders
   - Macros and assertions
   - Property-based testing
   - Mutation testing
   - Performance testing
   - And more...

2. **[API Reference](API_REFERENCE.md)** - Complete API documentation
   - All types and functions
   - Parameters and return values
   - Examples for each API

3. **[Architecture](ARCHITECTURE.md)** - Design principles and patterns
   - Extension patterns
   - Design decisions
   - Best practices

4. **Examples Directory** - Working code examples
   ```bash
   cargo make test-examples
   ```

## Troubleshooting

### Compilation Errors

**Error**: `cannot find crate 'chicago_tdd_tools'`
- **Solution**: Verify the path in `Cargo.toml` is correct
- **Check**: Run `cargo make check` to see detailed error messages

**Error**: `edition 2021 is required`
- **Solution**: Add `edition = "2021"` to your `Cargo.toml` `[package]` section

**Error**: `cannot find macro 'chicago_test'`
- **Solution**: Ensure you have `use chicago_tdd_tools::prelude::*;` at the top

### Runtime Errors

**Error**: `TestFixture::new()` panics
- **Solution**: For async tests, ensure tokio runtime is available
- **Check**: Use `chicago_fixture_test!` for automatic fixture setup

**Error**: Property-based tests don't compile
- **Solution**: Enable `property-testing` feature flag
- **Check**: Verify feature is enabled in `Cargo.toml`

**Error**: Testcontainers tests fail
- **Solution**: Ensure Docker is running
- **Check**: Run `docker ps` to verify Docker is accessible

### Performance Test Issues

**Issue**: Performance tests fail on non-x86_64
- **Solution**: RDTSC is x86_64-specific; tests automatically fall back to `std::time::Instant`
- **Note**: Tick budget still applies, but measurement method differs

## Platform-Specific Notes

### Linux

- Works out of the box
- RDTSC available on x86_64
- Docker required for testcontainers feature

### macOS

- Works out of the box
- RDTSC available on x86_64 (Intel Macs)
- Apple Silicon (ARM) uses `std::time::Instant` fallback
- Docker Desktop required for testcontainers feature

### Windows

- Works out of the box
- RDTSC available on x86_64
- Docker Desktop required for testcontainers feature

## Next Steps

You're ready to start writing tests! Here's a suggested learning path:

1. ✅ **Complete this guide** - You're here!
2. 📖 **Read User Guide** - Learn all features
3. 🔍 **Explore API Reference** - Understand all APIs
4. 🏗️ **Review Architecture** - Understand design principles
5. 💡 **Try Examples** - Run `cargo make test-examples`
6. 🚀 **Write Your Tests** - Apply Chicago TDD principles

## Need Help?

- **Documentation**: Check [User Guide](USER_GUIDE.md) and [API Reference](API_REFERENCE.md)
- **Examples**: Run `cargo make test-examples` to see working code
- **Architecture**: Review [Architecture](ARCHITECTURE.md) for design patterns
- **Issues**: Check GitHub issues for common problems

Happy testing! 🎉
