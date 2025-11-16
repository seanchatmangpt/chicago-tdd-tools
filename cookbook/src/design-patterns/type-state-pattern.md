# Pattern 15: Type State Enforcement

> 🔧 **HOW-TO** | Use types to enforce call order at compile time

## Quick Reference

| Aspect | Details |
|--------|---------|
| **Problem Solved** | Runtime call-order enforcement can be bypassed; inconsistent state and flakiness |
| **Core Solution** | Model phases as distinct types; methods consume self and return next state |
| **When to Use** | ✅ Prescribed call orders, ✅ Multi-phase workflows, ✅ Builder patterns |
| **When NOT to Use** | ❌ Optional steps (use builder instead), ❌ Dynamic order (use state machines) |
| **Difficulty** | Medium - Requires understanding PhantomData and type transitions |

## The Problem

APIs with a prescribed call order (like AAA: Arrange → Act → Assert) can be called out of order. Runtime checks rely on documentation and can be bypassed, leading to subtle bugs and test flakiness.

## The Solution

Encode each phase as a distinct type. Methods consume `self` (requiring move) and return the next phase type. The type system enforces order: you cannot call methods in the wrong sequence because the type doesn't support it. Use `PhantomData` to add type information with zero runtime cost.

## Essential Code Example

```rust
// Marker types for phases (empty, zero-cost)
pub struct Arrange;
pub struct Act;
pub struct Assert;

pub struct TestState<Phase> {
    context: TestContext,
    _phase: std::marker::PhantomData<Phase>,
}

impl TestState<Arrange> {
    pub fn new() -> Self {
        Self {
            context: TestContext::new(),
            _phase: std::marker::PhantomData,
        }
    }

    // Consume self, return Act phase
    pub fn act(self) -> TestState<Act> {
        TestState {
            context: self.context,
            _phase: std::marker::PhantomData,
        }
    }
}

impl TestState<Act> {
    // Only available in Act phase
    pub fn assert(self) -> TestState<Assert> { /* ... */ }
}

// Usage: Type system enforces order
let test = TestState::<Arrange>::new()
    .act()              // Phase 1: can only call act()
    .assert();          // Phase 2: can only call assert()

// test.assert().act()  // Compile error! Assert doesn't have act()
```

## Implementation Checklist

- [ ] Each phase is a distinct type (struct or empty enum)
- [ ] Methods consume self and return next phase
- [ ] PhantomData carries type information (zero cost)
- [ ] Compile error if methods called out of order
- [ ] Transition methods have clear names (act, assert, etc.)
- [ ] Documentation explains the state machine

## The Gotcha (Most Common Mistake)

Implementing multiple methods on each type, allowing bypassing of order:

```rust
// ❌ WRONG: Can call in any order
impl TestState<Arrange> {
    pub fn act(self) -> TestState<Act> { /* ... */ }
    pub fn assert(&mut self) { /* ... */ }  // Allowed without going to Act!
}

// ✅ RIGHT: Each phase has only its valid transitions
impl TestState<Arrange> {
    pub fn act(self) -> TestState<Act> { /* ... */ }
}

impl TestState<Act> {
    pub fn assert(self) -> TestState<Assert> { /* ... */ }
}
```

**Why**: Each type must only expose methods for valid next states. The type system then enforces order.

## Codebase Example

File: `src/core/state.rs`
Purpose: Type-level state machines for test phases

## Related Patterns

- **Before this**: [Pattern 1: AAA Pattern](../testing-patterns/aaa-pattern.md) (what to enforce)
- **Use with**: [Pattern 14: Compile-Time](compile-time-validation.md) (validate at compile time)
- **Next**: [Pattern 11: Zero-Cost](zero-cost-abstractions.md) (no runtime overhead)

---

**Why It Works**: The Rust type system enforces that you can only call methods that exist on the current phase type. Wrong-order calls don't compile.

**Production Checklist**:
- [ ] Each phase is well-defined
- [ ] Transitions happen via method calls
- [ ] Type system prevents wrong order
- [ ] PhantomData adds no runtime cost
- [ ] Documentation shows valid state diagram
