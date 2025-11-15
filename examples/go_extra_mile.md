# Go the Extra Mile: 1st/2nd/3rd Idea Progression

**Category:** Explanation
**Level:** Intermediate to Advanced
**Prerequisites:** Understanding of testing and type systems
**Features Required:** `otel`, `weaver` (optional)

---

## Overview

This example demonstrates the "go the extra mile" paradigm with progressive enhancement: from basic implementation to maximum value solutions through the 1st/2nd/3rd idea progression.

**What you'll learn:**
- The 80/20 principle in action
- Progressive enhancement philosophy
- When to stop vs. when to go further
- Type-level validation patterns
- OTEL/Weaver integration progression

---

## Quick Start

```bash
# Without OTEL/Weaver
cargo run --example go_extra_mile

# With full features
cargo run --example go_extra_mile --features otel,weaver
```

---

## Prerequisites

- Rust 1.70+ (Edition 2021)
- Chicago TDD Tools installed
- Optional: `otel` and `weaver` features for full example

---

## Philosophy

### The Three Ideas

| Idea | Scope | Effort | Value | When to Use |
|------|-------|--------|-------|-------------|
| **1st** | Solve immediate problem | 10% | 20% | Quick prototype, throwaway code |
| **2nd** | 80/20 sweet spot | 20% | 80% | **Most production code** |
| **3rd** | Maximum value | 70% | 100% | Critical systems, frameworks |

### 80/20 Thinking

The **2nd idea** typically provides **80% more value** with only **20% more effort** than the 1st idea. This is the sweet spot for most use cases.

---

## Example: Number Parsing

### 1st Idea: Solve the Immediate Problem

Parse `u32` only - narrow scope, works but limited:

```rust
pub fn parse_u32_first_idea(input: &str) -> Result<u32, String> {
    input.parse().map_err(|e| format!("Parse error: {e}"))
}

// Usage
let result = parse_u32_first_idea("42")?;
assert_eq!(result, 42);
```

**Characteristics:**
- ✓ Solves immediate problem
- ✓ Fast to implement
- ✗ Works for `u32` only
- ✗ No telemetry
- ✗ No type safety beyond Rust basics

---

### 2nd Idea: Go Bigger (80/20 Sweet Spot)

Generic version - works for all number types:

```rust
pub fn parse_number_second_idea_no_otel<T: std::str::FromStr>(
    input: &str
) -> Result<T, String>
where
    T::Err: std::fmt::Display,
{
    input.parse().map_err(|e| format!("Parse error: {e}"))
}

// Usage
let u32_value: u32 = parse_number_second_idea_no_otel("42")?;
let i32_value: i32 = parse_number_second_idea_no_otel("-42")?;
let f64_value: f64 = parse_number_second_idea_no_otel("123.456")?;
```

**Characteristics:**
- ✓ Works for all number types (generic)
- ✓ 80% more value than 1st idea
- ✓ Minimal additional code (~5 lines)
- ✗ No telemetry yet
- ✗ No type-level validation

**This is usually the best choice for production code.**

---

### 2nd Idea Enhanced: Add OTEL Instrumentation

Same generic version, now with telemetry:

```rust
#[cfg(feature = "otel")]
pub fn parse_number_second_idea<T: std::str::FromStr>(
    input: &str,
    span_name: &str,
) -> Result<(T, Span), String>
where
    T::Err: std::fmt::Display,
{
    // Create OTEL span for operation
    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis() as u64;

    let mut span = Span::new_active(
        SpanContext::root(TraceId(12345), SpanId(67890), 1),
        span_name.to_string(),
        start_time,
        BTreeMap::new(),
        Vec::new(),
        SpanStatus::Unset,
    );

    span.attributes.insert("input".to_string(), input.to_string());
    span.attributes.insert("type".to_string(), std::any::type_name::<T>().to_string());

    // Parse and complete span
    let result = input.parse().map_err(|e| format!("Parse error: {}", e));
    let end_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;
    span.complete(end_time)?;

    result.map(|value| (value, span))
}

// Usage
let (value, span) = parse_number_second_idea::<u32>("42", "parse_number")?;
let validator = SpanValidator::new();
validator.validate(&span)?;
```

**Added Value:**
- ✓ Observability with OTEL spans
- ✓ Span validation
- ✓ Debugging capabilities

---

### 3rd Idea: Maximum Value - Type-Level Validation

Type-safe validated numbers that prevent entire classes of errors:

```rust
pub struct ValidatedNumberNoOtel<T> {
    value: T,
}

impl<T: std::str::FromStr> ValidatedNumberNoOtel<T>
where
    T::Err: std::fmt::Display,
{
    pub fn parse(input: &str) -> Result<Self, String> {
        input
            .parse()
            .map(|value| Self { value })
            .map_err(|e| format!("Parse error: {e}"))
    }

    pub const fn value(&self) -> &T {
        &self.value
    }
}

// Usage
let validated = ValidatedNumberNoOtel::<u32>::parse("42")?;
assert_eq!(*validated.value(), 42);
// validated.value is guaranteed to be valid - type system enforces it
```

**With OTEL + Weaver:**

```rust
#[cfg(feature = "otel")]
pub struct ValidatedNumber<T> {
    value: T,
    span: Span,
}

impl<T: std::str::FromStr + std::fmt::Display> ValidatedNumber<T> {
    pub fn parse(input: &str, span_name: &str) -> Result<Self, String> {
        // Full OTEL instrumentation + validation
        // ... (see example code for full implementation)
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn span(&self) -> &Span {
        &self.span
    }
}

// Usage
let validated = ValidatedNumber::<u32>::parse("42", "validated_parse")?;
assert_eq!(*validated.value(), 42);

// Validate span with OTEL
let validator = SpanValidator::new();
validator.validate(validated.span())?;

// Validate with Weaver (if available)
#[cfg(feature = "weaver")]
weaver_validator.validate(validated.span())?;
```

**Maximum Value:**
- ✓ Type-safe (validated at construction)
- ✓ Full OTEL spans and metrics
- ✓ Weaver semantic convention validation
- ✓ Prevents entire class of errors at compile time
- ✗ More code (70% more effort than 2nd idea)

---

## Decision Framework

### When to Use Each Idea

| Situation | Use | Reason |
|-----------|-----|--------|
| Prototype/spike | 1st | Fast iteration |
| Internal tools | 1st or 2nd | Depending on reuse |
| Production feature (most cases) | **2nd** | Best value/effort ratio |
| Shared library | 2nd or 3rd | Consider reuse level |
| Critical path | 3rd | Maximum safety needed |
| Framework/core infrastructure | 3rd | Worth the investment |

### Questions to Ask

1. **Will this be reused?** → Consider 2nd idea
2. **Is this critical?** → Consider 3rd idea
3. **Is this a prototype?** → 1st idea is fine
4. **Do I need observability?** → 2nd idea with OTEL
5. **Do I need type safety?** → 3rd idea

---

## Real-World Example

### Scenario: Parsing User Input

**Context:** Web API parsing request parameters

**1st Idea:** Parse string to u32
- Good for: Quick MVP
- Problem: What about other types?

**2nd Idea:** Generic parser
- Good for: Production API
- Benefit: Works for all types
- Cost: Just generics (minimal effort)

**3rd Idea:** Validated type with telemetry
- Good for: Critical authentication/payment systems
- Benefit: Type-safe + observable + validated
- Cost: More code, but worth it for critical paths

**Decision:** Use **2nd idea** for most endpoints, **3rd idea** for payment/auth endpoints.

---

## Common Patterns

### Pattern 1: Start with 2nd Idea

```rust
// Don't start with:
fn parse_specific_type(...) -> Result<SpecificType, Error>

// Start with:
fn parse_generic<T: FromStr>(...) -> Result<T, Error>
```

### Pattern 2: Upgrade Strategically

```rust
// Most code: 2nd idea
fn parse<T: FromStr>(input: &str) -> Result<T, Error>

// Critical paths: 3rd idea
struct Validated<T> { value: T }
impl<T> Validated<T> {
    fn parse(input: &str) -> Result<Self, Error>
}
```

---

## Troubleshooting

### When to Stop

**Stop at 2nd idea if:**
- Solves the problem completely
- No additional business value from 3rd idea
- Effort/benefit ratio not justified

**Continue to 3rd idea if:**
- Critical system requiring maximum safety
- Framework/library with many users
- Compliance/security requirements

---

## Next Steps

After understanding the philosophy, apply it:

1. **[Basic Test](basic_test.md)** - Apply 80/20 thinking to tests
2. **[Advanced Features](advanced_features.md)** - See 3rd idea patterns
3. **[Pattern Cookbook](../cookbook/src/README.md)** - Design patterns

---

## Related Documentation

- [Examples README](README.md) - All examples overview
- [Architecture](../docs/reference/ARCHITECTURE.md) - Design philosophy
- [API Reference](../docs/reference/API_REFERENCE.md) - Complete API documentation

---

## Reference

### Key Functions

**1st Idea:**
- `parse_u32_first_idea(input) -> Result<u32, String>`

**2nd Idea:**
- `parse_number_second_idea_no_otel<T>(input) -> Result<T, String>`
- `parse_number_second_idea<T>(input, span_name) -> Result<(T, Span), String>` (with OTEL)

**3rd Idea:**
- `ValidatedNumberNoOtel::<T>::parse(input) -> Result<ValidatedNumberNoOtel<T>, String>`
- `ValidatedNumber::<T>::parse(input, span_name) -> Result<ValidatedNumber<T>, String>` (with OTEL)

### Philosophy

- **1st Idea:** Immediate solution (20% value, 10% effort)
- **2nd Idea:** 80/20 sweet spot (80% value, 20% effort) - **Use this most often**
- **3rd Idea:** Maximum value (100% value, 70% effort)

---

**Quality is the default. Prevention beats detection.**

*Example: go_extra_mile.rs | Version: 1.2.0 | Updated: 2025-11-15*
