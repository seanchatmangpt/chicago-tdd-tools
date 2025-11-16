# Pattern 17: Builder-Driven Test Data

> 🔧 **HOW-TO** | Use fluent builders to reduce test data verbosity

## Quick Reference

| Aspect | Details |
|--------|---------|
| **Problem Solved** | Verbose test data scatters intent; changes break hundreds of tests; duplication of defaults |
| **Core Solution** | Wrap base builder; provide domain-specific fluent methods with sensible defaults |
| **When to Use** | ✅ Domain objects, ✅ Complex nested structures, ✅ Multiple optional fields |
| **When NOT to Use** | ❌ Simple flat objects (direct construction is clearer), ❌ Builder code larger than usage (too much overhead) |
| **Difficulty** | Low - Standard builder pattern |

## The Problem

Complex domain objects require multiple fields. Hand-building them in every test is verbose and scatters intent. When requirements change, hundreds of tests need updates. Defaults are duplicated across tests.

## The Solution

Create domain-specific builders that wrap the generic `TestDataBuilder`. Provide fluent methods with sensible defaults. Builders document domain requirements while reducing test setup verbosity.

## Essential Code Example

```rust
use chicago_tdd_tools::core::builders::TestDataBuilder;

pub struct OrderBuilder {
    base: TestDataBuilder,
}

impl OrderBuilder {
    pub fn new() -> Self {
        Self {
            base: TestDataBuilder::new()
                .with_var("status", "pending")
                .with_var("total", "0.00"),
        }
    }

    pub fn with_customer_id(mut self, id: &str) -> Self {
        self.base = self.base.with_var("customer_id", id);
        self
    }

    pub fn with_amount(mut self, amount: &str) -> Self {
        self.base = self.base.with_var("total", amount);
        self
    }

    pub fn build(self) -> serde_json::Value {
        self.base.build_json().expect("valid json")
    }
}

// Usage: Clear intent, one line per override
let order = OrderBuilder::new()
    .with_customer_id("cust-123")
    .with_amount("99.99")
    .build();
```

## Implementation Checklist

- [ ] Domain builder wraps TestDataBuilder (composition)
- [ ] Sensible defaults in `new()`
- [ ] Fluent methods for each overridable field
- [ ] Build method returns ready-to-use data
- [ ] No duplication of base builder logic
- [ ] Documentation explains default values

## The Gotcha (Most Common Mistake)

Duplicating TestDataBuilder logic in domain builders:

```rust
// ❌ WRONG: Duplication of builder logic
pub struct OrderBuilder {
    fields: HashMap<String, String>,
}
impl OrderBuilder {
    pub fn build(self) -> serde_json::Value {
        // Duplicated serialization logic
    }
}

// ✅ RIGHT: Composition (reuse base builder)
pub struct OrderBuilder {
    base: TestDataBuilder,  // Reuse, don't duplicate
}
impl OrderBuilder {
    pub fn build(self) -> serde_json::Value {
        self.base.build_json().expect("valid")  // Delegation
    }
}
```

**Why**: Duplicating builder logic is a maintenance burden. Composition keeps you in sync with base improvements.

## Codebase Example

File: `src/core/builders.rs`
Purpose: Generic TestDataBuilder that domain builders compose

## Related Patterns

- **Before this**: [Pattern 8: Composition](../architecture-patterns/composition-over-duplication.md) (wrap, don't copy)
- **Use with**: [Pattern 1: AAA Pattern](../testing-patterns/aaa-pattern.md) (builders simplify Arrange)
- **Next**: [Pattern 11: Zero-Cost](zero-cost-abstractions.md) (builders compile away)

---

**Why It Works**: Builders provide fluent APIs that reduce verbosity while centralizing defaults.

**Production Checklist**:
- [ ] All complex objects have builders
- [ ] Builders compose base builder (no duplication)
- [ ] Sensible defaults reduce override noise
- [ ] Fluent API is clear and readable
- [ ] Tests using builder are easier to maintain
