# Advanced Rust Features Example

**Category:** Explanation
**Level:** Advanced
**Prerequisites:** Understanding of Rust type system
**Features Required:** `async` (optional)

---

## Overview

This example demonstrates hyper-advanced Rust features used in Chicago TDD Tools to provide compile-time guarantees and zero-cost abstractions.

**What you'll learn:**
- Type-level arithmetic with const generics
- Type state pattern for compile-time correctness
- Async traits (Rust 1.75+)
- Zero-cost abstractions
- Compile-time error prevention

---

## Quick Start

```bash
cargo run --example advanced_features
```

---

## Prerequisites

- Rust 1.70+ (Edition 2021)
- Rust 1.75+ for async trait features
- Understanding of Rust type system and generics

---

## Key Concepts

### 1. Const Generics and Type-Level Arithmetic

```rust
use chicago_tdd_tools::core::type_level::SizeValidatedArray;

const ARRAY: SizeValidatedArray<8, 8> = SizeValidatedArray::new([0u8; 8]);
assert_eq!(ARRAY.size(), 8);
```

**Benefits:**
- Compile-time size validation
- Zero runtime overhead
- Type-safe array operations

### 2. Type State Pattern

Enforce test phase ordering at compile time:

```rust
use chicago_tdd_tools::core::state::{Arrange, TestState};

// Can only progress: Arrange → Act → Assert
let arrange = TestState::<Arrange>::new()
    .with_arrange_data(vec![1, 2, 3]);

let act = arrange.act();  // ✓ Valid transition
// arrange.assert();      // ✗ Compile error! Can't skip Act phase

let act = act.execute(|data| {
    let mut result = data.unwrap_or_default();
    result.push(4);
    result
});

let assert = act.assert();  // ✓ Valid transition
```

**Benefits:**
- Compile-time phase enforcement
- Invalid transitions are compile errors
- Impossible to skip phases or call methods in wrong order

### 3. Async Traits (Rust 1.75+)

```rust
#[cfg(feature = "async")]
use chicago_tdd_tools::core::async_fixture::AsyncFixtureManager;

// Async methods in traits (Rust 1.75+)
// See src/core/async_fixture.rs for implementation
```

**Benefits:**
- Async fixture management
- Same ergonomics as sync code
- No runtime overhead

---

## Code Examples

### Example 1: Type-Level Size Validation

```rust
const fn example_type_level_arithmetic() {
    // Create size-validated array using const generics
    const ARRAY: SizeValidatedArray<8, 8> = SizeValidatedArray::new([0u8; 8]);

    // Size is compile-time constant
    assert_eq!(ARRAY.size(), 8);
    assert_eq!(ARRAY.data().len(), 8);
}
```

### Example 2: Type State Pattern

```rust
fn example_type_state_pattern() {
    // Start with Arrange phase (type system enforces order)
    let arrange_state = TestState::<Arrange>::new()
        .with_arrange_data(vec![1, 2, 3]);

    // Transition to Act phase (only possible from Arrange)
    let act_state = arrange_state.act();
    let act_state = act_state.execute(|data| {
        let mut result = data.unwrap_or_default();
        result.push(4);
        result
    });

    // Transition to Assert phase (only possible from Act)
    let assert_state = act_state.assert();
    assert!(assert_state.assert_that(|result| {
        result.map(|r| r.len() == 4).unwrap_or(false)
    }));

    // Type system prevents calling methods in wrong order:
    // - Cannot call act() on TestState<Assert>
    // - Cannot call assert() on TestState<Arrange>
    // - Cannot create TestState<Act> directly
}
```

---

## Advanced Patterns

### Pattern 1: Compile-Time Guarantees

The type system catches errors before code runs:

```rust
// ✓ Valid: Proper phase progression
let state = TestState::<Arrange>::new()
    .with_arrange_data(data)
    .act()
    .execute(|d| process(d))
    .assert();

// ✗ Compile error: Cannot skip phases
let state = TestState::<Arrange>::new()
    .assert();  // ERROR: Method not available on TestState<Arrange>
```

### Pattern 2: Zero-Cost Abstractions

Advanced features compile to the same code as manual implementations:

```rust
// This high-level code:
let state = TestState::<Arrange>::new().with_arrange_data(vec![1, 2, 3]);

// Compiles to the same assembly as:
struct TestState { data: Vec<i32> }
let state = TestState { data: vec![1, 2, 3] };
```

**No runtime overhead for:**
- Type state transitions
- Const generic validations
- Sealed trait enforcement

---

## Philosophy

### Poka-Yoke (Error-Proofing)

Advanced features prevent entire classes of errors at compile time:

| Error Class | Prevention Method | Example |
|------------|------------------|---------|
| Wrong phase order | Type state pattern | Cannot call assert() before act() |
| Invalid sizes | Const generics | Array size validated at compile time |
| External implementations | Sealed traits | Only intended types can implement |

### Developer Experience (DX)

**Goal:** Maximize DX through compile-time guarantees

**Benefits:**
1. **Catch errors early** - Compilation, not runtime
2. **Better IDE support** - Type system guides correct usage
3. **Self-documenting** - Types encode constraints
4. **Zero cost** - No runtime overhead

---

## Troubleshooting

### Error: "method not found in `TestState<Arrange>`"

**Cause:** Trying to call method from wrong phase

**Fix:** Follow the correct phase progression:
```rust
// ✓ Correct
TestState::<Arrange>::new().act().assert()

// ✗ Wrong
TestState::<Arrange>::new().assert()
```

### Error: "async feature required"

**Cause:** Using async features without feature flag

**Fix:**
```toml
[dev-dependencies]
chicago-tdd-tools = { path = "../chicago-tdd-tools", features = ["async"] }
```

### Error: "mismatched types: expected `8`, found `10`"

**Cause:** Const generic size mismatch

**Fix:** Ensure size parameters match:
```rust
// ✓ Correct
const ARRAY: SizeValidatedArray<8, 8> = SizeValidatedArray::new([0u8; 8]);

// ✗ Wrong
const ARRAY: SizeValidatedArray<8, 10> = SizeValidatedArray::new([0u8; 8]);
```

---

## Next Steps

After understanding advanced features, explore:

1. **[Go the Extra Mile](go_extra_mile.md)** - Progressive enhancement philosophy
2. **[Basic Test](basic_test.md)** - Apply concepts to practical testing
3. **[Pattern Cookbook](../cookbook/src/README.md)** - Design patterns using these features

---

## Related Documentation

- [Examples README](README.md) - All examples overview
- [Architecture](../docs/reference/ARCHITECTURE.md) - Design principles
- [API Reference](../docs/reference/API_REFERENCE.md) - Complete API documentation

---

## Reference

### Key Types

- `SizeValidatedArray<const SIZE: usize, const VALIDATED_SIZE: usize>` - Size-validated array
- `TestState<Phase>` - Type state pattern (Arrange, Act, Assert)
- `AsyncFixtureProvider` - Async trait for fixtures (requires `async` feature)

### Key Concepts

- **Const Generics** - Generic parameters that are constant values
- **Type State** - Encode state machines in types
- **Zero-Cost** - No runtime overhead for abstractions
- **Compile-Time** - Errors caught before code runs
- **Sealed Traits** - Prevent external implementations

### Feature Flags

- `async` - Enable async trait features (Rust 1.75+)

---

**Quality is the default. Prevention beats detection.**

*Example: advanced_features.rs | Version: 1.2.0 | Updated: 2025-11-15*
