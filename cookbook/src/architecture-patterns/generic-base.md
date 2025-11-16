# Pattern 6: Generic Base Layer

> 🔧 **HOW-TO** | Keep core lean: generic capabilities, no domain logic

## Quick Reference

| Aspect | Details |
|--------|---------|
| **Problem Solved** | Base layer bloats with domain code; consumers pull unused dependencies and bloat builds |
| **Core Solution** | Core = generic only (fixtures, builders, assertions); domain logic in extensions |
| **When to Use** | ✅ Reusable testing libraries, ✅ Multi-team frameworks, ✅ Open-source crates |
| **When NOT to Use** | ❌ Single domain (use domain layer directly), ❌ Monorepos (use capability grouping) |
| **Difficulty** | Medium - Requires identifying generic vs. domain code |

## The Problem

If the base layer includes domain code, every consumer pulls unused dependencies, slowing builds and creating tight coupling. If the base is too minimal, each consumer must reinvent the wheel.

## The Solution

Keep the base layer focused on generic testing primitives: fixtures, builders, assertions, macros, state machines. Organize by capability (core, testing, validation, observability, integration). Domain-specific helpers live in extension crates that compose the base layer.

## Essential Code Example

```rust
// src/lib.rs - Generic base (no domain)
pub mod core;          // fixtures, builders, assertions
pub mod testing;       // property, mutation, snapshots
pub mod validation;    // guards, coverage
pub mod observability; // telemetry
pub mod integration;   // containers

pub use core::{fixture, builders, assertions};

// Note: NO domain code here. Extensions add domain-specific fixtures.
```

## Implementation Checklist

- [ ] Base layer has no domain-specific types or constants
- [ ] All modules depend only on stdlib and optional features
- [ ] Features are optional (not required for base functionality)
- [ ] Re-export high-level APIs at the crate root
- [ ] Extensions compose the base, not copy it
- [ ] Capability groups are stable (rarely reorganized)

## The Gotcha (Most Common Mistake)

Adding domain logic to the base layer to save one dependency:

```rust
// ❌ WRONG: Domain code in base
pub mod customers {
    pub struct CustomerFixture { /* ... */ }
}
pub mod orders {
    pub struct OrderFixture { /* ... */ }
}
// Now every user of base pulls in customer and order logic

// ✅ RIGHT: Base is generic; domain extensions compose it
pub mod core { /* fixtures, builders, assertions */ }
// users create: customers-extension, orders-extension crates
// that depend on chicago-tdd-tools::core
```

**Why**: Domain logic couples the base to specific use cases. Generic primitives stay stable and reusable.

## Codebase Example

File: `src/lib.rs`
Purpose: Shows generic module structure with no domain coupling

## Related Patterns

- **Before this**: [Pattern 8: Composition](composition-over-duplication.md) (how to compose the base)
- **Next**: [Pattern 7: Extension Layer](extension-layer.md) (build domain extensions on top)
- **Use with**: [Pattern 10: Capability Grouping](capability-groups.md) (organize modules)

---

**Why It Works**: Generic primitives stay small and reusable. Domain-specific logic lives where it belongs—in extension crates.

**Production Checklist**:
- [ ] Base crate compiles without domain features
- [ ] No domain-specific builder methods
- [ ] Extensions are optional imports
- [ ] Build time is acceptable with/without features
- [ ] Documentation doesn't mention domain concepts
