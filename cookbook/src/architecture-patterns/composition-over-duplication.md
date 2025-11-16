# Pattern 8: Composition Over Duplication

> 🔧 **HOW-TO** | Wrap existing primitives instead of copying them

## Quick Reference

| Aspect | Details |
|--------|---------|
| **Problem Solved** | Duplicated helpers diverge; bug fixes in base don't reach copies; maintenance nightmare |
| **Core Solution** | Wrap base primitives (don't copy); extend wrappers with domain methods |
| **When to Use** | ✅ Adding domain methods, ✅ Reusing base builders, ✅ Customizing base fixtures |
| **When NOT to Use** | ❌ Missing base functionality (contribute to base instead), ❌ Conflicting behavior (redesign pattern) |
| **Difficulty** | Low - Standard composition pattern |

## The Problem

Copying helpers breaks the single source of truth. Bug fixes in the base don't reach copies. Over time, copies diverge and become unmaintainable. Teams waste time synchronizing copies instead of fixing actual bugs.

## The Solution

Wrap base primitives instead of copying them. Create domain builders that compose base builders. Create domain fixtures that wrap base fixtures. Extend wrappers with domain-specific methods, but keep the base behavior intact.

## Essential Code Example

```rust
use chicago_tdd_tools::core::builders::TestDataBuilder;

pub struct OrderBuilder {
    base: TestDataBuilder,
}

impl OrderBuilder {
    pub fn new() -> Self {
        Self { base: TestDataBuilder::new() }
    }

    // Domain-specific method using base builder
    pub fn with_amount(mut self, amount: u64) -> Self {
        self.base = self.base.with_var("amount", amount.to_string());
        self
    }

    pub fn build_json(self) -> serde_json::Value {
        self.base.build_json().expect("json")
    }
}
```

## Implementation Checklist

- [ ] Wrap base primitives, don't copy methods
- [ ] Add domain-specific methods to wrappers
- [ ] Keep base behavior unchanged
- [ ] Delegate to base for core logic
- [ ] Tests verify base behavior is preserved
- [ ] If base is missing functionality, contribute upstream first

## The Gotcha (Most Common Mistake)

Copying and modifying base helpers to "save time":

```rust
// ❌ WRONG: Copy leads to divergence
pub fn setup_fixture() {
    // Copied from base, but with custom logic added
    let db = Database::new();  // What if base changes this?
    // ...
}

// ✅ RIGHT: Compose, don't copy
pub fn setup_fixture() -> DomainFixture {
    let base = BaseFixture::new();  // Inherit base behavior
    let domain = DomainFixture::new(base);  // Wrap it
    domain.with_custom_config()
}
```

**Why**: Copies diverge immediately. Composition keeps you in sync with the base.

## Codebase Example

File: `src/core/builders.rs` and extension examples
Purpose: Shows composition of generic builders into domain builders

## Related Patterns

- **Before this**: [Pattern 7: Extension Layer](extension-layer.md) (create extensions)
- **Use with**: [Pattern 6: Generic Base](generic-base.md) (base to compose from)
- **Next**: [Pattern 9: Single Source](single-source-of-truth.md) (maintain consistency)

---

**Why It Works**: Wrapped primitives inherit base improvements automatically. Domain logic stays in one place.

**Production Checklist**:
- [ ] No duplicated logic from base
- [ ] Base updates don't break wrappers
- [ ] Composition is transparent to callers
- [ ] Wrapper methods are clearly domain-specific
- [ ] Tests verify both base and domain behavior
