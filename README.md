# Chicago TDD Tools

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

Testing framework for Chicago TDD (Classicist Test-Driven Development) in Rust. Enforces AAA pattern, provides fixtures, builders, and advanced testing capabilities.

## Chicago TDD Principles

This framework enforces Chicago TDD (Classicist) principles:

- **State-Based Testing**: Tests verify outputs and state, not implementation details
- **Real Collaborators**: Uses actual dependencies, not mocks
- **Behavior Verification**: Tests verify what code does, not how it does it
- **AAA Pattern**: All tests follow Arrange-Act-Assert structure

See [Chicago TDD Standards](.cursor/rules/chicago-tdd-standards.mdc) for complete methodology.

## Dog Fooding

The framework tests itself using its own tools. All framework tests use `test!` macros and framework features, demonstrating real-world usage patterns and validating framework ergonomics.

See [Dog Fooding Documentation](docs/DOG_FOODING.md) for details.

## Quick Start

### Installation

**Step 1**: Install `cargo-make` (required for build system):
```bash
cargo install cargo-make
```

**Step 2**: Add dependency to `Cargo.toml`:
```toml
[dev-dependencies]
chicago-tdd-tools = { path = "../chicago-tdd-tools" }
tokio = { version = "1.0", features = ["rt", "macros"] }
```

**Note**: For optional features (property-testing, testcontainers, weaver, etc.), enable them:
```toml
[dev-dependencies]
chicago-tdd-tools = { path = "../chicago-tdd-tools", features = ["property-testing", "testcontainers"] }
```

**Step 3**: Verify installation:
```bash
cargo make check
```

### Optional Features

Enable features as needed:
```toml
[dev-dependencies]
chicago-tdd-tools = { 
    path = "../chicago-tdd-tools",
    features = [
        "property-testing",  # Property-based testing
        "testcontainers",     # Docker container support
        "weaver",            # Weaver live validation
        "otel"               # OTEL span/metric validation
    ]
}
```

**Common Feature Combinations**:
- **Basic testing**: No features needed (default)
- **Property testing**: `property-testing`
- **Integration testing**: `testcontainers`
- **Observability testing**: `otel`, `weaver`

### Your First Test

**Synchronous Test**:
```rust
use chicago_tdd_tools::prelude::*;

test!(test_example, {
    // Arrange: Set up test data
    let input = 5;
    
    // Act: Execute feature
    let result = input * 2;
    
    // Assert: Verify behavior
    assert_eq!(result, 10);
});
```

**Async Test**:
```rust
use chicago_tdd_tools::prelude::*;

async_test!(test_async_example, {
    // Arrange: Set up test data
    let expected = 10;
    
    // Act: Execute async operation
    let result = async {
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        5 * 2
    }.await;
    
    // Assert: Verify behavior
    assert_eq!(result, expected);
});
```

**Fixture Test**:
```rust
use chicago_tdd_tools::prelude::*;

fixture_test!(test_with_fixture, fixture, {
    // Arrange: Fixture automatically created
    let counter = fixture.test_counter();
    
    // Act: Execute test
    let result = counter + 1;
    
    // Assert: Verify behavior
    assert!(result > 0);
});
```

**Run Tests**:
```bash
cargo make test
```

## Features

### Core Features (Always Available)

- **Fixtures**: Reusable test fixtures with state management and automatic cleanup
- **Builders**: Fluent builders for test data (JSON, HashMap, domain-specific)
- **Assertions**: Comprehensive assertion utilities (`assert_ok!`, `assert_err!`, `assert_in_range!`, etc.)
- **Macros**: AAA pattern enforcement (`test!`, `async_test!`, `fixture_test!`)
- **Performance Testing**: RDTSC-based tick measurement and budget validation
- **Guards**: Constraint enforcement (`MAX_RUN_LEN` ≤ 8, `MAX_BATCH_SIZE`)
- **JTBD Validation**: Jobs To Be Done validation framework
- **Coverage**: Test coverage analysis and reporting

### Optional Features (Enable in `Cargo.toml`)

#### Individual Features

- **`property-testing`**: Property-based testing with proptest (random test generation, finding edge cases)
- **`mutation-testing`**: Mutation testing for test quality validation
- **`snapshot-testing`**: Snapshot testing with insta (output comparison, regression testing)
- **`fake-data`**: Fake data generation for test data (realistic test data creation)
- **`concurrency-testing`**: Concurrency testing with loom (thread model checking, deterministic testing)
- **`parameterized-testing`**: Parameterized tests with rstest (multiple inputs, test matrices)
- **`cli-testing`**: CLI testing with trycmd (command execution testing, golden files)
- **`testcontainers`**: Docker container support for integration testing
- **`otel`**: OpenTelemetry span/metric validation
- **`weaver`**: Weaver live validation integration (requires `otel`)
- **`async`**: Async fixture providers (async traits, Rust 1.75+)
- **`benchmarking`**: Criterion benchmarking support
- **`workflow-engine`**: Workflow-specific features

#### Feature Groups (Convenience Bundles)

For better DX, common feature combinations are available as feature groups:

- **`testing-extras`**: Most common advanced testing features (`property-testing`, `snapshot-testing`, `fake-data`)
  - Use when: You want comprehensive test coverage with property-based and snapshot testing
- **`testing-full`**: All testing features (property, snapshot, mutation, concurrency, parameterized, cli, fake-data)
  - Use when: You need maximum testing capabilities for comprehensive test suites
- **`observability-full`**: Complete observability stack (`otel`, `weaver`)
  - Use when: You need full observability validation with Weaver integration
- **`integration-full`**: Full integration testing (`testcontainers`, `weaver`)
  - Use when: You need integration testing with Docker containers and Weaver observability

**Examples**: Enable features individually or use feature groups:

```toml
# Individual features
[dev-dependencies]
chicago-tdd-tools = { 
    path = "../chicago-tdd-tools",
    features = ["property-testing", "snapshot-testing", "fake-data"]
}

# Feature groups (recommended for common combinations)
[dev-dependencies]
chicago-tdd-tools = { 
    path = "../chicago-tdd-tools",
    features = ["testing-extras"]  # Enables property-testing, snapshot-testing, fake-data
}

# Combine feature groups with individual features
[dev-dependencies]
chicago-tdd-tools = { 
    path = "../chicago-tdd-tools",
    features = ["testing-extras", "testcontainers"]  # testing-extras + testcontainers
}
```

## Module Organization

Modules are organized into capability groups for better discoverability:

- **`core/`**: Core testing infrastructure (fixtures, async_fixture, builders, assertions, macros, state, const_assert, alert)
- **`testing/`**: Advanced testing techniques (property, mutation, snapshot, concurrency, cli, generator)
- **`validation/`**: Quality & validation (coverage, guards, jtbd, performance)
- **`observability/`**: Telemetry & observability (otel, weaver)
- **`integration/`**: Integration testing (testcontainers)

All modules are re-exported at the crate root for backward compatibility. See [Architecture](docs/ARCHITECTURE.md) for details.

## Build System

**CRITICAL**: Always use `cargo make` commands, never direct `cargo` commands. The build system handles proc-macro crates, includes timeouts, and ensures consistency.

```bash
cargo make check      # Compile check (with timeout)
cargo make test       # Run tests (excludes testcontainers by default)
cargo make test-integration  # Run testcontainers integration tests (requires Docker)
cargo make lint       # Run clippy
cargo make pre-commit # Pre-commit validation
```

See [Build System Practices](.cursor/rules/build-system-practices.mdc) for details.

## Documentation

### Getting Started
- **[Quick Guide](docs/QUICK_GUIDE.md)** - Essential patterns (80% of use cases)
- **[Getting Started](docs/GETTING_STARTED.md)** - Quick start guide with verified examples

### Comprehensive Guides
- **[User Guide](docs/USER_GUIDE.md)** - Complete usage guide with patterns and best practices
- **[API Reference](docs/API_REFERENCE.md)** - Complete API documentation
- **[Architecture](docs/ARCHITECTURE.md)** - Design principles and extension patterns

### Methodology & Practices
- **[Chicago TDD Standards](.cursor/rules/chicago-tdd-standards.mdc)** - Testing methodology and standards
- **[Dog Fooding](docs/DOG_FOODING.md)** - Framework self-testing principles
- **[SPR Guide](docs/SPR_GUIDE.md)** - Sparse Priming Representation methodology
- **[Build System Practices](.cursor/rules/build-system-practices.mdc)** - Build system and workflow practices

### Examples
- **[Examples](examples/)** - Working code examples demonstrating all features

## Requirements

### Required
- **Rust**: Edition 2021 (Rust 1.70+)
- **Cargo**: Latest stable
- **cargo-make**: Required for build system (`cargo install cargo-make`)
- **Tokio**: Required for async tests (included in dev-dependencies)

### Optional
- **Docker**: Required for `testcontainers` feature (must be running for integration tests)
- **Criterion**: Optional for `benchmarking` feature (install separately for benches/)

**Verify Installation**:
```bash
rustc --version  # Should be 1.70+
cargo --version  # Latest stable
cargo make --version  # Should be installed
```

## License

MIT
