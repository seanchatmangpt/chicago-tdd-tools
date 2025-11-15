# Mura Inventory - Examples Folder

## Step 1: Identify Mura (Unevenness)

### Documentation Style Inconsistency

**Issue**: Inconsistent function-level documentation style inside `#[cfg(test)]` modules.

**Details**:
- `macro_examples.rs`: Uses `//` for function-level docs (4 functions)
  - `test_basic_aaa_pattern`
  - `test_result_handling`
  - `test_error_handling`
  - `test_with_custom_message`
- `cli_testing.rs`, `concurrency_testing.rs`, `otel_weaver_testing.rs`, `snapshot_testing.rs`: Use `///` for function-level docs (18 functions total)
  - `cli_testing.rs`: 3 functions
  - `concurrency_testing.rs`: 3 functions
  - `otel_weaver_testing.rs`: 8 functions
  - `snapshot_testing.rs`: 4 functions

**Impact**: Medium - Creates confusion about documentation standards. Test modules don't generate public docs, so `//` is more appropriate than `///`.

**Files Affected**: 5 files (1 uses `//`, 4 use `///`)

---

### Function Naming Pattern Inconsistency

**Issue**: Different naming patterns for example/test functions.

**Details**:
- `example_*` functions: 11 functions across 3 files
  - `basic_test.rs`: 3 functions (`example_fixture_creation`, `example_data_building`, `example_error_handling`)
  - `testcontainers_example.rs`: 6 functions (`example_basic_container`, `example_container_with_ports`, etc.)
  - `advanced_features.rs`: 2 functions (`example_type_level_arithmetic`, `example_type_state_pattern`)
- `test_*` functions: 10 functions inside `#[cfg(test)]` modules
  - `cli_testing.rs`: 3 functions
  - `concurrency_testing.rs`: 3 functions
  - `snapshot_testing.rs`: 4 functions
- `main()` functions: 7 files have `main()` functions

**Impact**: Low - Different patterns serve different purposes, but could be more consistent.

**Files Affected**: All 11 files

---

### Module Structure Inconsistency

**Issue**: Different module structures across example files.

**Details**:
- Files with `#[cfg(test)]` modules: 6 files
  - `cli_testing.rs`
  - `concurrency_testing.rs`
  - `macro_examples.rs`
  - `otel_weaver_testing.rs`
  - `snapshot_testing.rs`
  - `go_extra_mile.rs`
- Files with standalone functions: 5 files
  - `basic_test.rs`
  - `testcontainers_example.rs`
  - `advanced_features.rs`
  - `property_testing.rs`
  - `mutation_testing.rs`
- Files with both `main()` and test modules: 6 files

**Impact**: Low - Different structures serve different purposes (examples vs tests).

**Files Affected**: All 11 files

---

### Return Type Inconsistency

**Issue**: Inconsistent return types for `example_*` functions.

**Details**:
- `Result<(), Box<dyn std::error::Error>>`: 6 functions
  - `basic_test.rs`: 2 functions
  - `testcontainers_example.rs`: 4 functions
- `()`: 5 functions
  - `basic_test.rs`: 1 function
  - `testcontainers_example.rs`: 2 functions
  - `advanced_features.rs`: 2 functions

**Impact**: Medium - Inconsistent error handling patterns. Some functions handle errors, others don't.

**Files Affected**: 3 files

---

## Step 2: Measure Variability

### Documentation Style Consistency
- **Violations**: 1 file uses `//`, 4 files use `///` inside test modules
- **Inconsistency Score**: Medium (22% use `//`, 78% use `///`)
- **Standard**: Should use `//` for test module functions (test modules don't generate public docs)

### Function Naming Consistency
- **Patterns**: 3 different patterns (`example_*`, `test_*`, `main()`)
- **Inconsistency Score**: Low (different patterns serve different purposes)
- **Standard**: Current patterns are acceptable, but could be documented

### Module Structure Consistency
- **Patterns**: 2 different patterns (test modules vs standalone functions)
- **Inconsistency Score**: Low (different structures serve different purposes)
- **Standard**: Current patterns are acceptable

### Return Type Consistency
- **Patterns**: 2 different patterns (`Result<...>` vs `()`)
- **Inconsistency Score**: Medium (45% use `Result`, 55% use `()`)
- **Standard**: Should use `Result` for functions that can fail

---

## Step 3: Standardize

### Standard Definition

#### Documentation Standards
- **Inside `#[cfg(test)]` modules**: Use `//` for function-level documentation
  - Rationale: Test modules don't generate public documentation, so `//` is more appropriate
- **Outside test modules**: Use `///` for function-level documentation
  - Rationale: Public API documentation requires `///`

#### Function Naming Standards
- **Example functions**: Use `example_*` prefix for standalone example functions
- **Test functions**: Use `test_*` prefix for functions inside `#[cfg(test)]` modules
- **Main functions**: Use `main()` for executable examples

#### Return Type Standards
- **Example functions that can fail**: Use `Result<(), Box<dyn std::error::Error>>`
- **Example functions that cannot fail**: Use `()`
- **Test functions**: Use `()` (test framework handles errors via panics)

### Reference Implementation

**Documentation Style**: Use `macro_examples.rs` as reference
- Uses `//` for function-level docs inside `#[cfg(test)]` module
- Consistent with Rust best practices for test modules

**Return Types**: Use `basic_test.rs` and `testcontainers_example.rs` as reference
- Functions that can fail return `Result<(), Box<dyn std::error::Error>>`
- Proper error propagation with `?` operator

---

## Step 4: Apply Consistently

### Priority Order

1. **High Priority**: Documentation style consistency (affects 4 files)
2. **Medium Priority**: Return type consistency (affects 3 files)
3. **Low Priority**: Function naming and module structure (acceptable as-is, but document)

### Implementation Plan

1. **Fix Documentation Style** (4 files):
   - Change `///` to `//` in `cli_testing.rs` (3 functions)
   - Change `///` to `//` in `concurrency_testing.rs` (3 functions)
   - Change `///` to `//` in `otel_weaver_testing.rs` (8 functions)
   - Change `///` to `//` in `snapshot_testing.rs` (4 functions)

2. **Fix Return Types** (3 files):
   - Review `basic_test.rs`: 1 function uses `()`, consider if it should return `Result`
   - Review `testcontainers_example.rs`: 2 functions use `()`, consider if they should return `Result`
   - Review `advanced_features.rs`: 2 functions use `()`, consider if they should return `Result`

3. **Document Standards**: Add documentation explaining the standards

---

## Step 5: Control

### Automated Checks
- **Documentation**: Add clippy check for doc comment style in test modules
- **Return Types**: Manual review (difficult to automate)

### Code Review Checklist
- [ ] Function-level docs in test modules use `//` not `///`
- [ ] Example functions that can fail return `Result`
- [ ] Function naming follows standards (`example_*`, `test_*`, `main()`)

### Documentation
- Document standards in `docs/process/CODING_STANDARDS.md`
- Add examples showing correct patterns

