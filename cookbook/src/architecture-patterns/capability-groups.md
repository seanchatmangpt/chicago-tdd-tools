# Pattern 10: Capability Grouping

## Context

You are browsing the Chicago TDD Tools codebase or designing a new module. You need to know where functionality belongs and how to expose it.

## Problem

Without a consistent module taxonomy, features surface haphazardly. Consumers struggle to find capabilities, and maintainers duplicate structure.

## Solution

Group modules by capability: `core` for foundational primitives, `testing` for advanced techniques, `validation` for guardrails, `observability` for telemetry, and `integration` for external systems. Re-export each group at the crate root to support both granular and high-level imports. New modules join one of these groups or motivate a new, clearly named capability.

## Forces

- Discoverability vs. granularity: capability groups provide short import paths while preserving modularity
- Stability vs. evolution: groups rarely change, making documentation and IDE tooling reliable
- Compilation vs. optionality: feature flags enable or disable entire capability slices

## Examples

```rust
// src/lib.rs
pub mod core;          // fixtures, builders, assertions, macros
pub mod testing;       // property testing, mutation testing, snapshots
pub mod validation;    // guards, coverage, performance
pub mod observability; // otel, weaver
pub mod integration;   // testcontainers
```

## Related Patterns

- Pattern 6: Generic Base Layer
- Pattern 9: Single Source of Truth
- Pattern 19: Feature Gate Slices
