# Pattern 8: Composition Over Duplication

## Context

You are adding a feature to an extension crate and need functionality already available in the base layer.

## Problem

Copying helpers or macros breaks the single source of truth. Over time the copies diverge, and bug fixes must be applied in multiple places.

## Solution

Compose existing primitives. Wrap fixtures inside domain fixtures, embed builders into higher-level builders, and use assertion macros rather than writing bespoke checks. When missing functionality is truly generic, add it to the base crate instead of forking it downstream.

## Forces

- Launch speed vs. long-term maintenance: composition keeps future upgrades cheap
- Ergonomics vs. explicitness: wrappers can augment APIs without obscuring the base
- Ownership vs. contribution: contribute upstream when the behavior benefits all users

## Examples

```rust
pub struct OrderBuilder {
    base: TestDataBuilder,
}

impl OrderBuilder {
    pub fn new() -> Self {
        Self { base: TestDataBuilder::new() }
    }

    pub fn with_amount(mut self, amount: u64) -> Self {
        self.base = self.base.with_var("amount", amount.to_string());
        self
    }

    pub fn build_json(self) -> serde_json::Value {
        self.base.build_json().expect("json")
    }
}
```

## Related Patterns

- Pattern 6: Generic Base Layer
- Pattern 7: Extension Layer
- Pattern 9: Single Source of Truth
