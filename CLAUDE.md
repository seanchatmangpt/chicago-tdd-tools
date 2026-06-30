# CLAUDE.md — Chicago TDD Tools Guide

## What Is This Project?

A Rust testing framework that enforces Chicago-style TDD (Classicist approach) through compile-time guarantees. The core principle: **if it compiles, correctness follows**. Quality is not an afterthought—it's encoded in the type system.

**Stats:** ~8,000 LOC | Rust 2021+ | MIT License | 445+ unit tests | Zero production panics

## Core Philosophy: Poka-Yoke Design

This framework prevents errors at three levels:

1. **Compile-time**: Type-level state machines enforce AAA pattern. Sealed traits prevent invalid test states.
2. **Build-time**: Git hooks prevent `.unwrap()` and `panic!()` from being committed. Clippy treats all warnings as errors.
3. **Runtime**: Result-based error handling everywhere. No panics in production code.

**Key decisions flow from this**: When something could fail at runtime, we encode it as a type constraint and fail at compile time instead.

## Structure at a Glance

```
src/
├── core/              # Foundational testing primitives
│   ├── fixture.rs     # Test setup/teardown
│   ├── builders.rs    # Fluent builders for test data
│   ├── assertions.rs  # Assert helpers
│   ├── state.rs       # Type-level AAA enforcement
│   ├── poka_yoke.rs   # Error prevention
│   ├── alert.rs       # Structured logging
│   └── macros/        # Test/assertion macros
├── testing/           # Advanced techniques
│   ├── property/      # Property-based testing
│   ├── mutation/      # Mutation testing
│   ├── snapshot/      # Snapshot testing
│   ├── concurrency/   # Concurrency testing (loom)
│   └── cli/           # CLI testing (trycmd)
├── validation/        # Quality assurance
│   ├── coverage/      # Coverage analysis
│   ├── guards/        # Constraint enforcement
│   ├── jtbd/          # Jobs to be done
│   └── performance/   # Tick/RDTSC measurement
├── observability/     # OTEL & Weaver
│   ├── otel/          # OTEL validation
│   ├── weaver/        # Weaver live-check
│   └── ocel/          # OCEL process mining
├── integration/       # Integration support
│   └── testcontainers/  # Docker containers
├── sector_stacks/     # Production-grade workflows
├── swarm/             # Distributed coordination
├── operator_registry.rs  # YAWL workflow patterns
└── lib.rs             # Root exports + prelude
```

**Organization principle**: Modules grouped by capability, not alphabetically. All re-exported at crate root for backward compatibility.

## Critical Constraints

### 1. Always Use `cargo make`, Never Raw `cargo`

```bash
# ✅ Correct
cargo make test
cargo make lint
cargo make pre-commit

# ❌ Never this
cargo test
cargo clippy
```

**Why**: cargo-make handles proc-macros correctly, enforces timeouts (prevents hanging), provides consistent build environment.

### 2. No Production Panics or Unwraps

Git hooks and CI checks prevent:
- `.unwrap()` in production code
- `.expect()` in production code
- `panic!()` in production code
- `todo!()` in production code
- `unimplemented!()` in production code

**Correct patterns**:
```rust
// ✅ Propagate errors
let value = result?;

// ✅ Handle with default
let value = match result {
    Ok(v) => v,
    Err(e) => { alert_warning!("Failed: {}", e); default }
};

// ✅ If let pattern
let value = if let Ok(v) = result { v } else { default };

// Test-only exceptions
#[allow(clippy::unwrap_used)]  // JUSTIFICATION: Test code only
let value = result.unwrap();
```

### 3. All Clippy Warnings Are Errors

```bash
# This will fail the build
cargo make lint
```

Configured in `Cargo.toml`:
- `clippy::all` (deny)
- `clippy::pedantic` (deny)
- `clippy::nursery` (deny)
- `clippy::cargo` (deny)
- `-D warnings` (all warnings are errors)

If you need to allow something, use inline justification:
```rust
#[allow(clippy::too_many_arguments)]
// JUSTIFICATION: This function legitimately needs 8+ args for the domain model
fn process(...) { }
```

### 4. Timeout SLAs (Enforced in Makefile.toml)

| Operation | Timeout | Context |
|-----------|---------|---------|
| check | 30s | Fast compilation check |
| lint | 300s | Cold-start with all features |
| fmt | 30s | Code formatting |
| test-unit | 300s | Fast unit tests |
| test-integration | 300s | Docker tests |
| build-release | 30s | Release build |

If a command hangs, it fails. By design. Prevents CI zombies.

## Development Workflow

### Before Every Commit (Required)

```bash
cargo make pre-commit  # fmt + lint + unit tests
```

This catches 95% of issues before CI sees them.

### When Adding a Feature

```bash
# 1. Write failing test first (TDD)
# 2. Implement minimal code
# 3. Run this before committing
cargo make pre-commit
cargo make ci-local    # Simulate full CI pipeline

# 4. Commit
git add . && git commit -m "feat: description"
```

### When Something Fails

```bash
# Reproduce CI environment locally
cargo make ci-local

# Check for specific issues
cargo make fmt         # Auto-fix formatting
cargo make lint        # See what clippy complains about
cargo make test-unit   # Re-run tests
```

## Testing Organization

### Unit Tests (Fast, No Docker)

```rust
use chicago_tdd_tools::prelude::*;

test!(test_example, {
    // Arrange
    let input = 5;
    // Act
    let result = input * 2;
    // Assert
    assert_eq!(result, 10);
});
```

Location: `tests/` or inline in `src/` with `#[cfg(test)]`

### Integration Tests (Requires Docker)

```rust
fixture_test!(test_with_container, fixture, {
    let container = fixture.docker_container();
    // Use Docker-backed services
});
```

Location: `tests/` with integration test files

Run with: `cargo make test-integration`

### Advanced Testing Techniques

**Property-based** (random test generation):
```rust
test!(test_property, {
    let gen = PropertyTestGenerator::<100, 5>::new();
    // Verify property holds for all generated inputs
});
```

**Snapshot** (output comparison):
```rust
test!(test_snapshot, {
    assert_snapshot!(data.to_string());
});
```
Requires `snapshot-testing` feature.

**Mutation** (test quality validation):
Intentionally break code and verify tests catch it. Target: ≥80% mutation score.

**Concurrency** (thread safety):
Uses `loom` for deterministic testing. Find races before production.

## Feature Flags

**Always available**: `test!`, `async_test!`, `fixture_test!`, builders, assertions

**Enable as needed**:

```toml
[dev-dependencies]
chicago-tdd-tools = { path = ".", features = ["testing-extras"] }
```

**Common bundles**:
- `testing-extras` — property + snapshot + fake data (80% use case)
- `otel` — OpenTelemetry span/metric validation
- `weaver` — Weaver semantic convention live-check
- `testcontainers` — Docker container support
- `ocel-generation` — OCEL 2.0 process mining

See `Cargo.toml` for full feature list.

## Critical Files to Understand

| File | Purpose |
|------|---------|
| `Cargo.toml` | Dependencies, features, lints (source of truth for quality settings) |
| `Makefile.toml` | Build system, timeouts, development tasks |
| `src/lib.rs` | Public API, module organization, re-exports |
| `.github/workflows/ci.yml` | CI pipeline definition |
| `docs/process/CODE_REVIEW_CHECKLIST.md` | What reviewers expect |

## Logging & Alerts

Never use `println!` or `eprintln!` in production code:

```rust
// Use alert macros instead
alert_critical!("Database failed: {}", error);  // log::error!
alert_warning!("Retry attempt {}", n);          // log::warn!
alert_info!("Processing {} items", count);      // log::info!
alert_success!("Completed");                    // log::info!
alert_debug!("State: {:?}", state);             // log::debug!
```

Requires `log` feature (enabled by default).

## Observability & Weaver

### Bootstrap (First Time)

```bash
cargo make weaver-bootstrap  # Download Weaver CLI + semantic conventions
```

### Smoke Test (Verify It Works)

```bash
cargo make weaver-smoke  # No Docker required
```

### Integration Tests with Weaver

```bash
cargo make test-integration  # Runs Weaver validation tests
# Or skip if Docker unavailable
WEAVER_ALLOW_SKIP=1 cargo make test-integration
```

## Procedural Macros

Located in `proc_macros/` crate. Critical macros:

- `#[chicago_test]` — Zero-boilerplate test macro
- `#[fixture]` — Automatic fixture setup/teardown
- `#[derive(TestBuilder)]` — Fluent builder generation
- `#[scaffold(...)]` — Code generation (ggen-backed)

These are re-exported from `chicago_tdd_tools` and available in the prelude.

## Common Patterns (DO)

1. ✅ Use `cargo make` exclusively
2. ✅ Run `cargo make pre-commit` before every commit
3. ✅ Follow AAA pattern in all tests (Arrange-Act-Assert)
4. ✅ Use `?` operator for error propagation
5. ✅ Add tests when adding features (TDD approach)
6. ✅ Use alert macros for logging
7. ✅ Document non-obvious decisions inline
8. ✅ Measure performance before optimizing

## Common Anti-Patterns (DON'T)

1. ❌ `.unwrap()`, `.expect()`, `panic!()` in production code
2. ❌ `println!` or `eprintln!` for logging
3. ❌ Ignoring clippy warnings
4. ❌ Skipping tests when adding features
5. ❌ Using raw `cargo` instead of `cargo make`
6. ❌ Adding dependencies without justification
7. ❌ Breaking backward compatibility without discussion
8. ❌ Committing without running `cargo make pre-commit`

## Git Workflow

**Important**: FIX FORWARD ONLY. Never `git reset --hard` or destructive operations.

- Solve problems by **adding commits**, not removing them
- `git revert` (creates new commit) is allowed if absolutely necessary
- Debug, find root cause, apply targeted fix

## Quick Reference

```bash
# Setup (once)
cargo install cargo-make
cargo make install-hooks

# Daily workflow
cargo make pre-commit              # Before every commit
cargo make check                   # Quick check
cargo make test                    # All tests
cargo make lint                    # Code quality
cargo make docs                    # Generate API docs

# Debugging CI
cargo make ci-local                # Simulate full pipeline
cargo make fmt && cargo make check # Fast feedback loop

# Special cases
cargo make test-integration        # With Docker
cargo make weaver-bootstrap        # First Weaver setup
cargo make coverage                # Coverage report
```

## When Something Breaks

**Most common**: Running `cargo test` instead of `cargo make test`
- **Fix**: Use `cargo make test` only

**Proc-macro compilation fails**: Probably used raw cargo
- **Fix**: `cargo make clean && cargo make check`

**Lint fails locally, passed before**: New code introduced clippy violations
- **Fix**: `cargo make lint` to see what, then fix inline or allow with justification

**Tests pass locally, fail CI**: Architecture differences or missing Docker
- **Fix**: `cargo make ci-local` to reproduce, then fix

**Weaver tests skip**: Docker not available
- **Fix**: Start Docker or set `WEAVER_ALLOW_SKIP=1`

## Getting Help

- **Project philosophy**: README.md (why this framework exists)
- **API documentation**: `cargo make docs`
- **Code review expectations**: `docs/process/CODE_REVIEW_CHECKLIST.md`
- **Examples**: `examples/` directory (18+ complete, working examples)
- **Architecture**: `docs/reference/ARCHITECTURE.md`
- **Troubleshooting**: `docs/getting-started/GETTING_STARTED.md`

## Key Dependencies to Know

| Crate | Purpose | Why |
|-------|---------|-----|
| `tokio` | Async runtime | Used in all async tests |
| `serde` | Serialization | Test data structures |
| `proptest` | Property testing | Random test generation (optional) |
| `insta` | Snapshot testing | Output comparison (optional) |
| `testcontainers` | Docker support | Integration tests (optional) |
| `opentelemetry` | OTEL SDK | Observability (optional) |
| `loom` | Concurrency testing | Thread safety verification (optional) |
| `trybuild` | Compile-fail tests | Compile-time validation (dev-only) |

## Recent Work (Last 30 Days)

- **v26.6.30**: Proc macros exposed in public API
- **v26.6.29**: Track E feedback loop with Weaver integration
- **v26.6.24**: BLAKE3 receipt chain validation + OCEL utilities
- **v26.6.121**: OCEL 2.0 process mining, governance module, wave orchestration, 43 YAWL patterns

See `CHANGELOG.md` for full history.

---

**Version:** 1.2.0 | **Updated:** 2026-06-29 | **Principle:** Compile-time prevention > runtime detection
