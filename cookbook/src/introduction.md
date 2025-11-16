# Introduction

Christopher Alexander famously observed that great environments share a pattern language – a network of proven responses to recurring forces. Chicago TDD Tools embodies the same philosophy for Rust testing. Instead of isolated utilities, the framework codifies high-leverage patterns that push teams toward dependable, behavior-focused tests and extendable architecture.

This cookbook distills those patterns. Each entry is written in Alexander's form so you can quickly scan the context, recognize the tension, and apply the solution. Read the patterns sequentially to see how they reinforce each other, or jump to the problem you have today.

The language is organized into three families:

1. **Testing Patterns** – maintainable, behavior-driven tests that fail fast and verify real outcomes.
2. **Architecture Patterns** – structural choices that keep the framework extensible and consistent.
3. **Design Patterns** – type-level techniques, zero-cost abstractions, and compile-time validation.

Combine these ingredients to build resilient Rust systems aligned with Chicago TDD principles: state-based testing, real collaborators, behavior verification, and the AAA pattern.

---

## Core Team Philosophy

Every pattern in this cookbook embodies **four principles** that the chicago-tdd-tools core team lives by:

### 🚫 **Poka-Yoke: Prevent Errors at Compile-Time**
Mistakes are prevented by the type system, not caught at runtime. **If it compiles, correctness follows.**
- Pattern 1 (AAA) + Pattern 14 (Compile-Time Validation) → Wrong test structure = compiler error
- Pattern 15 (Type State) → Wrong call order = compiler error
- Pattern 13 (Sealed Traits) → Unsafe downstream code = compiler error

**Core team rule:** Use types to encode invariants. Never rely on documentation or runtime checks.

### 🎯 **FMEA: Quantify Risk Reduction**
Every pattern reduces Risk Priority Numbers (RPN). Patterns aren't optional—they prevent failures.
- Pattern 2 (Error Paths) → Production panics: RPN 180 → 36
- Pattern 4 (Resource Cleanup) → Resource leaks: RPN 112 → 22
- Pattern 18 (Timeout Defense) → Hanging tests: RPN 120 → 24

**Core team rule:** If a pattern doesn't reduce a known failure mode, don't use it.

### 🔒 **Production Safety: No Unwrap, Panic, or Println**
Production code must be bulletproof. Errors propagate, never crash.
- Pattern 2 (Error Paths) teaches `?` operator, never `.unwrap()`
- Pattern 5 (Real Collaborators) catches integration issues before production
- Pattern 16 (Fixture Lifecycle) guarantees cleanup even on failure

**Core team rule:** Clippy denies `unwrap_used`, `expect_used`, `panic` in production code.

### ⚡ **80/20 Thinking: Maximum Value, Minimum Effort**
Learn 5 patterns first (45 min), get 80% of benefits. Everything else adds specialized depth.
- Foundation: Pattern 1 (AAA) + Pattern 6 (Generic Base)
- High-impact testing: Pattern 2 (Error Paths) + Pattern 5 (Real Collaborators)
- High-impact organization: Pattern 10 (Capability Groups)

**Core team rule:** Start with foundations, add patterns when you hit their problem.

---

**Not sure where to start?** Choose your path:

| Your Situation | Go To | Time |
|---|---|---|
| **I'm in a hurry** | [Quick Reference Card](quick-reference.md) - One-page lookup | 2 min |
| **I want to avoid common mistakes** | [Common Mistakes](common-mistakes.md) - Learn from gotchas | 10 min |
| **I need a learning order** | [Pattern Dependencies](pattern-dependencies.md) - Foundations first, then build | 5 min |
| **I need to solve a problem** | [Choosing Your Pattern](choosing-your-pattern.md) - Decision guide | 5 min |
| **I want to see patterns in action** | [Real-World Scenarios](real-world-scenarios.md) - 5 examples | 15 min |
| **I want a quick overview** | [All Patterns Reference](all-patterns-reference.md) - All 20 at a glance | 10 min |
| **I want to learn systematically** | Choose a learning sequence below | 60-120 min |
| **I want a specific pattern** | Jump to pattern reference below | Varies |

---

## Learning Sequences

Learn patterns progressively through structured tutorials:

| Sequence | Focus | Time | Difficulty |
|----------|-------|------|------------|
| [Testing Patterns](tutorials/learning-testing-patterns.md) | How to write better tests | 90 min | Beginner |
| [Architecture Patterns](tutorials/learning-architecture-patterns.md) | How to organize code | 60 min | Intermediate |
| [Design Patterns](tutorials/learning-design-patterns.md) | Type safety and optimization | 120 min | Advanced |

**Recommended**: Follow in order (Testing → Architecture → Design).

---

## Quick Links

**Quick Start (Fastest paths)**:
- **[Quick Reference Card](quick-reference.md)** - One-page lookup table, 2 min
- **[Common Mistakes](common-mistakes.md)** - Avoid the gotchas, 10 min
- **[Real-World Scenarios](real-world-scenarios.md)** - See patterns in action, 15 min

**Navigation & Discovery**:
- **[Pattern Dependencies & Learning Order](pattern-dependencies.md)** - Learning paths and foundations first, 5 min
- **[Choosing Your Pattern](choosing-your-pattern.md)** - Decision matrices to find the right pattern
- **[All Patterns Reference](all-patterns-reference.md)** - All 20 patterns at a glance

**Deep Learning**:
- **[Testing Patterns](testing-patterns/)** - Learn to write great tests
- **[Architecture Patterns](architecture-patterns/)** - Learn to organize code
- **[Design Patterns](design-patterns/)** - Learn advanced safety and design
