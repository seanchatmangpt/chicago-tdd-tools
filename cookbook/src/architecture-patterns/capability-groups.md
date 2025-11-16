# Pattern 10: Capability Grouping

> 🔧 **HOW-TO** | Organize modules by capability, not alphabetically

## Quick Reference

| Aspect | Details |
|--------|---------|
| **Problem Solved** | Features surface randomly; consumers struggle to find capabilities; structure is unmaintainable |
| **Core Solution** | Group modules by capability (core, testing, validation, observability, integration) |
| **When to Use** | ✅ Multi-module crates, ✅ Public APIs, ✅ Feature-gated functionality |
| **When NOT to Use** | ❌ Single-module crates, ❌ Internal organization only (use folders), ❌ Organizational structure (use teams) |
| **Difficulty** | Low - Pure organization pattern |

## The Problem

Without a consistent module taxonomy, new features end up in random places. Consumers can't find capabilities. Maintainers duplicate structure. Documentation becomes a treasure hunt to find the right module.

## The Solution

Organize modules into capability groups: **core** (fixtures, builders, assertions), **testing** (advanced techniques), **validation** (guardrails), **observability** (telemetry), **integration** (external systems). Each group is stable; new modules join existing groups or motivate a new group.

## Essential Code Example

```rust
// src/lib.rs - Organize by capability groups
pub mod core;          // fixtures, builders, assertions, macros
pub mod testing;       // property, mutation, snapshot testing
pub mod validation;    // guards, coverage, performance
pub mod observability; // OTEL, Weaver telemetry
pub mod integration;   // testcontainers, docker support

// Re-export high-level APIs
pub use core::{fixture, builders, assertions};
pub use testing::property;
```

## Implementation Checklist

- [ ] Each module belongs to exactly one capability group
- [ ] Group names are stable (rarely reorganized)
- [ ] Each group is named after "what it does", not "where it is"
- [ ] High-level APIs are re-exported at crate root
- [ ] New modules fit an existing group or create a clear new one
- [ ] Feature flags align with capability groups when optional

## The Gotcha (Most Common Mistake)

Alphabetical organization or organization by implementation (utilities, helpers, core_utils):

```rust
// ❌ WRONG: Alphabetical (hard to find things)
pub mod builders;
pub mod core;
pub mod fixtures;
pub mod guards;
pub mod observability;
pub mod testing;

// ❌ WRONG: Implementation-focused (confusing for users)
pub mod internal;      // What's inside?
pub mod utils;         // What utilities?
pub mod helpers;       // Which helpers?

// ✅ RIGHT: Capability-focused (users know where to look)
pub mod core;          // Fixtures, builders, assertions
pub mod validation;    // Guards, coverage, performance
pub mod observability; // Telemetry, observability
```

**Why**: Users care about capabilities, not implementation. Capability grouping helps them find what they need.

## Codebase Example

File: `src/lib.rs`
Purpose: Shows the five stable capability groups and their purpose

## Related Patterns

- **Before this**: [Pattern 6: Generic Base](generic-base.md) (what to include)
- **Use with**: [Pattern 9: Single Source](single-source-of-truth.md) (organize constants within groups)
- **Next**: [Pattern 19: Feature Gates](../design-patterns/feature-gating.md) (gate capability groups)

---

**Why It Works**: Capability grouping makes module organization predictable. Users can guess where features are, and maintainers have a framework for growth.

**Production Checklist**:
- [ ] Module organization is documented in README
- [ ] Each group has a clear purpose
- [ ] No module belongs to multiple groups
- [ ] Re-exports make common paths short
- [ ] Feature flags align with groups
