# CLAUDE.md - AI Assistant Guide for Chicago TDD Tools

**Last Updated:** 2025-11-14
**Version:** 1.1.0
**Purpose:** Comprehensive guide for AI assistants working with the Chicago TDD Tools codebase

---

## Table of Contents

1. [Repository Overview](#repository-overview)
2. [Codebase Structure](#codebase-structure)
3. [Development Workflows](#development-workflows)
4. [Key Conventions and Patterns](#key-conventions-and-patterns)
5. [Build System and Tooling](#build-system-and-tooling)
6. [Testing Strategies](#testing-strategies)
7. [CI/CD Pipeline](#cicd-pipeline)
8. [Quality Standards](#quality-standards)
9. [Observability and Weaver](#observability-and-weaver)
10. [Common Tasks](#common-tasks)
11. [Documentation Structure](#documentation-structure)
12. [Error Prevention Patterns](#error-prevention-patterns)

---

## Repository Overview

### What is Chicago TDD Tools?

Chicago TDD Tools is a comprehensive Rust testing framework designed to enforce **Chicago-style Test-Driven Development (Classicist TDD)** principles. The framework emphasizes:

- **State-based testing** - Verify outputs/state, not implementation details
- **Real collaborators** - Use actual dependencies, minimize mocks
- **Behavior verification** - Verify what code does, not how it does it
- **AAA Pattern enforcement** - Arrange-Act-Assert structure enforced at compile time

### Project Statistics

- **Language:** Rust (Edition 2021, Rust 1.70+)
- **Lines of Code:** ~8,000 lines
- **License:** MIT
- **Repository:** https://github.com/seanchatmangpt/knhk
- **Current Version:** 1.1.0

### Core Philosophy: Elite Rust Principles

The codebase embodies several core Rust design principles:

1. **Type-First Thinking** - Types encode invariants; compiler enforces correctness
2. **Zero-Cost Abstractions** - Performance through generics, const generics, and macros
3. **Poka-Yoke Design** - Error prevention at compile time (Japanese: "mistake-proofing")
4. **Compile-Time Guarantees** - If it compiles, invariants are maintained
5. **80/20 Thinking** - "Go the extra mile" - solve 80% of problems with 20% extra effort

---

## Codebase Structure

### Directory Layout

```
chicago-tdd-tools/
├── src/                          # Main library (~8000 lines)
│   ├── lib.rs                    # Library root with prelude and module exports
│   ├── core/                     # Core testing infrastructure
│   │   ├── fixture.rs            # Test fixtures and setup utilities
│   │   ├── async_fixture.rs      # Async fixtures (async feature)
│   │   ├── builders.rs           # Fluent builders for test data
│   │   ├── assertions.rs         # Assertion helpers and utilities
│   │   ├── state.rs              # Type-level AAA enforcement
│   │   ├── poka_yoke.rs          # Error prevention through type safety
│   │   ├── const_assert.rs       # Compile-time assertions
│   │   ├── alert.rs              # Visual alert helpers (log integration)
│   │   └── macros/               # Test and assertion macros
│   ├── testing/                  # Advanced testing techniques
│   │   ├── property/             # Property-based testing (proptest)
│   │   ├── mutation/             # Mutation testing framework
│   │   ├── snapshot/             # Snapshot testing (insta)
│   │   ├── concurrency/          # Concurrency testing (loom)
│   │   ├── cli/                  # CLI testing (trycmd)
│   │   └── generator/            # Test code generation
│   ├── validation/               # Quality & constraint validation
│   │   ├── coverage/             # Test coverage analysis
│   │   ├── guards/               # Guard constraint enforcement
│   │   ├── jtbd/                 # Jobs To Be Done validation
│   │   └── performance/          # RDTSC benchmarking/tick measurement
│   ├── observability/            # Telemetry & observability
│   │   ├── otel/                 # OpenTelemetry span/metric validation
│   │   ├── weaver/               # Weaver live validation
│   │   └── unified.rs            # Unified observability API
│   ├── integration/              # Integration testing
│   │   └── testcontainers/       # Docker container support
│   └── bin/
│       └── weaver_smoke.rs       # Weaver smoke test binary
├── proc_macros/                  # Procedural macros (separate crate)
│   └── src/lib.rs                # #[tdd_test], #[fixture], #[derive(TestBuilder)]
├── tests/                        # Integration & compilation tests
│   ├── common.rs                 # Shared test utilities
│   ├── go_extra_mile_tests.rs    # "Go extra mile" patterns
│   ├── weaver_integration.rs     # Weaver integration tests
│   └── compile-fail/             # Compile-fail tests (trybuild)
├── examples/                     # Usage examples (8 example files)
├── cookbook/                     # Alexander-style pattern language
│   └── src/                      # 20 documented patterns
├── docs/                         # User-facing documentation
│   ├── getting-started/          # Quick/full guides, troubleshooting
│   ├── reference/                # API reference, architecture
│   ├── features/                 # Feature documentation
│   ├── process/                  # Development process (FMEA, SPR)
│   ├── releases/                 # Changelog & release notes
│   └── research/                 # Analysis & innovation docs
├── .github/workflows/            # CI/CD configuration
├── scripts/                      # Utility scripts (git hooks, etc.)
└── Makefile.toml                 # Build system (cargo-make)
```

### Module Organization: Capability Groups

Modules are organized by **functionality**, not alphabetically:

- **core/** - Foundational testing primitives (fixtures, builders, assertions)
- **testing/** - Advanced testing techniques (property, mutation, snapshot, concurrency)
- **validation/** - Quality assurance & constraints (coverage, guards, JTBD, performance)
- **observability/** - Telemetry & observability (OTEL, Weaver)
- **integration/** - Integration testing support (testcontainers)

### Backward Compatibility

Modules are re-exported at the crate root for backward compatibility:
- New code should use capability group paths (e.g., `core::fixture`, `validation::guards`)
- Old code using crate root imports will continue to work

---

## Development Workflows

### Essential Pre-Commit Workflow

**ALWAYS run before committing:**

```bash
cargo make pre-commit  # Format + Lint + Unit Tests
```

This command runs:
1. `cargo make fmt` - Format code (~5s)
2. `cargo make lint` - Clippy with all/pedantic/nursery/cargo lints (~5s)
3. `cargo make test-unit` - Unit tests only (~10s)
4. `cargo make dead-code-check` - Detect undeclared modules

**Expected total time:** ~20s

### CI Simulation (Recommended)

To catch CI failures before pushing:

```bash
cargo make ci-local  # Simulate CI environment locally
```

This runs the same checks as GitHub Actions CI, including:
- Format check with git diff validation
- Clippy lint check with retry logic
- Unit tests with environment validation
- Production code safety checks (unwrap/expect)

### Git Hooks Installation (Highly Recommended)

Install pre-commit hooks to prevent unwrap/expect in production code:

```bash
cargo make install-hooks
```

This prevents production panics by catching `.unwrap()` and `.expect()` calls before commit.

### Standard Development Cycle

```bash
# 1. Check compilation
cargo make check

# 2. Format code
cargo make fmt

# 3. Run unit tests (fast iteration)
cargo make test-unit

# 4. Run linter before committing
cargo make lint

# 5. Pre-commit validation
cargo make pre-commit
```

### Release Workflow

```bash
# Comprehensive release validation
cargo make release-validate

# Full release pipeline (validate + CI + docs)
cargo make release
```

---

## Key Conventions and Patterns

### 1. Poka-Yoke Design (Error Prevention)

**Core Principle:** Prevent errors at compile time, not runtime.

**Implementation:**
- All clippy warnings treated as errors (`-D warnings`)
- Production code must not use `.unwrap()` or `.expect()`
- Type system enforces invariants
- CI fails on any lint warnings

**Example:**
```rust
// ❌ NEVER in production code
let value = result.unwrap();

// ✅ ALWAYS use proper error handling
let value = result?;
// or
let value = match result {
    Ok(v) => v,
    Err(e) => return Err(e.into()),
};
```

### 2. Type-First Thinking

**Core Principle:** Use types as a design tool. If it compiles, invariants are maintained.

**Techniques:**
- PhantomData for type-level state machines
- Const generics for compile-time parameters
- Sealed traits for API safety
- Type state enforcement

**Example (from src/core/state.rs):**
```rust
// Type-level state machine for AAA pattern
pub struct Arrange;
pub struct Act;
pub struct Assert;

pub struct TestCase<State> {
    _state: PhantomData<State>,
}
```

### 3. 80/20 Thinking: "Go the Extra Mile"

For every task, consider three ideas:

1. **First idea:** Solve the immediate problem
2. **Second idea:** Solve 80% of related problems with 20% additional effort (SWEET SPOT)
3. **Third idea:** Maximum value with type-level solutions (quality-first)

**Prefer the second idea:** Maximum value with reasonable effort.

**Example:** See `tests/go_extra_mile_tests.rs` for implementation patterns.

### 4. AAA Pattern Enforcement

All tests must follow **Arrange-Act-Assert** pattern:

```rust
use chicago_tdd_tools::prelude::*;

test!(test_example, {
    // Arrange - Set up test data
    let input = 5;

    // Act - Execute the behavior
    let result = input * 2;

    // Assert - Verify the outcome
    assert_eq!(result, 10);
});
```

Macros enforce this pattern at compile time through type-level state machines.

### 5. Zero-Cost Abstractions

**Principles:**
- Generics over trait objects (static dispatch > dynamic dispatch)
- Const generics for compile-time parameters
- Inline annotations for hot paths
- Stack allocation over heap (`&str` > `String`, references > owned)

### 6. Macro Pattern Enforcement

**Available Macros:**

**Test Macros:**
- `test!` - Synchronous tests with AAA enforcement
- `async_test!` - Async tests (1s timeout)
- `fixture_test!` - Async tests with automatic fixture setup
- `performance_test!` - Performance tests with tick budget validation

**Assertion Macros:**
- `assert_ok!` - Assert Result is Ok
- `assert_err!` - Assert Result is Err
- `assert_in_range!` - Assert value in range
- `assert_eq_msg!` - Assert with custom message

**Alert Macros:**
- `alert_critical!` - Critical severity (log::error!)
- `alert_warning!` - Warning severity (log::warn!)
- `alert_info!` - Info severity (log::info!)
- `alert_success!` - Success message (log::info!)
- `alert_debug!` - Debug severity (log::debug!)

**Procedural Macros:**
- `#[tdd_test]` - Zero-boilerplate tests with AAA validation
- `#[fixture]` - Automatic fixture setup/teardown
- `#[derive(TestBuilder)]` - Automatic fluent builder generation

### 7. Documentation Standards: Alexander-Style Patterns

The codebase documents 20 patterns across three categories:

**Testing Patterns (5):**
1. AAA Pattern
2. Error Path Testing (80% of bugs)
3. Boundary Conditions
4. Resource Cleanup
5. Real Collaborators

**Architecture Patterns (5):**
6. Generic Base Layer
7. Extension Layer
8. Composition Over Duplication
9. Single Source of Truth
10. Capability Grouping

**Design Patterns (10):**
11. Zero-Cost Abstractions
12. Type Safety with GATs
13. Sealed Traits for API Safety
14. Compile-Time Validation
15. Type State Enforcement
16. Fixture Lifecycle Management
17. Builder-Driven Test Data
18. Timeout Defense in Depth
19. Feature Gate Slices
20. Macro Pattern Enforcement

**See:** `cookbook/src/` for detailed pattern documentation.

---

## Build System and Tooling

### Critical: Always Use cargo-make

**⚠️ NEVER use `cargo` commands directly. ALWAYS use `cargo make`.**

**Reasons:**
1. Handles proc-macro crates correctly
2. Includes timeout protection (prevents hanging)
3. Single source of truth for build configuration
4. Consistent across development and CI

### Required Tool Installation

```bash
# Required: cargo-make (MUST be installed first)
cargo install cargo-make

# Verify installation
cargo make --version

# Recommended: Git hooks for production code safety
cargo make install-hooks

# Optional but useful
cargo install cargo-nextest      # Parallel test runner
cargo install cargo-llvm-cov      # Coverage reporting
cargo install cargo-insta         # Snapshot testing
cargo install cargo-mutants       # Mutation testing
```

### Core Build Commands

```bash
# Compilation checks (5s timeout)
cargo make check              # Compile check without building
cargo make build              # Debug build
cargo make build-release      # Release build (30s timeout)

# Formatting (5s timeout)
cargo make fmt                # Format all code

# Linting (300s timeout for cold start)
cargo make lint               # Clippy with all/pedantic/nursery/cargo
cargo make clippy             # Alias for lint

# Testing
cargo make test               # Unit tests only (excludes integration)
cargo make test-unit          # Unit tests only (same as test)
cargo make test-integration   # Integration tests (requires Docker)
cargo make test-all           # Unit + integration tests

# Specialized testing
cargo make test-property      # Property-based tests
cargo make test-mutation      # Mutation testing
cargo make test-snapshot      # Snapshot tests
cargo make test-examples      # Example tests
cargo make test-timings       # Generate timing report for slow tests

# Coverage (manual, not part of pre-commit)
cargo make coverage           # Coverage report
cargo make coverage-report    # HTML coverage report

# Cleanup
cargo make clean              # Clean build artifacts
cargo make clean-all-home     # Clean all Rust projects in ~/

# Pre-commit validation (CRITICAL)
cargo make pre-commit         # Format + Lint + Unit Tests

# CI simulation
cargo make ci-local           # Simulate CI environment locally
cargo make ci                 # Full CI pipeline

# Release validation
cargo make release-validate   # Comprehensive release validation
cargo make release            # Full release pipeline

# Weaver integration
cargo make weaver-bootstrap   # Download Weaver + registry
cargo make weaver-smoke       # Weaver smoke test

# Documentation
cargo make docs               # Generate and open docs
cargo make docs-build         # Build docs without opening
cargo make cookbook-build     # Build pattern cookbook (mdBook)
cargo make cookbook-serve     # Serve cookbook locally
```

### Timeout SLAs

All commands have timeout protection:

| Operation | Timeout | Notes |
|-----------|---------|-------|
| Quick checks (fmt, check) | 5s | Fast validation |
| Compilation (debug) | 5-10s | Local warm cache |
| Compilation (release) | 30s | Optimization passes |
| Lint (clippy) | 300s (5min) | CI cold-start with all features |
| Unit tests | 1s per test | Individual test timeout |
| Integration tests | 30s | Docker containers |
| Coverage | 30s | Manual task only |
| Audit | 15s | Network operations |
| Documentation | 20s | Doc generation |

### Feature Flags

**Core features (always available):**
- `workflow-engine` - Workflow-specific features
- `mutation-testing` - Mutation testing framework
- `async` - Async performance utilities
- `benchmarking` - Criterion benchmarking support

**Advanced testing features:**
- `property-testing` - proptest integration
- `snapshot-testing` - insta integration
- `fake-data` - fake data generation
- `concurrency-testing` - loom integration
- `parameterized-testing` - rstest integration
- `cli-testing` - trycmd integration

**Observability features:**
- `otel` - OpenTelemetry validation
- `weaver` - Weaver live validation

**Integration features:**
- `testcontainers` - Docker container support

**Convenience bundles:**
- `testing-extras` - property + snapshot + fake-data (most common)
- `testing-full` - All testing features
- `observability-full` - otel + weaver
- `integration-full` - testcontainers + weaver

**Default features:**
- `logging` - Alert helpers with log crate integration

**Usage example:**
```toml
[dev-dependencies]
chicago-tdd-tools = { version = "1.1.0", features = ["testing-extras"] }
```

---

## Testing Strategies

### Test Organization Principles

1. **Mirror source structure** - tests parallel src/ organization
2. **Shared utilities** - `tests/common.rs` consolidates reusable helpers
3. **Clear separation** - Unit tests vs integration tests
4. **Consistent naming** - Descriptive test names following pattern conventions

### Test Categories (Expert Testing Focus)

The framework emphasizes testing where bugs actually occur:

1. **Error Path Testing (80% of bugs)** - Test error conditions, not just happy path
2. **Boundary Condition Testing** - Test edge cases and limits
3. **Resource Cleanup Testing** - Test cleanup in error & panic paths
4. **Concurrency Testing** - Test thread safety and race conditions
5. **Real Dependency Testing** - Use real collaborators, not mocks

### Unit Tests: Fast Iteration

**Location:** `tests/*.rs` files or `#[cfg(test)]` modules

**Characteristics:**
- Fast execution (< 1s per test)
- No Docker dependencies
- Excluded from integration tests
- Run with `cargo make test-unit`

**Pattern:**
```rust
use chicago_tdd_tools::prelude::*;

test!(test_unit_example, {
    // Arrange
    let value = 42;

    // Act
    let result = value + 1;

    // Assert
    assert_eq!(result, 43);
});
```

### Integration Tests: Real Dependencies

**Location:** `tests/testcontainers/`, `tests/weaver_integration.rs`

**Characteristics:**
- Requires Docker
- 30s timeout
- MUST fail if Docker is stopped
- Run with `cargo make test-integration`

**Pattern:**
```rust
use chicago_tdd_tools::prelude::*;

fixture_test!(test_integration_example, fixture, {
    // Fixture provides Docker container setup
    let container = fixture.docker_container();

    // Act on real service
    let result = container.execute_query();

    // Assert on real behavior
    assert_ok!(result);
});
```

### Property-Based Testing

**Feature:** `property-testing`

**Usage:**
```rust
use chicago_tdd_tools::testing::property::*;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_property(value in 0..100) {
        // Property: value * 2 should always be even
        assert_eq!((value * 2) % 2, 0);
    }
}
```

### Snapshot Testing

**Feature:** `snapshot-testing`

**Usage:**
```rust
use chicago_tdd_tools::testing::snapshot::*;

test!(test_snapshot, {
    let result = complex_data_structure();
    assert_snapshot!(result);  // insta integration
});
```

**Commands:**
```bash
cargo make snapshot-review    # Review changes
cargo make snapshot-accept    # Accept all changes
cargo make snapshot-reject    # Reject all changes
```

### Mutation Testing

**Feature:** `mutation-testing`

**Usage:**
```bash
cargo make test-mutation          # Framework's mutation testing
cargo make test-mutation-mutants  # cargo-mutants (requires install)
```

### Concurrency Testing

**Feature:** `concurrency-testing`

**Usage:**
```rust
use chicago_tdd_tools::testing::concurrency::*;

#[test]
fn test_concurrent_access() {
    loom::model(|| {
        // Test concurrent access patterns
    });
}
```

### Compile-Fail Tests

**Purpose:** Verify that invalid code fails to compile

**Location:** `tests/compile-fail/`

**Usage:**
```rust
use trybuild;

#[test]
fn test_compile_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/*.rs");
}
```

---

## CI/CD Pipeline

### GitHub Actions Workflows

**Location:** `.github/workflows/`

**Workflows:**
1. `ci.yml` - Main CI pipeline (runs on every push)
2. `release.yml` - Release automation
3. `docs.yml` - Documentation generation
4. `benchmark.yml` - Performance benchmarking
5. `stale.yml` - Stale issue management
6. `clear-cache.yml` - Cache cleanup

### CI Pipeline Jobs

**ci.yml** - Main pipeline (runs on all branch pushes):

| Job | Purpose | Matrix | Timeout |
|-----|---------|--------|---------|
| **fmt** | Code formatting check | Ubuntu latest | 5min |
| **lint** | Clippy linting (all/pedantic/nursery/cargo) | Ubuntu/macOS/Windows × stable/beta/nightly | 10min |
| **test** | Unit tests with retry logic | Ubuntu/macOS/Windows × stable/beta | 10min |
| **unwrap-check** | Production code safety (no unwrap/expect) | Ubuntu latest | 5min |
| **coverage** | Code coverage (70% target, warning-only) | Ubuntu latest | 10min |
| **ci** | Final validation (all jobs must pass) | Ubuntu latest | - |

### FMEA Improvements Applied

The CI pipeline implements several Failure Mode and Effects Analysis (FMEA) improvements:

1. **Multi-OS Testing** (RPN: 315 → 45)
   - Tests run on Linux, macOS, Windows
   - Catches OS-specific issues early

2. **Test Retry Logic** (RPN: 120 → 24)
   - Tests retry up to 3 times on failure
   - Handles flaky tests gracefully

3. **Production Code Safety** (RPN: 180 → 36)
   - CI enforces no `.unwrap()` or `.expect()` in production code
   - Prevents production panics

4. **Coverage Tracking** (RPN: 336 → 67)
   - 70% coverage target (warning-only, will become blocking)
   - Codecov integration for historical tracking

5. **Branch Push Validation** (RPN: 560 → 56)
   - CI runs on all branch pushes, not just main
   - Early feedback on PR branches

### CI Failure Handling

**When CI fails:**

1. **Format failure:**
   ```bash
   cargo make fmt
   git add .
   git commit --amend --no-edit
   ```

2. **Lint failure:**
   ```bash
   cargo make lint  # See specific warnings
   # Fix issues and commit
   ```

3. **Test failure:**
   ```bash
   cargo make test-unit       # Run tests locally
   cargo make ci-local        # Simulate CI environment
   ```

4. **Unwrap/expect failure:**
   ```bash
   # Install hooks to prevent future occurrences
   cargo make install-hooks
   # Fix code to use ? or proper error handling
   # See docs/process/SPR_GUIDE.md for patterns
   ```

---

## Quality Standards

### Code Quality: Poka-Yoke Enforcement

**Critical Rules:**

1. **No clippy warnings** - All warnings treated as errors (`-D warnings`)
2. **No unwrap/expect in production** - Use `?` or proper error handling
3. **No panic in production** - Use `Result` for error handling
4. **Structured logging** - Use `alert!` macros or `log!` macros, never `println!`
5. **Explicit ownership** - Clear lifetimes and ownership
6. **Zero-cost abstractions** - Prefer generics over trait objects

### Lint Configuration

**Enabled clippy groups:**
- `clippy::all` - All default lints
- `clippy::pedantic` - Pedantic lints
- `clippy::nursery` - Experimental lints
- `clippy::cargo` - Cargo-specific lints

**Critical denials:**
- `clippy::unwrap_used` - No `.unwrap()` in production
- `clippy::expect_used` - No `.expect()` in production
- `clippy::panic` - No `panic!()` in production
- `clippy::todo` - No `todo!()` placeholders
- `clippy::unimplemented` - No `unimplemented!()` placeholders

**Allow exceptions only with justification:**
```rust
#[allow(clippy::unwrap_used)]  // JUSTIFICATION: Test code only
fn test_helper() {
    let value = result.unwrap();  // OK in test code
}
```

### Error Handling Patterns

**See:** `docs/process/SPR_GUIDE.md` for comprehensive patterns.

**Quick reference:**

```rust
// ❌ NEVER
fn bad() -> String {
    get_value().unwrap()  // CI will fail
}

// ✅ ALWAYS - Option 1: Propagate errors
fn good() -> Result<String, Error> {
    Ok(get_value()?)
}

// ✅ ALWAYS - Option 2: Pattern match
fn good_alt() -> String {
    match get_value() {
        Ok(v) => v,
        Err(e) => {
            alert_warning!("Failed to get value: {}", e);
            String::from("default")
        }
    }
}

// ✅ ALWAYS - Option 3: if let
fn good_if_let() -> String {
    if let Ok(v) = get_value() {
        v
    } else {
        String::from("default")
    }
}
```

### Logging Standards

**Use alert macros for visual severity indicators:**

```rust
use chicago_tdd_tools::prelude::*;

alert_critical!("Database connection failed: {}", error);
alert_warning!("Retrying operation: attempt {}", attempt);
alert_info!("Processing {} items", count);
alert_success!("Operation completed successfully");
alert_debug!("Internal state: {:?}", state);
```

**Or use log crate directly (when logging feature enabled):**

```rust
use log::{error, warn, info, debug};

error!("Critical error: {}", msg);
warn!("Warning: {}", msg);
info!("Info: {}", msg);
debug!("Debug: {}", msg);
```

**Never use println! or eprintln! in production code.**

### Performance Awareness

**Principles:**
1. **Minimize allocations** - Reuse buffers, use references
2. **Stack > Heap** - Prefer stack allocation when possible
3. **References > Owned** - Use `&str` instead of `String` when possible
4. **Static dispatch > Dynamic dispatch** - Use generics over trait objects
5. **Measure, don't guess** - Use `cargo make test-timings` to identify slow tests

---

## Observability and Weaver

### Weaver Integration (Dogfooding)

The framework dogfoods Weaver for semantic convention validation.

**Quick Start:**

```bash
# 1. Bootstrap Weaver CLI + registry
cargo make weaver-bootstrap

# 2. Run smoke test (version check + telemetry span)
cargo make weaver-smoke

# 3. Run full integration (requires Docker + weaver feature)
cargo make test-integration
```

### Weaver Live-Check

**Feature:** `weaver`

**Purpose:** Validate OpenTelemetry spans/metrics against semantic conventions in real-time

**Usage:**
```rust
use chicago_tdd_tools::observability::weaver::*;

fixture_test!(test_weaver_validation, fixture, {
    // Fixture provides Weaver container
    let weaver = fixture.weaver_instance();

    // Send telemetry
    send_otel_span("http.request", attributes);

    // Validate with Weaver
    let result = weaver.validate_span("http.request");
    assert_ok!(result);
});
```

### Skipping Weaver Tests (When Necessary)

**Default:** Weaver tests fail fast when prerequisites are missing (quality is the default)

**Temporary skip:**
```bash
export WEAVER_ALLOW_SKIP=1
cargo make test
```

**Use case:** Local development when Docker is not available

### OpenTelemetry Validation

**Feature:** `otel`

**Purpose:** Validate OTEL instrumentation without Weaver

**Usage:**
```rust
use chicago_tdd_tools::observability::otel::*;

test!(test_otel_span, {
    let span = create_test_span("my.operation");

    // Validate span attributes
    assert_eq!(span.name(), "my.operation");
    assert!(span.has_attribute("service.name"));
});
```

---

## Common Tasks

### When Making Changes to Production Code

```bash
# 1. Make changes
# 2. Format
cargo make fmt

# 3. Check compilation
cargo make check

# 4. Run tests
cargo make test-unit

# 5. Lint
cargo make lint

# 6. Pre-commit validation (runs all above)
cargo make pre-commit

# 7. Commit
git add .
git commit -m "feat: your change description"

# 8. Before push (optional but recommended)
cargo make ci-local
```

### When Adding New Features

```bash
# 1. Create feature branch
git checkout -b feature/my-feature

# 2. Implement feature with tests (TDD approach)
#    - Write failing test first
#    - Implement minimal code to pass
#    - Refactor

# 3. Validate
cargo make pre-commit

# 4. Run full test suite
cargo make test-all  # If Docker available

# 5. Check coverage (optional)
cargo make coverage-report

# 6. Commit and push
git add .
git commit -m "feat: add new feature"
git push -u origin feature/my-feature
```

### When Fixing CI Failures

```bash
# 1. Simulate CI locally
cargo make ci-local

# 2. If format issues
cargo make fmt
git add .
git commit --amend --no-edit

# 3. If lint issues
cargo make lint
# Fix issues
git add .
git commit --amend --no-edit

# 4. If test failures
cargo make test-unit
# Fix tests
git add .
git commit --amend --no-edit

# 5. If unwrap/expect issues
# Install hooks for prevention
cargo make install-hooks
# Fix code to use ? or proper error handling
# See docs/process/SPR_GUIDE.md
git add .
git commit --amend --no-edit
```

### When Adding New Dependencies

```bash
# 1. Add to Cargo.toml with justification comment
# 2. Update feature flags if optional
# 3. Test with all feature combinations
cargo make test --all-features
cargo make test --no-default-features

# 4. Run security audit
cargo make audit

# 5. Document in CLAUDE.md if significant
```

### When Writing Documentation

```markdown
# Follow documentation standards:
# 1. Keep README.md focused on 80% use cases
# 2. Full details go in docs/ subdirectories
# 3. Patterns go in cookbook/
# 4. API reference auto-generated from rustdoc

# Build docs
cargo make docs-build

# Serve cookbook
cargo make cookbook-serve
```

### When Preparing a Release

```bash
# 1. Update version in Cargo.toml and proc_macros/Cargo.toml
# 2. Update CHANGELOG.md
# 3. Create release notes in docs/releases/

# 4. Validate release
cargo make release-validate

# 5. Full release pipeline
cargo make release

# 6. Tag release
git tag -a v1.x.x -m "Release v1.x.x"
git push origin v1.x.x
```

---

## Documentation Structure

### Documentation Hierarchy

**Quick Start (80% use cases):**
- `README.md` - Quick start guide
- `docs/getting-started/QUICK_GUIDE.md` - Essential patterns and macro reference

**Getting Started:**
- `docs/getting-started/GETTING_STARTED.md` - Complete setup guide with troubleshooting
- `docs/getting-started/USER_GUIDE.md` - Comprehensive usage guide

**Reference:**
- `docs/reference/API_REFERENCE.md` - Complete API documentation
- `docs/reference/ARCHITECTURE.md` - Design principles and module organization
- `docs/reference/SLA_REFERENCE.md` - Timeout standards and constraints

**Patterns and Learning:**
- `cookbook/src/` - Alexander-style pattern language (20 patterns)
  - `testing-patterns/` - 5 testing patterns
  - `architecture-patterns/` - 5 architecture patterns
  - `design-patterns/` - 10 design patterns
- `docs/features/` - Per-feature documentation
- `examples/` - 8 example files demonstrating features

**Process Documentation:**
- `docs/process/SPR_GUIDE.md` - Error handling patterns (SPR = Systematic Problem Resolution)
- `docs/process/FMEA_TESTS_BUILD_ACTIONS.md` - Failure mode analysis
- `docs/process/CODE_REVIEW_CHECKLIST.md` - Review guidelines
- `docs/process/TEST_ISOLATION_GUIDE.md` - Test isolation patterns
- `docs/process/DOG_FOODING.md` - Dogfooding Weaver

**Analysis and Research:**
- `docs/analysis/` - Root cause analyses
- `docs/research/` - Innovation and problem-solving documentation

**Releases:**
- `docs/releases/CHANGELOG.md` - Version history
- `docs/releases/RELEASE_NOTES_*.md` - Release-specific notes

### Documentation Index

**See:** `docs/README.md` for complete documentation navigation.

---

## Error Prevention Patterns

### FMEA (Failure Mode and Effects Analysis)

The codebase implements several FMEA improvements to reduce Risk Priority Numbers (RPN):

| Failure Mode | Original RPN | Current RPN | Improvement |
|--------------|-------------|-------------|-------------|
| Tests pass locally, fail in CI | 105 | 21 | Multi-OS testing, ci-local simulation |
| Clippy warnings accumulate | 112 | 11 | CI enforcement, pre-commit checks |
| Production panics (unwrap/expect) | 180 | 36 | CI checks, git hooks, lint enforcement |
| Tests fail intermittently | 120 | 24 | Retry logic, test isolation |
| Coverage regressions | 336 | 67 | Coverage tracking, Codecov integration |
| Release failures | Multiple | Reduced | Comprehensive release-validate task |

### Poka-Yoke Mechanisms

**Compile-Time Prevention:**
1. Type-level state machines enforce AAA pattern
2. Sealed traits prevent incorrect API usage
3. Const assertions validate constraints at compile time
4. Clippy lints deny dangerous patterns

**Build-Time Prevention:**
1. cargo-make ensures consistent build configuration
2. Timeout enforcement prevents hanging operations
3. Pre-commit hooks catch unwrap/expect

**Runtime Prevention:**
1. Result-based error handling (no panics)
2. Structured logging (no println!)
3. Timeout SLAs for all operations

### Root Cause Prevention

**Single Source of Truth:**
- `Makefile.toml` - All build commands
- `cargo make lint` - Lint configuration (matches CI)
- `Cargo.toml [lints]` section - Global lint levels

**Automated Enforcement:**
- CI runs on all branch pushes (not just main)
- Pre-commit hooks prevent common mistakes
- Multi-OS testing catches platform issues
- Retry logic handles flaky tests

**Quality Gates:**
- Format check (blocks merge)
- Lint check (blocks merge)
- Test check (blocks merge)
- Unwrap/expect check (blocks merge)
- Coverage tracking (warning, will block at 70%)

---

## AI Assistant Guidelines

### When Working with This Codebase

**DO:**
1. Always use `cargo make` commands, never raw `cargo`
2. Run `cargo make pre-commit` before suggesting commits
3. Follow Poka-Yoke principles (prevent errors at compile time)
4. Use proper error handling (?, match, if let) - never unwrap/expect
5. Add tests when adding features (TDD approach)
6. Use alert macros for logging, never println!
7. Document patterns in cookbook/ when adding significant patterns
8. Check FMEA improvements when modifying CI/build systems
9. Follow AAA pattern in all tests
10. Consider "80/20 thinking" - go the extra mile with 20% extra effort

**DON'T:**
1. Use `.unwrap()` or `.expect()` in production code
2. Use `panic!()`, `todo!()`, or `unimplemented!()` in production
3. Use `println!` or `eprintln!` for logging
4. Skip tests when adding features
5. Ignore clippy warnings (fix or justify with #[allow])
6. Add dependencies without justification
7. Commit without running `cargo make pre-commit`
8. Modify CI without understanding FMEA implications
9. Break backward compatibility without discussion
10. Use alphabetical organization (use capability groups)

### Code Review Checklist

When reviewing code changes:

1. ✅ Tests added for new functionality
2. ✅ No unwrap/expect in production code
3. ✅ Proper error handling (Result, ?)
4. ✅ Structured logging (alert macros or log crate)
5. ✅ AAA pattern in tests
6. ✅ No clippy warnings
7. ✅ Documentation updated
8. ✅ `cargo make pre-commit` passes
9. ✅ Backward compatibility maintained
10. ✅ FMEA considerations for critical paths

**See:** `docs/process/CODE_REVIEW_CHECKLIST.md` for complete checklist.

### Common Pitfalls to Avoid

1. **Using cargo instead of cargo make**
   - Symptom: Proc-macro errors, missing timeouts
   - Fix: Always use `cargo make` commands

2. **Unwrap/expect in production**
   - Symptom: CI unwrap-check failure
   - Fix: Use `?`, `match`, or `if let`
   - Prevention: `cargo make install-hooks`

3. **Ignoring clippy warnings**
   - Symptom: CI lint failure
   - Fix: Run `cargo make lint` and fix issues
   - Prevention: `cargo make pre-commit`

4. **Skipping tests**
   - Symptom: CI test failure, coverage regression
   - Fix: Write tests following AAA pattern
   - Prevention: TDD approach (test first)

5. **Breaking backward compatibility**
   - Symptom: User code breaks on upgrade
   - Fix: Re-export at crate root, deprecate gradually
   - Prevention: Check existing public API usage

---

## Quick Reference Card

### Essential Commands

```bash
# Pre-commit (ALWAYS run before commit)
cargo make pre-commit

# Development cycle
cargo make check && cargo make test-unit && cargo make lint

# CI simulation
cargo make ci-local

# Full test suite (requires Docker)
cargo make test-all

# Release validation
cargo make release-validate
```

### File Locations

| What | Where |
|------|-------|
| Source code | `src/` |
| Tests | `tests/` |
| Examples | `examples/` |
| Documentation | `docs/` |
| Patterns | `cookbook/src/` |
| CI config | `.github/workflows/` |
| Build config | `Makefile.toml` |

### Getting Help

| Question | Resource |
|----------|----------|
| Quick start | `README.md` |
| API reference | `docs/reference/API_REFERENCE.md` |
| Error handling | `docs/process/SPR_GUIDE.md` |
| Patterns | `cookbook/src/` |
| Architecture | `docs/reference/ARCHITECTURE.md` |
| Troubleshooting | `docs/getting-started/GETTING_STARTED.md` |
| All docs | `docs/README.md` |

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.1.0 | 2025-11-14 | Initial CLAUDE.md creation based on current codebase state |

---

## Contributing

When contributing to this codebase:

1. Read this entire CLAUDE.md file
2. Install required tools (`cargo make`, git hooks)
3. Follow Poka-Yoke principles
4. Run `cargo make pre-commit` before every commit
5. Update this file if adding significant patterns or conventions

**Quality is the default. Prevention is better than detection.**

---

**Last Updated:** 2025-11-14
**Maintained by:** KNHK Team
**License:** MIT
