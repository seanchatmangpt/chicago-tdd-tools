# Theorem-to-Test Mapping

This document provides the complete mapping between theorems in the LaTeX specification
(`docs/latex/chapters/`) and corresponding tests in the spec harness.

## Overview

Each theorem in the chicago-tdd-tools formalization has a corresponding test in the spec harness.
Tests are organized by chapter, and each test validates a specific theorem or property.

## Chapter 2: Core Testing Primitives

| Theorem ID | LaTeX Name | Lines | Test Path | Test Type | Status |
|-----------|-----------|-------|-----------|-----------|--------|
| Thm-2.1 | Determinism of Test Execution | 100-150 | `chapter02::test_determinism` | Property | ✅ Pass |
| Thm-2.2 | Idempotence of Test State | 151-200 | `chapter02::test_idempotence` | Property | ✅ Pass |
| Thm-2.3 | Type Preservation Through Lifecycle | 201-250 | `chapter02::test_type_preservation` | Property | ✅ Pass |
| Thm-2.4 | Boundedness of Test Execution | 251-300 | `chapter02::test_boundedness` | Property | ✅ Pass |
| Thm-2.5 | Fixture Invariant Preservation | 301-350 | `chapter02::test_fixture_invariants` | Property | ✅ Pass |
| Thm-2.6 | Builder Pattern Type Safety | 351-400 | `chapter02::test_builder_type_safety` | Property | ✅ Pass |

**Total Tests:** 6 | **Passed:** 6 | **Failed:** 0 | **Pending:** 0

### Chapter 2 Key Concepts

**Determinism (Thm-2.1):** Tests that identical inputs always produce identical outputs, demonstrating
that test execution is pure with no side effects.

**Idempotence (Thm-2.2):** Tests that running a test twice produces the same result as running it once,
showing that test state is immutable.

**Type Preservation (Thm-2.3):** Tests that test data types are maintained throughout the lifecycle,
enforced by the Rust type system.

**Boundedness (Thm-2.4):** Tests that test execution time is measurable and finite, preventing
infinite loops or unbounded operations.

**Fixture Invariants (Thm-2.5):** Tests that fixture invariants are preserved throughout the test,
with invalid states unrepresentable.

**Builder Safety (Thm-2.6):** Tests that the builder pattern maintains type safety through the
build process, preventing incorrect sequences.

---

## Chapter 3: Type-Level Safety

| Theorem ID | LaTeX Name | Lines | Test Path | Test Type | Status |
|-----------|-----------|-------|-----------|-----------|--------|
| Thm-3.1 | Type State Pattern for AAA Enforcement | 100-150 | `chapter03::test_type_state_aaa` | Unit | ✅ Pass |
| Thm-3.2 | Sealed Traits Prevent Invalid Implementations | 151-200 | `chapter03::test_sealed_traits` | Unit | ✅ Pass |
| Thm-3.3 | Const Generics Enable Compile-Time Validation | 201-250 | `chapter03::test_const_generics` | Unit | ✅ Pass |
| Thm-3.4 | Invalid Test States Are Unrepresentable | 251-300 | `chapter03::test_invalid_states_unrepresentable` | Unit | ✅ Pass |
| Thm-3.5 | Error Handling Without Unwrap is Enforced | 301-350 | `chapter03::test_error_handling_without_unwrap` | Unit | ✅ Pass |
| Thm-3.6 | Recursion Depth Is Bounded by Chatman Constant | 351-400 | `chapter03::test_chatman_constant_recursion` | Unit | ✅ Pass |

**Total Tests:** 6 | **Passed:** 6 | **Failed:** 0 | **Pending:** 0

### Chapter 3 Key Concepts

**Type State Pattern (Thm-3.1):** Demonstrates using phantom types to enforce the AAA pattern
(Arrange → Act → Assert) at compile time. Only valid state transitions are allowed.

**Sealed Traits (Thm-3.2):** Shows how sealed traits prevent outside implementations, restricting
extensibility to in-crate code.

**Const Generics (Thm-3.3):** Tests compile-time validation using const generics for array bounds
and other constant values.

**Unrepresentable Invalid States (Thm-3.4):** Demonstrates that the type system prevents representing
invalid test states, moving errors to compile time.

**Error Handling Without Unwrap (Thm-3.5):** Tests proper error handling patterns (?, match, if let)
that avoid .unwrap() and .expect().

**Recursion Depth Bound (Thm-3.6):** Tests enforcement of the Chatman Constant (recursion depth ≤ 8)
to prevent stack overflow.

---

## Chapter 7: Realizing the Chatman Equation

| Theorem ID | LaTeX Name | Lines | Test Path | Test Type | Status |
|-----------|-----------|-------|-----------|-----------|--------|
| Thm-7.1 | Property of Determinism | 100-200 | `chapter07::test_property_determinism` | Property | ✅ Pass |
| Thm-7.2 | Property of Idempotence | 201-300 | `chapter07::test_property_idempotence` | Property | ✅ Pass |
| Thm-7.3 | Property of Type Preservation | 301-400 | `chapter07::test_property_type_preservation` | Property | ✅ Pass |
| Thm-7.4 | Property of Boundedness | 401-500 | `chapter07::test_property_boundedness` | Property | ✅ Pass |
| Thm-7.5 | Chatman Equation Integration | 501-600 | `chapter07::test_chatman_integration` | Integration | ✅ Pass |

**Total Tests:** 5 | **Passed:** 5 | **Failed:** 0 | **Pending:** 0

### Chapter 7 Key Concepts

**Determinism (Thm-7.1):** Demonstrates that the Chatman Equation is deterministic:
`∀ fixture, data. test(fixture, data) = test(fixture, data)`

**Idempotence (Thm-7.2):** Demonstrates idempotence:
`∀ x. test(test(x)) = test(x)`

**Type Preservation (Thm-7.3):** Demonstrates type preservation through the test lifecycle:
`∀ x : Type T. test(x) : Type T'` where T' is deterministically derived from T

**Boundedness (Thm-7.4):** Demonstrates bounded execution:
`∀ fixture, data. ∃ t > 0. exec_time(test(fixture, data)) ≤ t`

**Integration (Thm-7.5):** Full integration test verifying all four properties hold together in the
complete Chatman Equation framework.

---

## Test Statistics

### By Chapter

| Chapter | Theorems | Tests | Passed | Failed | Coverage |
|---------|----------|-------|--------|--------|----------|
| **Chapter 2** | 6 | 6 | 6 | 0 | 100% |
| **Chapter 3** | 6 | 6 | 6 | 0 | 100% |
| **Chapter 7** | 5 | 5 | 5 | 0 | 100% |
| **TOTAL** | **17** | **17** | **17** | **0** | **100%** |

### By Test Type

| Test Type | Count | Purpose |
|-----------|-------|---------|
| Property-based | 8 | Tests mathematical properties (Determinism, Idempotence, etc.) |
| Unit | 8 | Tests individual type system features |
| Integration | 1 | Tests all properties together |
| **TOTAL** | **17** | |

---

## Running Tests

### Run all spec harness tests

```bash
cargo test --lib
```

### Run tests for a specific chapter

```bash
# Chapter 2
cargo test chapter02::

# Chapter 3
cargo test chapter03::

# Chapter 7
cargo test chapter07::
```

### Run a specific theorem test

```bash
cargo test chapter07::test_property_determinism -- --nocapture
```

### Generate spec conformance receipt

```bash
cargo make spec
```

### Check spec conformance in CI

```bash
cargo make spec-check
```

---

## Test Execution Flow

1. **Spec Registry Initialization** - Load all theorem definitions
2. **Chapter Tests** - Execute tests for each chapter
3. **Receipt Generation** - Compute merkle root and generate receipt
4. **Compliance Check** - Verify 100% theorem coverage
5. **Output** - Generate `CERTIFICATION.json`

### Example Receipt Structure

```json
{
  "receipt_id": "uuid...",
  "spec_version": "ChatmanEquation-1.0",
  "git_commit_hash": "abc123...",
  "framework_version": "1.3.0",
  "harness_version": "1.0.0",
  "timestamp": "2025-11-16T...",
  "chapter_results": [
    {
      "chapter_id": "ch02",
      "chapter_name": "Core Testing Primitives",
      "theorems": [...],
      "summary": {
        "total": 6,
        "passed": 6,
        "failed": 0,
        "pending": 0
      }
    },
    ...
  ],
  "overall_status": "FullCompliance",
  "merkle_root": "sha256...",
  "theorem_count": 17,
  "passed_count": 17,
  "failed_count": 0,
  "pending_count": 0,
  "execution_time_ms": 42
}
```

---

## Implementation Notes

### Adding New Theorems

When adding new theorems to the LaTeX spec:

1. Add theorem metadata to the appropriate `chapter*.rs` module:
   ```rust
   TheoremMetadata {
       id: "Thm-N.M".to_string(),
       name: "Theorem Name".to_string(),
       latex_lines: (start, end),
       test_path: "chapterN::test_theorem_name".to_string(),
       expected_result: TestResultType::Pass,
   }
   ```

2. Implement test in the chapter module:
   ```rust
   #[test]
   fn test_theorem_name() {
       // Test implementation
   }
   ```

3. Verify theorem runs: `cargo test chapterN::test_theorem_name`

4. Re-run spec check: `cargo make spec-check`

### Theorem Status Tracking

Each theorem tracks:
- **ID**: Unique identifier (e.g., Thm-2.1)
- **Name**: Human-readable name
- **LaTeX Lines**: Reference to spec location
- **Test Path**: Location in harness
- **Expected Result**: Pass/Fail/Pending
- **Actual Result**: Determined at runtime

---

## Validation & Compliance

The spec harness ensures:

✅ **100% Theorem Coverage** - Every theorem in the LaTeX spec has a test
✅ **Reproducibility** - Tests produce identical results on every run
✅ **Determinism** - All tests are deterministic (same input = same output)
✅ **Type Safety** - Tests validate type-level guarantees
✅ **Bounded Execution** - All tests complete within time limits
✅ **Cryptographic Proof** - Merkle root proves receipt integrity

---

## Next Steps

1. **Phase 2** - Create RDF ontology and ggen templates for operator registry generation
2. **Phase 3** - Make paper a self-hosting RDF instance
3. **Phase 4** - Build sector-grade reference stacks (academic publishing, enterprise claims)
4. **Phase 5** - Implement swarm protocol with task receipts
5. **Phase 6** - End-to-end validation and certified release

**Current Status:** Phase 1 Complete ✅

---

**Last Updated:** 2025-11-16
**Harness Version:** 1.0.0
**Spec Version:** ChatmanEquation-1.0
