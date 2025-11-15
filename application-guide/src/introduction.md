# Introduction: Chicago TDD Philosophy

## What is Chicago-Style TDD?

Chicago-style TDD (also called "Classicist" TDD) is a testing approach that emphasizes:

1. **Type Safety First** - Use Rust's type system to prevent entire categories of errors
2. **Real Dependencies** - Test with actual implementations, not mocks
3. **Error Prevention** - Mistakes are prevented at compile time, not caught at runtime
4. **Quality by Default** - Quality is the default state; prevention beats detection

This contrasts with London-style TDD, which uses mocks and message-passing verification.

## Core Principles

### 1. Poka-Yoke Design

"Poka-yoke" (fool-proofing) means designing systems to prevent mistakes before they happen.

**Example**: Instead of writing a test to catch `panic!()` usage, the compiler enforces that production code never panics:

```rust
// ❌ Compiler error (blocked by clippy `unwrap_used` deny)
let value = result.unwrap();

// ✅ Compiler accepts - proper error handling
let value = result?;
```

### 2. Type-Level Correctness

Use Rust's type system as the primary design tool. If it compiles, correctness follows.

**Example**: A validated number type that proves its value is valid:

```rust
pub struct ValidatedNumber<T> {
    value: T,  // Can only be constructed via parse()
}

impl<T: FromStr> ValidatedNumber<T> {
    pub fn parse(input: &str) -> Result<Self, String> {
        // Validation logic here
        input.parse().map(|value| Self { value })
    }
}
```

### 3. Real Collaborators

Test with actual dependencies, not mocks. This catches integration bugs and makes refactoring safer.

**Why?** Mocks can hide bugs at integration boundaries. Real implementations reveal actual behavior.

```rust
// Chicago-style: Use the real implementation
let fixture = TestFixture::new()?;  // Real setup
let result = actual_function(&fixture)?;  // Real code

// London-style: Would use a mock instead
// let fixture = MockFixture::new();
```

### 4. The 80/20 Principle

When designing solutions, consider three ideas:

- **1st Idea**: Solve the immediate problem (narrow scope, simple)
- **2nd Idea**: 80% more value with 20% more effort (sweet spot, usually best)
- **3rd Idea**: Maximum value, but evaluate carefully (maximum scope, most complex)

Example: Number parsing

- 1st Idea: Parse `u32` only
- 2nd Idea: Generic parser works for all number types
- 3rd Idea: Type-validated parser with OTEL instrumentation and Weaver validation

Choose the 2nd idea most of the time. Only go to 3rd idea when type safety is critical.

## Why Chicago TDD for Rust?

Rust's type system makes Chicago-style TDD especially powerful:

| Aspect | Benefit |
|--------|---------|
| **Compile-time guarantees** | Many bugs prevented before testing |
| **Ownership system** | Resource cleanup guaranteed |
| **Result type** | Explicit error handling (no surprises) |
| **Trait system** | Generic code with real implementations |
| **Macros** | Test framework enforcement at compile time |

## Common Misconceptions

### "Mocks are Required"

❌ **False** in Chicago TDD. Use real dependencies when possible.

Use mocks only for:
- External services (APIs, databases)
- Expensive operations (file I/O, network)
- Non-deterministic behavior (time, random numbers)

### "Tests Should Only Test Public APIs"

❌ **Partially true**. Chicago TDD tests the behavior, not the interface.

Test internal functions if they:
- Have complex logic
- Are hard to test through public APIs
- Need boundary condition verification

### "100% Coverage Means Bug-Free Code"

❌ **False**. Coverage measures code execution, not correctness.

Focus on:
- Error paths (the real bugs are here)
- Boundary conditions
- State transitions

**Example**: 100% coverage of a sorting function doesn't guarantee it handles duplicates correctly.

## Testing with Chicago TDD Tools

Chicago TDD Tools provides:

- **Type-level AAA enforcement**: Arrange-Act-Assert structure at compile time
- **Data builders**: Fluent API for complex test data
- **Fixture management**: Test isolation and cleanup
- **Assertion helpers**: Clear, readable assertions
- **Advanced techniques**: Property-based, mutation, snapshot testing
- **Observability**: OTEL and Weaver integration

All with zero-cost abstractions and compile-time error prevention.

## The Testing Pyramid

Chicago TDD follows a testing pyramid:

```
        △ (Few)
       /|\
      / | \    E2E Tests (slow, fragile)
     /  |  \
    /____|____\
    |    |    |    Integration Tests (medium speed)
    |____|____|
    |         |    Unit Tests (fast, many)
    |_________|
    ▽ (Many)
```

**Unit Tests** (bottom): Fast, isolated, test one function

**Integration Tests** (middle): Medium speed, test multiple components together

**E2E Tests** (top): Slow, test the entire system

Chicago TDD emphasizes unit tests and integration tests. E2E tests are less important if unit+integration tests are comprehensive.

## AAA Pattern

Every test follows Arrange-Act-Assert:

```rust
test!(test_parsing, {
    // Arrange: Set up test data
    let input = "42";

    // Act: Execute the code under test
    let result = input.parse::<u32>();

    // Assert: Verify the result
    assert_ok!(&result);
    assert_eq!(result.unwrap(), 42);
});
```

This pattern ensures tests are:
- **Clear**: Anyone can see what's being tested
- **Complete**: Setup, execution, and verification are separate
- **Correct**: Focused on one behavior per test

## Test Organization

### Parallel Tests

By default, tests run in parallel. Ensure each test is independent:

```rust
test!(test1, {
    // Use unique data: TestFixture::new()?
    // Don't rely on global state
    // Don't use file/network resources shared with other tests
});
```

### Fixture Isolation

Each fixture is independent:

```rust
test!(test_with_fixture, {
    let fixture1 = TestFixture::new()?;
    let fixture2 = TestFixture::new()?;
    // fixture1 and fixture2 are completely independent
    // Automatic cleanup when they drop
});
```

## Advanced Topics Preview

Chicago TDD Tools provides techniques for:

1. **Property-Based Testing**: Generate random test data and verify properties hold
2. **Mutation Testing**: Validate test quality by introducing mutations
3. **Snapshot Testing**: Golden files to detect unintended changes
4. **CLI Testing**: Test command-line interfaces with trycmd
5. **Concurrency Testing**: Deterministic thread testing with loom
6. **Observability**: OTEL instrumentation and Weaver validation

We'll explore all of these in later sections.

## Getting Started

The best way to learn is by doing. Here's the recommended learning path:

1. **[Core Patterns](core/README.md)** - Master fixtures, data builders, and assertions
2. **[Error Path Testing](core/error-paths.md)** - Learn where bugs hide
3. **[Advanced Techniques](advanced/README.md)** - Pick techniques for your use case
4. **[Real-World Applications](guides/real-world.md)** - See complete examples
5. **[Best Practices](guides/best-practices.md)** - Avoid common pitfalls

## Key Takeaways

✅ Chicago TDD emphasizes **type safety** and **error prevention**

✅ Use **real dependencies**, not mocks

✅ Follow the **80/20 principle** when designing solutions

✅ Every test follows **Arrange-Act-Assert**

✅ Focus on **error paths** (where bugs hide)

❌ Don't rely on 100% coverage (it's not enough)

❌ Don't use mocks for everything

❌ Don't skip boundary condition testing

---

**Next**: [Core Testing Patterns](core/README.md)
