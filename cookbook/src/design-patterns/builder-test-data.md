# Pattern 17: Builder-Driven Test Data

## Context

Domain objects require multiple fields or nested structures. Hand-building them in tests scatters intent and duplicates defaults.

## Problem

Verbose setup obscures the behavior under test. When requirements change, hundreds of tests need updates.

## Solution

Wrap `TestDataBuilder` (or create your own builder) to provide fluent helpers and sensible defaults. Expose domain-specific methods (`with_customer_id`, `with_balance`) and return JSON or HashMap structures ready for assertions. Builders live close to the domain, yet reuse the underlying generic builder to avoid duplication.

## Forces

- Expressiveness vs. coupling: builders should reflect domain language without leaking implementation details
- Defaults vs. explicitness: provide safe defaults but allow overrides
- Reuse vs. specialization: share base builder logic; extensions add convenience

## Examples

```rust
pub struct CustomerBuilder {
    base: TestDataBuilder,
}

impl CustomerBuilder {
    pub fn new() -> Self {
        Self {
            base: TestDataBuilder::new()
                .with_var("status", "active"),
        }
    }

    pub fn with_id(mut self, id: &str) -> Self {
        self.base = self.base.with_var("customer_id", id.to_string());
        self
    }

    pub fn build(self) -> serde_json::Value {
        self.base.build_json().expect("valid json")
    }
}
```

## Related Patterns

- Pattern 8: Composition Over Duplication
- Pattern 11: Zero-Cost Abstractions
- Pattern 19: Feature Gate Slices
