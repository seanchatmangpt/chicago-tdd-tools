# Pattern 14: Compile-Time Validation

> 📚 Reference

## Pattern at a Glance

| Aspect | Details |
|--------|---------|
| **Problem** | Runtime validation checks add overhead; missing invariants cause bugs; validation can be bypassed |
| **Solution** | Use const generics, type markers, and compile-time assertions (const_assert!) to validate invariants |
| **When to Use** | Numeric limits, feature combinations, configuration constraints |
| **When NOT to Use** | User input (must validate at runtime), values only known at runtime |
| **Trade-offs** | Moves checks to compile time (which increases compile times slightly) but guarantees zero-cost runtime enforcement |
| **Complexity** | Medium |
| **Real-World Example** | [src/core/const_assert.rs](file:///Users/sac/chicago-tdd-tools/src/core/const_assert.rs) |

## The Problem

Runtime validation adds overhead and can be bypassed in rarely tested code paths. An invariant violation might only surface in production under specific conditions, making debugging difficult.

## The Solution

Encode invariants as types and constants. Use const generics to parameterize limits. Use type-level markers to track state. Use `const_assert!` to validate at compile time. Move as much validation as possible to compile time where the compiler enforces it.

## Essential Code Example

```rust
use chicago_tdd_tools::core::const_assert;

// Compile-time validated array size
pub struct ValidatedArray<const SIZE: usize, const MAX: usize> {
    data: [u8; SIZE],
    _marker: std::marker::PhantomData<[u8; MAX]>,
}

// This fails to compile if SIZE > MAX
impl<const SIZE: usize, const MAX: usize> ValidatedArray<SIZE, MAX> {
    pub const fn new() -> Self {
        const_assert!(SIZE <= MAX);
        Self {
            data: [0u8; SIZE],
            _marker: std::marker::PhantomData,
        }
    }
}

// Usage
let arr = ValidatedArray::<5, 10>::new();  // Compiles
// let arr = ValidatedArray::<15, 10>::new();  // Compile error!
```

## Implementation Checklist

- [ ] Invariants are encoded as type parameters (const generics)
- [ ] Violations prevent compilation (not runtime)
- [ ] Error messages explain the invariant
- [ ] Runtime validation is only for unpredictable values
- [ ] Constructors return Result for runtime validation
- [ ] Type markers track state without runtime cost

## The Gotcha (Most Common Mistake)

Validating at runtime what could be validated at compile time:

```rust
// ❌ WRONG: Runtime validation of compile-time value
pub fn process<T>(data: T, max_size: usize) {
    if std::mem::size_of::<T>() > max_size {
        panic!("Size too large");  // Could have failed at compile time!
    }
}

// ✅ RIGHT: Compile-time validation
pub fn process<T, const MAX: usize>(data: T) {
    const_assert!(std::mem::size_of::<T>() <= MAX);  // Compile error if violated
}
```

**Why**: Compile-time validation is zero-cost and impossible to bypass.

## Real-World Example

- **Code location**: [src/core/const_assert.rs](file:///Users/sac/chicago-tdd-tools/src/core/const_assert.rs)
- **Explanation**: The `const_assert!` macro compiles if and only if the boolean expression evaluates to true, enforcing size or value limits at compile time.

## Related Patterns

- **Before this**: [Pattern 11: Zero-Cost](zero-cost-abstractions.md) (pair with generics)
- **Use with**: [Pattern 15: Type State](type-state-pattern.md) (encode state at compile time)
- **Next**: [Pattern 18: Timeout Defense](timeout-defense.md) (validate timeout constants)

---

**Why It Works**: Compile-time validation is free (zero runtime cost) and impossible to bypass. Violations are caught before shipping.

**Production Checklist**:
- [ ] Invariants that can be const are compile-time checked
- [ ] const generics document constraints
- [ ] Error messages guide users to valid values
- [ ] No runtime checks for compile-time known values
- [ ] Performance is not impacted by validation
