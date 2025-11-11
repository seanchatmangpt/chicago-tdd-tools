# Chicago TDD Tools Playground

Comprehensive playground demonstrating all features of chicago-tdd-tools. This serves as both a validation suite and a reference implementation that demonstrates all capabilities of the framework.

## Purpose

This playground validates that all features work correctly for end users and provides copyable examples that can be adapted for real projects.

## Features Demonstrated

### Core Features (Always Available)
- **Fixtures**: Test fixtures with state management and isolation
- **Async Fixtures**: Async fixture providers (requires `async` feature, Rust 1.75+)
- **Builders**: Fluent builders for test data
- **Assertions**: Comprehensive assertion utilities
- **Macros**: AAA pattern enforcement (`test!`, `async_test!`, `fixture_test!`, etc.)
- **State**: Type-level AAA pattern enforcement
- **Type Level**: Type-level programming with const generics
- **Const Assert**: Compile-time assertions
- **Alert**: Visual problem indicators

### Testing Features (Optional)
- **Property Testing**: Property-based testing with const generics
- **Mutation Testing**: Test quality validation
- **Snapshot Testing**: Output comparison and regression testing
- **Concurrency Testing**: Deterministic thread model checking
- **CLI Testing**: Command-line tool testing with golden files
- **Generator**: Test code generation
- **Parameterized Testing**: Multiple inputs with rstest

### Validation Features (Always Available)
- **Coverage**: Test coverage analysis and reporting
- **Guards**: Guard constraint enforcement (MAX_RUN_LEN ≤ 8, MAX_BATCH_SIZE)
- **JTBD**: Jobs To Be Done validation framework
- **Performance**: RDTSC benchmarking and tick measurement

### Observability Features (Optional)
- **OTEL**: OpenTelemetry span/metric validation
- **Weaver**: Weaver live validation integration (requires `otel`)

### Integration Features (Optional)
- **Testcontainers**: Docker container support for integration testing

## Usage

### Running Examples

```bash
# Run all examples
cargo run --bin playground

# Run specific example module
cargo run --example core::fixtures
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for specific feature
cargo test --features property-testing
cargo test --features snapshot-testing
cargo test --features testcontainers
```

### Feature Flags

All features are enabled by default in this playground. To test with specific features:

```bash
# Test with only core features
cargo test --no-default-features

# Test with specific feature groups
cargo test --features testing-extras
cargo test --features observability-full
cargo test --features integration-full
```

## Project Structure

```
playground/
├── Cargo.toml          # All features enabled
├── PROJECT_CHARTER.md  # Project charter and goals
├── README.md           # This file
├── src/
│   ├── main.rs        # Main entry point
│   ├── core/          # Core features examples
│   ├── testing/       # Testing features examples
│   ├── validation/    # Validation features examples
│   ├── observability/ # Observability features examples
│   └── integration/   # Integration features examples
└── tests/             # Comprehensive test suite
```

## Examples by Category

### Core Features
- `src/core/fixtures.rs` - Test fixtures
- `src/core/async_fixtures.rs` - Async fixtures
- `src/core/builders.rs` - Test data builders
- `src/core/assertions.rs` - Assertion utilities
- `src/core/macros.rs` - Test macros
- `src/core/state.rs` - Type-level state
- `src/core/type_level.rs` - Type-level programming
- `src/core/const_assert.rs` - Compile-time assertions
- `src/core/alert.rs` - Alert helpers

### Testing Features
- `src/testing/property.rs` - Property-based testing
- `src/testing/mutation.rs` - Mutation testing
- `src/testing/snapshot.rs` - Snapshot testing
- `src/testing/concurrency.rs` - Concurrency testing
- `src/testing/cli.rs` - CLI testing
- `src/testing/generator.rs` - Test code generation
- `src/testing/parameterized.rs` - Parameterized testing

### Validation Features
- `src/validation/coverage.rs` - Coverage analysis
- `src/validation/guards.rs` - Guard constraints
- `src/validation/jtbd.rs` - JTBD validation
- `src/validation/performance.rs` - Performance validation

### Observability Features
- `src/observability/otel.rs` - OTEL validation
- `src/observability/weaver.rs` - Weaver validation

### Integration Features
- `src/integration/testcontainers.rs` - Docker containers

## Requirements

### Required
- Rust 1.70+ (1.75+ for `async` feature)
- Cargo
- cargo-make (for build system)

### Optional
- Docker (for `testcontainers` feature)
- Weaver binary (for `weaver` feature)

## Success Criteria

✅ All features demonstrated with working examples  
✅ All tests pass (100% success rate)  
✅ All features compile and run correctly  
✅ Playground serves as reference for end users  
✅ Clear examples and usage instructions  

## Copying Examples

All examples in this playground are designed to be copied and adapted for your projects. Each example demonstrates:

1. **Arrange**: Set up test data and context
2. **Act**: Execute the feature under test
3. **Assert**: Verify behavior and outputs

Follow the AAA pattern in all examples for consistency with Chicago TDD principles.

## Contributing

When adding new examples:

1. Follow the AAA pattern (Arrange-Act-Assert)
2. Include comprehensive comments
3. Add tests that validate the example works
4. Update this README with new examples
5. Ensure all features compile and tests pass

## License

MIT (same as chicago-tdd-tools)

