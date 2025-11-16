# Quick Reference Card: All 20 Patterns

One-page lookup for all Chicago TDD patterns. Each row = core idea in 1 sentence.

> **Need a learning order instead of a quick lookup?** See [Pattern Dependencies & Learning Order](pattern-dependencies.md) for recommended paths (45 min to 4 hours).

## Testing Patterns (Patterns 1-5)

| # | Pattern | Problem | Core Solution | Key Benefit |
|---|---------|---------|----------------|-------------|
| 1 | **AAA** | Tests mix setup/verify | Arrange → Act → Assert | Clear failure diagnosis |
| 2 | **Error Paths** | Happy path tests miss bugs | Test each error variant | Catches regressions early |
| 3 | **Boundaries** | Off-by-one errors hide | Test below/at/above limit | Prevents boundary bugs |
| 4 | **Cleanup** | Forgotten cleanup cascades | Use fixtures with Drop | Automatic resource release |
| 5 | **Real Collaborators** | Mocks hide integration gaps | Use actual services/containers | Production confidence |

## Architecture Patterns (Patterns 6-10)

| # | Pattern | Problem | Core Solution | Key Benefit |
|---|---------|---------|----------------|-------------|
| 6 | **Generic Base** | Base layer bloats with domain code | Keep core generic, add domain in extensions | Stays small and reusable |
| 7 | **Extension Layer** | Domain logic scattered in base | Create extension crates composing base | Independent evolution |
| 8 | **Composition** | Copy-paste breaks single source | Wrap base primitives, don't copy | Stays in sync with base |
| 9 | **Single Source** | Constants drift across modules | Centralize, re-export when needed | Guaranteed consistency |
| 10 | **Capability Groups** | Features surface randomly | Group modules by capability | Predictable discovery |

## Design Patterns (Patterns 11-20)

| # | Pattern | Problem | Core Solution | Key Benefit |
|---|---------|---------|----------------|-------------|
| 11 | **Zero-Cost** | Runtime polymorphism slows hot paths | Use generics, const generics, macros | No performance cost |
| 12 | **Type Safety (GATs)** | References escape fixture scope | Bind lifetimes to fixture with GATs | Compiler prevents bugs |
| 13 | **Sealed Traits** | Downstream code breaks invariants | Seal trait, implement only internally | Safe future evolution |
| 14 | **Compile-Time** | Runtime validation can be bypassed | Validate at compile time with types | Impossible to violate |
| 15 | **Type State** | Call order can be wrong | Model phases as types, enforce via methods | Wrong order = compile error |
| 16 | **Fixture Lifecycle** | Manual cleanup is error-prone | Use fixtures with guaranteed teardown | Works even on failure |
| 17 | **Builder Test Data** | Test setup is verbose and repetitive | Wrap base builder with domain methods | Fluent, readable tests |
| 18 | **Timeout Defense** | Single timeout layer fails silently | Layer timeouts at test/runner/process | Multiple safety nets |
| 19 | **Feature Gates** | All features bloat build time | Group features into curated slices | Users only pay for what they use |
| 20 | **Macro Enforcement** | Developers skip best practices | Embed practices in macros | Using macro guarantees compliance |

---

## Lookup by Problem

**Need to write better tests?**
- Pattern 1: Structure tests (AAA)
- Pattern 2: Test error cases
- Pattern 3: Test boundaries
- Pattern 4: Clean up resources
- Pattern 5: Use real dependencies

**Need to organize code?**
- Pattern 6: Keep base lean
- Pattern 7: Build extensions on base
- Pattern 8: Compose, don't copy
- Pattern 9: Single source for constants
- Pattern 10: Group by capability

**Need compile-time safety?**
- Pattern 11: No performance cost (generics)
- Pattern 12: Bind lifetimes (GATs)
- Pattern 13: Prevent implementations (sealed)
- Pattern 14: Validate early (const generics)
- Pattern 15: Enforce call order (type state)

**Need to manage complexity?**
- Pattern 16: Handle lifecycle automatically
- Pattern 17: Reduce setup verbosity
- Pattern 18: Prevent hangs (timeouts)
- Pattern 19: Keep builds fast (feature gates)
- Pattern 20: Enforce consistency (macros)

---

## Common Pattern Combinations

**Building a robust test suite:**
1. → Pattern 1 (AAA structure)
2. → Pattern 2 (error paths)
3. → Pattern 3 (boundaries)
4. → Pattern 5 (real collaborators)
5. → Pattern 16 (cleanup)

**Designing extensible architecture:**
1. → Pattern 6 (generic base)
2. → Pattern 10 (capability groups)
3. → Pattern 7 (extensions)
4. → Pattern 8 (composition)
5. → Pattern 9 (single source)

**Maximizing compile-time safety:**
1. → Pattern 14 (compile-time validation)
2. → Pattern 15 (type state)
3. → Pattern 13 (sealed traits)
4. → Pattern 12 (GATs for lifetimes)
5. → Pattern 11 (zero-cost abstractions)

---

## Quick Stats

| Category | Count | Avg Lines | Time to Apply |
|----------|-------|-----------|----------------|
| Testing | 5 | 90 | 10 min |
| Architecture | 5 | 85 | 15 min |
| Design | 10 | 95 | 20 min |
| **Total** | **20** | **90** | **15 min average** |

**Learning paths:**
- **Quick Start**: Patterns 1, 2, 5 (30 min)
- **Solid Foundation**: Patterns 1-5, 6, 10 (2 hours)
- **Production Ready**: All 20 patterns (4 hours)

---

## When to Use Each Pattern Family

**Testing Patterns:** Every test, starting day one
**Architecture Patterns:** When code is getting complex or domain grows
**Design Patterns:** When performance matters or you need compile-time guarantees

---

**Pro Tip:** Start with Pattern 1 (AAA). It's the foundation. Then add Patterns 2-5 as you build test confidence. Add Architecture patterns as teams grow. Add Design patterns when you hit performance/safety walls.
