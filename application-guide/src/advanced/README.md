# Advanced Testing Techniques

Welcome to advanced testing techniques! These specialized methods help you test complex scenarios effectively.

## Overview

Advanced techniques include:

1. **Property-Based Testing** - Generate random data and verify properties
2. **Mutation Testing** - Validate test quality by introducing mutations
3. **Snapshot Testing** - Golden files to detect unintended changes
4. **CLI Testing** - Test command-line interfaces
5. **Concurrency Testing** - Deterministic thread testing

## When to Use Advanced Techniques

| Technique | Best For | When to Use |
|-----------|----------|------------|
| **Property-Based** | Mathematical properties, edge cases | Complex algorithms, parsing |
| **Mutation** | Test quality validation | Assessing test effectiveness |
| **Snapshot** | Stable output, complex structures | API responses, generated code |
| **CLI** | Command-line tools | CLI applications, scripts |
| **Concurrency** | Thread-safe code | Concurrent systems, race conditions |

## Quick Reference

### Property-Based Testing

```rust
// Test that addition is commutative
test!(test_addition_commutative, {
    let strategy = ProptestStrategy::new().with_cases(100);
    strategy.test(any::<(u32, u32)>(), |(a, b)| a + b == b + a);
});
```

### Mutation Testing

```rust
// Verify tests catch mutations
let mut tester = MutationTester::new(data);
tester.apply_mutation(MutationOperator::ChangeValue(...));
let caught = tester.test_mutation_detection(|data| check_data(data));
assert!(caught);  // Tests should catch the mutation
```

### Snapshot Testing

```rust
// Golden file testing
let output = generate_report();
assert_matches!(output, "report");  // Compares with snapshot
```

### CLI Testing

```rust
// Test CLI commands
let output = CliTest::new("myapp", vec!["list", "--verbose"])
    .run()?;
assert!(output.contains("item1"));
```

### Concurrency Testing

```rust
// Deterministic thread testing
loom::model(|| {
    let data = Arc::new(Mutex::new(0));
    thread::spawn({
        let data = data.clone();
        move || *data.lock().unwrap() += 1;
    });
});
```

## Learning Path

1. **Start with** [Property-Based Testing](property-testing.md) - Easy to understand, powerful
2. **Then explore** [Mutation Testing](mutation-testing.md) - Validates your tests
3. **Add** [Snapshot Testing](snapshot-testing.md) - For regression detection
4. **Test CLIs with** [CLI Testing](cli-testing.md) - If you have CLI tools
5. **Thread-safe code with** [Concurrency Testing](concurrency-testing.md) - For concurrent systems

## Combining Techniques

You can combine advanced techniques:

```rust
test!(comprehensive_test, {
    // Use fixtures and data builders (core)
    let fixture = TestFixture::new()?;
    let data = TestDataBuilder::new()...build_json()?;

    // Use property-based testing (advanced)
    let strategy = ProptestStrategy::new().with_cases(50);

    // Use mutation testing (advanced)
    let mut tester = MutationTester::new(data.clone());

    // Use snapshot testing (advanced)
    let result = process(&data)?;
    assert_matches!(result, "expected_output");
});
```

## Real-World Scenarios

### Scenario 1: JSON Parser

- **Core**: Basic tests with fixtures
- **Property-Based**: Test parsing properties (round-trip)
- **Mutation**: Validate test quality
- **Snapshot**: Compare against golden files

### Scenario 2: CLI Tool

- **Core**: Basic command tests
- **CLI Testing**: Full CLI integration
- **Snapshot**: Compare output with golden files
- **Property-Based**: Random argument generation

### Scenario 3: Concurrent System

- **Core**: Basic thread tests
- **Concurrency**: Deterministic testing with loom
- **Property-Based**: Test invariants across threads
- **Mutation**: Validate synchronization correctness

## Feature Flags

Enable features for advanced techniques:

```toml
[dev-dependencies]
chicago-tdd-tools = { version = "1.3", features = [
    "property-testing",      # Property-based testing
    "mutation-testing",      # Mutation testing
    "snapshot-testing",      # Snapshot testing
    "cli-testing",           # CLI testing
    "concurrency-testing",   # Concurrency testing
    "testing-extras",        # All of above (most common)
    "testing-full",          # All testing features
] }
```

## Performance Considerations

Advanced techniques can be slower:

| Technique | Speed | Trade-off |
|-----------|-------|-----------|
| Unit tests | Fast (ms) | Limited scenarios |
| Property-based | Medium (seconds) | Comprehensive coverage |
| Mutation | Slow (minutes) | High confidence |
| Snapshot | Fast (ms) | Brittle to changes |
| Concurrency | Slow (seconds) | Deterministic |

**Recommendation**:
- Use core patterns for 80% of tests (fast feedback)
- Use advanced techniques for critical paths (high confidence)

## Common Pitfalls

❌ **Over-using advanced techniques**
- Use property-based for properties, not all tests
- Use mutation occasionally, not always

❌ **Ignoring performance**
- Property-based with 10,000 cases is overkill
- Limit mutation test scope

❌ **Replacing core patterns**
- Advanced techniques complement, not replace, core patterns
- Still need AAA pattern and error paths

✅ **Best practices**:
- Use core patterns as foundation
- Add advanced techniques strategically
- Balance confidence with speed

## Sections

- [Property-Based Testing](property-testing.md)
- [Mutation Testing](mutation-testing.md)
- [Snapshot Testing](snapshot-testing.md)
- [CLI Testing](cli-testing.md)
- [Concurrency Testing](concurrency-testing.md)

## Next Steps

👉 **Start with [Property-Based Testing](property-testing.md)**

