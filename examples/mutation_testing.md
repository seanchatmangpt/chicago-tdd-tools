# Mutation Testing Example

**Category:** How-To Guide
**Level:** Intermediate
**Prerequisites:** Understanding of testing concepts
**Features Required:** None

---

## Overview

This example demonstrates mutation testing with Chicago TDD tools. Mutation testing validates test quality by introducing mutations (changes) to code and verifying tests catch them.

**What you'll learn:**
- Using `MutationTester` to apply mutations
- Testing mutation detection
- Calculating mutation scores
- Measuring test quality

---

## Quick Start

```bash
cargo run --example mutation_testing
```

---

## Prerequisites

- Rust 1.70+ (Edition 2021)
- Chicago TDD Tools installed
- No additional features required

---

## Key Concepts

### Mutation Testing

Validates test quality by introducing small changes (mutations) to code and verifying that tests fail. If tests don't catch mutations, they may not be testing the right behavior.

**Process:**
1. Apply mutation to code/data
2. Run tests
3. Tests should **fail** (mutation detected)
4. Calculate mutation score (% of mutations caught)

### Mutation Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `RemoveKey` | Remove key from data structure | `{"a": 1}` → `{}` |
| `AddKey` | Add key to data structure | `{"a": 1}` → `{"a": 1, "b": 2}` |
| `ChangeValue` | Change value in data structure | `{"a": 1}` → `{"a": 2}` |
| `NegateCondition` | Negate boolean condition | `if x > 0` → `if x <= 0` |

### Mutation Score

Percentage of mutations caught by tests:

```
Mutation Score = (Mutations Caught / Total Mutations) × 100%
```

**Target:** ≥ 80% mutation score

---

## Code Examples

### Example 1: Basic Mutation Testing

```rust
use chicago_tdd_tools::mutation::*;
use std::collections::HashMap;

// Arrange: Create data and tester
let mut data = HashMap::new();
data.insert("key1".to_string(), "value1".to_string());
let mut tester = MutationTester::new(data);

// Apply mutation
tester.apply_mutation(MutationOperator::RemoveKey("key1".to_string()));

// Act: Test if mutation is caught
let caught = tester.test_mutation_detection(|data| {
    // Test: Data should have at least one key
    !data.is_empty()
});

// Assert: Mutation should be caught
assert!(caught, "Mutation should be detected");
```

### Example 2: Calculate Mutation Score

```rust
use chicago_tdd_tools::mutation::*;
use std::collections::HashMap;

let mut data = HashMap::new();
data.insert("key1".to_string(), "value1".to_string());
let mut tester = MutationTester::new(data);

// Apply mutations
tester.apply_mutation(MutationOperator::RemoveKey("key1".to_string()));
tester.apply_mutation(MutationOperator::AddKey("key2".to_string(), "value2".to_string()));

// Test detection
let caught = tester.test_mutation_detection(|data| data.contains_key("key1"));

// Calculate score
let total_mutations = 2;
let caught_mutations = if caught { total_mutations } else { 0 };
let score = MutationScore::calculate(caught_mutations, total_mutations);

println!("Mutation score: {}%", score.score());
assert!(score.is_acceptable(), "Score should be >= 80%");
```

---

## Mutation Operators

### RemoveKey

Removes a key from a data structure:

```rust
// Before: {"a": 1, "b": 2}
tester.apply_mutation(MutationOperator::RemoveKey("a".to_string()));
// After: {"b": 2}
```

**Tests should:**
- Verify presence of required keys
- Check data completeness

### AddKey

Adds a key to a data structure:

```rust
// Before: {"a": 1}
tester.apply_mutation(MutationOperator::AddKey("b".to_string(), "2".to_string()));
// After: {"a": 1, "b": 2}
```

**Tests should:**
- Verify exact key count
- Check for unexpected keys

### ChangeValue

Changes a value in a data structure:

```rust
// Before: {"a": 1}
tester.apply_mutation(MutationOperator::ChangeValue("a".to_string(), "2".to_string()));
// After: {"a": 2}
```

**Tests should:**
- Verify exact values
- Check value correctness

### NegateCondition

Negates boolean conditions:

```rust
// Before: if x > 0
// After: if x <= 0
```

**Tests should:**
- Test boundary conditions
- Verify control flow

---

## Common Patterns

### Pattern 1: Data Integrity Testing

```rust
let mut tester = MutationTester::new(required_data);
tester.apply_mutation(MutationOperator::RemoveKey("required_field".to_string()));

let caught = tester.test_mutation_detection(|data| {
    data.contains_key("required_field")
});

assert!(caught, "Should detect missing required field");
```

### Pattern 2: Value Validation

```rust
let mut tester = MutationTester::new(data_with_values);
tester.apply_mutation(MutationOperator::ChangeValue("status".to_string(), "invalid".to_string()));

let caught = tester.test_mutation_detection(|data| {
    data.get("status") == Some(&"valid".to_string())
});

assert!(caught, "Should detect invalid status");
```

### Pattern 3: Completeness Testing

```rust
let mut tester = MutationTester::new(minimal_data);
tester.apply_mutation(MutationOperator::AddKey("extra".to_string(), "value".to_string()));

let caught = tester.test_mutation_detection(|data| {
    data.len() == expected_field_count
});

assert!(caught, "Should detect extra fields");
```

---

## Measuring Test Quality

### Good Tests (High Mutation Score)

```rust
// ✓ Good: Tests verify behavior
test!(test_requires_key, {
    let data = required_data();
    assert!(data.contains_key("required"));
});

// Mutation score: 100% - mutation caught
```

### Weak Tests (Low Mutation Score)

```rust
// ✗ Weak: Doesn't verify behavior
test!(test_data_exists, {
    let data = required_data();
    assert!(!data.is_empty());  // Too general
});

// Mutation score: 0% - mutation not caught
```

---

## Best Practices

### 1. Test Specific Behavior

```rust
// ✓ Good: Specific check
assert_eq!(data.get("status"), Some(&"active".to_string()));

// ✗ Bad: General check
assert!(!data.is_empty());
```

### 2. Aim for 80%+ Mutation Score

```rust
let score = MutationScore::calculate(caught, total);
assert!(score.is_acceptable(), "Mutation score should be >= 80%");
```

### 3. Test All Paths

```rust
// Test both success and error paths
test!(test_success_path, { /* ... */ });
test!(test_error_path, { /* ... */ });
```

---

## Troubleshooting

### Low Mutation Score

**Cause:** Tests not verifying specific behavior

**Fix:**
- Add assertions for specific values
- Test boundary conditions
- Verify error paths

### Mutations Not Detected

**Cause:** Tests too general or missing coverage

**Fix:**
- Make assertions more specific
- Add tests for uncovered code paths
- Test edge cases

---

## Next Steps

After mastering mutation testing, explore:

1. **[Property Testing](property_testing.md)** - Random test generation
2. **[Concurrency Testing](concurrency_testing.md)** - Thread safety
3. **[Snapshot Testing](snapshot_testing.md)** - Output stability

---

## Related Documentation

- [Examples README](README.md) - All examples overview
- [API Reference](../docs/reference/API_REFERENCE.md) - Complete API documentation
- [Test Isolation Guide](../docs/process/TEST_ISOLATION_GUIDE.md) - Testing best practices

---

## Reference

### Key Types

- `MutationTester<T>` - Mutation tester for type T
- `MutationOperator` - Type of mutation to apply
- `MutationScore` - Mutation score calculation

### Key Functions

- `MutationTester::new(data) -> MutationTester<T>`
- `MutationTester::apply_mutation(operator)`
- `MutationTester::test_mutation_detection(test) -> bool`
- `MutationScore::calculate(caught, total) -> MutationScore`
- `MutationScore::score() -> u8`
- `MutationScore::is_acceptable() -> bool`

### Mutation Operators

- `MutationOperator::RemoveKey(key)` - Remove key
- `MutationOperator::AddKey(key, value)` - Add key
- `MutationOperator::ChangeValue(key, value)` - Change value
- `MutationOperator::NegateCondition` - Negate condition

---

**Quality is the default. Prevention beats detection.**

*Example: mutation_testing.rs | Version: 1.2.0 | Updated: 2025-11-15*
