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

### CLI Commands

The playground provides a unified CLI using the `playg` command with noun-verb patterns:

```bash
# Show help
playg --help

# Show status of all core features
playg core stat

# List available core examples
playg core list

# Execute one or more examples
playg core exec --names "fixtures"
playg core exec --names "fixtures builders assert"

# Show testing features status
playg test stat

# List available test examples
playg test list

# Execute test examples
playg test exec --names "gen"

# Show validation features status
playg valid stat

# Execute validation checks
playg valid exec --names "cov guard"

# Show observability features
playg obs stat

# Run OTEL demo (if otel feature enabled)
playg obs otel

# Run Weaver demo (if weaver feature enabled)
playg obs weav

# Show integration features
playg integ stat

# Run testcontainers demo (if testcontainers feature enabled)
playg integ contain
```

All commands return JSON output by default, making them suitable for scripting and automation.

### Running Examples (Legacy)

```bash
# Run all examples (legacy method)
cargo run --bin playground

# Run specific example module (legacy method)
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


## Chatman Engine validation (feature: chatman)

Feature-gated validation suite exercising the Chatman Engine v26.7.9
(`praxis-graphlaw`'s `chatman` module, workflow wf_255e0807) through every
chicago-tdd-tools capability. Off by default: the default playground
build/test is unaffected (the `chatman` test targets use
`required-features = ["chatman"]` and are never compiled without the flag).

```bash
cargo test --features chatman
```

Note: this suite compiles only once praxis's `chatman` module lands/compiles
(workflow wf_255e0807); the `praxis-graphlaw` path dependency is optional and
pulled in solely by the `chatman` feature.

### Capability -> test-file map

| Capability | Test file |
|---|---|
| AAA macros (`test!`, `fixture_test!`, `async_test!`) | `tests/chatman_01_macros_aaa.rs` |
| Assertion family (`assert_ok/err/fail/matches/contains/subset/eq_msg!`) | `tests/chatman_02_assertions.rs` |
| Builders + fixtures (`GenericTestDataBuilder`, `FixtureProvider`, `ScopedMetadata`) | `tests/chatman_03_builders_fixtures.rs` |
| Property testing (`ProptestStrategy`, pinned BLAKE3-derived seeds) | `tests/chatman_04_property.rs` |
| Parameterized testing (`param_test!` boundary/routing matrices) | `tests/chatman_05_parameterized.rs` |
| Snapshot testing (`SnapshotAssert`: 29-name Refusal contract, envelope JSON) | `tests/chatman_06_snapshot.rs` |
| Receipt chains (`ReceiptChainBuilder`, `Blake3ChainValidator`) | `tests/chatman_07_receipts_chain.rs` |
| Typestate + model checking (`StateMachine` S1-S6, `ModelChecker`) | `tests/chatman_08_typestate_state_machine.rs` |
| Mutation testing (`MutationTester`, `MutationScore`) | `tests/chatman_09_mutation_score.rs` |
| Performance (`performance_test!`, `TickCounter`; Chatman Constant <= 8 ticks, lenient reference gate) | `tests/chatman_10_performance.rs` |

Shared helpers: `tests/chatman_common/mod.rs` (deterministic envelope/seed
builders, refusal exemplars — no production logic).

Framework quirk: `SnapshotAssert` invokes insta inside chicago-tdd-tools
itself, so accepted snapshots are stored in the framework repo at
`../src/testing/snapshots/chicago_tdd_tools__testing__snapshot__chatman_*.snap`,
not under `playground/tests/snapshots/`.
