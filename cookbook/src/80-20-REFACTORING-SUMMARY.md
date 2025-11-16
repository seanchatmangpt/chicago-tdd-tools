# 80/20 Pattern Refactoring: Core Team Approach

**Apply Chicago TDD philosophy to patterns**: Practical, action-oriented, immediately valuable.

---

## What "80/20" Means for Patterns

The core team (CLAUDE.md) emphasizes:
- **Prevention over detection** - Make wrong code impossible
- **Type-first thinking** - Use the type system
- **Zero-cost abstractions** - No runtime overhead
- **Proven patterns** - Tested in production
- **Action-oriented** - Immediately applicable

**Apply this**: Show the 20% of pattern knowledge that solves 80% of problems. Cut philosophy, emphasize application.

---

## Refactoring Blueprint

### Remove (Too Much Context)
```
❌ Long "Context" sections (philosophical)
❌ "Forces" section (abstract tensions)
❌ Multiple similar code examples
❌ "Why This Matters" rambling
❌ Advanced/edge case subsections
❌ Historical background
```

### Keep (Essential Value)
```
✅ Quick Reference table (problem/solution/when-to-use)
✅ The Problem (1 paragraph, clear)
✅ The Solution (1 paragraph, core idea)
✅ 1 Essential Code Example (working, copy-paste ready)
✅ Implementation Checklist (step-by-step)
✅ Common Mistake / Gotcha (where devs fail)
✅ Real Codebase Link (proof it's used)
✅ Related Patterns (which patterns go together)
```

### Add (Core Team Value)
```
✨ "The Gotcha" - When/why this fails
✨ Production Checklist - Before shipping
✨ Type System Benefits - Compile-time safety
✨ "Why It Works" (1 sentence mechanism)
✨ Perf Implications - If relevant
```

---

## Pattern Length Target

| Metric | Before | Target | Reduction |
|--------|--------|--------|-----------|
| **Lines** | 240+ | 80-100 | 65% less |
| **Code Examples** | 6+ | 2-3 | 50% less |
| **Time to Apply** | 30 min research | 10 min | Clear path |
| **Information Density** | Verbose | High | More value per line |

---

## Refactored Pattern Template

```markdown
# Pattern N: [Name]

> 🔧 **HOW-TO** | [One-line purpose that solves the problem]

## The Problem

[1 paragraph: What breaks without this? Why is it painful?]

## The Solution

[1 paragraph: Core idea. "Use X to achieve Y"]

## Essential Code Example

[1 code example - max 15 lines, shows it working]

## Implementation Checklist

- [ ] Step 1
- [ ] Step 2
- [ ] Step 3
- [ ] Verify with Step 4

## The Gotcha (When This Backfires)

[Most common mistake with ❌ bad + ✅ good code examples]

## Codebase Example

File: `path/to/file.rs:line-range`
Purpose: [What it's doing]

## Related Patterns

- **Before**: Pattern X (prerequisite)
- **After**: Pattern Y (next level)
- **Combine**: Pattern Z (works together)

---

**Why It Works**: [1-2 sentence mechanism]

**Production Ready**: [1-2 sentence production checklist]
```

---

## 80/20 Philosophy for Each Pattern Family

### Testing Patterns (1-5)
Focus on **preventing test bugs**, not testing philosophy:
- Make tests readable (AAA)
- Test the hard cases (Error Paths, Boundaries)
- Use real code (Real Collaborators)
- Clean up automatically (Cleanup)

**80/20 Goal**: You can't write bad tests (structure prevents it)

### Architecture Patterns (6-10)
Focus on **preventing structural problems**, not design elegance:
- Eliminate duplication (Generic Base, Composition)
- Safely extend (Extension Layer)
- Keep data consistent (Single Source)
- Organize by capability (Capability Groups)

**80/20 Goal**: Code structure enforces correctness

### Design Patterns (11-20)
Focus on **preventing whole bug categories**, not advanced Rust:
- Use type system (Type Safety, Type State, Sealed)
- Compile-time checks (Compile-Time, Macros)
- Resource safety (Fixtures, Timeout Defense)
- Feature reliability (Feature Gates)

**80/20 Goal**: Compile time catches bugs, not runtime

---

## Refactoring Priority (Highest Impact First)

| Priority | Pattern | Why First |
|----------|---------|-----------|
| 🔴 P0 | AAA (1) | Foundation, used in every test |
| 🔴 P0 | Error Paths (2) | Prevents missing test cases |
| 🔴 P0 | Real Collaborators (5) | Biggest architecture choice |
| 🟠 P1 | Type State (15) | Prevents entire bug categories |
| 🟠 P1 | Sealed Traits (13) | API safety |
| 🟠 P1 | Compile-Time (14) | Shift errors left |
| 🟡 P2 | All others | Refactor in learning order |

---

## Refactoring Checklist

### Before Publishing Each Pattern

- [ ] **Brevity**: ≤ 100 lines (was it verbose?)
- [ ] **Clarity**: First 2 minutes explains core idea
- [ ] **Actionability**: Can apply in one sitting
- [ ] **Real Code**: Links to actual codebase
- [ ] **The Gotcha**: Shows most common mistake
- [ ] **Checklist**: Step-by-step application
- [ ] **Relationships**: Shows pattern dependencies
- [ ] **Why It Works**: One-sentence mechanism

### Quality Questions

Ask yourself for each pattern:

1. **Can a junior dev apply this in 10 minutes?** If no, it's too complex.
2. **Does it include "when this backfires"?** If no, add the gotcha.
3. **Is there a real codebase example?** If no, add file:line reference.
4. **Does it show the most common mistake?** If no, add before/after code.
5. **Can I skip the theoretical parts and still apply it?** If no, trim philosophy.

---

## Success Criteria

A refactored pattern succeeds when:

✅ **It's actionable**: Devs apply it same day
✅ **It's realistic**: Shows common mistakes (not just happy path)
✅ **It's proven**: Real codebase example
✅ **It's quick**: 5-minute read, 10-minute apply
✅ **It's connected**: Shows related patterns
✅ **It's safe**: Includes "when this backfires"
✅ **It's complete**: Has checklist for verification

---

## Example: AAA Pattern Refactored

**Original**: 240 lines, philosophical, multiple examples
**Refactored**: 80 lines, action-oriented, essential example + gotcha

**Key changes**:
- ❌ Removed: Context, Forces, Advanced sections
- ✅ Added: Gotcha section with common mistake
- ✅ Shortened: Problem from 2 paragraphs to 1
- ✅ Simplified: Solution from 3 paragraphs to 1
- ✅ Kept: Quick reference table (high value)
- ✅ Focused: One essential code example instead of four

**Result**: Faster to read, faster to apply, more valuable

---

## How to Refactor (Template Approach)

For each pattern:

1. **Read the original** (take notes on key ideas)
2. **Write "The Problem"** (1 paragraph: What breaks?)
3. **Write "The Solution"** (1 paragraph: Core idea)
4. **Add Essential Code** (1 example, 10-15 lines)
5. **Add Checklist** (4-6 steps to apply)
6. **Add Gotcha** (most common mistake with bad/good code)
7. **Find Codebase Example** (file:line where it's used)
8. **Add Related Patterns** (which patterns go together)
9. **Write "Why It Works"** (1-2 sentences)
10. **Review**: Does it pass all success criteria?

---

## Next Steps

1. **Phase 1 (P0 Patterns)**: Refactor AAA, Error Paths, Real Collaborators
2. **Phase 2 (P1 Patterns)**: Refactor Type State, Sealed, Compile-Time
3. **Phase 3 (P2 Patterns)**: Refactor remaining 14 patterns
4. **Validation**: Run through success criteria for each
5. **Integration**: Link refactored patterns to tutorials and decision matrix

---

## Philosophy Summary

**Before**: Patterns as philosophical constructs (Alexander style)
**After**: Patterns as practical tools (Chicago TDD Core Team style)

**Before**: "Here's a pattern language for design"
**After**: "Here's how to apply this, avoid this mistake, and ship this code"

The refactoring preserves pattern knowledge but reframes it for **immediate action and maximum value**.
