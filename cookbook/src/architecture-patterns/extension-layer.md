# Pattern 7: Extension Layer

> 🔧 How-to

## Pattern at a Glance

| Aspect | Details |
|--------|---------|
| **Problem** | Domain code in base bloats all users; extensions need their own helpers but avoid copy-paste |
| **Solution** | Create extension crate; compose base fixtures/builders; add domain-specific methods |
| **When to Use** | Multi-team product, independent evolution needed, shared domain patterns |
| **When NOT to Use** | Single crate (put helpers in local modules), utility functions (put in base) |
| **Trade-offs** | Adds wrapper layers, but enables independent evolution of domain-specific helpers without bloating the core |
| **Complexity** | Low |
| **Real-World Example** | [src/core/fixture.rs](file:///Users/sac/chicago-tdd-tools/src/core/fixture.rs) |

## The Problem

Domain teams need custom fixtures and builders, but embedding them in the core base bloats all users. Copy-pasting helpers from the base layer leads to drift—bugs fixed upstream aren't reflected downstream.

## The Solution

Create an extension crate that depends on the base. Wrap base fixtures inside domain fixtures. Re-export from the extension so teams import domain-specific types. The extension becomes the 20% layered on top of the base's 80%.

## Essential Code Example

```rust
// my-domain-testing crate/src/lib.rs
use chicago_tdd_tools::core::fixture::TestFixture;

pub struct DomainFixture {
    base: TestFixture,
    context: DomainContext,
}

impl DomainFixture {
    pub fn new(config: DomainConfig) -> Self {
        Self {
            base: TestFixture::new(),
            context: DomainContext::from(config),
        }
    }

    // Domain-specific helpers
    pub fn with_default_user(&mut self) -> &mut Self {
        self.context.setup_user();
        self
    }
}
```

## Implementation Checklist

- [ ] Extension crate depends on base, not vice versa
- [ ] Wrap base fixtures/builders, don't copy them
- [ ] Add domain-specific methods to wrappers
- [ ] Re-export key types from extension lib.rs
- [ ] Document which base features are used
- [ ] No duplication of base logic

## The Gotcha (Most Common Mistake)

Copy-pasting helpers from base to extension, creating maintenance burden:

```rust
// ❌ WRONG: Duplicated helpers
// extension/lib.rs copies helpers from chicago-tdd-tools
pub fn setup_database() { /* duplicated */ }
pub fn teardown() { /* duplicated */ }

// ✅ RIGHT: Wrap and extend
use chicago_tdd_tools::core::fixture::TestFixture;

pub fn setup_domain() -> DomainFixture {
    // Uses base fixture internally
    DomainFixture::new(TestFixture::new())
}
```

**Why**: Duplicated code diverges. Bug fixes in the base don't reach copies. Wrappers stay in sync.

## Real-World Example

- **Code location**: [src/core/fixture.rs](file:///Users/sac/chicago-tdd-tools/src/core/fixture.rs)
- **Explanation**: The generic `TestFixture` serves as the base layer, which is extended and composed into specific domain test fixtures.

## Related Patterns

- **Before this**: [Pattern 6: Generic Base](generic-base.md) (base to extend)
- **After this**: [Pattern 8: Composition](composition-over-duplication.md) (how to compose effectively)
- **Use with**: [Pattern 9: Single Source](single-source-of-truth.md) (maintain invariants)

---

**Why It Works**: Extensions stay loosely coupled to the base. When base APIs change, only the wrapper wrapper, not all domain code.

**Production Checklist**:
- [ ] No base helpers are duplicated in extension
- [ ] Extension compiles independently
- [ ] Base updates don't break domain code
- [ ] Domain fixture exports key methods only
- [ ] Tests document extension usage patterns
