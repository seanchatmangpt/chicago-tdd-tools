# Pattern 11: Zero-Cost Abstractions

> 🔧 **HOW-TO** | Use generics and macros instead of runtime polymorphism

## Quick Reference

| Aspect | Details |
|--------|---------|
| **Problem Solved** | Runtime polymorphism slows hot paths; duplicating code sacrifices readability |
| **Core Solution** | Use generics, const generics, and macros; compile to identical machine code as bespoke implementations |
| **When to Use** | ✅ Performance-critical code, ✅ Type-level abstractions, ✅ Macro-driven APIs |
| **When NOT to Use** | ❌ Infrequently called code (use simple trait objects), ❌ Heterogeneous collections (use trait objects) |
| **Difficulty** | Medium - Requires understanding generics and const generics |

## The Problem

Runtime polymorphism adds overhead via vtables and indirect calls. Duplicating code to avoid polymorphism sacrifices maintainability. Neither option is ideal for hot paths.

## The Solution

Use generics and const generics to specialize code at compile time. Each call site gets its own monomorphized version with identical performance to hand-written code. Use macros for DSLs that expand to specialized code. Prefer stack allocation and references over heap allocation.

## Essential Code Example

```rust
use chicago_tdd_tools::validation::performance::measure_ticks;

// Generic: compiled to specialized code per call site (zero-cost)
pub fn measure_operation<F, T>(operation: F) -> (T, u64)
where
    F: FnOnce() -> T,
{
    measure_ticks(operation)  // No vtable lookup, inlined at call site
}

// Called with concrete types - compiler monomorphizes
let (result, ticks) = measure_operation(|| compute_value());
```

## Implementation Checklist

- [ ] Use generics instead of trait objects where performance matters
- [ ] Use const generics for compile-time constants
- [ ] Prefer `&T` over `Box<T>` for parameters
- [ ] Profile before optimizing (don't prematurely apply pattern)
- [ ] Macros generate specialized code, not boilerplate
- [ ] Document performance implications in comments

## The Gotcha (Most Common Mistake)

Using trait objects or runtime dispatch when generics would work:

```rust
// ❌ WRONG: Runtime polymorphism in hot path
pub fn process<T>(item: &dyn Handler<T>) {  // vtable lookup every call
    item.handle();
}

// ✅ RIGHT: Generic (monomorphized, zero-cost)
pub fn process<T, H: Handler<T>>(item: &H) {
    item.handle();  // Inlined, no vtable
}
```

**Why**: Generics compile to specialized code per type. Trait objects require runtime dispatch.

## Codebase Example

File: `src/validation/performance/mod.rs`
Purpose: Shows generic `measure_ticks` that specializes per operation

## Related Patterns

- **Before this**: [Pattern 10: Capability Grouping](../architecture-patterns/capability-groups.md) (organize APIs)
- **Next**: [Pattern 12: Type Safety](type-safety-patterns.md) (combine with type safety)
- **Use with**: [Pattern 14: Compile-Time](compile-time-validation.md) (validate at compile time)

---

**Why It Works**: Generics are monomorphized (specialized) at compile time. The resulting binary code is identical to hand-written specialized code.

**Production Checklist**:
- [ ] Performance-critical paths use generics, not trait objects
- [ ] No unnecessary heap allocations
- [ ] Stack allocation preferred when sizes are known
- [ ] Benchmarks confirm zero-cost abstraction claims
- [ ] Binary size is monitored (monomorphization can bloat)
