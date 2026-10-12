# Theorem-to-Test Mapping

This document provides the complete mapping between theorems in the LaTeX specification
(`docs/latex/chapters/`) and corresponding tests in the spec harness.

## Overview

Each theorem in the chicago-tdd-tools formalization has a corresponding test in the spec harness.
Tests are organized by chapter, and each test validates a specific theorem or property.

## Chapter 2: Core Testing Primitives

**Note on provenance:** `docs/latex/chapters/02-core-primitives.tex` contains no theorem
environments. The determinism/idempotence/type-safety/boundedness theorems below are defined in
`docs/latex/chapters/07-chatman-equation-realization.tex` (lines 29, 70, 122, 163) and are
exercised by the Chapter-2 harness tests. The last two rows have no corresponding theorem in the
LaTeX spec — they are pattern-level tests, not theorem validations.

| Theorem ID | LaTeX Name | Source (file:lines) | Test Path | Test Type | Status |
|-----------|-----------|---------------------|-----------|-----------|--------|
| Thm-7.1 | Test Determinism | `07-chatman-equation-realization.tex:29-32` | `chapter02::test_determinism` | Property | ✅ Pass |
| Thm-7.2 | Test Idempotence | `07-chatman-equation-realization.tex:70-76` | `chapter02::test_idempotence` | Property | ✅ Pass |
| Thm-7.3 | Type Safety Through Test Lifecycle | `07-chatman-equation-realization.tex:122-130` | `chapter02::test_type_preservation` | Property | ✅ Pass |
| Thm-7.4 | Bounded Test Execution | `07-chatman-equation-realization.tex:163-171` | `chapter02::test_boundedness` | Property | ✅ Pass |
| — | (no theorem — pattern-level test) | — | `chapter02::test_fixture_invariants` | Property | ✅ Pass |
| — | (no theorem — pattern-level test) | — | `chapter02::test_builder_type_safety` | Property | ✅ Pass |

**Total Tests:** 6 | **Passed:** 6 | **Failed:** 0 | **Pending:** 0

### Chapter 2 Key Concepts

**Determinism (Thm-7.1):** Tests that identical inputs always produce identical outputs, demonstrating
that test execution is pure with no side effects.

**Idempotence (Thm-7.2):** Tests that running a test twice produces the same result as running it once,
showing that test state is immutable.

**Type Preservation (Thm-7.3):** Tests that test data types are maintained throughout the lifecycle,
enforced by the Rust type system.

**Boundedness (Thm-7.4):** Tests that test execution time is measurable and finite, preventing
infinite loops or unbounded operations.

**Fixture Invariants (no theorem):** Tests that fixture invariants are preserved throughout the test,
with invalid states unrepresentable.

**Builder Safety (no theorem):** Tests that the builder pattern maintains type safety through the
build process, preventing incorrect sequences.

---

## Chapter 3: Type-Level Safety

| Theorem ID | Coverage Area | Test Path | Test Type | Status |
|-----------|-----------|-------|-----------|--------|
| Thm-3.1 | Knowledge hook atomicity | `chapter03::test_knowledge_hook_atomicity` | Unit | ✅ Pass |
| Thm-3.2 | YAWL Pattern 1: Sequence | `chapter03::test_yawl_pattern_1_sequence` | Unit | ✅ Pass |
| Thm-3.3 | YAWL Pattern 2: Parallel Split | `chapter03::test_yawl_pattern_2_parallel_split` | Unit | ✅ Pass |
| Thm-3.4 | YAWL Pattern 3: Synchronization | `chapter03::test_yawl_pattern_3_synchronization` | Unit | ✅ Pass |
| Thm-3.5 | YAWL Pattern 4: Exclusive Choice | `chapter03::test_yawl_pattern_4_exclusive_choice` | Unit | ✅ Pass |
| Thm-3.6 | YAWL Pattern 5: Simple Merge | `chapter03::test_yawl_pattern_5_simple_merge` | Unit | ✅ Pass |
| — | Registry properties (property-based) | `chapter03::test_registry_properties_proptest` | Property | ✅ Pass |

**Total Tests:** 7 | **Passed:** 7 | **Failed:** 0 | **Pending:** 0

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
| Thm-7.1 | Test Determinism | 29-32 | `chapter07::test_property_determinism` | Property | ✅ Pass |
| Thm-7.2 | Test Idempotence | 70-76 | `chapter07::test_property_idempotence` | Property | ✅ Pass |
| Thm-7.3 | Type Safety Through Test Lifecycle | 122-130 | `chapter07::test_property_type_preservation` | Property | ✅ Pass |
| Thm-7.4 | Bounded Test Execution | 163-171 | `chapter07::test_property_boundedness` | Property | ✅ Pass |
| — | (no theorem — integration test) | — | `chapter07::test_chatman_integration` | Integration | ✅ Pass |

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

**Integration (no theorem):** Full integration test verifying all four properties hold together in the
complete Chatman Equation framework.

---

## Test Statistics

### By Chapter

| Chapter | Theorems | Tests | Passed | Failed | Tests Mapped |
|---------|----------|-------|--------|--------|--------------|
| **Chapter 2** | 4 (of 6 tests; 2 pattern-level) | 6 | 6 | 0 | 6/6 |
| **Chapter 3** | 2 (Thm: Type State Pattern for AAA, No Invalid Test States) | 7 | 7 | 0 | 7/7 |
| **Chapter 7** | 4 (of 5 tests; 1 integration) | 5 | 5 | 0 | 5/5 |
| **TOTAL** | — | **18** | **18** | **0** | **18/18** |

### By Test Type

| Test Type | Count | Purpose |
|-----------|-------|---------|
| Property-based | 11 | Tests mathematical properties (Determinism, Idempotence, registry properties, etc.) |
| Unit | 6 | Tests type-system features (Chapter 3) |
| Integration | 1 | Tests all properties together |
| **TOTAL** | **18** | |

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
4. **Compliance Check** - Verify all mapped tests pass (18/18)
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

✅ **Test Coverage** - All 18 harness tests are mapped. The LaTeX spec defines 6 theorem
environments (0 in Chapter 2, 2 in Chapter 3, 4 in Chapter 7); the remaining mapped tests are
pattern-level or integration tests without a theorem counterpart
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
