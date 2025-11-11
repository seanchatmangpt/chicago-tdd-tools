# Pattern 19: Feature Gate Slices

## Context

The framework offers advanced capabilities (property testing, mutation testing, testcontainers, OTEL) that not every project needs.

## Problem

Enabling every feature increases compile times and pulls in heavy dependencies. Disabling a feature accidentally can break tests silently.

## Solution

Group related features into named slices in `Cargo.toml` (e.g., `testing-extras`, `observability-full`). Document the slice and expose `cfg`-gated APIs accordingly. Tests and examples import the feature-specific modules only when the feature is active, keeping the base lean.

## Forces

- Modularity vs. convenience: slices reduce duplication but still allow fine-grained toggles
- Discoverability vs. complexity: a small number of curated slices keeps onboarding simple
- Compatibility vs. optionality: code must compile cleanly with features disabled

## Examples

```toml
[features]
default = ["logging"]
testing-extras = ["property-testing", "snapshot-testing", "fake-data"]
observability-full = ["otel", "weaver"]
```

```rust
#[cfg(feature = "weaver")]
pub mod weaver;
```

## Related Patterns

- Pattern 6: Generic Base Layer
- Pattern 10: Capability Grouping
- Pattern 20: Macro Pattern Enforcement
