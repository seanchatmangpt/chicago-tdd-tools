# Property-Based Testing

Property-based testing generates random test data and verifies that properties hold for all inputs.

## What is a Property?

A **property** is a logical assertion that should hold for all valid inputs:

```rust
// Property: Addition is commutative
// For all a, b: a + b == b + a

// Property: Parsing and formatting is round-trip safe
// For all x: parse(format(x)) == x

// Property: Sorted list has no inversions
// For all lists: list[i] <= list[i+1]
```

## Property-Based vs. Example-Based

### Example-Based (Traditional)

```rust
test!(test_addition_examples, {
    assert_eq!(2 + 3, 5);
    assert_eq!(0 + 5, 5);
    assert_eq!(10 + 0, 10);
});
```

**Limitation**: Only tests specific examples. What about `u32::MAX + 1`?

### Property-Based

```rust
test!(test_addition_property, {
    let strategy = ProptestStrategy::new().with_cases(1000);
    strategy.test(any::<(u32, u32)>(), |(a, b)| {
        a + b == b + a  // Checks for 1000 random pairs
    });
});
```

**Advantage**: Tests 1000 random cases automatically.

## Getting Started

### Basic Property Test

```rust
use chicago_tdd_tools::property::*;
use proptest::prelude::*;

test!(test_parsing_property, {
    let strategy = ProptestStrategy::new().with_cases(100);

    strategy.test(any::<u32>(), |num| {
        let formatted = format!("{}", num);
        let parsed: u32 = formatted.parse().unwrap();
        num == parsed  // Property: round-trip works
    });
});
```

### Using Generators

Generate specific types:

```rust
test!(test_string_properties, {
    let strategy = ProptestStrategy::new().with_cases(100);

    // Test with strings of 1-100 characters
    strategy.test("[a-zA-Z0-9]{1,100}", |s| {
        // Property: non-empty string remains non-empty
        !s.is_empty()
    });
});
```

## Common Properties to Test

### 1. Commutativity

```rust
// Property: a + b == b + a
strategy.test(any::<(i32, i32)>(), |(a, b)| {
    a + b == b + a
});
```

### 2. Associativity

```rust
// Property: (a + b) + c == a + (b + c)
strategy.test(any::<(i32, i32, i32)>(), |(a, b, c)| {
    (a + b) + c == a + (b + c)
});
```

### 3. Distributivity

```rust
// Property: a * (b + c) == (a * b) + (a * c)
strategy.test(any::<(i32, i32, i32)>(), |(a, b, c)| {
    a * (b + c) == (a * b) + (a * c)
});
```

### 4. Identity

```rust
// Property: a + 0 == a
strategy.test(any::<i32>(), |a| {
    a + 0 == a
});
```

### 5. Inverse

```rust
// Property: a - a == 0
strategy.test(any::<i32>(), |a| {
    a - a == 0
});
```

## Real-World Example: JSON Parsing

```rust
test!(test_json_parsing_properties, {
    let strategy = ProptestStrategy::new().with_cases(500);

    strategy.test(any::<(String, i32, bool)>(), |(name, age, active)| {
        // Create JSON
        let json = format!(
            r#"{{"name":"{}","age":{},"active":{}}}"#,
            name, age, active
        );

        // Parse it
        let parsed: Result<MyData, _> = serde_json::from_str(&json);

        // Property: Valid input parses successfully
        parsed.is_ok()
    });
});
```

## Real-World Example: String Validation

```rust
test!(test_email_validation, {
    let strategy = ProptestStrategy::new().with_cases(200);

    // Test valid emails
    strategy.test(
        r"[a-zA-Z0-9]+@[a-zA-Z0-9]+\.[a-zA-Z]{2,}",
        |email| {
            // Property: Valid email passes validation
            validate_email(email).is_ok()
        }
    );
});
```

## Shrinking

When a property fails, shrinking finds the minimal failing case:

```rust
test!(test_with_shrinking, {
    let strategy = ProptestStrategy::new().with_cases(100);

    strategy.test(any::<Vec<i32>>(), |vec| {
        // Property fails for some input
        vec.len() < 10  // This might fail for vec![1,2,3,...,100]

        // Shrinking finds minimal failure: vec with length >= 10
    });
});
```

## Combining Strategies

Test multiple values together:

```rust
test!(test_combined_strategy, {
    let strategy = ProptestStrategy::new().with_cases(100);

    // Test with tuple of (String, u32, bool)
    strategy.test(
        (any::<String>(), 1u32..100u32, any::<bool>()),
        |(name, age, active)| {
            // Test with all three values
            !name.is_empty() && age > 0
        }
    );
});
```

## Configuration

### Number of Cases

```rust
// Test with 100 random cases
let strategy = ProptestStrategy::new().with_cases(100);

// Test with 1000 cases (slower, more thorough)
let strategy = ProptestStrategy::new().with_cases(1000);

// Test with 10 cases (faster, less thorough)
let strategy = ProptestStrategy::new().with_cases(10);
```

### Random Seed

For reproducible tests:

```rust
let mut generator = PropertyTestGenerator::<100, 5>::new()
    .with_seed(42);  // Use specific seed

let data = generator.generate_test_data();
```

## When to Use Property-Based Testing

✅ **Use for:**
- Mathematical properties (commutativity, associativity)
- Round-trip properties (serialize/deserialize)
- Parsing and formatting
- List operations (sort, filter, map)
- State machine transitions

❌ **Don't use for:**
- Specific business logic (use example tests)
- Performance testing (use benchmarks)
- Complex setup (use fixtures)

## Best Practices

✅ **Do:**
- Test actual properties (not specific values)
- Use meaningful generators
- Start with 100-500 cases
- Check edge cases manually

❌ **Don't:**
- Replace example tests (both have value)
- Use excessive cases (slows down tests)
- Ignore failed cases
- Only use randomly generated data

## Performance

- 100 cases: ~100ms per property
- 1000 cases: ~1s per property
- 10,000 cases: ~10s per property

**Recommendation**: Start with 100-500 cases. Use more for critical code.

## Troubleshooting

### Property Fails Intermittently

Use shrinking output to find minimal case:

```rust
test!(test_debug_failure, {
    let strategy = ProptestStrategy::new().with_cases(1000);
    strategy.test(any::<(u32, u32)>(), |(a, b)| {
        // If fails: check shrunk output
        // Example: shrunk to (0, 0) or (u32::MAX, 0)
        (a as u64) + (b as u64) < u64::MAX
    });
});
```

### Property Too Strict

Relax constraints:

```rust
// Too strict: a * b == b * a (fails due to overflow)
// Better: a.checked_mul(b) == b.checked_mul(a)
```

## Next Steps

Learn mutation testing: [Mutation Testing](mutation-testing.md)

---

## Summary

Property-based testing:
- ✅ Tests properties for all inputs
- ✅ Finds edge cases automatically
- ✅ Includes shrinking to find minimal failures
- ✅ Great for algorithms and parsing

Use with fixtures and data builders for comprehensive testing.

