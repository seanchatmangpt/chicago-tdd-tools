# Pattern 19: Feature Gate Slices

> 🔧 **HOW-TO** | Group features into curated slices to reduce bloat and build time

## Quick Reference

| Aspect | Details |
|--------|---------|
| **Problem Solved** | Enabling all features bloats build time and dependencies; accidental disables break tests silently |
| **Core Solution** | Group related features into named slices (testing-extras, observability-full); cfg-gate APIs |
| **When to Use** | ✅ Optional capabilities, ✅ Heavy dependencies, ✅ Multi-feature combinations |
| **When NOT to Use** | ❌ Essential features (put in base), ❌ Too many slices (> 5, becomes confusing) |
| **Difficulty** | Low - Pure configuration |

## The Problem

The framework offers advanced capabilities (property testing, containers, telemetry) that not every project needs. Enabling every feature increases compile times and pulls in heavy dependencies. Accidental feature disabling breaks tests silently.

## The Solution

Group related features into curated slices (e.g., testing-extras combines property-testing + snapshot-testing + fake-data). Expose APIs only when features are active via `#[cfg(feature = "...")]`. Keep the base lean; let users opt into advanced capabilities.

## Essential Code Example

```toml
# Cargo.toml: Curated feature slices
[features]
default = ["logging"]

# Base features
logging = ["log"]

# Convenience slices
testing-extras = ["property-testing", "snapshot-testing", "fake-data"]
observability-full = ["otel", "weaver"]
integration-full = ["testcontainers", "docker"]

# Fine-grained controls
property-testing = ["proptest"]
snapshot-testing = ["insta"]
otel = ["opentelemetry"]
```

```rust
// src/lib.rs: Gate APIs by feature
#[cfg(feature = "property-testing")]
pub mod property;

#[cfg(feature = "observability-full")]
pub mod observability;
```

## Implementation Checklist

- [ ] Related features grouped into slices
- [ ] Slices have clear, descriptive names
- [ ] APIs are `#[cfg(feature = "...")]` gated
- [ ] Code compiles with features disabled
- [ ] Documentation explains which features enable which APIs
- [ ] No more than 5-7 named slices (avoid complexity)

## The Gotcha (Most Common Mistake)

Too many fine-grained features or inconsistent feature gates:

```toml
// ❌ WRONG: Too many slices, hard to choose
[features]
foo = ["a"]
bar = ["b"]
baz = ["c", "d"]
// User confusion: which one to enable?

// ✅ RIGHT: Curated slices with clear purpose
[features]
testing-extras = ["property-testing", "snapshot-testing", "fake-data"]
observability-full = ["otel", "weaver"]
```

**Why**: Too many choices is paralyzing. Curated slices guide users to sensible combinations.

## Codebase Example

File: `Cargo.toml`
Purpose: Defines feature slices and their contents

## Related Patterns

- **Before this**: [Pattern 10: Capability Grouping](../architecture-patterns/capability-groups.md) (organize modules)
- **Use with**: [Pattern 6: Generic Base](../architecture-patterns/generic-base.md) (keep base lean)
- **Next**: [Pattern 20: Macro Enforcement](macro-enforcement.md) (enforce feature requirements in macros)

---

**Why It Works**: Feature slices reduce choice paralysis. Curated combinations solve common needs.

**Production Checklist**:
- [ ] Base crate compiles without optional features
- [ ] Each feature slice is well-documented
- [ ] Documentation shows which features enable which APIs
- [ ] Tests cover both with/without features
- [ ] No accidental feature requirements in base
- [ ] CI tests multiple feature combinations
