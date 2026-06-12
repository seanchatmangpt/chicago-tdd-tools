# Pattern 20: Macro Pattern Enforcement

> 📚 Reference

## Pattern at a Glance

| Aspect | Details |
|--------|---------|
| **Problem** | Developers skip timeouts, forget AAA comments, or mix raw test macros; no enforcement mechanism |
| **Solution** | Embed best practices in macros; using macros guarantees compliance |
| **When to Use** | Enforcing conventions, centralizing boilerplate, feature requirements |
| **When NOT to Use** | Optional patterns (don't force), complex macros with large code expansions |
| **Trade-offs** | Restricts test flexibility, but guarantees compliance across the entire team without code reviews |
| **Complexity** | Hard |
| **Real-World Example** | [src/core/macros/test.rs](file:///Users/sac/chicago-tdd-tools/src/core/macros/test.rs) |

## The Problem

Developers forget to add timeouts, skip AAA comments, or use raw `#[test]` instead of framework macros. Consistency depends on discipline, which fails at scale. No enforcement mechanism.

## The Solution

Embed best practices inside macros. Using the macro guarantees compliance. For example, `test!` injects AAA comments, `async_test!` wraps with timeouts, and feature-gated macros require specific features.

## Essential Code Example

```rust
// Macro that enforces timeouts + AAA structure
#[macro_export]
macro_rules! test {
    ($name:ident, $body:block) => {
        #[test]
        fn $name() {
            // Injected: AAA structure  comment
            // Arrange - Set up test data
            // Act - Execute the one behavior
            // Assert - Verify the result
            $body
        }
    };
}

// Macro that enforces async + timeout
#[macro_export]
macro_rules! async_test {
    ($name:ident, $body:block) => {
        #[tokio::test]
        async fn $name() {
            let _guard = tokio::time::timeout(
                std::time::Duration::from_secs(1),
                async { $body },
            ).await;
        }
    };
}

// Macro that enforces feature requirement
#[cfg(feature = "otel")]
#[macro_export]
macro_rules! otel_test {
    ($name:ident, $body:block) => {
        #[test]
        fn $name() { $body }
    };
}

#[cfg(not(feature = "otel"))]
#[macro_export]
macro_rules! otel_test {
    ($($tt:tt)*) => {
        compile_error!("OTEL tests require --features otel");
    };
}
```

## Implementation Checklist

- [ ] Macros embed best practices (timeouts, AAA structure, feature checks)
- [ ] Using macro guarantees compliance
- [ ] Compile errors guide users if feature missing
- [ ] Macro expansion stays small (< 10 lines)
- [ ] Macro names clearly indicate purpose (async_test, fixture_test, etc.)
- [ ] Documentation explains what the macro enforces

## The Gotcha (Most Common Mistake)

Over-engineered macros that try to enforce too much or macros with complex expansions:

```rust
// ❌ WRONG: Macro tries to enforce code structure (impossible)
macro_rules! test_with_structure {
    ($name:ident, $arrange:block, $act:block, $assert:block) => {
        // Syntax is awkward; users won't use it
    }
}

// ✅ RIGHT: Macro enforces framework requirements, not code structure
macro_rules! async_test {
    ($name:ident, $body:block) => {
        #[tokio::test]
        async fn $name() {
            tokio::time::timeout(Duration::from_secs(1), async { $body }).await
        }
    };
}
```

**Why**: Macros are best for enforcing framework requirements (timeouts, features), not code structure.

## Real-World Example

- **Code location**: [src/core/macros/test.rs](file:///Users/sac/chicago-tdd-tools/src/core/macros/test.rs)
- **Explanation**: Restricts test definitions to custom macros like `test!` and `fixture_test!` that automatically inject timeouts and AAA structure.

## Related Patterns

- **Before this**: [Pattern 1: AAA Pattern](../testing-patterns/aaa-pattern.md) (what to enforce)
- **Use with**: [Pattern 18: Timeout Defense](timeout-defense.md) (macros enforce timeouts)
- **Use with**: [Pattern 19: Feature Gates](feature-gating.md) (macros enforce features)

---

**Why It Works**: Code using the macro automatically gets best practices. No discipline required.

**Production Checklist**:
- [ ] Core testing macros (test!, async_test!, fixture_test!) enforce timeouts
- [ ] Feature-gated macros require appropriate features
- [ ] Macro expansions are small and fast
- [ ] Error messages guide users to solutions
- [ ] Documentation explains each macro's enforcement
