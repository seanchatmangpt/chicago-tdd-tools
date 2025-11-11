# Pattern 9: Single Source of Truth

## Context

Constants, toggle lists, and feature matrices are needed across modules and extensions.

## Problem

Duplicating configuration (timeouts, guard limits, feature lists) invites drift. Teams change one copy and forget the rest, producing inconsistent behavior.

## Solution

Centralize invariants inside the module that owns them and re-export when necessary. Examples include timeout constants in `core::macros::test`, guard limits in `validation::guards`, and feature combinations in `Cargo.toml`. Extensions read these definitions instead of defining their own copies.

## Forces

- Accessibility vs. encapsulation: invariants must be easy to import without exposing internals
- Flexibility vs. safety: allow customization through builders or configuration rather than duplicating constants
- Documentation vs. code: comments and docs should reference the single source, not restated values

## Examples

```rust
// src/core/macros/test.rs
pub const DEFAULT_UNIT_TEST_TIMEOUT_SECONDS: u64 = 1;
pub const DEFAULT_INTEGRATION_TEST_TIMEOUT_SECONDS: u64 = 30;

// src/validation/guards.rs
pub const MAX_RUN_LEN: usize = 8;
```

## Related Patterns

- Pattern 8: Composition Over Duplication
- Pattern 10: Capability Grouping
- Pattern 18: Timeout Defense in Depth
