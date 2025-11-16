# Real-World Scenarios: Patterns in Action

This guide shows how patterns combine to solve actual problems you'll encounter.

> **Want to learn patterns in order?** See [Pattern Dependencies & Learning Order](pattern-dependencies.md) for recommended learning paths.

---

## Scenario 1: Building a Database Query Service

**The Problem:** You're writing a database abstraction that needs to be:
- Easy to test with real and fake databases
- Type-safe (queries are validated at compile time)
- Performant (no runtime dispatch overhead)
- Extensible (teams add domain-specific queries)

**Patterns Used:**

1. **Pattern 6: Generic Base Layer**
   - Core: `TestFixture`, `TestDataBuilder`, assertion macros
   - No database-specific code in base

2. **Pattern 7: Extension Layer**
   - Create `database-testing` crate that depends on base
   - Add `DatabaseFixture` wrapping `TestFixture`
   - Provide `QueryBuilder` for fluent query construction

3. **Pattern 5: Real Collaborators**
   - Integration tests use actual database (Postgres in container)
   - Unit tests use fake database
   - Both follow same interface

4. **Pattern 11: Zero-Cost Abstractions**
   - Query validation via generics, not trait objects
   - Query builder uses const generics for compile-time table names
   - No runtime cost for type safety

5. **Pattern 14: Compile-Time Validation**
   - Invalid queries fail at compile time
   - Column names checked against schema
   - SQL injection prevented by construction

6. **Pattern 1: AAA Pattern**
   ```rust
   fixture_test!(test_query, fixture, {
       // Arrange: Set up database with known data
       let db = fixture.postgres_database().await?;
       db.insert_user("alice", 30).await?;

       // Act: Execute the query
       let user = db.find_user("alice").await?;

       // Assert: Verify result
       assert_eq!(user.name, "alice");
       assert_eq!(user.age, 30);
       Ok(())
   });
   ```

**Result:** Type-safe, fast database tests that catch bugs before production.

---

## Scenario 2: Designing an Event System with Extensibility

**The Problem:** You're building an event system where:
- Multiple teams add custom event handlers
- Handlers must follow consistent patterns
- You can't let downstream code break invariants
- Code must be performant

**Patterns Used:**

1. **Pattern 13: Sealed Traits**
   - Public `EventHandler` trait is sealed
   - Only framework can implement
   - Safe to evolve without breaking users

2. **Pattern 12: Type Safety (GATs)**
   - Handler lifetime is bound to registry lifetime
   - Compiler prevents handlers outliving registry
   - No use-after-free bugs possible

3. **Pattern 11: Zero-Cost Abstractions**
   - Event routing uses generics, not dynamic dispatch
   - Each event type gets specialized handler code
   - No vtable lookup overhead

4. **Pattern 15: Type State Enforcement**
   - Event lifecycle: Created → Dispatching → Done
   - Can't call methods out of order
   - Type system enforces flow

5. **Pattern 8: Composition Over Duplication**
   - Teams create `domain-events` crate
   - Wrap base `EventRegistry`
   - Add domain-specific event types

6. **Pattern 20: Macro Enforcement**
   ```rust
   #[event_handler]  // Macro enforces pattern compliance
   async fn on_user_created(event: UserCreated) -> Result<()> {
       // Macro ensures: timeout, error handling, logging
       Ok(())
   }
   ```

**Result:** Extensible event system with compile-time safety and zero overhead.

---

## Scenario 3: Testing a Complex Business Flow

**The Problem:** You're testing an order processing system with:
- Multiple error paths (payment fails, inventory out, shipping unavailable)
- Complex state transitions (pending → processing → shipped → delivered)
- Real external services (payment processor, shipping provider)
- Builders for complex test data

**Patterns Used:**

1. **Pattern 1: AAA Pattern**
   - Every test follows clear structure
   - Easy to add new tests as requirements change

2. **Pattern 2: Error Path Testing**
   ```rust
   param_test! {
       #[case(PaymentError::InvalidCard)]
       #[case(PaymentError::InsufficientFunds)]
       #[case(PaymentError::Timeout)]
       fn test_payment_errors(error: PaymentError) {
           // Each error has its own test
           // Guaranteed all errors are tested
       }
   }
   ```

3. **Pattern 3: Boundary Conditions**
   - Order total: below limit, at limit, above limit
   - Quantity: 0, 1, max
   - Test edge cases explicitly

4. **Pattern 5: Real Collaborators**
   - Integration tests use testcontainers (real Postgres, Stripe sandbox)
   - Unit tests use mocked services
   - No surprises in production

5. **Pattern 17: Builder-Driven Test Data**
   ```rust
   let order = OrderBuilder::new()
       .with_customer_id("cust-123")
       .with_amount("99.99")
       .with_items(vec![
           ItemBuilder::new().with_sku("BOOK").with_qty(2),
           ItemBuilder::new().with_sku("PEN").with_qty(10),
       ])
       .build();
   ```

6. **Pattern 15: Type State Enforcement**
   - Order state machine: New → Pending → Confirmed → Shipped → Delivered
   - Can't call methods out of order
   - Compiler enforces valid state transitions

7. **Pattern 16: Fixture Lifecycle**
   - Fixtures auto-cleanup: database rolled back, containers stopped
   - No manual cleanup code
   - Works even if test fails

**Result:** Comprehensive test suite that documents business logic and catches regressions.

---

## Scenario 4: Open-Sourcing a Testing Framework

**The Problem:** You're open-sourcing a testing framework and need:
- Lean core (not every project needs every feature)
- Extensible (teams add domain-specific helpers)
- Fast builds (heavy dependencies optional)
- Type-safe APIs

**Patterns Used:**

1. **Pattern 6: Generic Base Layer**
   - Core: `test!`, `async_test!`, fixtures, assertions
   - No domain logic, minimal dependencies

2. **Pattern 10: Capability Grouping**
   ```
   core/          // fixtures, builders, assertions, macros
   testing/       // property, mutation, snapshot testing
   validation/    // guards, coverage, performance
   observability/ // telemetry (OTEL, Weaver)
   integration/   // containers (testcontainers)
   ```

3. **Pattern 19: Feature Gate Slices**
   ```toml
   [features]
   default = ["logging"]
   testing-extras = ["property-testing", "snapshot-testing"]
   observability-full = ["otel", "weaver"]
   ```

4. **Pattern 20: Macro Enforcement**
   - `test!` injects AAA comments and timeouts
   - Using macro guarantees compliance with patterns
   - `property_test!` requires `property-testing` feature

5. **Pattern 9: Single Source of Truth**
   ```rust
   pub const DEFAULT_UNIT_TEST_TIMEOUT_SECONDS: u64 = 1;
   pub const DEFAULT_INTEGRATION_TEST_TIMEOUT_SECONDS: u64 = 30;
   // Nextest.toml references these constants
   // CI ensures consistency
   ```

6. **Pattern 7: Extension Layer**
   - Users create `myteam-testing` crate
   - Depends on `chicago-tdd-tools`
   - Adds domain-specific builders and fixtures

**Result:** Framework that scales from single team to community, staying lean and extensible.

---

## Scenario 5: Refactoring Legacy Code to Add Type Safety

**The Problem:** You have legacy code that:
- Works but has hidden bugs
- Hard to refactor without breaking things
- Needs gradual type-safety improvements

**Patterns Used:**

1. **Pattern 15: Type State Enforcement**
   - Introduce phase types gradually
   - Existing code continues to work
   - New code is type-checked

2. **Pattern 14: Compile-Time Validation**
   - Add const generics for numeric limits
   - Migration: accept both old and new paths
   - Gradually move to compile-time checks

3. **Pattern 13: Sealed Traits**
   - Seal critical traits
   - Prevents downstream code from breaking invariants
   - Safe to refactor internal implementation

4. **Pattern 12: Type Safety (GATs)**
   - New APIs use GATs for lifetime safety
   - Old APIs continue working
   - Gradual migration path

5. **Pattern 9: Single Source of Truth**
   - Extract constants from scattered code
   - Centralize configuration
   - Version it in git

**Result:** Gradually add type safety to legacy systems without big-bang rewrites.

---

## When to Use Each Scenario

| Scenario | Situation | Key Takeaway |
|----------|-----------|--------------|
| 1: Database Service | Building domain-specific abstraction | Combine base + extension + real collaborators |
| 2: Event System | Need extensibility without breaking safety | Sealed traits + type state + generics |
| 3: Business Logic | Complex flows with multiple error paths | AAA + error paths + boundaries + builders |
| 4: Open-Source | Lean core that scales | Feature gates + capability groups + macros |
| 5: Legacy Refactor | Add type safety to existing code | Gradual migration using type state + const |

---

## The Pattern Combination Strategy

**For testing:**
1. Start with Pattern 1 (AAA)
2. Add Pattern 2 (error paths)
3. Add Pattern 5 (real collaborators)
4. Add Pattern 17 (builders) as tests grow

**For architecture:**
1. Start with Pattern 10 (capability groups)
2. Add Pattern 6 (generic base)
3. Add Pattern 7 (extensions) when team grows
4. Add Pattern 9 (single source) when constants multiply

**For safety:**
1. Use Pattern 14 (compile-time) for invariants
2. Add Pattern 15 (type state) for workflows
3. Add Pattern 13 (sealed traits) for APIs
4. Add Pattern 12 (GATs) for lifetime safety

**Pro tip:** Don't use all patterns at once. Start with the ones that solve your current problem, then add others as complexity grows.
