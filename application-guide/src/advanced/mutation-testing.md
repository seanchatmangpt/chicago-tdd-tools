# Mutation Testing

> 🔧 **HOW-TO** | 📚 **REFERENCE** | Validate test quality by introducing mutations

Mutation testing validates test quality by introducing mutations (changes) to code and verifying tests catch them.

## Quick Reference: Mutation Testing API

| Component | Purpose | Key Methods |
|-----------|---------|-------------|
| `MutationTester::new()` | Create a tester for data | `apply_mutation()`, `test_mutation_detection()` |
| `MutationOperator::RemoveKey()` | Remove data key | Parameter: key to remove |
| `MutationOperator::AddKey()` | Add new key-value | Parameters: key, value |
| `MutationOperator::ChangeValue()` | Change existing value | Parameters: key, new_value |
| `MutationOperator::SwapValues()` | Swap two values | Parameters: key1, key2 |
| `MutationOperator::ToggleBoolean()` | Flip bool true↔false | Parameter: key |
| `MutationOperator::NumericDelta()` | Adjust numeric by delta | Parameters: key, delta_value |
| `MutationOperator::StringCase()` | Change string case | Parameters: key, CaseMode |
| `MutationScore::calculate()` | Compute score | Parameters: caught, total |
| `SpanStatus` enum | Mutation test results | Variants: Ok, Error, Unknown |

## Why Mutation Testing?

High code coverage doesn't guarantee good tests:

```rust
fn dangerous_function(x: u32) -> u32 {
    if x > 0 {
        return x * 2;  // Intentional bug: should be x * 3
    }
    0
}

test!(test_bad_coverage, {
    // This test gives 100% code coverage
    assert_eq!(dangerous_function(5), 10);  // This passes even with bug!
    // But tests don't verify the result is CORRECT

    // Bad test - doesn't verify behavior
    assert!(dangerous_function(5) > 0);  // Passes even if returns 99
});
```

Mutation testing fixes this:

```rust
test!(test_with_mutation, {
    // Arrange
    let mut tester = MutationTester::new(dangerous_function);

    // Apply mutation: change * 2 to * 3
    tester.apply_mutation(MutationOperator::ChangeValue(...));

    // Act: Test catches the mutation
    let caught = tester.test_mutation_detection(|func| {
        func(5) == 10  // This will fail with mutation
    });

    // Assert: Mutation was caught
    assert!(caught);  // ✅ Good test catches mutation
});
```

## Basic Mutation Testing

### Creating a Mutation Tester

```rust
use chicago_tdd_tools::mutation::*;
use std::collections::HashMap;

test!(test_mutation_basic, {
    let mut data = HashMap::new();
    data.insert("key1".to_string(), "value1".to_string());

    // Create tester
    let mut tester = MutationTester::new(data);

    // Apply mutation: remove a key
    tester.apply_mutation(MutationOperator::RemoveKey("key1".to_string()));

    // Test detects the mutation
    let caught = tester.test_mutation_detection(|data| {
        !data.is_empty()  // Should fail with mutation
    });

    assert!(caught);
});
```

## Mutation Operators

### RemoveKey

Remove a key from the data:

```rust
tester.apply_mutation(MutationOperator::RemoveKey("key".to_string()));
```

### AddKey

Add a new key:

```rust
tester.apply_mutation(MutationOperator::AddKey(
    "new_key".to_string(),
    "new_value".to_string()
));
```

### ChangeValue

Change a value:

```rust
tester.apply_mutation(MutationOperator::ChangeValue(
    "key".to_string(),
    "different_value".to_string()
));
```

### SwapValues

Swap values between two keys:

```rust
tester.apply_mutation(MutationOperator::SwapValues(
    "key1".to_string(),
    "key2".to_string()
));
```

### ToggleBoolean

Toggle a boolean value (flip true/false):

```rust
tester.apply_mutation(MutationOperator::ToggleBoolean(
    "is_active".to_string()
));
```

### NumericDelta

Change a numeric value by a delta:

```rust
tester.apply_mutation(MutationOperator::NumericDelta(
    "count".to_string(),
    10  // Add 10 to the value
));
```

### StringCase

Change string case:

```rust
use chicago_tdd_tools::mutation::CaseMode;

tester.apply_mutation(MutationOperator::StringCase(
    "name".to_string(),
    CaseMode::Upper  // or Lower, Mixed
));
```

## Mutation Score

Calculate how many mutations your tests catch:

```rust
test!(test_mutation_score, {
    let mut data = HashMap::new();
    data.insert("key1".to_string(), "value1".to_string());
    data.insert("key2".to_string(), "value2".to_string());

    let mut tester = MutationTester::new(data);

    // Apply 3 mutations
    let mut caught = 0;

    // Mutation 1: Remove key1
    tester.apply_mutation(MutationOperator::RemoveKey("key1".to_string()));
    if tester.test_mutation_detection(|d| d.contains_key("key1")) {
        caught += 1;
    }

    // Mutation 2: Remove key2
    tester.apply_mutation(MutationOperator::RemoveKey("key2".to_string()));
    if tester.test_mutation_detection(|d| d.contains_key("key2")) {
        caught += 1;
    }

    // Mutation 3: Add key3
    tester.apply_mutation(MutationOperator::AddKey("key3".to_string(), "value3".to_string()));
    if tester.test_mutation_detection(|d| d.len() == 2) {
        caught += 1;
    }

    // Calculate score
    let score = MutationScore::calculate(caught, 3);
    assert!(score.is_acceptable());  // >= 80%

    alert_info!("Mutation score: {}%", score.score());
});
```

## Real-World Example: User Service

```rust
test!(test_user_service_mutations, {
    let user = User {
        id: 123,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    let mut tester = MutationTester::new(user);
    let mut caught = 0;
    let mut total = 0;

    // Test 1: Mutation removes user ID
    total += 1;
    tester.apply_mutation(MutationOperator::ChangeValue(
        "id".to_string(),
        "0".to_string()
    ));
    if tester.test_mutation_detection(|u| u.id > 0) {
        caught += 1;
    }

    // Test 2: Mutation changes name
    total += 1;
    tester.apply_mutation(MutationOperator::ChangeValue(
        "name".to_string(),
        "Bob".to_string()
    ));
    if tester.test_mutation_detection(|u| u.name == "Alice") {
        caught += 1;
    }

    // Test 3: Mutation changes email
    total += 1;
    tester.apply_mutation(MutationOperator::ChangeValue(
        "email".to_string(),
        "bob@example.com".to_string()
    ));
    if tester.test_mutation_detection(|u| u.email == "alice@example.com") {
        caught += 1;
    }

    let score = MutationScore::calculate(caught, total);
    alert_info!("User service mutation score: {}%", score.score());
    assert!(score.is_acceptable());
});
```

## Interpreting Results

### High Mutation Score (>80%)

✅ **Good**: Tests are catching mutations

```
Mutation Score: 95%
- 95 out of 100 mutations caught
- Tests are effective
- High confidence in code quality
```

### Low Mutation Score (<80%)

⚠️ **Warning**: Some mutations slip through

```
Mutation Score: 60%
- Only 60 out of 100 mutations caught
- 40 mutations go undetected
- May have weak tests or untested branches
```

### Mutations Caught vs. Missed

```
Mutation "Remove key1" → CAUGHT (test failed)
Mutation "Change value" → MISSED (test still passed!)
```

When mutations are missed, improve tests:

```rust
// Before: Weak test
assert!(data.contains_key("key1"));

// After: Strong test
assert_eq!(data.get("key1").unwrap(), "expected_value");
```

## Mutation Testing Workflow

1. **Write tests** (core patterns)
2. **Measure coverage** (80%+ code coverage)
3. **Run mutation tests** (catch mutations)
4. **Improve weak tests** (missing mutations)
5. **Reach 80%+ mutation score**

## When to Use Mutation Testing

✅ **Use for:**
- Validating test suite quality
- Critical code paths
- Security-sensitive code
- Core algorithms

❌ **Don't use for:**
- Every test (slow)
- Simple tests
- Learning phase
- Every build

**Recommendation**: Use mutation testing:
- During development (spot check)
- For critical code (ensure quality)
- Occasionally in CI (weekly)
- Not every build (too slow)

## Performance

- Small mutation set: 1-10 mutations (seconds)
- Medium set: 10-100 mutations (minutes)
- Large set: 100+ mutations (hours)

**Recommendation**: Limit to 20-50 mutations for regular testing.

## Best Practices

✅ **Do:**
- Focus on critical code
- Test both success and error paths
- Improve tests that miss mutations
- Calculate mutation score

❌ **Don't:**
- Run mutation tests on every commit (slow)
- Expect 100% mutation score (impossible)
- Ignore missed mutations
- Over-optimize for mutation score

## Common Mutations to Check

| Mutation | What to Test |
|----------|--------------|
| RemoveKey | Verify required keys exist |
| AddKey | Verify exact set of keys |
| ChangeValue | Verify exact value, not just type |
| SwapValues | Verify values aren't accidentally swapped |
| ToggleBoolean | Verify both true and false cases |
| NumericDelta | Test boundary values and edge cases |
| StringCase | Verify case-sensitive comparisons |

## Troubleshooting

### Mutation Not Caught

Your test is too weak:

```rust
// ❌ Weak - passes even with mutation
assert!(user.id > 0);

// ✅ Strong - catches changes
assert_eq!(user.id, 123);
```

### Mutation Score Unrealistic

Adjust your mutations to match actual bugs:

```rust
// Only test realistic mutations
tester.apply_mutation(MutationOperator::ChangeValue(...));

// Skip mutations that don't matter
// (e.g., changing comments, unused variables)
```

## Next Steps

Learn snapshot testing: [Snapshot Testing](snapshot-testing.md)

---

## Summary

Mutation testing:
- ✅ Validates test quality
- ✅ Catches weak tests
- ✅ Increases confidence
- ✅ Target 80%+ score

Use for critical code to ensure maximum quality.

