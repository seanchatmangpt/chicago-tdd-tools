# Pattern Dependencies & Learning Order

This guide shows which patterns depend on which others, helping you build a foundation before moving to advanced patterns.

---

## Quick Answer: Minimum Viable Pattern Sets

**Just starting?** Choose your path:

| Your Goal | Minimum Patterns | Time | Patterns |
|-----------|------------------|------|----------|
| **Write better tests** | 3 patterns | 30 min | 1, 2, 5 |
| **Organize your code** | 2 patterns | 20 min | 6, 10 |
| **Add type safety** | 2 patterns | 40 min | 14, 15 |
| **Production ready** | 8 patterns | 2 hours | 1-3, 5-7, 10 |
| **Complete mastery** | All 20 patterns | 4 hours | 1-20 in order |

---

## Dependency Graph: Visual

```
FOUNDATIONAL LAYER
├── Pattern 1: AAA ................. (No dependencies)
│   ├── Pattern 2: Error Paths
│   ├── Pattern 3: Boundaries
│   ├── Pattern 4: Cleanup ......... Pattern 16 depends on this
│   ├── Pattern 5: Real Collaborators
│   ├── Pattern 17: Builder Test Data
│   └── Pattern 18: Timeout Defense
│
└── Pattern 6: Generic Base ........ (No dependencies)
    ├── Pattern 7: Extension Layer
    ├── Pattern 8: Composition (also needs 7)
    ├── Pattern 9: Single Source
    ├── Pattern 10: Capability Groups
    └── Pattern 19: Feature Gates

INDEPENDENT PATTERNS (Apply to any of above)
├── Pattern 11: Zero-Cost Abstractions
├── Pattern 12: Type Safety (GATs)
├── Pattern 13: Sealed Traits
├── Pattern 14: Compile-Time Validation
├── Pattern 15: Type State Enforcement
└── Pattern 20: Macro Enforcement
```

---

## Foundation Patterns (Must Learn First)

### Pattern 1: AAA Pattern
**What it is:** Arrange-Act-Assert structure for tests
**Why first:** All other testing patterns build on this foundation
**Prerequisite:** None
**Time:** 10 min

**Unlocks:** Patterns 2, 3, 4, 5, 16, 17, 18

---

### Pattern 6: Generic Base Layer
**What it is:** Keep core layer generic, domain logic in extensions
**Why first:** All architecture patterns build on this structure
**Prerequisite:** None
**Time:** 10 min

**Unlocks:** Patterns 7, 8, 9, 10, 19

---

## Testing Patterns (Depend on Pattern 1)

Learn these in any order after mastering Pattern 1:

| Pattern | Builds On | Core Idea | Time |
|---------|-----------|-----------|------|
| **2: Error Paths** | 1: AAA | Test each error variant | 15 min |
| **3: Boundaries** | 1: AAA | Test below/at/above limits | 15 min |
| **4: Cleanup** | 1: AAA | RAII for resource release | 10 min |
| **5: Real Collaborators** | 1: AAA | Use actual services, not mocks | 15 min |

**Recommended order:** 1 → 2 → 3 → 5 → 4

**Why this order:**
1. Learn AAA first (foundation)
2. Add error paths (80% of bugs)
3. Add boundary testing (off-by-one prevention)
4. Use real collaborators (confidence boost)
5. Add cleanup (resource safety)

---

## Architecture Patterns (Depend on Pattern 6)

### Core Pattern: Pattern 6 (Generic Base)
Learn this first. Builds foundation for all architecture patterns.

### Tier 1 (Depends only on 6)

| Pattern | What It Adds | Time |
|---------|-------------|------|
| **10: Capability Groups** | How to organize modules | 10 min |
| **9: Single Source of Truth** | Centralize constants | 10 min |

**Learn after 6:** 6 → 10 OR 6 → 9 (either order)

### Tier 2 (Depends on 6 + Tier 1)

| Pattern | Depends On | What It Adds | Time |
|---------|-----------|-------------|------|
| **7: Extension Layer** | 6 | Domain logic in extensions | 10 min |
| **8: Composition** | 6, 7 | Wrap don't copy | 10 min |

**Learn after Tier 1:** 6 → 10 → 7 → 8 OR 6 → 7 → 8

### Pattern 19: Feature Gates
**Depends on:** 6: Generic Base
**When to learn:** After you have multiple features in your crate
**Time:** 10 min

---

## Design Patterns (Independent)

These patterns don't depend on each other. Learn based on your needs:

### For Zero-Cost Performance

**Order:** 11 → 12 → 13 → 14 → 15

| # | Pattern | Problem | Time |
|---|---------|---------|------|
| **11** | Zero-Cost Abstractions | Trait objects slow hot paths | 15 min |
| **14** | Compile-Time Validation | Validate at runtime vs compile time | 15 min |
| **12** | Type Safety (GATs) | Lifetimes escape scope | 20 min |
| **15** | Type State Enforcement | Call order can be wrong | 15 min |
| **13** | Sealed Traits | Downstream breaks invariants | 10 min |

**Why this order:** Build on each other's concepts for safety

### For Extensibility & Governance

**Order:** 13 → 20 → 19

| # | Pattern | What It Does | Time |
|---|---------|-------------|------|
| **13** | Sealed Traits | Prevent downstream implementations | 10 min |
| **20** | Macro Enforcement | Embed best practices in macros | 15 min |
| **19** | Feature Gates | Group features into slices | 10 min |

### For Reliability & Correctness

**Order:** 16 → 17 → 18

| # | Pattern | What It Does | Time |
|---|---------|-------------|------|
| **16** | Fixture Lifecycle | Automatic cleanup via Drop | 10 min |
| **17** | Builder Test Data | Fluent test data construction | 10 min |
| **18** | Timeout Defense | Layer timeouts for safety | 10 min |

---

## Dependency Matrix: Full Reference

### Minimal Set Per Pattern

Shows the **minimum** other patterns you should master before this one:

| Pattern | Must Know First | Should Know First | Optional Context |
|---------|-----------------|-------------------|------------------|
| **1: AAA** | None | None | None |
| **2: Error Paths** | 1 | None | None |
| **3: Boundaries** | 1 | 2 | None |
| **4: Cleanup** | 1 | None | None |
| **5: Real Collaborators** | 1 | 2, 3 | None |
| **6: Generic Base** | None | None | None |
| **7: Extension Layer** | 6 | 10 | None |
| **8: Composition** | 6, 7 | 8 | None |
| **9: Single Source** | 6 | None | None |
| **10: Capability Groups** | 6 | None | None |
| **11: Zero-Cost** | None | None | 6 (for context) |
| **12: Type Safety (GATs)** | None | None | 1 (for context) |
| **13: Sealed Traits** | None | None | 6 (for context) |
| **14: Compile-Time** | None | None | 1 (for context) |
| **15: Type State** | None | None | 1 (for context) |
| **16: Fixture Lifecycle** | 1, 4 | None | None |
| **17: Builder Test Data** | 1 | 6 (for builders) | None |
| **18: Timeout Defense** | 1 | None | None |
| **19: Feature Gates** | 6 | None | None |
| **20: Macro Enforcement** | None | None | 1, 6 (for context) |

---

## Recommended Learning Paths

### Path 1: Testing Excellence (90 minutes)

For teams focused on test quality. Covers all testing patterns.

```
Week 1: Foundation (30 min)
  → Pattern 1: AAA Pattern (10 min)
  → Pattern 2: Error Paths (10 min)
  → Pattern 3: Boundaries (10 min)

Week 2: Depth (30 min)
  → Pattern 5: Real Collaborators (15 min)
  → Pattern 4: Cleanup (15 min)

Week 3: Advanced Testing (30 min)
  → Pattern 16: Fixture Lifecycle (10 min)
  → Pattern 17: Builder Test Data (10 min)
  → Pattern 18: Timeout Defense (10 min)
```

**Total:** Patterns 1-5, 16-18 mastered (7 patterns)
**Time:** 90 minutes
**Benefit:** Write bulletproof tests, catch 80% of bugs

---

### Path 2: Architecture & Organization (60 minutes)

For teams designing extensible systems.

```
Week 1: Foundation (20 min)
  → Pattern 6: Generic Base Layer (10 min)
  → Pattern 10: Capability Groups (10 min)

Week 2: Extensions (20 min)
  → Pattern 7: Extension Layer (10 min)
  → Pattern 9: Single Source of Truth (10 min)

Week 3: Composition (20 min)
  → Pattern 8: Composition Over Duplication (10 min)
  → Pattern 19: Feature Gates (10 min)
```

**Total:** Patterns 6-10, 19 mastered (6 patterns)
**Time:** 60 minutes
**Benefit:** Organize code for reuse, reduce duplication, manage complexity

---

### Path 3: Type Safety & Performance (120 minutes)

For teams needing compile-time guarantees and zero-cost abstractions.

```
Week 1: Zero-Cost (30 min)
  → Pattern 11: Zero-Cost Abstractions (15 min)
  → Pattern 14: Compile-Time Validation (15 min)

Week 2: Type-Level Safety (30 min)
  → Pattern 12: Type Safety (GATs) (20 min)
  → Pattern 15: Type State Enforcement (10 min)

Week 3: Advanced Safety (30 min)
  → Pattern 13: Sealed Traits (10 min)
  → Pattern 20: Macro Enforcement (15 min)
  → Pattern 18: Timeout Defense (5 min - context)

Week 4: Extensibility (30 min)
  → Review Patterns 1, 6, 7 (15 min - context for how these interact)
  → Pattern 13: Sealed Traits deep dive (15 min)
```

**Total:** Patterns 11-15, 18, 20 mastered + context on 1, 6, 7 (8 patterns)
**Time:** 120 minutes
**Benefit:** Compile-time safety, zero-cost abstractions, type-level design

---

### Path 4: Production Ready (180+ minutes)

Complete mastery of all 20 patterns, recommended order.

```
PHASE 1: TESTING FOUNDATION (45 min)
  Pattern 1: AAA Pattern (10 min)
  Pattern 2: Error Paths (10 min)
  Pattern 3: Boundaries (10 min)
  Pattern 5: Real Collaborators (10 min)
  Pattern 4: Cleanup (5 min)

PHASE 2: ARCHITECTURE FOUNDATION (40 min)
  Pattern 6: Generic Base Layer (10 min)
  Pattern 10: Capability Groups (10 min)
  Pattern 7: Extension Layer (10 min)
  Pattern 8: Composition Over Duplication (10 min)

PHASE 3: ADVANCED TESTING (40 min)
  Pattern 16: Fixture Lifecycle (10 min)
  Pattern 17: Builder Test Data (10 min)
  Pattern 18: Timeout Defense (10 min)
  Pattern 4: Cleanup (revisit) (10 min)

PHASE 4: ADVANCED ARCHITECTURE (30 min)
  Pattern 9: Single Source of Truth (10 min)
  Pattern 19: Feature Gates (10 min)
  Pattern 20: Macro Enforcement (10 min)

PHASE 5: DESIGN PATTERNS (80 min)
  Pattern 11: Zero-Cost Abstractions (15 min)
  Pattern 14: Compile-Time Validation (15 min)
  Pattern 12: Type Safety (GATs) (20 min)
  Pattern 15: Type State Enforcement (15 min)
  Pattern 13: Sealed Traits (15 min)
```

**Total:** All 20 patterns (some with revisits)
**Time:** 180+ minutes
**Benefit:** Complete mastery, ready to design complex systems

---

## Decision Tree: Which Patterns Do I Need?

### Question 1: What's your main goal?

**A) Better tests** → Path 1 (Testing Excellence)
**B) Better code organization** → Path 2 (Architecture)
**C) Type safety & performance** → Path 3 (Type Safety)
**D) Everything** → Path 4 (Production Ready)

### Question 2: How much time do you have?

**< 1 hour:**
- Pattern 1: AAA (10 min)
- Pattern 6: Generic Base (10 min)
- Quick Reference Card (5 min)
- Read: Common Mistakes (10 min)
- Read: Real-World Scenarios (15 min)

**1-2 hours:** Choose Path 1, 2, or 3

**2+ hours:** Choose Path 4

### Question 3: What's your biggest pain point right now?

**"Tests are hard to write"** → Pattern 1, 2, 5 (30 min)
**"Code is becoming scattered"** → Pattern 6, 10, 7 (30 min)
**"Need type safety"** → Pattern 14, 15, 12 (50 min)
**"Tests are slow"** → Pattern 11, 18 (25 min)
**"Resource management is messy"** → Pattern 4, 16 (20 min)
**"Too many bugs in error handling"** → Pattern 2, 3, 5 (40 min)

---

## Dependency Principles

### Rule 1: Learn Foundations First

Always master foundational patterns before patterns that depend on them:
- Learn **Pattern 1** before 2, 3, 4, 5, 16, 17, 18
- Learn **Pattern 6** before 7, 8, 9, 10, 19

### Rule 2: You Can Skip Non-Dependent Patterns

Design patterns (11-15, 20) don't depend on each other. Skip if they don't solve your problem.

**Example:** If you don't care about performance, skip Pattern 11.
**Example:** If you don't use trait objects, skip Pattern 13.

### Rule 3: Learn in Small Groups

Don't try to master all 20 at once. Pick a path (90 min, 120 min, or 180 min) and complete it.

**Why:** Patterns reinforce each other. Learning a path gives you working systems, not isolated ideas.

### Rule 4: Revisit Patterns in Context

Come back to patterns once you've hit the problem they solve.

**Example:** Pattern 19 (Feature Gates) is irrelevant until your crate has 3+ optional features.

---

## Anti-Pattern: Skipping Foundations

**❌ WRONG:** Try to use Pattern 7 (Extension Layer) without Pattern 6 (Generic Base)
- Result: Confused about what goes in "base" vs "extension"

**❌ WRONG:** Try Pattern 12 (Type Safety) without Pattern 1 (AAA)
- Result: GATs seem like unnecessary complexity

**❌ WRONG:** Try Pattern 20 (Macro Enforcement) without knowing what you want to enforce
- Result: Over-engineered macros that nobody uses

**✅ RIGHT:** Learn foundations (1, 6) first, then specialized patterns
- Result: Each pattern solves real problems you've hit

---

## Quick Reference: What Each Pattern Enables

After mastering Pattern 1 (AAA), you unlock:
- ✅ Pattern 2: Test more error cases (+ 15 min)
- ✅ Pattern 3: Catch boundary bugs (+ 15 min)
- ✅ Pattern 4: Clean up resources (+ 10 min)
- ✅ Pattern 5: Integration testing (+ 15 min)
- ✅ Pattern 16: Auto cleanup (+ 10 min)
- ✅ Pattern 17: Builder test data (+ 10 min)
- ✅ Pattern 18: Timeout safety (+ 10 min)

After mastering Pattern 6 (Generic Base), you unlock:
- ✅ Pattern 7: Extensible code (+ 10 min)
- ✅ Pattern 8: No duplication (+ 10 min)
- ✅ Pattern 9: Shared constants (+ 10 min)
- ✅ Pattern 10: Clear organization (+ 10 min)
- ✅ Pattern 19: Feature management (+ 10 min)

Design patterns (11-15, 20) stand alone.

---

## Pro Tip: The 80/20 Minimum

**If you only have 45 minutes,** master these 5 patterns:

1. **Pattern 1: AAA** (10 min) - Test structure
2. **Pattern 2: Error Paths** (10 min) - Real bug detection
3. **Pattern 6: Generic Base** (10 min) - Code organization
4. **Pattern 10: Capability Groups** (10 min) - Discoverability
5. **Pattern 5: Real Collaborators** (5 min) - Integration confidence

**Result:** You'll have 80% of the value with these 5 patterns. Everything else adds depth.

