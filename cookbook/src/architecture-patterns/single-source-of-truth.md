# Pattern 9: Single Source of Truth

> 🔧 How-to

## Pattern at a Glance

| Aspect | Details |
|--------|---------|
| **Problem** | Duplicate constants drift; teams change one copy and forget others; inconsistent behavior |
| **Solution** | Centralize constants in owning module; re-export when needed |
| **When to Use** | Timeouts, limits, feature lists, configuration matrices |
| **When NOT to Use** | Runtime configuration (use builders), per-test overrides (parameterize instead) |
| **Trade-offs** | Cross-module dependencies are explicit, but guarantees consistency across all components |
| **Complexity** | Low |
| **Real-World Example** | [src/core/macros/test.rs](file:///Users/sac/chicago-tdd-tools/src/core/macros/test.rs) |

## The Problem

Duplicating constants (timeouts, limits, features) across modules causes drift. One team changes a timeout constant but forgets the second copy in a different module. The system behaves inconsistently: some parts use 5s, others 30s.

## The Solution

Centralize each constant in the module that owns it. Re-export from `lib.rs` so other modules can import it. Document where constants should be used. Extensions read these constants instead of defining their own copies.

## Essential Code Example

```rust
// src/core/macros/test.rs - Single source for timeouts
pub const DEFAULT_UNIT_TEST_TIMEOUT_SECONDS: u64 = 1;
pub const DEFAULT_INTEGRATION_TEST_TIMEOUT_SECONDS: u64 = 30;

// src/lib.rs - Re-export for visibility
pub use core::macros::{
    DEFAULT_UNIT_TEST_TIMEOUT_SECONDS,
    DEFAULT_INTEGRATION_TEST_TIMEOUT_SECONDS,
};

// Any module needing the timeout imports it
use chicago_tdd_tools::DEFAULT_UNIT_TEST_TIMEOUT_SECONDS;
```

## Implementation Checklist

- [ ] Each constant is defined in exactly one place
- [ ] Constant ownership is clear (comment showing owner)
- [ ] Re-export from crate root for visibility
- [ ] Extensions import, not redefine constants
- [ ] Documentation references the single source
- [ ] No duplicated constants in other modules

## The Gotcha (Most Common Mistake)

Defining the "same" constant in multiple places because it seems local:

```rust
// ❌ WRONG: Duplicate constants
// src/core/macros/test.rs
const TIMEOUT_SECS: u64 = 30;

// src/validation/guards.rs (forgot this uses the same timeout)
const TIMEOUT_SECS: u64 = 30;  // Same value, will diverge!

// ✅ RIGHT: Single definition
// src/core/macros/test.rs
pub const DEFAULT_TEST_TIMEOUT_SECS: u64 = 30;

// src/validation/guards.rs
use chicago_tdd_tools::core::macros::DEFAULT_TEST_TIMEOUT_SECS;
```

**Why**: "Same value" isn't the same source. Accidental changes to one copy create bugs in unexpected places.

## Real-World Example

- **Code location**: [src/core/macros/test.rs](file:///Users/sac/chicago-tdd-tools/src/core/macros/test.rs)
- **Explanation**: Defines the default test timeouts in one file, which are then re-exported at the crate root and imported across modules.

## Related Patterns

- **Before this**: [Pattern 8: Composition](composition-over-duplication.md) (apply to constants)
- **Use with**: [Pattern 10: Capability Grouping](capability-groups.md) (organize modules)
- **Next**: [Pattern 18: Timeout Defense](../design-patterns/timeout-defense.md) (uses timeout constants)

---

**Why It Works**: One definition guarantees consistency. Changes propagate everywhere automatically.

**Production Checklist**:
- [ ] No constant is defined more than once
- [ ] All re-exports are at lib.rs or module root
- [ ] Documentation points to single source
- [ ] Tests verify constants are consistent
- [ ] Feature gates don't duplicate constants
