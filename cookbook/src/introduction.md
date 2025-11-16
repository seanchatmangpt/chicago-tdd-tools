# Introduction

Christopher Alexander famously observed that great environments share a pattern language – a network of proven responses to recurring forces. Chicago TDD Tools embodies the same philosophy for Rust testing. Instead of isolated utilities, the framework codifies high-leverage patterns that push teams toward dependable, behavior-focused tests and extendable architecture.

This cookbook distills those patterns. Each entry is written in Alexander's form so you can quickly scan the context, recognize the tension, and apply the solution. Read the patterns sequentially to see how they reinforce each other, or jump to the problem you have today.

The language is organized into three families:

1. **Testing Patterns** – maintainable, behavior-driven tests that fail fast and verify real outcomes.
2. **Architecture Patterns** – structural choices that keep the framework extensible and consistent.
3. **Design Patterns** – type-level techniques, zero-cost abstractions, and compile-time validation.

Combine these ingredients to build resilient Rust systems aligned with Chicago TDD principles: state-based testing, real collaborators, behavior verification, and the AAA pattern.

---

## Getting Started

**Not sure where to start?** Choose your path:

| Your Situation | Go To | Time |
|---|---|---|
| **I need to solve a problem** | [Choosing Your Pattern](choosing-your-pattern.md) | 5 min |
| **I want a quick overview** | [All Patterns Reference](all-patterns-reference.md) | 10 min |
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

- **[Choosing Your Pattern](choosing-your-pattern.md)** - Decision matrices to find the right pattern
- **[All Patterns Reference](all-patterns-reference.md)** - All 20 patterns at a glance
- **[Testing Patterns](testing-patterns/)** - Learn to write great tests
- **[Architecture Patterns](architecture-patterns/)** - Learn to organize code
- **[Design Patterns](design-patterns/)** - Learn advanced safety and design
