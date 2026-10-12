# Test Directory Structure

This document explains the organization of tests in the Chicago TDD Tools project.

## Principles

### 1. Mirror Source Structure
Tests mirror the source module structure for easy navigation and maintenance. Each module in `src/` should have corresponding tests in `tests/`.

### 2. Shared Test Utilities
Common test helpers live in `tests/common/` (a directory module: `mod.rs`, `integration.rs`). Test binaries include them via `#[path]`/`mod` declarations or `include!("test_common.inc")` for shared snippets.

### 3. Clear Separation
- **Unit tests**: Test individual modules in isolation (`tests/{module}_tests.rs`)
- **Integration tests**: Test module interactions and real-world scenarios (`tests/{module}_integration.rs` or `tests/{module}_integration_tests.rs`)
- **Docker-gated tests**: Tests requiring real containers (testcontainers/weaver) live under `tests/testcontainers/` behind the `testcontainers` feature

### 4. Consistent Naming
Use pattern: `tests/{module}_tests.rs` for suites, `tests/{module}_integration.rs` for integration suites.

## Current Structure

Verified against the tree (do not edit this list without re-running `ls tests/`):

```
tests/
├── common/                             # Shared test utilities (directory module)
│   ├── mod.rs                          #   docker_available, require_docker, etc.
│   └── integration.rs                  #   integration test helpers
├── testcontainers.rs                   # Shim that include!s tests/testcontainers/*.rs
├── testcontainers/                     # Testcontainers suites (feature-gated, Docker)
│   ├── tests.rs                        #   Unit tests for the testcontainers module
│   ├── integration.rs                  #   Integration tests (real containers)
│   ├── expert.rs                       #   Expert scenarios (warmup, etc.)
│   └── weaver.rs                       #   Weaver live-check integration tests
├── snapshots/                          # Insta snapshot fixtures
│   └── star_toml_e2e__star_toml_default_config_snapshot.snap
├── test_common.inc                     # Shared snippet included by test files
├── blake3_receipt_tests.rs             # BLAKE3 receipt chain tests
├── challenger_m1_2_tests.rs            # Challenger M1.2 tests
├── dependency_lock_guard.rs            # Dependency lock guard tests
├── governance_tests.rs                 # Governance module tests
├── hyper_advanced_integration.rs       # Hyper advanced integration tests
├── ocel_tests.rs                       # OCEL 2.0 process mining tests
├── star_toml_e2e.rs                    # Star TOML end-to-end tests (snapshot-backed)
├── v1_3_0_integration.rs               # v1.3.0 integration tests
├── weaver_integration.rs               # Weaver integration tests
├── weaver_macro_tests.rs               # Weaver macro tests
├── go_extra_mile_tests.rs              # "Go the extra mile" pattern tests
├── cli_proof_tests.rs                  # cli_proof module tests (feature cli-proof)
├── prelude_surface_tests.rs            # Prelude public-surface tests
├── trybuild.rs                         # Compile-fail suite harness (trybuild)
└── compile_fail/                       # Compile-fail fixtures (seed *.stderr via TRYBUILD=overwrite)
```

## Test File Organization

### testcontainers/ (via tests/testcontainers.rs shim)
Suites organized by category:
1. `tests.rs` — Error path testing, boundary conditions, feature testing (ports, env vars, wait conditions), ExecResult structure, container client, stress testing
2. `integration.rs` — Resource cleanup and real container interactions
3. `expert.rs` — Expert-level scenarios (e.g. image warmup)
4. `weaver.rs` — Weaver Docker image availability, command execution in containers, live-check capabilities

### Integration suites
`*_integration.rs` / `*_integration_tests.rs` files exercise module interactions and
real-world scenarios; Docker-dependent ones call `require_docker()` first.

## Using Shared Utilities

To use shared test utilities in a test file:

```rust
#[cfg(all(feature = "testcontainers", test))]
mod my_tests {
    include!("test_common.inc");
    // or reference tests/common via #[path] mod declarations

    #[test]
    fn test_something() {
        require_docker(); // Test will fail if Docker is not available
        // Test code here...
    }
}
```

**Important**: Integration tests that require Docker should use `require_docker()`, which will cause the test to fail if Docker is unavailable.

## Running Tests

```bash
just test-unit          # Fast unit tests (no Docker), 300s timeout
just test-integration   # Integration tests (requires Docker)
just test-all           # Unit + integration
```

## Adding New Tests

When adding tests for a new module:

1. Create `tests/{module}_tests.rs` (or `tests/{module}_integration.rs` for integration suites)
2. Use shared utilities from `tests/common/` or `include!("test_common.inc")`
3. Follow the organizational patterns established in existing test files
4. Document test categories with section comments

## Migration History

The test structure was consolidated from:
- `testcontainers_error_tests.rs` → merged into `testcontainers/tests.rs`
- `testcontainers_feature_tests.rs` → merged into `testcontainers/tests.rs`
- `testcontainers_expert_tests.rs` → became `testcontainers/expert.rs`
- Flat `common.rs` → became the `tests/common/` directory module

This consolidation:
- Eliminated duplicate test code
- Improved maintainability
- Made test organization clearer
- Reduced file count while maintaining test coverage
