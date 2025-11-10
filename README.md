# Chicago TDD Tools

[![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)](https://github.com/seanchatmangpt/knhk)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![Status](https://img.shields.io/badge/status-production--ready-success.svg)](https://github.com/seanchatmangpt/knhk)

A comprehensive testing framework for **Chicago TDD** (Classicist Test-Driven Development) methodology in Rust. Provides reusable fixtures, builders, helpers, and advanced testing capabilities following Chicago TDD principles.

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dev-dependencies]
chicago-tdd-tools = { path = "../chicago-tdd-tools" }
tokio = { version = "1.0", features = ["rt", "macros"] }
```

### Your First Test

Create `tests/my_test.rs`:

```rust
use chicago_tdd_tools::prelude::*;

chicago_test!(test_basic, {
    // Arrange: Set up test data
    let input = 5;
    
    // Act: Execute feature
    let result = input * 2;
    
    // Assert: Verify behavior
    assert_eq!(result, 10);
});
```

Run it:

```bash
cargo make test
```

**See [Getting Started Guide](docs/GETTING_STARTED.md) for a complete walkthrough.**

## Features

### Core Components

| Feature | Description | Status |
|---------|-------------|--------|
| **Test Fixtures** | Reusable fixtures with automatic cleanup | ✅ Stable |
| **Test Data Builders** | Fluent builders for test data | ✅ Stable |
| **Assertion Helpers** | Comprehensive assertion utilities | ✅ Stable |
| **Macros** | AAA pattern enforcement and test helpers | ✅ Stable |
| **Property-Based Testing** | QuickCheck-style random test generation | ✅ Stable |
| **Mutation Testing** | Test quality validation through mutations | ✅ Stable |
| **Performance Testing** | RDTSC benchmarking and tick measurement | ✅ Stable |
| **Guards & Constraints** | Guard constraint enforcement (MAX_RUN_LEN ≤ 8) | ✅ Stable |
| **JTBD Validation** | Jobs To Be Done validation framework | ✅ Stable |
| **Testcontainers** | Integration testing with Docker containers | ✅ Stable |
| **OTEL/Weaver** | OpenTelemetry span/metric validation | ✅ Stable |

### Feature Flags

Enable optional features in `Cargo.toml`:

```toml
[dev-dependencies]
chicago-tdd-tools = { 
    path = "../chicago-tdd-tools",
    features = [
        "property-testing",    # Property-based testing
        "mutation-testing",    # Mutation testing
        "testcontainers",      # Docker container support
        "otel",                # OTEL validation
        "weaver",              # Weaver live validation (requires otel)
    ]
}
```

## Requirements

- **Rust**: Edition 2021 (Rust 1.70+)
- **Tokio**: Required for async tests (included in dev-dependencies)
- **Docker**: Required for `testcontainers` feature (optional)

## Usage Examples

### Async Test with Fixture

```rust
use chicago_tdd_tools::prelude::*;

chicago_fixture_test!(test_with_fixture, fixture, {
    // Arrange: Fixture automatically created
    let counter = fixture.test_counter();
    
    // Act: Execute test
    let result = counter + 1;
    
    // Assert: Verify behavior
    assert!(result > 0);
});
```

### Test Data Builder

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
});
```

### Performance Testing

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

### Property-Based Testing

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

## Chicago TDD Principles

This framework enforces Chicago TDD principles:

1. **State-Based Testing**: Tests verify outputs and state, not implementation
2. **Real Collaborators**: Uses actual dependencies, not mocks
3. **Behavior Verification**: Tests verify what code does, not how
4. **AAA Pattern**: All tests follow Arrange-Act-Assert structure

## Documentation

| Document | Description |
|----------|-------------|
| **[Getting Started](docs/GETTING_STARTED.md)** | 5-minute quick start guide with verified examples |
| **[User Guide](docs/USER_GUIDE.md)** | Comprehensive usage guide with patterns and best practices |
| **[API Reference](docs/API_REFERENCE.md)** | Complete API documentation with examples |
| **[Architecture](docs/ARCHITECTURE.md)** | Design principles, architecture, and extension patterns |

## Examples

Run examples with:

```bash
cargo make test-examples
```

Available examples:

- `basic_test.rs` - Basic fixture and builder usage
- `macro_examples.rs` - Macro usage patterns
- `property_testing.rs` - Property-based testing examples
- `mutation_testing.rs` - Mutation testing examples
- `testcontainers_example.rs` - Docker container integration

## Development

### Build Commands

```bash
cargo make check      # Check compilation
cargo make build      # Build in debug mode
cargo make test       # Run all tests
cargo make lint       # Run clippy
cargo make fmt        # Format code
cargo make pre-commit # Run pre-commit checks
```

### Running Tests

```bash
# All tests
cargo make test

# Unit tests only
cargo make test-unit

# Examples only
cargo make test-examples

# Property-based tests
cargo make test-property

# Mutation tests
cargo make test-mutation
```

## Benefits

### ✅ Reduced Boilerplate

- **60% less code** per test compared to standard Rust tests
- Reusable fixtures and builders eliminate repetitive setup
- Consistent patterns across all tests

### ✅ Better Test Quality

- **Property-based testing** finds edge cases automatically
- **Mutation testing** validates test quality and coverage
- **Chicago TDD** ensures correct testing patterns

### ✅ Maintainability

- Centralized fixtures reduce duplication
- Reusable builders for common data patterns
- Consistent helpers across projects

### ✅ Performance Validation

- **RDTSC benchmarking** for hot path validation
- **Tick budget enforcement** (Chatman Constant: ≤8 ticks)
- **Guard constraints** prevent performance regressions

## Troubleshooting

### Common Issues

**Issue**: `TestFixture::new()` fails
- **Solution**: Ensure tokio runtime is available for async tests

**Issue**: Property-based tests don't compile
- **Solution**: Enable `property-testing` feature flag

**Issue**: Testcontainers tests fail
- **Solution**: Ensure Docker is running and `testcontainers` feature is enabled

**Issue**: Performance tests fail on non-x86_64
- **Solution**: RDTSC is x86_64-specific; tests fall back to `std::time::Instant` on other platforms

### Getting Help

- Check the [User Guide](docs/USER_GUIDE.md) for detailed examples
- Review [API Reference](docs/API_REFERENCE.md) for complete API documentation
- See `examples/` directory for working code examples
- Review [Architecture](docs/ARCHITECTURE.md) for extension patterns

## Performance Characteristics

- **Zero-cost abstractions**: Macros expand to efficient code
- **Minimal overhead**: Fixtures use atomic counters for isolation
- **Fast execution**: Property-based tests use efficient generators
- **Hot path validation**: RDTSC provides cycle-accurate measurement

## Security Considerations

- **No unsafe code** in core framework (except RDTSC on x86_64)
- **Input validation** in all public APIs
- **Error handling** prevents panics in production code paths
- **Resource cleanup** via RAII patterns

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Contributing

Contributions welcome! See the [User Guide](docs/USER_GUIDE.md) for development patterns and Chicago TDD principles.

## Repository

- **GitHub**: [seanchatmangpt/knhk](https://github.com/seanchatmangpt/knhk)
- **Version**: 1.0.0
- **Rust Edition**: 2021
