# Pattern Refactoring: 80/20 Core Team Approach

**Goal**: Refactor all 20 patterns to be action-oriented, concise, and immediately valuable.

---

## Core Team Philosophy Applied

| CLAUDE.md Principle | How to Apply to Patterns |
|---|---|
| **80/20 Thinking** | Show the 20% that solves 80% of problems, cut philosophy |
| **Quality Default** | Prevention patterns over detection patterns |
| **Type-First Thinking** | Emphasize type system, compile-time checks |
| **Zero-Cost Abstractions** | Explain perf implications upfront |
| **Proven Patterns** | Link to actual codebase, not theory |
| **Prevention over Detection** | Poka-yoke design, make wrong code impossible |

---

## New Pattern Template (Lightweight)

```markdown
# Pattern: [Name]

> 🔧 **HOW-TO** | [One-line purpose]

## The Problem

[1 paragraph: What breaks without this pattern]

## The Solution

[1 paragraph: Core idea. "Use X to achieve Y"]

## Essential Code Example

[1 code example that shows it working - max 15 lines]

## Implementation Checklist

- [ ] Step 1
- [ ] Step 2
- [ ] Step 3

## The Gotcha (When This Backfires)

[Most common mistake or edge case - 3-4 sentences]

## Codebase Example

[File path + line number where this pattern is used]

## Related Patterns

- Pattern X - Use before this
- Pattern Y - Use after this

---

**Why This Works**: [1 sentence on the mechanism]
```

---

## Refactoring Checklist

### Remove (Too Verbose)
- [ ] Long "Context" sections (combine with Problem)
- [ ] "Forces" section (too abstract)
- [ ] "Why This Matters" paragraphs (should be obvious from problem)
- [ ] Multiple redundant code examples (keep 1 core example)
- [ ] Advanced/theory sections (focus on practical)
- [ ] Long explanations (use bullets/checklists instead)

### Keep (High Value)
- [ ] Quick Reference table (problem/solution/trade-offs)
- [ ] Problem statement (clear and brief)
- [ ] 1 essential code example (working, copy-paste ready)
- [ ] Common mistakes section (where devs go wrong)
- [ ] Implementation checklist (step-by-step application)
- [ ] Real codebase link (proof it's used)
- [ ] Pattern relationships (which patterns go together)

### Add (Core Team Value)
- [ ] "The Gotcha" section (when/why this fails)
- [ ] Production checklist (before shipping)
- [ ] Perf implications (if relevant)
- [ ] Type system benefits (compile-time safety angle)
- [ ] One-line "why it works" (mechanism clarity)

---

## Example: Pattern 1 AAA Refactored (80/20)

**Before**: 240 lines
**After**: 80 lines

```markdown
# Pattern 1: AAA Pattern

> 🔧 **HOW-TO** | Structure tests for readability and debugging

## The Problem

Tests that intermingle setup, logic, and verification hide intent and make failures hard to diagnose.

## The Solution

Divide every test into three explicit phases: **Arrange** (setup), **Act** (execute), **Assert** (verify). Each phase has one job.

## Essential Code Example

```rust
test!(test_calculate_discount, {
    // Arrange
    let price = 100.0;
    let discount_pct = 10;

    // Act
    let final_price = apply_discount(price, discount_pct);

    // Assert
    assert_eq!(final_price, 90.0);
});
```

## Implementation Checklist

- [ ] Each test has 3 phases: Arrange → Act → Assert
- [ ] Comments label each phase
- [ ] Act phase calls one function (no side effects)
- [ ] Assert uses specific assertions, not conditionals
- [ ] Test name describes what's being tested

## The Gotcha

**Most common mistake**: Mixing Arrange and Act by calling a setup function that also executes. This hides what's actually being tested.

```rust
// ❌ BAD: Can't tell what's being tested
let result = setup_and_process(100);  // What are we testing?

// ✅ GOOD: Crystal clear
let value = 100;           // Arrange
let result = process(value);  // Act
assert_eq!(result, 200);   // Assert
```

## Codebase Example

Used throughout: `examples/basic_test.rs`, `tests/go_extra_mile_tests.rs`

## Related Patterns

- **Before this**: Read first (foundation)
- **After this**: Pattern 2 (Error Path Testing), Pattern 5 (Real Collaborators)
- **Use with**: Pattern 17 (Builder Test Data) for complex Arrange phases

---

**Why It Works**: Explicit structure makes intent obvious, and when tests fail, you know exactly which phase broke.
```

---

## Apply to All 20 Patterns

### Testing Patterns (1-5)
- [ ] Pattern 1: AAA - Done (example above)
- [ ] Pattern 2: Error Paths - Refactor
- [ ] Pattern 3: Boundaries - Refactor
- [ ] Pattern 4: Cleanup - Refactor
- [ ] Pattern 5: Real Collaborators - Refactor

### Architecture Patterns (6-10)
- [ ] Pattern 6: Generic Base - Refactor
- [ ] Pattern 7: Extension - Refactor
- [ ] Pattern 8: Composition - Refactor
- [ ] Pattern 9: Single Source - Refactor
- [ ] Pattern 10: Capability Groups - Refactor

### Design Patterns (11-20)
- [ ] Pattern 11: Zero-Cost - Refactor
- [ ] Pattern 12: Type Safety/GATs - Refactor
- [ ] Pattern 13: Sealed Traits - Refactor
- [ ] Pattern 14: Compile-Time - Refactor
- [ ] Pattern 15: Type State - Refactor
- [ ] Pattern 16: Fixture Lifecycle - Refactor
- [ ] Pattern 17: Builder Test Data - Refactor
- [ ] Pattern 18: Timeout Defense - Refactor
- [ ] Pattern 19: Feature Gates - Refactor
- [ ] Pattern 20: Macro Enforcement - Refactor

---

## Quality Metrics

Each refactored pattern should:

| Metric | Target | How to Measure |
|--------|--------|---|
| **Length** | 80-100 lines max | Word count |
| **Code examples** | 1 essential + 1 gotcha example | Number of code blocks |
| **Actionability** | Can implement in 10 minutes | Time to apply pattern |
| **Clarity** | 1st read understanding | Readability score |
| **Value density** | 80% of value in 20% of text | Information per line |

---

## Refactoring Order

**Highest impact first** (apply core team 80/20):

1. **Testing Patterns** (Patterns 1-5) - Used in every test
2. **Real Collaborators** (Pattern 5) - Most impactful architecture choice
3. **Type State** (Pattern 15) - Prevents entire bug categories
4. **Sealed Traits** (Pattern 13) - API safety
5. All others by frequency of use in codebase

---

## Success Criteria

After refactoring:
- ✅ Each pattern is 2-3 pages (not 5+)
- ✅ Can apply pattern in one sitting
- ✅ "Aha moment" comes in first 2 minutes
- ✅ Includes real codebase example
- ✅ Includes "the gotcha" (common mistake)
- ✅ Links to related patterns
- ✅ Production-ready checklist
- ✅ No philosophy, only action
