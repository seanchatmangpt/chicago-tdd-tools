# Running Feature Examples Tutorial

**Estimated time**: 20-30 minutes

Explore advanced testing techniques beyond core features: property-based testing, mutation testing, snapshot testing, concurrency testing, and more.

## What Are Feature Examples?

Feature examples demonstrate advanced testing capabilities that you enable with optional feature flags:

- **Property Testing** - Generate random test data to verify properties
- **Mutation Testing** - Check if tests catch intentional code changes
- **Snapshot Testing** - Compare complex outputs against stored snapshots
- **Concurrency Testing** - Detect race conditions deterministically
- **CLI Testing** - Test command-line tools like black boxes
- **Generator** - Generate test code automatically

## Prerequisites

- Complete [Getting Started Tutorial](getting-started.md)
- Complete [Running Core Examples Tutorial](running-core-examples.md)
- You're in the `playground` directory
- Optional: Docker for some integration tests

## Enabling Features

By default, the playground enables all features. To work with specific features:

```bash
# All features (default)
cargo run --all-features -- test stat

# Core features only
cargo run --no-default-features -- test stat

# Specific feature group
cargo run --features "property-testing" -- test stat
```

## Feature Group 1: Testing Features

### Property-Based Testing

Property testing generates random inputs to verify properties hold **for all inputs**.

#### Run it

```bash
cargo run -- test exec --names "prop"
```

#### What you'll see

```json
{
  "example": "property",
  "status": "success",
  "message": "Property-based testing finds edge cases",
  "details": {
    "properties_tested": 5,
    "cases_generated": 100,
    "edge_cases_found": 3,
    "assertions_passed": 500
  }
}
```

#### What it demonstrates

```rust
test!(test_addition_commutativity, {
    // Property: a + b == b + a (always true for all numbers)
    for i in 0..100 {
        let a = rand::random::<u32>();
        let b = rand::random::<u32>();

        // This property holds for ALL valid inputs
        assert_eq!(a + b, b + a, "Addition must be commutative");
    }
});
```

**Key learnings:**
- Properties are invariants that hold for all valid inputs
- Random generation finds edge cases automatically
- More thorough than manual test cases

### Mutation Testing

Mutation testing checks if tests catch intentional code changes (mutants).

#### Run it

```bash
cargo run -- test exec --names "mut"
```

#### What you'll see

```json
{
  "example": "mutation",
  "status": "success",
  "message": "Mutation testing validates test quality",
  "details": {
    "mutations_applied": 50,
    "mutations_caught": 48,
    "mutation_score": 96,
    "weak_tests": 2
  }
}
```

#### What it demonstrates

```rust
test!(test_mutation_catches_bugs, {
    // Arrange
    let mut data = HashMap::new();
    data.insert("key", "value");

    // Act: Apply mutation (hypothetical code change)
    // What if someone changed insert to remove?
    let mutated = data.remove("key");

    // Assert: Test MUST catch this change
    assert_eq!(mutated, Some("value"));
    assert!(data.is_empty());  // Catch removal
});
```

**Key learnings:**
- Mutation score shows test thoroughness
- 80%+ mutation score is excellent
- Helps find weak tests that don't verify behavior

### Snapshot Testing

Snapshot testing compares complex outputs against stored reference snapshots.

#### Run it

```bash
cargo run -- test exec --names "snap"
```

#### What you'll see

```json
{
  "example": "snapshot",
  "status": "success",
  "message": "Snapshots prevent unexpected output changes",
  "details": {
    "snapshots_verified": 8,
    "new_snapshots": 0,
    "matches": 8,
    "mismatches": 0
  }
}
```

#### What it demonstrates

```rust
test!(test_serialization_snapshot, {
    // Arrange
    let user = User {
        name: "Alice",
        email: "alice@example.com",
        created: 2025_11_15,
    };

    // Act: Serialize to JSON
    let json = serde_json::to_string_pretty(&user)?;

    // Assert: Compare against snapshot
    assert_snapshot!(json);
    // First run: creates snapshot file
    // Second run: compares against it
    // Change output? Update with: cargo make snapshot-accept
});
```

**Key learnings:**
- Perfect for JSON, HTML, binary comparisons
- Prevents accidental output changes
- Easy snapshot review workflow

### Concurrency Testing

Concurrency testing detects race conditions by running code in all possible thread interleavings.

#### Run it

```bash
cargo run -- test exec --names "conc"
```

#### What you'll see

```json
{
  "example": "concurrency",
  "status": "success",
  "message": "Concurrency testing catches race conditions",
  "details": {
    "thread_models": 100,
    "interleavings_checked": 10000,
    "race_conditions_found": 0,
    "safe": true
  }
}
```

#### What it demonstrates

```rust
test!(test_concurrent_access, {
    // Uses loom for deterministic thread checking
    loom::model(|| {
        let data = Arc::new(Mutex::new(0));
        let data_clone = data.clone();

        // Spawn thread
        let handle = loom::thread::spawn(move || {
            let mut guard = data_clone.lock().unwrap();
            *guard += 1;
        });

        // Loom exhaustively checks all interleavings
        handle.join().unwrap();
    });
});
```

**Key learnings:**
- Loom tests all possible thread interleavings
- Detects data races deterministically
- Essential for multi-threaded code

## Feature Group 2: Validation Features

### Coverage Analysis

Coverage shows which code is tested.

#### Run it

```bash
cargo run -- valid exec --names "cov"
```

#### What you'll see

```json
{
  "example": "coverage",
  "status": "success",
  "message": "Coverage analysis guides testing",
  "details": {
    "lines_covered": 245,
    "lines_total": 300,
    "coverage_percent": 81.7,
    "uncovered_lines": 55
  }
}
```

### Guard Constraints

Guards enforce compile-time constraints on code.

#### Run it

```bash
cargo run -- valid exec --names "guard"
```

#### What you'll see

```json
{
  "example": "guards",
  "status": "success",
  "message": "Guard constraints enforce maximum run length",
  "details": {
    "constraints_verified": 5,
    "max_run_length_verified": true,
    "max_batch_size_verified": true
  }
}
```

## Feature Group 3: Observability Features

### OTEL Validation (requires docker or `otel` feature)

Test OpenTelemetry instrumentation.

#### Check if available

```bash
cargo run -- obs stat
```

#### Run it (if available)

```bash
cargo run -- obs otel
```

## Combining Multiple Examples

Run a full testing suite:

```bash
cargo run -- test exec --names "prop mut snap conc"
```

### What you'll see

```json
[
  { "example": "property", "status": "success", ... },
  { "example": "mutation", "status": "success", ... },
  { "example": "snapshot", "status": "success", ... },
  { "example": "concurrency", "status": "success", ... }
]
```

## Recommended Learning Path

### Week 1: Learn Advanced Testing
1. **Day 1**: Property-based testing - Find edge cases automatically
2. **Day 2**: Mutation testing - Validate test quality
3. **Day 3**: Snapshot testing - Prevent output regressions
4. **Day 4**: Concurrency testing - Ensure thread safety
5. **Day 5**: Review and practice

### Week 2: Apply to Real Projects
1. Identify weak tests with mutation testing
2. Add property tests for mathematical operations
3. Use snapshots for serialization tests
4. Add concurrency tests for shared state

## Practice Exercises

### Exercise 1: Property Test for Your Code

Find a function in your project and write a property test:

```rust
test!(test_your_property, {
    for _ in 0..100 {
        let input = random_input();
        let result = your_function(input);
        assert!(your_property(result)); // Property must hold
    }
});
```

### Exercise 2: Mutation Test Weak Tests

Run mutation testing on your test suite to find weak tests:

```bash
# Check mutation score
cargo make test-mutation
```

Improve tests that don't catch mutations.

### Exercise 3: Add Snapshot Test

Add snapshot testing to serialization:

```rust
#[test]
fn test_output_format() {
    let data = your_function();
    assert_snapshot!(serde_json::to_string_pretty(&data)?);
}
```

## Next Steps

- **See all CLI commands** → [CLI Command Reference](../reference/cli-commands.md)
- **Copy examples to your project** → [Copying Examples Tutorial](copying-examples.md)
- **Understand testing philosophy** → [Testing Philosophy](../explanation/testing-philosophy.md)
- **See all available examples** → [Example Inventory](../reference/example-inventory.md)

## Summary

You've explored:
- ✅ Property-based testing for finding edge cases
- ✅ Mutation testing for validating test quality
- ✅ Snapshot testing for output regression prevention
- ✅ Concurrency testing for race condition detection
- ✅ Coverage analysis and guard constraints
- ✅ Observability and OTEL validation

---

**Next**: [Copying Examples](copying-examples.md) to use in your projects
