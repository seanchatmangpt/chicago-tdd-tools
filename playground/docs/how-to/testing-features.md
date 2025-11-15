# How to Run Testing Feature Examples

**Quick reference** for advanced testing: property-based, mutation, snapshot, concurrency, CLI testing.

## Quick Commands

```bash
# Show all testing features
cargo run -- test stat

# List available test examples
cargo run -- test list

# Run specific examples
cargo run -- test exec --names "prop"
cargo run -- test exec --names "mut snap conc"

# Run all testing examples
cargo run -- test exec --names "prop mut snap conc cli gen"
```

## Testing Features Overview

| Feature | Purpose | When to Use |
|---------|---------|------------|
| **Property** | Random test generation | Find edge cases automatically |
| **Mutation** | Test quality validation | Verify tests catch bugs |
| **Snapshot** | Output regression prevention | JSON, HTML, serialization |
| **Concurrency** | Race condition detection | Multi-threaded code |
| **CLI** | Command-line tool testing | Test executables |
| **Generator** | Test code generation | Generate tests automatically |

## Property-Based Testing

```bash
cargo run -- test exec --names "prop"
```

**What it does:**
Generates hundreds of random inputs to verify properties hold **for all inputs**.

**Use when:**
- Mathematical operations (addition, multiplication)
- Properties that should hold universally
- Finding edge cases is hard

**Example property:**
```rust
test!(test_addition_commutative, {
    for _ in 0..100 {
        let a = rand::random::<u32>();
        let b = rand::random::<u32>();
        assert_eq!(a + b, b + a);  // Must hold for ALL inputs
    }
});
```

**Key benefits:**
- Finds edge cases automatically
- More thorough than manual tests
- Documents invariants clearly

## Mutation Testing

```bash
cargo run -- test exec --names "mut"
```

**What it does:**
Applies intentional code changes (mutations) and verifies tests catch them.

**Use when:**
- Want to know if tests are thorough
- Need mutation score (aim for 80%+)
- Finding weak tests

**Example:**
```rust
test!(test_mutation_score, {
    // Arrange test data
    let mut data = vec![1, 2, 3];

    // Act - what if someone changed push to pop?
    data.push(4);

    // Assert must catch the change
    assert_eq!(data.len(), 4);
});
```

**Key metrics:**
- Mutation score ≥ 80% = Excellent
- Mutation score 60-79% = Good
- Mutation score < 60% = Tests need strengthening

## Snapshot Testing

```bash
cargo run -- test exec --names "snap"
```

**What it does:**
Compares complex outputs (JSON, HTML) against stored reference snapshots.

**Use when:**
- Testing JSON/XML serialization
- HTML rendering
- API response formats
- Binary format changes

**Example:**
```rust
test!(test_api_response, {
    let response = api_call();
    assert_snapshot!(serde_json::to_string_pretty(&response)?);
});
```

**Workflow:**
1. First run: Creates snapshot file
2. Second run: Compares against it
3. Output changes? Review and update:
   ```bash
   cargo make snapshot-review
   cargo make snapshot-accept
   ```

**Key benefits:**
- Prevents unexpected output changes
- Easy to review with diffs
- Documents expected behavior

## Concurrency Testing

```bash
cargo run -- test exec --names "conc"
```

**What it does:**
Uses loom to test code in all possible thread interleavings.

**Use when:**
- Multi-threaded code
- Shared mutable state
- Detecting data races
- Lock ordering issues

**Example:**
```rust
test!(test_concurrent_access, {
    loom::model(|| {
        let data = Arc::new(Mutex::new(0));
        let data_clone = data.clone();

        let handle = loom::thread::spawn(move || {
            *data_clone.lock().unwrap() += 1;
        });

        handle.join().unwrap();
    });
});
```

**Key benefits:**
- Deterministic testing (no flakiness)
- Exhaustive interleaving check
- Catches data races reliably

## CLI Testing

```bash
cargo run -- test exec --names "cli"
```

**What it does:**
Tests command-line applications like black boxes.

**Use when:**
- Testing CLI tools
- Verifying output format
- Testing argument parsing
- Golden file comparison

## Test Code Generator

```bash
cargo run -- test exec --names "gen"
```

**What it does:**
Generates test code automatically from patterns.

**Use when:**
- Need boilerplate test structure
- Generating parameterized tests
- Creating test templates

## Recommended Testing Paths

### Path 1: Find Edge Cases
```bash
cargo run -- test exec --names "prop"
```
Property-based testing finds edge cases automatically.

### Path 2: Validate Test Quality
```bash
cargo run -- test exec --names "mut"
```
Mutation testing shows if tests are thorough.

### Path 3: Prevent Regressions
```bash
cargo run -- test exec --names "snap"
```
Snapshot testing catches unwanted output changes.

### Path 4: Ensure Thread Safety
```bash
cargo run -- test exec --names "conc"
```
Concurrency testing detects race conditions.

### Path 5: Complete Testing Toolkit
```bash
cargo run -- test exec --names "prop mut snap conc"
```
All advanced testing techniques together.

## Feature Enablement

Some features require feature flags:

```bash
# Property testing (uses proptest)
cargo run --features property-testing -- test exec --names "prop"

# Snapshot testing (uses insta)
cargo run --features snapshot-testing -- test exec --names "snap"

# All features
cargo run --all-features -- test stat
```

## Combining with Core Features

Testing features build on core features:

```rust
use chicago_tdd_tools::prelude::*;

test!(test_property_with_fixture, {
    // Core: Use fixture + builder
    fixture_test!(inner, fixture, {
        let user = UserBuilder::new()
            .with_name("Test")
            .build();

        // Advanced: Property-based verification
        for _ in 0..100 {
            let property_result = your_property(&user);
            assert!(property_result);
        }
    });
});
```

## Best Practices

1. **Start with properties** - Catch edge cases early
2. **Add mutation testing** - Validate test quality
3. **Use snapshots** - Prevent output regressions
4. **Test concurrency** - For multi-threaded code
5. **Combine all** - Build comprehensive test suite

## Troubleshooting

**Q: "Property test takes too long"**
A: Reduce cases:
```rust
for _ in 0..10 {  // Instead of 0..100
    // ...
}
```

**Q: "Snapshot mismatches"**
A: Review and update:
```bash
cargo make snapshot-review
cargo make snapshot-accept
```

**Q: "Concurrency test is slow"**
A: Normal - loom checks all interleavings. Limit scope to problematic code.

**Q: "Feature not available"**
A: Enable with flags:
```bash
cargo run --all-features -- test stat
```

## Next Steps

- **Copy to your project** → [Copying Examples](../tutorials/copying-examples.md)
- **Core features** → [Core Features Guide](core-features.md)
- **Validation features** → [Validation Features](validation-features.md)
- **See all examples** → [Example Inventory](../reference/example-inventory.md)

---

Build comprehensive test suites with property, mutation, snapshot, and concurrency testing.
