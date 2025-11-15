# Chicago TDD Tools - Examples

**Version:** 1.2.0 | **Updated:** 2025-11-15

This directory contains comprehensive examples demonstrating Chicago TDD Tools usage, organized using the [Diátaxis framework](https://diataxis.fr/) for technical documentation.

---

## Quick Navigation

| Category | Description | Examples |
|----------|-------------|----------|
| **[Tutorials](#tutorials)** | Step-by-step learning | Basic Test, Macro Examples |
| **[How-To Guides](#how-to-guides)** | Task-oriented solutions | Property Testing, Snapshot Testing, Mutation Testing, Concurrency Testing, CLI Testing, Testcontainers, OTEL/Weaver Testing |
| **[Explanation](#explanation)** | Concepts and philosophy | Go the Extra Mile, Advanced Features |
| **[Reference](#reference)** | API documentation | All examples include reference sections |

---

## Diátaxis Framework

This documentation follows the Diátaxis systematic approach to technical documentation:

- **Tutorials**: Learning-oriented guides for getting started
- **How-To Guides**: Task-oriented guides for solving specific problems
- **Explanation**: Understanding-oriented background and concepts
- **Reference**: Information-oriented technical descriptions

---

## Tutorials

Tutorials are learning-oriented guides that take you through a series of steps to complete a project. They help you learn by doing.

### 1. Basic Test (`basic_test.rs`)

**Start here if you're new to Chicago TDD Tools.**

Learn the fundamental patterns of Chicago TDD through hands-on examples:
- Creating test fixtures
- Building test data with fluent builders
- Handling errors properly in tests
- Following the AAA (Arrange-Act-Assert) pattern

**Run:**
```bash
cargo run --example basic_test
```

**📖 Full Documentation:** [basic_test.md](basic_test.md)

**What you'll learn:**
- TestFixture creation and usage
- TestDataBuilder patterns
- Error handling in tests
- AAA pattern enforcement

---

### 2. Macro Examples (`macro_examples.rs`)

Learn how to use Chicago TDD macros for writing concise, readable tests.

**Run:**
```bash
cargo test --example macro_examples
```

**📖 Full Documentation:** [macro_examples.md](macro_examples.md)

**What you'll learn:**
- `test!` macro for synchronous tests
- `assert_ok!` and `assert_err!` for Result handling
- Custom assertion messages
- Available test and assertion macros

**Key Macros:**
- `test!` - Synchronous test
- `async_test!` - Async test
- `fixture_test!` - Test with fixture
- `assert_ok!` - Assert Result is Ok
- `assert_err!` - Assert Result is Err

---

## How-To Guides

How-to guides are task-oriented and show you how to solve specific problems. They assume you understand the basics.

### 3. Property-Based Testing (`property_testing.rs`)

How to use property-based testing to verify properties hold for all inputs.

**Run:**
```bash
cargo run --example property_testing --features property-testing
```

**📖 Full Documentation:** [property_testing.md](property_testing.md)

**Use this when:**
- Testing mathematical properties (commutativity, distributivity)
- Finding edge cases automatically
- Testing with random data generation

**Features:**
- `PropertyTestGenerator` (original, backward compatible)
- `ProptestStrategy` (enhanced with proptest crate)
- Random test data generation
- Property shrinking (finding minimal failing cases)

---

### 4. Snapshot Testing (`snapshot_testing.rs`)

How to use snapshot testing for complex data structures and output stability.

**Run:**
```bash
cargo test --features snapshot-testing --example snapshot_testing
```

**📖 Full Documentation:** [snapshot_testing.md](snapshot_testing.md)

**Use this when:**
- Testing complex data structures
- Validating API responses
- Testing generated code
- Verifying configuration files

**Features:**
- String snapshots
- JSON snapshots
- Debug representation snapshots
- Custom snapshot settings
- Snapshot review workflow

**Workflow:**
```bash
cargo test                     # Create/verify snapshots
cargo insta review            # Review changes
cargo insta accept            # Accept changes
cargo insta reject            # Reject changes
```

---

### 5. Mutation Testing (`mutation_testing.rs`)

How to validate test quality by introducing mutations to code.

**Run:**
```bash
cargo run --example mutation_testing
```

**📖 Full Documentation:** [mutation_testing.md](mutation_testing.md)

**Use this when:**
- Measuring test quality
- Finding weak tests
- Ensuring tests verify behavior

**Features:**
- Mutation operators (RemoveKey, AddKey, ChangeValue, NegateCondition)
- Mutation score calculation
- Test detection verification

**Target:** >= 80% mutation score

---

### 6. Concurrency Testing (`concurrency_testing.rs`)

How to test concurrent code with model checking using loom.

**Run:**
```bash
cargo test --features concurrency-testing --example concurrency_testing
```

**📖 Full Documentation:** [concurrency_testing.md](concurrency_testing.md)

**Use this when:**
- Testing concurrent operations
- Finding race conditions
- Verifying thread safety

**Features:**
- Model checking (explores all thread interleavings)
- Concurrent counter testing
- Concurrent vector operations
- Custom configuration (threads, preemptions)

---

### 7. CLI Testing (`cli_testing.rs`)

How to test command-line interfaces using golden files.

**Run:**
```bash
cargo test --features cli-testing --example cli_testing
```

**📖 Full Documentation:** [cli_testing.md](cli_testing.md)

**Use this when:**
- Testing CLI applications
- Verifying command output
- Testing environment variables

**Features:**
- `CliCommandBuilder` - Fluent command building
- `CliAssertions` - Output verification
- `CliEnvironment` - Environment variable management
- Golden file testing (`.trycmd` files)

---

### 8. Testcontainers (`testcontainers_example.rs`)

How to use Docker containers in integration tests with automatic lifecycle management.

**Run:**
```bash
cargo run --example testcontainers_example --features testcontainers
```

**📖 Full Documentation:** [testcontainers_example.md](testcontainers_example.md)

**Use this when:**
- Testing with real databases (PostgreSQL, MySQL)
- Testing with message queues (Redis, RabbitMQ)
- Testing with external services
- Integration testing requiring Docker

**Features:**
- Basic containers
- Port mapping
- Environment variables
- Command execution
- Entrypoint override
- Wait conditions

**Container Types:**
- Basic containers (exit immediately)
- Service containers (stay running)
- Command containers (custom commands)

---

### 9. OTEL/Weaver Testing (`otel_weaver_testing.rs`)

How to validate observability telemetry with OpenTelemetry and Weaver.

**Run:**
```bash
cargo test --features otel,weaver --example otel_weaver_testing
```

**📖 Full Documentation:** [otel_weaver_testing.md](otel_weaver_testing.md)

**Use this when:**
- Validating OTEL spans and metrics
- Testing telemetry instrumentation
- Ensuring semantic convention compliance
- Integration testing with observability

**Features:**
- OTEL span validation
- OTEL metric validation
- Weaver live-check integration
- Unified observability API
- Custom configuration (ports, registry paths)

---

## Explanation

Explanation guides help you understand concepts, background, and design decisions.

### 10. Go the Extra Mile (`go_extra_mile.rs`)

**Philosophy:** 1st/2nd/3rd Idea Progression

Demonstrates the "go the extra mile" paradigm with progressive enhancement:

**Run:**
```bash
cargo run --example go_extra_mile --features otel,weaver
```

**📖 Full Documentation:** [go_extra_mile.md](go_extra_mile.md)

**Concepts:**
- **1st Idea**: Solve the immediate problem (narrow scope)
- **2nd Idea**: Go bigger (80/20 sweet spot - 80% more value with 20% effort)
- **3rd Idea**: Maximum value (type-safe, prevents entire classes of errors)

**Example: Number Parsing**
1. Parse u32 only
2. Generic parser for all number types
3. Type-level validated numbers with OTEL + Weaver

**Decision Framework:**
- 1st Idea: Works, but narrow scope
- 2nd Idea: **Usually best** - 80% more value, reasonable effort
- 3rd Idea: Maximum value, but evaluate effort vs. benefit

---

### 11. Advanced Features (`advanced_features.rs`)

**Concepts:** Advanced Rust features for zero-cost abstractions

Demonstrates hyper-advanced Rust features used in Chicago TDD Tools:

**Run:**
```bash
cargo run --example advanced_features
```

**📖 Full Documentation:** [advanced_features.md](advanced_features.md)

**Concepts:**
- **Type-Level Arithmetic**: Const generics for compile-time size validation
- **Type State Pattern**: Enforce test phase ordering at compile time
- **Async Traits**: Async methods in traits (Rust 1.75+)
- **Sealed Traits**: Prevent external implementations
- **Zero-Cost Abstractions**: No runtime overhead
- **Compile-Time Guarantees**: Errors caught before code runs

**AAA Pattern Enforcement:**
- `TestState<Arrange>` → `TestState<Act>` → `TestState<Assert>`
- Type system prevents calling methods in wrong order
- Compiler catches invalid state transitions

---

## Reference

All examples include comprehensive reference sections documenting:
- Key types and functions
- Parameters and return values
- Error conditions
- Usage examples

**Quick Reference:**

| Feature | Example | Command |
|---------|---------|---------|
| Basic Testing | `basic_test.rs` | `cargo run --example basic_test` |
| Macros | `macro_examples.rs` | `cargo test --example macro_examples` |
| Property Testing | `property_testing.rs` | `cargo run --example property_testing --features property-testing` |
| Snapshot Testing | `snapshot_testing.rs` | `cargo test --features snapshot-testing --example snapshot_testing` |
| Mutation Testing | `mutation_testing.rs` | `cargo run --example mutation_testing` |
| Concurrency Testing | `concurrency_testing.rs` | `cargo test --features concurrency-testing --example concurrency_testing` |
| CLI Testing | `cli_testing.rs` | `cargo test --features cli-testing --example cli_testing` |
| Testcontainers | `testcontainers_example.rs` | `cargo run --example testcontainers_example --features testcontainers` |
| OTEL/Weaver | `otel_weaver_testing.rs` | `cargo test --features otel,weaver --example otel_weaver_testing` |
| Go Extra Mile | `go_extra_mile.rs` | `cargo run --example go_extra_mile --features otel,weaver` |
| Advanced Features | `advanced_features.rs` | `cargo run --example advanced_features` |

---

## Running Examples

### Run All Examples

```bash
# Run all runnable examples
cargo make test-examples

# Or manually:
for example in basic_test advanced_features property_testing mutation_testing go_extra_mile testcontainers_example; do
    cargo run --example $example --all-features
done
```

### Run Specific Example

```bash
# Basic examples (no features required)
cargo run --example basic_test
cargo run --example advanced_features
cargo run --example macro_examples
cargo run --example mutation_testing

# Examples with feature requirements
cargo run --example property_testing --features property-testing
cargo test --features snapshot-testing --example snapshot_testing
cargo test --features concurrency-testing --example concurrency_testing
cargo test --features cli-testing --example cli_testing
cargo run --example testcontainers_example --features testcontainers
cargo test --features otel,weaver --example otel_weaver_testing
cargo run --example go_extra_mile --features otel,weaver
```

### Run with All Features

```bash
cargo run --example basic_test --all-features
```

---

## Feature Flags

Examples require different feature flags:

| Feature | Examples | Description |
|---------|----------|-------------|
| None | `basic_test`, `advanced_features`, `macro_examples`, `mutation_testing` | Core functionality |
| `property-testing` | `property_testing` | Property-based testing with proptest |
| `snapshot-testing` | `snapshot_testing` | Snapshot testing with insta |
| `concurrency-testing` | `concurrency_testing` | Concurrency testing with loom |
| `cli-testing` | `cli_testing` | CLI testing with trycmd |
| `testcontainers` | `testcontainers_example` | Docker container support |
| `otel` | `otel_weaver_testing`, `go_extra_mile` | OpenTelemetry validation |
| `weaver` | `otel_weaver_testing`, `go_extra_mile` | Weaver semantic convention validation |
| `async` | (Used in tests) | Async fixture support |

**Bundles:**
- `testing-extras`: `property-testing` + `snapshot-testing` + `fake-data` (most common)
- `testing-full`: All testing features
- `observability-full`: `otel` + `weaver`
- `integration-full`: `testcontainers` + all testing features

---

## Learning Path

### Beginner Path

1. **Start**: `basic_test.rs` - Learn core concepts
2. **Macros**: `macro_examples.rs` - Learn macro usage
3. **Advanced**: Pick examples relevant to your use case

### By Use Case

**Unit Testing:**
1. `basic_test.rs` - Core patterns
2. `macro_examples.rs` - Test macros
3. `property_testing.rs` - Property-based testing

**Integration Testing:**
1. `testcontainers_example.rs` - Docker containers
2. `otel_weaver_testing.rs` - Observability validation

**Advanced Testing:**
1. `mutation_testing.rs` - Test quality validation
2. `concurrency_testing.rs` - Thread safety verification
3. `snapshot_testing.rs` - Output stability

**CLI Applications:**
1. `cli_testing.rs` - CLI testing patterns

**Philosophy and Concepts:**
1. `go_extra_mile.rs` - Progressive enhancement
2. `advanced_features.rs` - Advanced Rust features

---

## Example Structure

All examples follow a consistent structure:

```rust
//! # Example Title - Comprehensive Guide
//!
//! Brief description of what this example demonstrates.
//!
//! ## Tutorial: Getting Started
//!
//! Step-by-step walkthrough of key concepts.
//!
//! ## Explanation: Concepts
//!
//! Background and conceptual understanding.
//!
//! ## How-to: Common Tasks
//!
//! Task-oriented guidance for specific problems.
//!
//! ## Reference: Quick Lookup
//!
//! API documentation and technical details.

// Example code with detailed comments
```

---

## Best Practices

### Running Examples

1. **Read the documentation first**: Each example has comprehensive inline documentation
2. **Run examples to see output**: Examples print helpful output explaining what's happening
3. **Examine the code**: Examples are heavily commented with best practices
4. **Run tests**: Many examples have test modules demonstrating usage

### Using Examples in Your Code

1. **Copy patterns, not code**: Examples demonstrate patterns - adapt to your use case
2. **Follow AAA pattern**: All examples use Arrange-Act-Assert structure
3. **Handle errors properly**: Examples show proper error handling with `?` and `match`
4. **Use macros**: Examples demonstrate macro usage for concise tests

### Feature Flags

1. **Start minimal**: Use core features first, add advanced features as needed
2. **Use bundles**: `testing-extras` covers most common use cases
3. **Check requirements**: Some examples require Docker (testcontainers) or external tools (Weaver)

---

## Common Patterns

### AAA Pattern

All examples follow Arrange-Act-Assert:

```rust
test!(my_test, {
    // Arrange: Set up test data and fixtures
    let input = 5;
    let expected = 10;

    // Act: Execute code under test
    let result = input * 2;

    // Assert: Verify expected behavior
    assert_eq!(result, expected);
});
```

### Error Handling

Examples demonstrate proper error handling:

```rust
// Use ? operator to propagate errors
let value = result?;

// Or use match for explicit handling
match result {
    Ok(value) => { /* handle success */ },
    Err(e) => { /* handle error */ },
}
```

### Fixture Usage

Examples show fixture patterns:

```rust
fixture_test!(my_test, fixture, {
    // fixture is automatically created and cleaned up
    let data = fixture.test_data();
    // ... test code ...
});
```

---

## Troubleshooting

### Example Won't Run

**Error**: Feature not enabled

**Solution**: Check feature requirements and enable with `--features`:
```bash
cargo run --example property_testing --features property-testing
```

### Test Examples Fail

**Error**: Docker not running (testcontainers)

**Solution**: Start Docker:
```bash
docker --version  # Verify Docker is installed
docker info       # Verify Docker is running
```

**Error**: Weaver not available

**Solution**: Bootstrap Weaver:
```bash
cargo make weaver-bootstrap
```

### Compilation Errors

**Error**: Unknown macro or type

**Solution**: Ensure you're using the prelude:
```rust
use chicago_tdd_tools::prelude::*;
```

---

## Related Documentation

- **Getting Started**: `docs/getting-started/GETTING_STARTED.md`
- **API Reference**: `docs/reference/API_REFERENCE.md`
- **Architecture**: `docs/reference/ARCHITECTURE.md`
- **Cookbook**: `cookbook/src/` - Pattern language
- **Process**: `docs/process/SPR_GUIDE.md`, `FMEA_TESTS_BUILD_ACTIONS.md`

---

## Contributing

When adding new examples:

1. **Follow Diátaxis framework**: Categorize as Tutorial, How-To, Explanation, or Reference
2. **Include all sections**: Tutorial, Explanation, How-To, Reference
3. **Add comprehensive comments**: Explain what, why, and how
4. **Follow AAA pattern**: All tests use Arrange-Act-Assert
5. **Update this README**: Add entry to appropriate section
6. **Test thoroughly**: Ensure example runs and tests pass

---

## Feedback

Found an issue or have a suggestion? Please [open an issue](https://github.com/seanchatmangpt/chicago-tdd-tools/issues).

---

**Quality is the default. Prevention beats detection.**

*Version 1.2.0 | Updated 2025-11-15 | Team KNHK | License MIT*
