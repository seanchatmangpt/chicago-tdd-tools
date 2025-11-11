# Pattern 6: Generic Base Layer

## Context

You want a reusable testing foundation that can support multiple domains without pulling in their dependencies.

## Problem

If the base includes domain code, every consumer drags in unused dependencies, slowing builds and introducing coupling. Conversely, a minimal base risks missing essential primitives.

## Solution

Keep the core crate focused on generic capabilities – fixtures, builders, assertions, macros, state tracking. Expose them through capability modules (`core`, `testing`, `validation`, `observability`, `integration`). Ensure modules depend only on the standard library and optional features. Domain-specific abstractions live in downstream crates that compose the base.

## Forces

- Reuse vs. specialization: core must be broadly useful without dictating domain models
- Build time vs. flexibility: optional features load only when needed
- Stability vs. growth: base APIs should be stable; extensions add behavior

## Examples

```rust
// src/lib.rs
pub mod core;
pub mod testing;
pub mod validation;
pub mod observability;
pub mod integration;

pub use core::{fixture, builders, assertions};
```

## Related Patterns

- Pattern 7: Extension Layer
- Pattern 8: Composition Over Duplication
- Pattern 10: Capability Grouping
