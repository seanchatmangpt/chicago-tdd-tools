# Chicago TDD Tools - Application Guide

Welcome to the practical application guide for Chicago TDD Tools! This guide focuses on **real-world usage patterns** and practical techniques for testing Rust applications.

## What You'll Learn

This guide covers:

- **Core Patterns**: Fixtures, data builders, assertions, and error path testing
- **Advanced Techniques**: Property-based testing, mutation testing, snapshot testing, CLI testing, and concurrency testing
- **Progressive Enhancement**: The "Go the Extra Mile" pattern for designing increasingly valuable solutions
- **Observability**: OTEL instrumentation and Weaver live-check validation
- **Real-World Applications**: Complete examples of testing CLI tools, web services, and integration scenarios
- **Best Practices**: Proven patterns and migration strategies

## Who This Guide Is For

- Developers writing Rust applications and want comprehensive testing
- Teams adopting Chicago-style (Classicist) TDD practices
- Engineers improving test quality and catching bugs earlier
- Anyone learning advanced testing techniques in Rust

## How to Use This Guide

1. **Start with the Introduction** to understand Chicago TDD principles
2. **Learn Core Patterns** for everyday testing scenarios
3. **Explore Advanced Techniques** for specific testing challenges
4. **Study Real-World Applications** to see complete examples
5. **Apply Best Practices** to improve your testing approach

## Quick Links

- [Pattern Cookbook](../cookbook/README.md) - Alexander-style patterns reference
- [API Reference](https://docs.rs/chicago-tdd-tools/) - Complete API documentation
- [GitHub Repository](https://github.com/seanchatmangpt/chicago-tdd-tools)

## Key Concepts

### Chicago-Style TDD (Classicist)

Chicago TDD emphasizes:
- **Type Safety**: Use Rust's type system to prevent errors at compile time
- **Real Dependencies**: Test with actual implementations, not mocks
- **Error Prevention**: Poka-yoke design prevents mistakes before they happen
- **Quality by Default**: Quality is the default, not an afterthought

### The 80/20 Principle

The "second idea" typically provides 80% more value with only 20% more effort. Learn when to:
- Keep it simple (1st idea)
- Apply the sweet spot (2nd idea)
- Go all-in (3rd idea)

## Getting Started

### Install Chicago TDD Tools

```toml
[dev-dependencies]
chicago-tdd-tools = { version = "1.3", features = ["testing-extras"] }
```

### Run Examples

```bash
# Run basic example
cargo run --example basic_test

# Run property-based testing
cargo run --example property_testing --features property-testing

# Run mutation testing
cargo run --example mutation_testing

# Run all examples
cargo run --example go_extra_mile
```

### Build This Guide

```bash
# Install mdbook if you haven't already
cargo install mdbook

# Build and serve the guide
cd application-guide
mdbook serve
```

Then visit `http://localhost:3000` in your browser.

## Example: Basic Test Structure

All tests follow the **AAA Pattern** (Arrange-Act-Assert):

```rust
use chicago_tdd_tools::prelude::*;

test!(my_test, {
    // Arrange: Set up test data
    let input = 5;

    // Act: Execute the code under test
    let result = input * 2;

    // Assert: Verify the result
    assert_eq!(result, 10);
});
```

## Table of Contents

1. [Introduction](introduction.md) - Chicago TDD philosophy and principles
2. [Core Testing Patterns](core/README.md) - Everyday testing with fixtures and builders
3. [Advanced Techniques](advanced/README.md) - Specialized testing for complex scenarios
4. [Go the Extra Mile](guides/extra-mile.md) - Progressive enhancement pattern
5. [Observability & Quality](guides/observability.md) - Telemetry and quality assurance
6. [Real-World Applications](guides/real-world.md) - Complete practical examples
7. [Best Practices](guides/best-practices.md) - Proven patterns and patterns to avoid

## Featured Examples

### From Examples Directory
- **basic_test.rs** - Getting started with fixtures and data builders
- **property_testing.rs** - Property-based testing with proptest
- **mutation_testing.rs** - Validating test quality
- **go_extra_mile.rs** - Progressive enhancement patterns
- **cli_testing.rs** - Testing command-line interfaces
- **snapshot_testing.rs** - Golden file testing
- **concurrency_testing.rs** - Thread-safe testing with loom

### From Playground
- **CLI tool** with comprehensive sub-commands
- **Integration tests** with Docker containers
- **Quality validation** with coverage and performance metrics
- **OTEL/Weaver** observability examples

## Next Steps

👉 **Start here**: [Introduction](introduction.md)

## Community & Support

- GitHub Issues: Report bugs or request features
- Discussions: Share ideas and patterns with the community
- Pattern Cookbook: Contribute Alexander-style patterns

---

**Chicago TDD Tools** - Testing with confidence, errors prevented at compile time.
