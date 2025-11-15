# Similar Patterns Analysis - After Macro Import Standardization

## Summary

After standardizing macro import patterns, we identified several similar inconsistencies (Mura) that follow the same pattern: **multiple ways to do the same thing, causing confusion and maintenance burden**.

## Pattern 1: Error Handling in Tests (Similar to Macro Imports)

### Status: ⚠️ INCONSISTENT

**Issue**: Multiple patterns for handling `Result` types in test code:
- `.unwrap()` - Simple, but provides poor error messages
- `.expect("message")` - Better error messages, but inconsistent messages
- `.unwrap_or_else(|e| panic!("Failed: {}", e))` - Most descriptive, but verbose

### Current State

**Pattern Distribution**:
- `.unwrap()`: 5 uses (mostly in `go_extra_mile_tests.rs`)
- `.expect("message")`: 5 uses (in `testcontainers/tests.rs`, `testcontainers/weaver.rs`)
- `.unwrap_or_else(|e| panic!(...))`: 15+ uses (in `weaver_integration.rs`, `weaver_macro_tests.rs`)

**Examples**:
```rust
// Pattern 1: Simple unwrap (poor error messages)
let span = span.unwrap();

// Pattern 2: expect with message (inconsistent messages)
let exec_result = result.expect("Exec should succeed after assert_ok");

// Pattern 3: unwrap_or_else with panic (most descriptive)
let fixture = WeaverTestFixture::new()
    .unwrap_or_else(|err| panic!("Failed to initialise Weaver fixture: {err}"));
```

### Impact
- **Inconsistent error messages**: Some tests have good error messages, others don't
- **Maintenance burden**: Different patterns in different files
- **User confusion**: Which pattern should I use?

### Standard Pattern (Proposed)
- **After `assert_ok!()`**: Use `.expect("descriptive message")` - `assert_ok!` already verified it's Ok
- **For critical setup**: Use `.unwrap_or_else(|e| panic!("Failed to {action}: {e}"))` - Most descriptive
- **For simple cases**: Use `.unwrap()` only when error is impossible (e.g., `SystemTime::now().duration_since(UNIX_EPOCH).unwrap()`)

### Reference Implementation
- **Best example**: `tests/weaver_integration.rs` - Uses `.unwrap_or_else(|e| panic!(...))` consistently
- **Acceptable example**: `tests/testcontainers/tests.rs` - Uses `.expect("message")` after `assert_ok!()`

---

## Pattern 2: Test Function Declaration (Similar to Macro Imports)

### Status: ⚠️ INCONSISTENT

**Issue**: Multiple patterns for declaring test functions:
- `test!(name, { body })` - Chicago TDD macro (39 uses)
- `#[test] fn name() { ... }` - Standard Rust (7 uses)
- `#[tokio::test] async fn name() { ... }` - Async tests (3 uses)

### Current State

**Pattern Distribution**:
- `test!` macro: 39 uses (most common)
- `#[test]`: 7 uses (for special cases like `#[should_panic]`)
- `#[tokio::test]`: 3 uses (for async tests)

**Examples**:
```rust
// Pattern 1: test! macro (standard)
test!(my_test, {
    // Arrange
    // Act
    // Assert
});

// Pattern 2: #[test] for special cases
#[test]
#[should_panic(expected = "Weaver live-check validation failed")]
fn weaver_macro_detects_violation() { ... }

// Pattern 3: #[tokio::test] for async
#[tokio::test]
async fn test_weaver_fixture_happy_path() { ... }
```

### Impact
- **Low impact**: Different patterns serve different purposes
- **Acceptable**: `#[test]` needed for `#[should_panic]`, `#[tokio::test]` needed for async
- **Standard**: `test!` macro is preferred for standard tests

### Standard Pattern (Established)
- **Standard tests**: Use `test!` macro (enforced in CODING_STANDARDS.md)
- **Special cases**: Use `#[test]` for `#[should_panic]` or other special attributes
- **Async tests**: Use `#[tokio::test]` for async tests (or `async_test!` macro if available)

### Status: ✅ DOCUMENTED
- Standards already documented in `docs/process/CODING_STANDARDS.md`
- Current usage is acceptable (different patterns serve different purposes)

---

## Pattern 3: Documentation Style in Examples (Similar to Macro Imports)

### Status: ✅ FIXED (Previously)

**Issue**: Inconsistent documentation style in `#[cfg(test)]` modules:
- `//` - Regular comments (1 file)
- `///` - Doc comments (4 files)

### Current State
- ✅ **Fixed**: All test modules now use `//` consistently
- ✅ **Documented**: Standards in `MURA_INVENTORY_EXAMPLES.md`

### Status: ✅ COMPLETE
- This was already fixed in a previous Mura elimination cycle

---

## Pattern 4: Import Organization (Similar to Macro Imports)

### Status: ✅ MOSTLY CONSISTENT

**Issue**: Import organization patterns:
- Some files group imports by type (std, external, local)
- Some files use `use chicago_tdd_tools::prelude::*;`
- Some files use explicit imports

### Current State
- **Import grouping**: Consistent (std → external → local)
- **Prelude usage**: Not consistently used (some files use it, others don't)
- **Explicit imports**: Most files use explicit imports

### Impact
- **Low impact**: Import organization is mostly consistent
- **Acceptable**: Different import styles are acceptable as long as they're organized

### Standard Pattern (Established)
- **Import order**: Alphabetical within groups (std → external → local)
- **Enforcement**: Clippy checks import organization

### Status: ✅ DOCUMENTED
- Standards documented in `docs/process/CODING_STANDARDS.md`

---

## Priority Recommendations

### High Priority
1. **Error Handling in Tests** - Standardize on `.unwrap_or_else(|e| panic!(...))` for critical setup, `.expect("message")` after `assert_ok!()`
   - **Impact**: Medium (affects test error messages and maintainability)
   - **Effort**: Low (mostly documentation and examples)

### Medium Priority
2. **Test Function Declaration** - Already documented, current usage is acceptable
   - **Impact**: Low (different patterns serve different purposes)
   - **Status**: ✅ Documented in CODING_STANDARDS.md

### Low Priority
3. **Documentation Style** - ✅ Already fixed
4. **Import Organization** - ✅ Already documented

---

## Similarities to Macro Import Issue

All these patterns share the same characteristics as the macro import issue:

1. **Multiple ways to do the same thing**: Different patterns for similar operations
2. **Inconsistent usage**: Different files use different patterns
3. **Maintenance burden**: Harder to maintain when patterns are inconsistent
4. **User confusion**: Unclear which pattern to use
5. **Documentation needed**: Standards need to be documented and enforced

---

## Next Steps

1. **Document error handling patterns** in `docs/process/CODING_STANDARDS.md`
2. **Add to code review checklist** - Verify error handling patterns in tests
3. **Update examples** - Show preferred error handling patterns
4. **Consider standardization** - If error handling inconsistency becomes a problem

---

## Summary

**Similar patterns found**: 4 patterns similar to macro import inconsistency
- **Error handling in tests**: ⚠️ Inconsistent (recommend standardization)
- **Test function declaration**: ✅ Documented (acceptable as-is)
- **Documentation style**: ✅ Fixed (already standardized)
- **Import organization**: ✅ Documented (mostly consistent)

**Recommendation**: Focus on **error handling in tests** as the next standardization target, as it has the highest impact and is most similar to the macro import issue.

