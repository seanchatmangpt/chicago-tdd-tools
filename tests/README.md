# Test Directory Structure

This document explains the organization of tests in the Chicago TDD Tools project.

## Principles

### 1. Mirror Source Structure
Tests mirror the source module structure for easy navigation and maintenance. Each module in `src/` should have corresponding tests in `tests/`.

### 2. Shared Test Utilities
Common test helpers are consolidated in `tests/common.rs` to avoid duplication. Use `include!("common.rs")` to access shared utilities.

### 3. Clear Separation
- **Unit tests**: Test individual modules in isolation (`tests/{module}_tests.rs`)
- **Integration tests**: Test module interactions and real-world scenarios (`tests/{module}_integration_tests.rs`)
- **Feature tests**: Test feature-specific functionality (included in unit tests)

### 4. Consistent Naming
Use pattern: `tests/{module}_tests.rs` for unit tests, `tests/{module}_integration_tests.rs` for integration tests.

## Current Structure

```
tests/
├── common.rs                           # Shared test utilities (docker_available, etc.)
├── testcontainers_tests.rs            # Unit tests for testcontainers module
├── testcontainers_integration_tests.rs # Integration tests (real containers)
├── testcontainers_expert_tests.rs     # Expert-level scenarios (warmup, etc.)
├── testcontainers_weaver_tests.rs     # Weaver integration tests
└── go_extra_mile_tests.rs             # Integration tests for "go the extra mile" patterns
```

## Test File Organization

### testcontainers_tests.rs
Unit tests organized by category:
1. Error path testing - Tests all error variants (80% of bugs)
2. Boundary condition testing - Tests edge cases
3. Feature testing - Tests specific features (ports, env vars, wait conditions)
4. ExecResult structure testing - Tests ExecResult behavior
5. Container client testing - Tests ContainerClient functionality
6. Stress testing - Tests concurrent operations

### testcontainers_integration_tests.rs
Integration tests organized by category:
1. Resource cleanup testing - Tests cleanup in all code paths
2. Integration testing - Tests real container interactions

### testcontainers_expert_tests.rs
Expert-level scenarios that don't fit standard categories:
- Warmup testing (pre-pull commonly used images)

### testcontainers_weaver_tests.rs
Weaver integration tests:
- Tests Weaver Docker image availability
- Tests Weaver command execution in containers
- Tests Weaver live-check capabilities

## Using Shared Utilities

To use shared test utilities in a test file:

```rust
#[cfg(all(feature = "testcontainers", test))]
mod my_tests {
    mod common {
        include!("common.rs");
    }
    use common::require_docker;
    
    #[test]
    fn test_something() {
        require_docker(); // Test will fail if Docker is not available
        // Test code here...
    }
}
```

**Important**: Integration tests that require Docker should use `require_docker()`, which will cause the test to fail if Docker is unavailable. Use `skip_if_docker_unavailable()` only for tests where Docker is optional.

## Adding New Tests

When adding tests for a new module:

1. Create `tests/{module}_tests.rs` for unit tests
2. Create `tests/{module}_integration_tests.rs` for integration tests (if needed)
3. Use shared utilities from `tests/common.rs`
4. Follow the organizational patterns established in existing test files
5. Document test categories with section comments

## Migration History

The test structure was consolidated from:
- `testcontainers_error_tests.rs` → merged into `testcontainers_tests.rs`
- `testcontainers_feature_tests.rs` → merged into `testcontainers_tests.rs`
- `testcontainers_expert_tests.rs` → consolidated, duplicates removed, unique tests kept

This consolidation:
- Eliminated duplicate test code
- Improved maintainability
- Made test organization clearer
- Reduced file count while maintaining test coverage

