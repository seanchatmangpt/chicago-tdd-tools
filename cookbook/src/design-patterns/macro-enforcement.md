# Pattern 20: Macro Pattern Enforcement

## Context

You need consistent test structure and timeouts without repeating boilerplate or relying on discipline alone.

## Problem

Developers forget to add timeouts, skip AAA comments, or mix direct `#[test]` usage with framework macros, leading to drift and inconsistent behavior.

## Solution

Embed enforcement inside macros. `test!` injects the AAA skeleton, `async_test!` and `fixture_test!` wrap bodies with `tokio::time::timeout`, and `weaver_test!` requires the `weaver` feature. Each macro centralizes best practices so using it guarantees compliance.

## Forces

- Consistency vs. flexibility: macros enforce conventions while allowing custom logic inside
- Zero cost vs. tooling: expansions must stay small and compile quickly
- Guidance vs. noise: failures should point to the missing convention explicitly

## Examples

```rust
#[macro_export]
macro_rules! async_test {
    ($name:ident, $body:block) => {
        $crate::async_test_with_timeout!($name, 1, $body);
    };
}
```

```rust
#[cfg(not(feature = "otel"))]
#[macro_export]
macro_rules! otel_test {
    ($($tt:tt)*) => {
        compile_error!("OTEL testing requires the 'otel' feature. Enable with: --features otel");
    };
}
```

## Related Patterns

- Pattern 1: AAA Pattern
- Pattern 18: Timeout Defense in Depth
- Pattern 19: Feature Gate Slices
