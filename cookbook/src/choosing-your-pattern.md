# Choosing Your Pattern

> 🗺️ **NAVIGATION** | Find the right pattern for your testing problem

This guide helps you choose the right pattern(s) based on your testing situation.

---

## Quick Pattern Finder

**What's your challenge?** Find it in the table below:

| Your Challenge | Pattern Family | Pattern Name | Go To |
|--|--|--|--|
| How do I structure a test? | Testing | Pattern 1: AAA Pattern | [Learn](testing-patterns/aaa-pattern.md) |
| How do I test error cases? | Testing | Pattern 2: Error Path Testing | [Learn](testing-patterns/error-path-testing.md) |
| How do I test edge cases? | Testing | Pattern 3: Boundary Conditions | [Learn](testing-patterns/boundary-conditions.md) |
| How do I clean up resources? | Testing | Pattern 4: Resource Cleanup | [Learn](testing-patterns/resource-cleanup.md) |
| Should I use mocks? | Testing | Pattern 5: Real Collaborators | [Learn](testing-patterns/real-collaborators.md) |
| How do I organize my code? | Architecture | Pattern 6: Generic Base Layer | [Learn](architecture-patterns/generic-base.md) |
| How do I extend without duplicating? | Architecture | Pattern 7: Extension Layer | [Learn](architecture-patterns/extension-layer.md) |
| How do I avoid code duplication? | Architecture | Pattern 8: Composition Over Duplication | [Learn](architecture-patterns/composition-over-duplication.md) |
| How do I avoid data inconsistency? | Architecture | Pattern 9: Single Source of Truth | [Learn](architecture-patterns/single-source-of-truth.md) |
| How do I organize large modules? | Architecture | Pattern 10: Capability Grouping | [Learn](architecture-patterns/capability-groups.md) |
| How do I optimize for performance? | Design | Pattern 11: Zero-Cost Abstractions | [Learn](design-patterns/zero-cost-abstractions.md) |
| How do I prevent type errors? | Design | Pattern 12: Type Safety with GATs | [Learn](design-patterns/type-safety-patterns.md) |
| How do I prevent API misuse? | Design | Pattern 13: Sealed Traits | [Learn](design-patterns/sealed-traits.md) |
| How do I validate at compile-time? | Design | Pattern 14: Compile-Time Validation | [Learn](design-patterns/compile-time-validation.md) |
| How do I enforce state machines? | Design | Pattern 15: Type State Enforcement | [Learn](design-patterns/type-state-pattern.md) |
| How do I manage fixture lifecycle? | Design | Pattern 16: Fixture Lifecycle Management | [Learn](design-patterns/fixture-lifecycle.md) |
| How do I build test data easily? | Design | Pattern 17: Builder-Driven Test Data | [Learn](design-patterns/builder-test-data.md) |
| How do I prevent timeouts? | Design | Pattern 18: Timeout Defense | [Learn](design-patterns/timeout-defense.md) |
| How do I manage feature flags? | Design | Pattern 19: Feature Gate Slices | [Learn](design-patterns/feature-gating.md) |
| How do I enforce patterns with macros? | Design | Pattern 20: Macro Pattern Enforcement | [Learn](design-patterns/macro-enforcement.md) |

---

## By Category

### Testing Patterns: "How Do I Write Better Tests?"

These patterns solve fundamental testing problems:

| Pattern | Problem | Solution |
|---------|---------|----------|
| **Pattern 1: AAA** | Tests are hard to read | Structure: Arrange, Act, Assert |
| **Pattern 2: Error Paths** | I don't test failures | Test both success and error cases |
| **Pattern 3: Boundaries** | I miss edge cases | Systematically test limits |
| **Pattern 4: Resource Cleanup** | Tests leak resources | Automatic fixture cleanup |
| **Pattern 5: Real Collaborators** | Mocks hide integration bugs | Test with real implementations |

**When to use**: All the time. These are foundational.

**Learning Path**: [Testing Patterns Learning Sequence](tutorials/learning-testing-patterns.md)

---

### Architecture Patterns: "How Do I Organize Code?"

These patterns solve structural problems:

| Pattern | Problem | Solution |
|---------|---------|----------|
| **Pattern 6: Generic Base** | Code is duplicate | Extract generic abstractions |
| **Pattern 7: Extension Layer** | I can't extend without modifying | Add layers for extensions |
| **Pattern 8: Composition** | DRY violations everywhere | Compose instead of duplicating |
| **Pattern 9: Single Source** | Data gets out of sync | One canonical source of truth |
| **Pattern 10: Capability Groups** | Module is too large | Organize by capability, not type |

**When to use**: During architecture phase and refactoring.

**Learning Path**: [Architecture Patterns Learning Sequence](tutorials/learning-architecture-patterns.md)

---

### Design Patterns: "How Do I Make Code Safer?"

These patterns solve design and safety problems:

| Pattern | Problem | Solution |
|---------|---------|----------|
| **Pattern 11: Zero-Cost** | Abstractions are slow | Zero-cost abstractions via generics |
| **Pattern 12: Type Safety** | Type errors at runtime | Use GATs for safety |
| **Pattern 13: Sealed Traits** | API is too easy to misuse | Seal traits to prevent misuse |
| **Pattern 14: Compile-Time** | Errors caught at runtime | Validate at compile-time |
| **Pattern 15: Type State** | State machines are error-prone | Encode states in types |
| **Pattern 16: Fixture Lifecycle** | Test setup is complex | Manage lifecycle with traits |
| **Pattern 17: Builder Test Data** | Building test data is tedious | Fluent builders for test data |
| **Pattern 18: Timeout Defense** | Tests hang forever | Timeout defense in depth |
| **Pattern 19: Feature Gates** | Feature flags are unreliable | Gate slices across codebase |
| **Pattern 20: Macro Enforcement** | Patterns are easy to violate | Use macros to enforce patterns |

**When to use**: During design and implementation.

**Learning Path**: [Design Patterns Learning Sequence](tutorials/learning-design-patterns.md)

---

## Decision Trees

### "I'm writing a test. Which pattern do I need?"

```
┌─ Start: Writing a test
│
├─ What am I testing?
│  ├─ Normal behavior ──→ Pattern 1: AAA Pattern
│  ├─ Error behavior ──→ Pattern 2: Error Path Testing
│  ├─ Edge cases ──→ Pattern 3: Boundary Conditions
│  └─ Setup/teardown ──→ Pattern 4: Resource Cleanup
│
├─ What should I test against?
│  ├─ Mock/fake ──→ Consider Pattern 5: Real Collaborators
│  └─ Real implementation ──→ Pattern 5: Real Collaborators ✓
│
└─ How do I build test data?
   └─ Complex data ──→ Pattern 17: Builder-Driven Test Data
```

---

### "I'm designing an architecture. Which patterns apply?"

```
┌─ Start: Designing architecture
│
├─ How do I organize modules?
│  ├─ By type (models, handlers, etc.) ──→ Consider Pattern 10: Capability Groups
│  └─ By capability ──→ Pattern 10: Capability Groups ✓
│
├─ How do I reuse code?
│  ├─ Copy-paste ──→ NO! Use Pattern 8: Composition Over Duplication
│  └─ Abstract base ──→ Pattern 6: Generic Base Layer
│
├─ How do I extend without modifying?
│  └─ Pattern 7: Extension Layer
│
└─ Where is the source of truth?
   └─ Pattern 9: Single Source of Truth
```

---

### "I'm designing APIs. Which patterns keep them safe?"

```
┌─ Start: Designing public API
│
├─ Can downstream code misuse my API?
│  └─ YES ──→ Pattern 13: Sealed Traits
│
├─ Should errors be compile-time or runtime?
│  ├─ Compile-time ──→ Pattern 14: Compile-Time Validation
│  └─ Runtime ──→ Less safe, but sometimes necessary
│
├─ Does state machine matter?
│  ├─ YES (auth states, connection states) ──→ Pattern 15: Type State Enforcement
│  └─ NO ──→ Continue
│
├─ Are lifetimes complex?
│  └─ YES ──→ Pattern 12: Type Safety with GATs
│
└─ Should this be in macros?
   └─ Pattern 20: Macro Pattern Enforcement
```

---

## Learning by Difficulty

### Beginner (Start Here)

1. **Pattern 1: AAA Pattern** - Foundation
2. **Pattern 2: Error Path Testing** - See what not to do
3. **Pattern 3: Boundary Conditions** - Edge cases matter
4. **Pattern 4: Resource Cleanup** - Don't leak resources
5. **Pattern 5: Real Collaborators** - Test with real code

**Time**: ~3 hours | **Content**: Testing fundamentals

### Intermediate (Build on Basics)

6. **Pattern 6: Generic Base Layer** - Code organization
7. **Pattern 8: Composition Over Duplication** - DRY principle
8. **Pattern 10: Capability Grouping** - Module organization
9. **Pattern 17: Builder-Driven Test Data** - Practical testing
10. **Pattern 14: Compile-Time Validation** - Type safety

**Time**: ~4 hours | **Content**: Architecture and safety

### Advanced (Master the Craft)

11. **Pattern 11: Zero-Cost Abstractions** - Performance
12. **Pattern 12: Type Safety with GATs** - Advanced types
13. **Pattern 15: Type State Enforcement** - State machines
14. **Pattern 13: Sealed Traits** - API design
15. **Pattern 18: Timeout Defense** - Robustness

**Time**: ~5 hours | **Content**: Advanced design and optimization

---

## Pattern Combination Guide

### "I want to write production-quality tests"

Use these patterns together:
1. **Pattern 1: AAA** - Structure your tests
2. **Pattern 2: Error Paths** - Test failures
3. **Pattern 3: Boundaries** - Test edge cases
4. **Pattern 4: Resource Cleanup** - Clean automatically
5. **Pattern 5: Real Collaborators** - Use real dependencies
6. **Pattern 17: Builder Test Data** - Build complex test data

**Expected outcome**: Comprehensive, maintainable test suite

### "I want to build a safe, extensible API"

Use these patterns together:
1. **Pattern 6: Generic Base** - Reusable abstractions
2. **Pattern 7: Extension Layer** - Allow extensibility
3. **Pattern 13: Sealed Traits** - Prevent misuse
4. **Pattern 14: Compile-Time Validation** - Validate early
5. **Pattern 15: Type State** - Enforce state machines
6. **Pattern 20: Macro Enforcement** - Enforce usage patterns

**Expected outcome**: Safe, extensible, hard-to-misuse API

### "I want maximum performance"

Use these patterns together:
1. **Pattern 11: Zero-Cost Abstractions** - Generic dispatch
2. **Pattern 12: Type Safety with GATs** - Type-safe lifetimes
3. **Pattern 14: Compile-Time Validation** - Zero runtime checks
4. **Pattern 8: Composition Over Duplication** - Avoid copies

**Expected outcome**: Fast code with safety guarantees

---

## FAQ

**Q: How many patterns should I learn?**
A: Start with Testing Patterns (5), then Architecture (5) for production code. Advanced designers learn all 20.

**Q: Do I need to learn them in order?**
A: No, but the beginner patterns are prerequisites for understanding advanced ones.

**Q: Can I use just one pattern?**
A: Yes, but patterns work together. Combine related patterns for best results.

**Q: Where do I go from here?**
A: Choose a [Learning Sequence](tutorials/) or pick a pattern you need right now.

**Q: How do patterns relate to the application guide?**
A: Application guide shows *how to apply patterns in practice*. Cookbook explains *why patterns exist*. Use both together.

---

## Next Steps

Choose your learning path:

- **[Testing Patterns Learning Path](tutorials/learning-testing-patterns.md)** (90 minutes)
- **[Architecture Patterns Learning Path](tutorials/learning-architecture-patterns.md)** (60 minutes)
- **[Design Patterns Learning Path](tutorials/learning-design-patterns.md)** (120 minutes)
- **[All Patterns Quick Reference](all-patterns-reference.md)** (Lookup table for all 20)

Or jump directly to the pattern you need from the Quick Finder above.

---

**Remember**: Patterns work together. As you learn each one, you'll recognize them appearing in others. That's the power of a pattern language.
