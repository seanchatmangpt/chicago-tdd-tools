# All Patterns: Quick Reference Card

> 🔧 **ALL PATTERNS: QUICK REFERENCE CARD** | **How-to** | Solve this problem

> 📚 **REFERENCE** | All 20 patterns at a glance

Quick lookup for all patterns. Use this to find a pattern and jump to its full description.

> **Want a learning order?** See [Pattern Dependencies & Learning Order](pattern-dependencies.md) for recommended paths.

---

## Testing Patterns (5)

| # | Pattern | Problem | Solution | Learn More |
|-|---------|---------|----------|-----------|
| 1 | **AAA Pattern** | Tests are unreadable | Structure into Arrange-Act-Assert | [→](testing-patterns/aaa-pattern.md) |
| 2 | **Error Path Testing** | Failures aren't tested | Test both success AND error paths | [→](testing-patterns/error-path-testing.md) |
| 3 | **Boundary Conditions** | Edge cases are missed | Systematically test limits | [→](testing-patterns/boundary-conditions.md) |
| 4 | **Resource Cleanup** | Tests leak resources | Automatic fixture cleanup | [→](testing-patterns/resource-cleanup.md) |
| 5 | **Real Collaborators** | Mocks hide bugs | Test with real implementations | [→](testing-patterns/real-collaborators.md) |

**Key**: These 5 patterns are the **foundation**. Use them in every test.

---

## Architecture Patterns (5)

| # | Pattern | Problem | Solution | Learn More |
|-|---------|---------|----------|-----------|
| 6 | **Generic Base Layer** | Code duplication | Extract generic abstractions | [→](architecture-patterns/generic-base.md) |
| 7 | **Extension Layer** | Can't extend without modifying | Add layers for safe extension | [→](architecture-patterns/extension-layer.md) |
| 8 | **Composition Over Duplication** | DRY violations | Compose instead of copying | [→](architecture-patterns/composition-over-duplication.md) |
| 9 | **Single Source of Truth** | Data inconsistencies | One canonical source | [→](architecture-patterns/single-source-of-truth.md) |
| 10 | **Capability Grouping** | Monolithic modules | Organize by capability | [→](architecture-patterns/capability-groups.md) |

**Key**: These 5 patterns organize code structure. Use during architecture phase.

---

## Design Patterns (10)

| # | Pattern | Problem | Solution | Learn More |
|-|---------|---------|----------|-----------|
| 11 | **Zero-Cost Abstractions** | Abstractions are slow | Use generics, compile away overhead | [→](design-patterns/zero-cost-abstractions.md) |
| 12 | **Type Safety with GATs** | Type errors at runtime | Generic Associated Types | [→](design-patterns/type-safety-patterns.md) |
| 13 | **Sealed Traits** | API is too easy to misuse | Seal traits to prevent misuse | [→](design-patterns/sealed-traits.md) |
| 14 | **Compile-Time Validation** | Errors caught at runtime | Validate during compilation | [→](design-patterns/compile-time-validation.md) |
| 15 | **Type State Enforcement** | State machines are error-prone | Encode states in the type system | [→](design-patterns/type-state-pattern.md) |
| 16 | **Fixture Lifecycle** | Complex test setup | Manage with sealed traits | [→](design-patterns/fixture-lifecycle.md) |
| 17 | **Builder-Driven Test Data** | Building test data is tedious | Fluent builders for data | [→](design-patterns/builder-test-data.md) |
| 18 | **Timeout Defense** | Tests hang indefinitely | Timeout defense in depth | [→](design-patterns/timeout-defense.md) |
| 19 | **Feature Gate Slices** | Feature flags are unreliable | Slice-based feature gating | [→](design-patterns/feature-gating.md) |
| 20 | **Macro Pattern Enforcement** | Patterns are easy to violate | Use macros to enforce | [→](design-patterns/macro-enforcement.md) |

**Key**: These 10 patterns provide safety, performance, and design tools. Use during implementation.

---

## Pattern Organization

### By Complexity (Learning Path)

**Phase 1 - Foundation (Read First)**
1. Pattern 1: AAA Pattern
2. Pattern 2: Error Path Testing
3. Pattern 3: Boundary Conditions

**Phase 2 - Production Ready (Read Next)**
4. Pattern 4: Resource Cleanup
5. Pattern 5: Real Collaborators
6. Pattern 17: Builder-Driven Test Data

**Phase 3 - Architecture (Advanced)**
6. Pattern 8: Composition Over Duplication
7. Pattern 10: Capability Grouping
8. Pattern 9: Single Source of Truth

**Phase 4 - Advanced Design (Mastery)**
11. Pattern 11: Zero-Cost Abstractions
12. Pattern 13: Sealed Traits
13. Pattern 15: Type State Enforcement
14. Pattern 20: Macro Pattern Enforcement

---

### By Category (Type System)

**Testing Patterns**: Patterns 1-5
**Architecture Patterns**: Patterns 6-10
**Design Patterns**: Patterns 11-20

---

### By Problem Domain

**Testing Problems**: Patterns 1-5, 17
**Code Organization**: Patterns 6-10
**Type Safety**: Patterns 12, 14, 15
**API Design**: Patterns 13, 20
**Performance**: Pattern 11
**Robustness**: Pattern 18
**Reliability**: Pattern 19

---

## How to Use This Card

1. **Find your problem** in the Problem column
2. **See the solution** in the Solution column
3. **Click Learn More** to read the full pattern
4. **Bookmark** the pattern for future reference

---

## Quick Links

| Want to... | Go to... |
|-----------|---------|
| Choose a pattern | [Choosing Your Pattern](choosing-your-pattern.md) |
| Learn testing | [Testing Learning Sequence](tutorials/learning-testing-patterns.md) |
| Learn architecture | [Architecture Learning Sequence](tutorials/learning-architecture-patterns.md) |
| Learn design | [Design Learning Sequence](tutorials/learning-design-patterns.md) |
| All 20 patterns | This page (you are here) |

---

## Pattern Dependencies

Some patterns build on others. Recommended learning order:

```
Pattern 1 (AAA)
  ├─→ Pattern 2 (Error Paths)
  ├─→ Pattern 3 (Boundaries)
  ├─→ Pattern 4 (Resource Cleanup)
  └─→ Pattern 5 (Real Collaborators)
       └─→ Pattern 17 (Builder Test Data)

Pattern 6 (Generic Base)
  └─→ Pattern 8 (Composition)
       └─→ Pattern 10 (Capability Groups)

Pattern 14 (Compile-Time)
  └─→ Pattern 15 (Type State)

Pattern 13 (Sealed Traits)
  └─→ Pattern 20 (Macro Enforcement)
```

---

## Statistics

| Metric | Value |
|--------|-------|
| Total Patterns | 20 |
| Testing Patterns | 5 |
| Architecture Patterns | 5 |
| Design Patterns | 10 |
| Difficulty Range | Beginner → Advanced |
| Total Learning Time | ~10 hours |
| Estimated Implementation | 2-3 weeks |

---

## Pro Tips

💡 **Tip 1**: You don't need to learn all 20 patterns at once. Start with Testing (1-5), then add what you need.

💡 **Tip 2**: Patterns often appear in combinations. When you use Pattern 5 (Real Collaborators), you'll probably also use Pattern 17 (Builder Test Data).

💡 **Tip 3**: Look for patterns in the codebase you're reading. The more you see patterns, the better you'll understand them.

💡 **Tip 4**: Bookmark the [Decision Guide](choosing-your-pattern.md). You'll return to it when solving problems.

---

**Next**: Choose your learning path or jump to a pattern you need right now!


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
