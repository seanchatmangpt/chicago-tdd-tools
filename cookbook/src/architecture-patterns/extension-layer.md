# Pattern 7: Extension Layer

## Context

A product team needs domain-specific fixtures, builders, or assertions on top of Chicago TDD Tools.

## Problem

Embedding domain logic inside the core crate makes it hard to evolve independently and risks breaking other users. Repeated copy-paste of generic primitives leads to drift.

## Solution

Create an extension crate that depends on the core. Compose base fixtures inside your domain fixture, forwarding behavior while adding fields and helpers. Re-export the pieces your team should import. Treat the base crate as an 80% solution and layer the remaining 20% locally.

## Forces

- Encapsulation vs. reuse: domain modules wrap core primitives rather than modify them
- Stability vs. iteration speed: upstream stays stable while the extension can iterate quickly
- Discoverability vs. sprawl: re-export only what the domain needs

## Examples

```rust
// workflow-fixture crate
use chicago_tdd_tools::core::fixture::TestFixture;

pub struct WorkflowFixture {
    base: TestFixture<()>,
    engine: WorkflowEngine,
}

impl WorkflowFixture {
    pub fn new(engine: WorkflowEngine) -> Self {
        let base = TestFixture::new().expect("fixture");
        Self { base, engine }
    }
}
```

## Related Patterns

- Pattern 6: Generic Base Layer
- Pattern 8: Composition Over Duplication
- Pattern 9: Single Source of Truth
