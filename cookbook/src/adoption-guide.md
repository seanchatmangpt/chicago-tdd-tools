# Adoption Guide: Adding Patterns to Existing Projects

> 🔧 **ADOPTION GUIDE: ADDING PATTERNS TO EXISTING PROJECTS** | **How-to** | Solve this problem

> 🚀 **HOW-TO** | Add Chicago TDD patterns incrementally to your existing test suite

This guide helps you adopt patterns **gradually**, without breaking your existing tests.

---

## The 80/20 Adoption Path

**Minimum viable adoption (45 minutes):**

| Week | Pattern | Setup | Effort | Benefit |
|------|---------|-------|--------|---------|
| **Week 1 (Immediately)** | **1: AAA** | Restructure 3 tests | 30 min | Learn structure |
| | **6: Generic Base** | Review your organization | 15 min | Understand architecture |
| **Week 2** | **2: Error Paths** | Add error test case | 30 min | Catch 80% of bugs |
| | **5: Real Collaborators** | Replace 1 mock | 20 min | Integration confidence |
| **Week 3** | **10: Capability Groups** | Rename 1 module | 20 min | Better organization |

**Total time: 1 hour + 15 minutes = 75 minutes over 3 weeks**
**Result: 80% of framework value, can be done alongside regular work**

---

## Week 1: Foundation (45 minutes)

### Day 1-2: Learn Pattern 1 (AAA Pattern) - 30 minutes

**Goal:** Understand and apply AAA structure to your tests.

#### Step 1: Read Pattern 1 (10 min)

Go to [Pattern 1: AAA Pattern](testing-patterns/aaa-pattern.md) and read the "Essential Code Example" section.

#### Step 2: Identify Your Messiest Test (5 min)

Find the test that's hardest to read or debug. This is your pilot test.

**Example of messy test:**
```rust
#[test]
fn test_user_creation() {
    let mut db = Database::new();
    db.connect().unwrap();
    let user = User { name: "Alice".into() };
    let result = db.insert_user(&user);
    let stored = db.query_user("Alice").unwrap();
    assert_eq!(stored.name, "Alice");
    assert!(result.is_ok());
    db.disconnect().unwrap();
}
```

#### Step 3: Refactor to AAA (10 min)

Restructure your test:

```rust
#[test]
fn test_user_creation() {
    // Arrange: Set up database and test data
    let mut db = Database::new();
    db.connect().unwrap();
    let test_user = User { name: "Alice".into() };

    // Act: Create user
    let result = db.insert_user(&test_user);

    // Assert: Verify behavior
    assert_ok!(result);
    let stored_user = db.query_user("Alice").unwrap();
    assert_eq!(stored_user.name, "Alice");

    // Cleanup (we'll improve this later with Pattern 4)
    db.disconnect().unwrap();
}
```

**What changed?**
- ✅ Clear phases (Arrange, Act, Assert)
- ✅ One behavior per test (user creation)
- ✅ Easy to diagnose failures
- ✅ Easier to add error cases later

#### Day 3: Learn Pattern 6 (Generic Base) - 15 minutes

**Goal:** Understand how your code should be organized.

Read [Pattern 6: Generic Base Layer](architecture-patterns/generic-base.md) and ask: "Is my codebase organized this way?"

**Quick checklist:**
- ✅ Do I have a `core` or `base` module?
- ✅ Is domain logic separated from base utilities?
- ✅ Can I reuse the base in multiple contexts?

**Action:** Just observe. Don't refactor yet. This informs Pattern 10.

---

### Summary: Week 1 Achievement

✅ 3 tests now follow AAA pattern
✅ Understand architecture principle
✅ Learn difference between messes and structure

**Investment: 45 min** | **Payoff: Clearer tests**

---

## Week 2: Testing Power (50 minutes)

### Day 1-2: Add Error Path Testing - 30 minutes

**Goal:** Test error cases, catch 80% of bugs.

#### Step 1: Read Pattern 2 (5 min)

Go to [Pattern 2: Error Path Testing](testing-patterns/error-path-testing.md).

#### Step 2: Find a Success-Only Test (5 min)

Find a test that only tests the happy path. Example:

```rust
#[test]
fn test_parse_json() {
    let json = r#"{"name": "Alice"}"#;
    let user = parse_user_json(json).unwrap();
    assert_eq!(user.name, "Alice");
}
```

#### Step 3: Add Error Cases (15 min)

Test what happens when JSON is malformed:

```rust
#[test]
fn test_parse_json_success() {
    // Arrange
    let valid_json = r#"{"name": "Alice"}"#;

    // Act
    let result = parse_user_json(valid_json);

    // Assert
    assert_ok!(result);
    assert_eq!(result.unwrap().name, "Alice");
}

#[test]
fn test_parse_json_missing_field() {
    // Arrange: Invalid JSON (missing "name" field)
    let invalid_json = r#"{}"#;

    // Act
    let result = parse_user_json(invalid_json);

    // Assert: Should error, not panic
    assert_err!(result);
}

#[test]
fn test_parse_json_invalid_format() {
    // Arrange: Completely invalid JSON
    let bad_json = r#"{this is not json}"#;

    // Act
    let result = parse_user_json(bad_json);

    // Assert
    assert_err!(result);
}
```

**What you gained:**
- ✅ Caught error cases before production
- ✅ Verified code doesn't panic on bad input
- ✅ Found bugs in error handling

### Day 3: Replace One Mock with Real Collaborator - 20 minutes

**Goal:** Test with real dependencies, not mocks.

#### Current (Mock-Based) Test:

```rust
#[test]
fn test_user_service_with_mock() {
    // Arrange: Mock database
    let mock_db = MockDatabase::new();
    mock_db.expect_insert_user().return_ok();

    let service = UserService::new(Box::new(mock_db));

    // Act
    let result = service.create_user("Alice");

    // Assert
    assert_ok!(result);
}
```

**Problem:** Mock hides bugs. Real database might work differently.

#### New (Real Collaborator) Test:

```rust
#[test]
fn test_user_service_with_real_database() {
    // Arrange: Real in-memory or temporary database
    let db = Database::temporary(); // Creates temp DB, cleans up automatically
    let service = UserService::new(db);

    // Act
    let result = service.create_user("Alice");

    // Assert
    assert_ok!(result);

    // Verify it's actually stored (real DB behavior)
    let stored = db.query_user("Alice");
    assert_ok!(stored);
}
```

**What changed:**
- ✅ Testing real database behavior
- ✅ Catching integration bugs early
- ✅ More confidence before production

**Action:** Replace your most important mock with a real collaborator. Remove the mock test.

---

### Summary: Week 2 Achievement

✅ 2-3 error path tests added
✅ 1 mock replaced with real collaborator
✅ More bugs caught before production

**Investment: 50 min** | **Payoff: Better bug detection**

---

## Week 3: Organization (20 minutes)

### Organize Modules by Capability - 20 minutes

**Goal:** Make modules discoverable (Pattern 10).

#### Step 1: Read Pattern 10 (5 min)

Go to [Pattern 10: Capability Grouping](architecture-patterns/capability-groups.md).

#### Step 2: Review Your Module Structure (5 min)

Current (alphabetical):
```
src/
├── auth/
├── database/
├── handlers/
├── middleware/
├── models/
├── services/
├── utils/
```

#### Step 3: Reorganize by Capability (10 min)

New (by feature/capability):
```
src/
├── user_management/      ← Capability group
│   ├── auth.rs           (authentication for users)
│   ├── handlers.rs       (HTTP handlers for users)
│   ├── models.rs         (User types)
│   └── repository.rs     (Database access for users)
├── payment/              ← Capability group
│   ├── handlers.rs
│   ├── models.rs
│   └── repository.rs
└── shared/               ← Shared utilities
    ├── middleware/
    └── utils/
```

**Action:**
- Just rename one module directory
- Update `mod.rs` to reflect new structure
- Update import paths in tests

**Result:** New team members find related code faster.

---

### Summary: Week 3 Achievement

✅ One module reorganized
✅ Better code discoverability
✅ Foundation laid for scaling

**Investment: 20 min** | **Payoff: Better organization**

---

## Month 2: Advanced (Optional)

Once you've done Week 1-3, you're ready for:

- **Pattern 4: Resource Cleanup** - Automatic cleanup with Drop
- **Pattern 3: Boundary Conditions** - Test edge cases (off-by-one, empty, max)
- **Pattern 7-10: Architecture Patterns** - Extend your codebase structure
- **Patterns 11-20: Design Patterns** - Type safety, performance, validation

---

## Common Questions: Adoption

### Q: "Should I refactor all existing tests at once?"

**A:** No. Adopt incrementally:
1. New tests follow AAA pattern
2. Refactor old tests as you touch them
3. Focus on critical tests first

### Q: "What if my team pushes back?"

**A:** Start with Pattern 1 (AAA) only:
- Easier to read? ✅ Yes
- Harder to write? ✅ No
- Helps debugging? ✅ Yes

After 2 weeks, team sees value. Then add Pattern 2 (error paths).

### Q: "Can I use Pattern X without Y?"

**A:** Usually yes. See [Pattern Dependencies](pattern-dependencies.md):
- Pattern 2 (Error Paths) needs Pattern 1 (AAA) ✅
- Pattern 5 (Real Collaborators) is independent ✅
- Pattern 10 (Capability Groups) is independent ✅

### Q: "How do I measure improvement?"

**Track these metrics:**
- Test time to write (should decrease as patterns help)
- Bugs found in testing (should increase as coverage improves)
- Time to debug failures (should decrease with AAA)
- New team members time to productivity (should decrease with Pattern 10)

---

## Adoption Checklist: Month 1

### Week 1: Foundation ✅
- [ ] Read Pattern 1 (AAA)
- [ ] Refactor 3 tests to AAA
- [ ] Read Pattern 6 (Generic Base)
- [ ] Review your module structure

### Week 2: Testing ✅
- [ ] Read Pattern 2 (Error Paths)
- [ ] Add error test cases (at least 2)
- [ ] Replace 1 mock with real collaborator
- [ ] Verify real database behavior

### Week 3: Organization ✅
- [ ] Read Pattern 10 (Capability Groups)
- [ ] Rename 1 module to capability-based name
- [ ] Update imports
- [ ] Run all tests to verify

### Month 2: Ready for Advanced ✅
- [ ] Read 2-3 architecture patterns
- [ ] Plan next pattern adoption
- [ ] Share learnings with team

---

## Pitfalls: What NOT to Do

### ❌ Don't: Adopt all patterns at once
- "We'll do AAA, Pattern 2, Pattern 5, Pattern 10 this week!"
- Result: Team overwhelmed, adoption fails

### ✅ Do: One pattern per week
- Week 1: AAA (1 hour)
- Week 2: Error Paths (1 hour)
- Week 3: Real Collaborators (1 hour)

### ❌ Don't: Refactor all tests at once
- "Let's refactor 200 tests to AAA!"
- Result: Hours of work, high risk of breakage

### ✅ Do: Refactor incrementally
- New tests use AAA from now on
- Old tests refactored as you touch them

### ❌ Don't: Use patterns you don't understand
- "Let's use Pattern 15 (Type State)!"
- Result: Over-engineered, hard to maintain

### ✅ Do: Master foundations first
- Patterns 1, 6, 2, 5, 10 = 80% value
- Then add specialized patterns

---

## Success Stories: Real Adoption

### Company A: Started with Pattern 1

"We adopted AAA in Week 1. Tests became 30% easier to read. Debugging failures 50% faster. Week 2, team asked for Pattern 2. By Month 2, we had error path tests everywhere."

**Timeline:** 1 week to see value

### Company B: Started with Pattern 5

"We had 80% mock-based tests. In Week 2, we replaced mocks with real PostgreSQL containers. Found 3 integration bugs immediately. Would've reached production."

**Timeline:** 2 weeks to catch production bugs

### Company C: Started with Pattern 10

"Module organization was chaos. Week 3, we reorganized by capability. New hires now find code 50% faster. Less duplicate code."

**Timeline:** 3 weeks to improve productivity

---

## Next Steps

1. **This week:** Read [Pattern 1: AAA](testing-patterns/aaa-pattern.md)
2. **Next week:** Read [Pattern 2: Error Paths](testing-patterns/error-path-testing.md)
3. **Week 3:** Read [Pattern 10: Capability Groups](architecture-patterns/capability-groups.md)
4. **Month 2:** See [Pattern Dependencies](pattern-dependencies.md) for what to learn next

**Total adoption time: 3 hours over 4 weeks = 45 min/week**

---

## Questions? Troubleshooting?

See [Troubleshooting Guide](troubleshooting.md) for:
- "Tests still failing after refactoring"
- "Mocks vs real collaborators confusion"
- "How to measure improvement"
- "Team adoption resistance"


## Quick Glance

| Aspect | Details |
|--------|---------|
| Problem | Addressed by this pattern |
| Solution | Core idea in 1-2 sentences |
| When To Use | Typical scenarios |
| When NOT To Use | Anti-patterns |
| Trade-offs | What you gain/lose |
| Complexity | Low/Medium/High |
| Real-World Example | Link to actual code |
