# Common Mistakes Across All Patterns

This guide aggregates "The Gotcha" from each pattern and shows how to avoid it.

---

## Testing Patterns

### Pattern 1: AAA - Mixing Arrange and Act

**The Mistake:**
```rust
test!(test_bad, {
    let result = setup_and_process(100);  // Can't tell what's being tested
    assert_eq!(result, 200);
});
```

**The Fix:**
```rust
test!(test_good, {
    let value = 100;              // Arrange
    let result = process(value);  // Act
    assert_eq!(result, 200);      // Assert
});
```

**Why It Matters:** When the test fails, you need to know which phase broke. Mixing them makes debugging impossible.

---

### Pattern 2: Error Paths - Testing `is_err()` Without Checking Variant

**The Mistake:**
```rust
test!(test_bad, {
    let result = validate(invalid_input);
    assert!(result.is_err());  // What error? Any error passes!
});
```

**The Fix:**
```rust
test!(test_good, {
    let result = validate(invalid_input);
    assert_err!(&result);
    assert_eq!(result.unwrap_err().kind(), ValidationError);
    assert!(result.unwrap_err().to_string().contains("empty"));
});
```

**Why It Matters:** If the error type changes, you won't catch it. Specific checks prevent regressions.

---

### Pattern 3: Boundaries - Testing Only Happy Path

**The Mistake:**
```rust
test!(test_bad, {
    let result = process_items(vec![1, 2, 3, 4, 5]);  // Safe middle ground
    assert_ok!(&result);
});
```

**The Fix:**
```rust
param_test! {
    #[case(9)]       // below max
    #[case(10)]      // at max
    #[case(11)]      // above max
    fn test_boundary(count: usize) {
        let items = vec![0; count];
        let result = process_items(items);
        match count {
            0..=10 => assert_ok!(&result),
            _ => assert_err!(&result),
        }
    }
}
```

**Why It Matters:** Off-by-one errors are the most common boundary bug.

---

### Pattern 4: Cleanup - Manual Cleanup After Assertions

**The Mistake:**
```rust
test!(test_bad, {
    let container = docker.run("postgres:16");
    let result = container.query("SELECT 1");
    assert_eq!(result, 1);  // If this fails, cleanup never runs!
    drop(container);
});
```

**The Fix:**
```rust
fixture_test!(test_good, fixture, {
    let container = fixture.postgres_container()?;
    let result = container.query("SELECT 1")?;
    assert_eq!(result, 1);  // Cleanup happens regardless
    Ok(())
});
```

**Why It Matters:** Explicit cleanup is bypassed by panics and early returns. RAII guarantees it.

---

### Pattern 5: Real Collaborators - Mixing Real and Mock

**The Mistake:**
```rust
let db = RealDatabase::connect()?;      // Real
let cache = MockCache::new();           // Mock
let result = query_with_cache(&db, &cache)?;  // Can't tell which failed
```

**The Fix:**
```rust
// All real
let db = RealDatabase::connect()?;
let cache = RealCache::connect()?;
let result = query_with_cache(&db, &cache)?;

// OR all mocked (for unit tests)
let db = MockDatabase::new();
let cache = MockCache::new();
let result = query_with_cache(&db, &cache)?;
```

**Why It Matters:** Mixed real/mock makes it impossible to know what failed when tests break.

---

## Architecture Patterns

### Pattern 6: Generic Base - Adding Domain Logic to Base

**The Mistake:**
```rust
pub mod customers { /* domain code */ }
pub mod orders { /* domain code */ }
// Every user of base pulls in customer and order logic!
```

**The Fix:**
```rust
pub mod core { /* generic: fixtures, builders, assertions */ }
// Users create: customers-extension, orders-extension crates
// that depend on chicago-tdd-tools::core
```

**Why It Matters:** Domain logic in base couples it to specific use cases, reducing reusability.

---

### Pattern 7: Extension - Copy-Pasting from Base

**The Mistake:**
```rust
// extension/lib.rs copies helpers from base
pub fn setup_database() { /* duplicated */ }
pub fn teardown() { /* duplicated */ }
// When base changes, copies diverge immediately
```

**The Fix:**
```rust
// Wrap, don't copy
pub fn setup_domain() -> DomainFixture {
    let base = BaseFixture::new();  // Inherit base behavior
    DomainFixture::new(base)        // Wrap it
}
```

**Why It Matters:** Duplicated code diverges immediately. Wrappers stay in sync.

---

### Pattern 8: Composition - Duplicating Builder Logic

**The Mistake:**
```rust
pub struct OrderBuilder {
    fields: HashMap<String, String>,
}
impl OrderBuilder {
    pub fn build(self) -> serde_json::Value {
        // Duplicated serialization logic
    }
}
```

**The Fix:**
```rust
pub struct OrderBuilder {
    base: TestDataBuilder,  // Reuse
}
impl OrderBuilder {
    pub fn build(self) -> serde_json::Value {
        self.base.build_json().expect("valid")  // Delegation
    }
}
```

**Why It Matters:** Composition inherits base improvements. Copies require manual synchronization.

---

### Pattern 9: Single Source - Duplicate Constants

**The Mistake:**
```rust
// src/core/macros/test.rs
const TIMEOUT_SECS: u64 = 30;

// src/validation/guards.rs (forgot this uses same timeout)
const TIMEOUT_SECS: u64 = 30;  // Will diverge!
```

**The Fix:**
```rust
// src/core/macros/test.rs (single source)
pub const DEFAULT_TEST_TIMEOUT_SECS: u64 = 30;

// src/validation/guards.rs (imports)
use chicago_tdd_tools::core::macros::DEFAULT_TEST_TIMEOUT_SECS;
```

**Why It Matters:** "Same value" isn't "same source." Copies diverge when someone forgets to update both.

---

### Pattern 10: Capability Groups - Alphabetical or Implementation-Based Organization

**The Mistake:**
```rust
pub mod builders;      // Alphabetical (hard to find things)
pub mod core;
pub mod fixtures;
pub mod guards;
pub mod observability;
pub mod testing;
```

**The Fix:**
```rust
pub mod core;          // Fixtures, builders, assertions
pub mod testing;       // Advanced techniques
pub mod validation;    // Guardrails
pub mod observability; // Telemetry
pub mod integration;   // External systems
```

**Why It Matters:** Users care about capabilities, not implementation. Capability grouping helps them find what they need.

---

## Design Patterns

### Pattern 11: Zero-Cost - Using Trait Objects in Hot Path

**The Mistake:**
```rust
pub fn process<T>(item: &dyn Handler<T>) {  // vtable lookup every call
    item.handle();
}
```

**The Fix:**
```rust
pub fn process<T, H: Handler<T>>(item: &H) {  // Inlined, no vtable
    item.handle();
}
```

**Why It Matters:** Generics monomorphize at compile time, producing identical code to hand-written specialization.

---

### Pattern 12: Type Safety (GATs) - Forgetting Lifetime Binding

**The Mistake:**
```rust
type Fixture<'a>;  // 'a is free; can outlive provider
```

**The Fix:**
```rust
type Fixture<'a> where Self: 'a;  // 'a cannot exceed self's lifetime
```

**Why It Matters:** Without binding 'a to Self, the lifetime is unconstrained and references can escape.

---

### Pattern 13: Sealed Traits - Making Sealed Public

**The Mistake:**
```rust
pub mod private { pub trait Sealed {} }  // Wrong! Sealed is public
pub trait AsyncFixtureProvider {  // Anyone can implement
    // ...
}
```

**The Fix:**
```rust
mod private { pub trait Sealed {} }  // Private module
pub trait AsyncFixtureProvider: private::Sealed {  // Require Sealed
    // ...
}
```

**Why It Matters:** Sealed must be private to prevent external implementations. Public Sealed defeats the pattern.

---

### Pattern 14: Compile-Time - Validating at Runtime What Should Be Compile-Time

**The Mistake:**
```rust
pub fn process<T>(data: T, max_size: usize) {
    if std::mem::size_of::<T>() > max_size {
        panic!("Size too large");  // Could have failed at compile time!
    }
}
```

**The Fix:**
```rust
pub fn process<T, const MAX: usize>(data: T) {
    const_assert!(std::mem::size_of::<T>() <= MAX);  // Compile error if violated
}
```

**Why It Matters:** Compile-time validation is zero-cost and impossible to bypass.

---

### Pattern 15: Type State - Allowing Multiple Methods on Same Type

**The Mistake:**
```rust
impl TestState<Arrange> {
    pub fn act(self) -> TestState<Act> { /* ... */ }
    pub fn assert(&mut self) { /* ... */ }  // Can call without going through Act!
}
```

**The Fix:**
```rust
impl TestState<Arrange> {
    pub fn act(self) -> TestState<Act> { /* ... */ }
}

impl TestState<Act> {
    pub fn assert(self) -> TestState<Assert> { /* ... */ }
}
```

**Why It Matters:** Each type must only expose valid next states. The compiler enforces order.

---

### Pattern 16: Fixture Lifecycle - Cleanup After Assertions

**The Mistake:**
```rust
async_test!(test_bad, {
    let db = Database::connect().await?;
    let result = db.query().await?;
    assert_eq!(result, 42);  // If this fails, cleanup never runs!
    db.close().await;
});
```

**The Fix:**
```rust
async_test!(test_good, fixture, {
    let db = fixture.get_database().await?;
    let result = db.query().await?;
    assert_eq!(result, 42);  // Cleanup happens regardless
    Ok(())
});
```

**Why It Matters:** Explicit cleanup is bypassed by panics. Fixtures guarantee cleanup via Drop.

---

### Pattern 17: Builder - Duplicating Builder Logic

**The Mistake:**
```rust
pub struct OrderBuilder {
    fields: HashMap<String, String>,
}
impl OrderBuilder {
    pub fn build(self) -> serde_json::Value {
        // Duplicated builder logic
    }
}
```

**The Fix:**
```rust
pub struct OrderBuilder {
    base: TestDataBuilder,  // Reuse
}
impl OrderBuilder {
    pub fn build(self) -> serde_json::Value {
        self.base.build_json().expect("valid")
    }
}
```

**Why It Matters:** Composition keeps you in sync. Duplication creates maintenance burden.

---

### Pattern 18: Timeout Defense - Single Layer Timeout

**The Mistake:**
```rust
async fn test_slow_op() {  // No test-level timeout, only process-level
    // If this hangs, unclear which test stalled
}
```

**The Fix:**
```rust
fixture_test_with_timeout!(test, fixture, 30, {  // Test-level
    let result = container.query().await?;
    Ok(())
});
// Plus runner-level: nextest.toml with profile timeouts
// Plus process-level: Makefile.toml emergency stop
```

**Why It Matters:** Layered timeouts provide defense in depth. Each layer catches what lower layers miss.

---

### Pattern 19: Feature Gates - Too Many Fine-Grained Features

**The Mistake:**
```toml
[features]
foo = ["a"]
bar = ["b"]
baz = ["c", "d"]
# User confusion: which one to enable?
```

**The Fix:**
```toml
[features]
testing-extras = ["property-testing", "snapshot-testing", "fake-data"]
observability-full = ["otel", "weaver"]
# Clear purpose, sensible combinations
```

**Why It Matters:** Too many choices is paralyzing. Curated slices guide users.

---

### Pattern 20: Macro Enforcement - Over-Engineered Macros

**The Mistake:**
```rust
macro_rules! test_with_structure {
    ($name:ident, $arrange:block, $act:block, $assert:block) => {
        // Syntax is awkward; users won't use it
    }
}
```

**The Fix:**
```rust
macro_rules! async_test {
    ($name:ident, $body:block) => {
        #[tokio::test]
        async fn $name() {
            tokio::time::timeout(Duration::from_secs(1), async { $body }).await
        }
    };
}
```

**Why It Matters:** Macros are best for enforcing framework requirements (timeouts, features), not code structure.

---

## Summary: Most Common Mistakes by Category

| Category | Most Common | Fix |
|----------|-------------|-----|
| Testing | Mixing Arrange/Act; not checking error type | Separate phases; assert error variant |
| Architecture | Copying code instead of composing | Wrap primitives instead of copying |
| Design | Validating at runtime what could be compile-time | Use const generics, type system |

**Golden Rule:** If you can enforce something at compile time, do it. It's free and impossible to bypass.
