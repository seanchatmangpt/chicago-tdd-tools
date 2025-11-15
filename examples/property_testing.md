# Property-Based Testing Example

**Category:** How-To Guide
**Level:** Intermediate
**Prerequisites:** Understanding of testing concepts
**Features Required:** `property-testing`

---

## Overview

This example demonstrates property-based testing where you define properties that should hold for all inputs, and the framework generates random test data to verify them.

**What you'll learn:**
- Using `PropertyTestGenerator` (original, backward compatible)
- Using `ProptestStrategy` (enhanced with proptest crate)
- Defining mathematical properties
- Test data generation and shrinking

---

## Quick Start

```bash
cargo run --example property_testing --features property-testing
```

---

## Prerequisites

- Rust 1.70+ (Edition 2021)
- Chicago TDD Tools with `property-testing` feature

**Add to Cargo.toml:**
```toml
[dev-dependencies]
chicago-tdd-tools = { path = "../chicago-tdd-tools", features = ["property-testing"] }
```

---

## Key Concepts

### Property-Based Testing

Instead of writing specific test cases, define properties that should hold for all inputs:

**Traditional Testing:**
```rust
#[test]
fn test_addition() {
    assert_eq!(2 + 2, 4);
    assert_eq!(5 + 3, 8);
    assert_eq!(10 + 20, 30);
}
```

**Property-Based Testing:**
```rust
// Property: Addition is commutative
strategy.test(any::<(u32, u32)>(), |(x, y)| x + y == y + x);
```

### Properties

Common mathematical properties to test:

| Property | Description | Example |
|----------|-------------|---------|
| **Commutativity** | Order doesn't matter | `a + b == b + a` |
| **Associativity** | Grouping doesn't matter | `(a + b) + c == a + (b + c)` |
| **Identity** | Identity element exists | `a + 0 == a` |
| **Distributivity** | Distribution over operation | `a * (b + c) == (a * b) + (a * c)` |
| **Idempotence** | Applying twice = applying once | `abs(abs(x)) == abs(x)` |

---

## Code Examples

### Example 1: PropertyTestGenerator (Original)

```rust
use chicago_tdd_tools::property::*;

let mut generator = PropertyTestGenerator::<10, 3>::new().with_seed(42);
let data = generator.generate_test_data();
println!("Generated {} items", data.len());
```

**Key Points:**
- Const generic parameters for size and depth
- Configurable random seed for reproducibility
- Backward compatible with existing code

### Example 2: ProptestStrategy (Enhanced)

```rust
use chicago_tdd_tools::property::*;
use proptest::prelude::*;

let strategy = ProptestStrategy::new().with_cases(100);

// Test addition commutativity
strategy.test(any::<(u32, u32)>(), |(x, y)| x + y == y + x);
println!("✓ Addition is commutative");

// Test multiplication distributivity
strategy.test(any::<(u32, u32, u32)>(), |(a, b, c)| {
    a * (b + c) == (a * b) + (a * c)
});
println!("✓ Multiplication is distributive");
```

**Key Points:**
- Uses `proptest` crate for sophisticated generation
- Automatic shrinking to find minimal failing cases
- Configurable number of test cases

---

## Property Examples

### Example 1: Commutativity

```rust
// Property: a + b == b + a
strategy.test(any::<(u32, u32)>(), |(x, y)| x + y == y + x);
```

### Example 2: Identity

```rust
// Property: a + 0 == a
strategy.test(any::<u32>(), |x| x + 0 == x);
```

### Example 3: Associativity

```rust
// Property: (a + b) + c == a + (b + c)
strategy.test(any::<(u32, u32, u32)>(), |(a, b, c)| {
    (a + b) + c == a + (b + c)
});
```

### Example 4: Distributivity

```rust
// Property: a * (b + c) == (a * b) + (a * c)
strategy.test(any::<(u32, u32, u32)>(), |(a, b, c)| {
    a * (b + c) == (a * b) + (a * c)
});
```

---

## Advanced Usage

### Custom Generators

```rust
use proptest::prelude::*;

// Generate specific ranges
let strategy = (1..100u32, 1..100u32);
ProptestStrategy::new().test(strategy, |(x, y)| {
    x + y > x && x + y > y
});
```

### Shrinking

When a property fails, proptest automatically finds the minimal failing case:

```rust
// If property fails for (x=1000, y=500),
// proptest will shrink to find minimal failing input,
// e.g., (x=1, y=1)
```

### Configurable Test Cases

```rust
let strategy = ProptestStrategy::new()
    .with_cases(1000);  // Run 1000 random test cases
```

---

## Common Patterns

### Pattern 1: Testing Pure Functions

```rust
fn reverse_twice<T: Clone>(vec: Vec<T>) -> Vec<T> {
    let mut v = vec.clone();
    v.reverse();
    v.reverse();
    v
}

// Property: reversing twice returns original
strategy.test(any::<Vec<u32>>(), |vec| {
    reverse_twice(vec.clone()) == vec
});
```

### Pattern 2: Testing Invariants

```rust
// Property: Length invariant after sorting
strategy.test(any::<Vec<u32>>(), |mut vec| {
    let original_len = vec.len();
    vec.sort();
    vec.len() == original_len
});
```

### Pattern 3: Testing Relationships

```rust
// Property: max(a, b) >= min(a, b)
strategy.test(any::<(u32, u32)>(), |(a, b)| {
    a.max(b) >= a.min(b)
});
```

---

## Troubleshooting

### Error: "property-testing feature required"

**Cause:** Feature not enabled

**Fix:**
```toml
[dev-dependencies]
chicago-tdd-tools = { path = "../chicago-tdd-tools", features = ["property-testing"] }
```

### Error: "property failed: ..."

**Cause:** Property doesn't hold for generated input

**Fix:**
- Check the shrunk input (minimal failing case)
- Verify your property is correct
- Fix the code or adjust the property

### Slow Tests

**Cause:** Too many test cases

**Fix:** Reduce number of cases:
```rust
let strategy = ProptestStrategy::new().with_cases(10);  // Faster
```

---

## Next Steps

After mastering property-based testing, explore:

1. **[Mutation Testing](mutation_testing.md)** - Test quality validation
2. **[Snapshot Testing](snapshot_testing.md)** - Output stability
3. **[Concurrency Testing](concurrency_testing.md)** - Thread safety

---

## Related Documentation

- [Examples README](README.md) - All examples overview
- [API Reference](../docs/reference/API_REFERENCE.md) - Complete API documentation
- [Pattern Cookbook](../cookbook/src/README.md) - Design patterns

---

## Reference

### Key Types

- `PropertyTestGenerator<const SIZE: usize, const DEPTH: usize>` - Original generator
- `ProptestStrategy` - Enhanced strategy with proptest

### Key Functions

- `PropertyTestGenerator::new() -> PropertyTestGenerator`
- `PropertyTestGenerator::with_seed(seed) -> PropertyTestGenerator`
- `PropertyTestGenerator::generate_test_data() -> Vec<TestData>`
- `ProptestStrategy::new() -> ProptestStrategy`
- `ProptestStrategy::with_cases(count) -> ProptestStrategy`
- `ProptestStrategy::test(strategy, property)`

### Common Properties

- Commutativity: `a ⊕ b == b ⊕ a`
- Associativity: `(a ⊕ b) ⊕ c == a ⊕ (b ⊕ c)`
- Identity: `a ⊕ e == a`
- Distributivity: `a ⊗ (b ⊕ c) == (a ⊗ b) ⊕ (a ⊗ c)`

---

**Quality is the default. Prevention beats detection.**

*Example: property_testing.rs | Version: 1.2.0 | Updated: 2025-11-15*
