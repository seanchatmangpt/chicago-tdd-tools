# CLAUDE.md - AI Assistant Guide for Chicago TDD Tools

**Version:** 1.1.0 | **Updated:** 2025-11-14
**Purpose:** Compressed guide for AI assistants. Assumes LLM understands Rust patterns.

---

## Core Identity

Rust testing framework enforcing Chicago-style TDD (Classicist) through compile-time guarantees. Type system encodes invariants—if it compiles, correctness follows. Quality default, prevention over detection.

**Stats:** ~8,000 LOC | Rust 2021+ | MIT License | https://github.com/seanchatmangpt/chicago-tdd-tools

**Principles:**
1. **Poka-Yoke Design** - Mistakes prevented at compilation, not runtime
2. **Type-First Thinking** - Types as primary design tool, compiler enforces correctness
3. **80/20 Thinking** - Second idea solves 80% of problems with 20% extra effort
4. **Zero-Cost Abstractions** - Performance through generics, const generics, macros
5. **AAA Pattern** - Arrange-Act-Assert enforced via type-level state machines

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

Organized by functionality, not alphabetically. Backward compatible via crate-root re-exports.

- **core/** - Foundational testing primitives (fixtures, builders, assertions)
- **testing/** - Advanced techniques (property, mutation, snapshot, concurrency)
- **validation/** - Quality assurance (coverage, guards, JTBD, performance)
- **observability/** - Telemetry (OTEL, Weaver)
- **integration/** - Integration support (testcontainers)

**New code:** Use capability paths (`core::fixture`). **Old code:** Crate root imports continue working.

---

## Critical Constraints

### Build System: cargo-make Mandatory

**⚠️ NEVER use `cargo` directly. ALWAYS use `cargo make`.**

Reasons: Handles proc-macros correctly, timeout protection prevents hanging, single source of truth.

```bash
# Required installation
cargo install cargo-make
cargo make --version  # Verify

# Recommended
cargo make install-hooks  # Prevents unwrap/expect in production
```

### Production Code Rules

1. **No `.unwrap()` or `.expect()`** - CI blocks, git hooks prevent
2. **No `panic!()`, `todo!()`, `unimplemented!()`** - Use `Result`
3. **No `println!` or `eprintln!`** - Use `alert!` macros or `log!` crate
4. **All clippy warnings = errors** - `-D warnings` (all/pedantic/nursery/cargo)
5. **Timeout enforcement** - Every operation has timeout SLA

**Error handling patterns:**
```rust
// ❌ NEVER
let value = result.unwrap();

// ✅ ALWAYS
let value = result?;  // Option 1: Propagate
// or
let value = match result { Ok(v) => v, Err(e) => { alert_warning!("..."); default } };  // Option 2
// or
let value = if let Ok(v) = result { v } else { default };  // Option 3
```

---

## Development Workflows

### Essential Pre-Commit (ALWAYS)

```bash
cargo make pre-commit  # Format + Lint + Unit Tests (~20s)
```

Runs: `fmt` (5s) + `lint` (5s) + `test-unit` (10s) + `dead-code-check`

### Standard Cycle

```bash
cargo make check              # Compilation check
cargo make fmt                # Format
cargo make test-unit          # Unit tests (fast, no Docker)
cargo make lint               # Clippy all/pedantic/nursery/cargo
cargo make pre-commit         # All above
cargo make ci-local           # Simulate CI (catches failures before push)
```

### Core Build Commands

```bash
# Compilation (5-30s timeouts)
cargo make check build build-release

# Testing
cargo make test               # Unit only (excludes integration)
cargo make test-unit          # Same as test
cargo make test-integration   # Requires Docker, MUST fail if Docker stopped
cargo make test-all           # Unit + integration

# Specialized
cargo make test-property test-mutation test-snapshot test-examples test-timings

# Quality
cargo make lint clippy        # 300s timeout (CI cold-start)
cargo make coverage coverage-report

# Validation
cargo make pre-commit         # Format + Lint + Unit Tests
cargo make ci-local           # Simulate GitHub Actions
cargo make release-validate   # Comprehensive release checks

# Weaver
cargo make weaver-bootstrap   # Download Weaver + registry
cargo make weaver-smoke       # Smoke test

# Documentation
cargo make docs docs-build cookbook-build cookbook-serve

# Cleanup
cargo make clean clean-all-home
```

### Timeout SLAs

| Operation | Timeout | Context |
|-----------|---------|---------|
| Quick checks (fmt, check) | 5s | Fast validation |
| Compilation (debug) | 5-10s | Local warm cache |
| Compilation (release) | 30s | Optimization passes |
| Lint (clippy) | 300s | CI cold-start with all features |
| Unit tests | 1s per test | Individual timeout |
| Integration tests | 30s | Docker containers |
| Coverage | 30s | Manual only |
| Audit | 15s | Network operations |
| Documentation | 20s | Doc generation |

### Feature Flags

**Core (always available):** `workflow-engine`, `mutation-testing`, `async`, `benchmarking`

**Advanced testing:** `property-testing`, `snapshot-testing`, `fake-data`, `concurrency-testing`, `parameterized-testing`, `cli-testing`

**Observability:** `otel`, `weaver`

**Integration:** `testcontainers`

**Bundles:** `testing-extras` (property+snapshot+fake, most common), `testing-full`, `observability-full`, `integration-full`

**Default:** `logging` (alert macros with log crate)

```toml
[dev-dependencies]
chicago-tdd-tools = { version = "1.1.0", features = ["testing-extras"] }
```

---

## Testing Patterns

### Test Organization

Mirror source structure. `tests/common.rs` for shared utilities. Clear unit vs integration separation.

**Expert focus:** Error paths (80% of bugs), boundary conditions, resource cleanup, concurrency, real dependencies.

### Unit Tests (Fast)

```rust
use chicago_tdd_tools::prelude::*;

test!(test_example, {
    // Arrange - Set up
    let input = 5;
    // Act - Execute
    let result = input * 2;
    // Assert - Verify
    assert_eq!(result, 10);
});
```

Characteristics: <1s execution, no Docker, run with `cargo make test-unit`

### Integration Tests (Requires Docker)

```rust
fixture_test!(test_integration, fixture, {
    let container = fixture.docker_container();
    let result = container.execute_query();
    assert_ok!(result);
});
```

Characteristics: 30s timeout, MUST fail if Docker unavailable, run with `cargo make test-integration`

### Available Macros

**Test:** `test!`, `async_test!`, `fixture_test!`, `performance_test!`
**Assert:** `assert_ok!`, `assert_err!`, `assert_in_range!`, `assert_eq_msg!`
**Alert:** `alert_critical!`, `alert_warning!`, `alert_info!`, `alert_success!`, `alert_debug!`
**Procedural:** `#[tdd_test]`, `#[fixture]`, `#[derive(TestBuilder)]`

### Advanced Testing

**Property-based:** `proptest` integration for random generation, edge case discovery
**Snapshot:** `insta` integration with `cargo make snapshot-review/accept/reject`
**Mutation:** Framework built-in + optional `cargo-mutants`
**Concurrency:** `loom` for deterministic thread model checking
**Compile-fail:** `trybuild` verifies invalid code fails compilation

---

## CI/CD Pipeline

### GitHub Actions (`.github/workflows/`)

Main: `ci.yml` (every push), `release.yml`, `docs.yml`, `benchmark.yml`, `stale.yml`, `clear-cache.yml`

### CI Pipeline Jobs (ci.yml)

| Job | Purpose | Matrix | Timeout |
|-----|---------|--------|---------|
| **fmt** | Format check | Ubuntu | 5min |
| **lint** | Clippy (all/pedantic/nursery/cargo) | Ubuntu/macOS/Windows × stable/beta/nightly | 10min |
| **test** | Unit tests with retry (3 attempts) | Ubuntu/macOS/Windows × stable/beta | 10min |
| **unwrap-check** | Production safety (no unwrap/expect) | Ubuntu | 5min |
| **coverage** | 70% target (warning-only) | Ubuntu | 10min |
| **ci** | Final validation (all must pass) | Ubuntu | - |

### FMEA Improvements (Risk Priority Number reductions)

| Failure Mode | Original RPN | Current RPN | Mitigation |
|--------------|-------------|-------------|------------|
| Tests pass locally, fail CI | 105 | 21 | Multi-OS, ci-local simulation |
| Clippy warnings accumulate | 112 | 11 | CI enforcement, pre-commit |
| Production panics (unwrap/expect) | 180 | 36 | CI checks, git hooks, lint deny |
| Flaky tests | 120 | 24 | Retry logic (3x), test isolation |
| Coverage regressions | 336 | 67 | Coverage tracking, Codecov |
| Branch-specific issues | 560 | 56 | CI on all branch pushes |

### CI Failure Recovery

```bash
# Format: cargo make fmt && git commit --amend --no-edit
# Lint: cargo make lint, fix, commit
# Test: cargo make ci-local, fix, commit
# Unwrap: cargo make install-hooks, refactor to use ?, commit
```

---

## Quality Standards

### Poka-Yoke Enforcement

**Compile-time:** Type-level state machines (AAA), sealed traits, const assertions, clippy denials
**Build-time:** cargo-make consistency, timeout enforcement, pre-commit hooks
**Runtime:** Result-based errors (no panics), structured logging, timeout SLAs

### Lint Configuration

**Enabled:** `clippy::all`, `clippy::pedantic`, `clippy::nursery`, `clippy::cargo`, `-D warnings`
**Denied:** `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`
**Allow only with justification:** `#[allow(clippy::unwrap_used)]  // JUSTIFICATION: Test code only`

### Logging Standards

```rust
alert_critical!("Database failed: {}", error);  // log::error!
alert_warning!("Retry attempt {}", n);          // log::warn!
alert_info!("Processing {} items", count);      // log::info!
alert_success!("Completed");                    // log::info!
alert_debug!("State: {:?}", state);             // log::debug!
```

Never use `println!` or `eprintln!` in production.

### Performance Principles

1. Minimize allocations (reuse buffers, use references)
2. Stack > Heap (prefer stack allocation)
3. References > Owned (`&str` > `String`)
4. Static dispatch > Dynamic (generics > trait objects)
5. Measure, don't guess (`cargo make test-timings`)

---

## Observability & Weaver

### Weaver Integration (Dogfooding)

Framework dogfoods Weaver for semantic convention validation.

```bash
cargo make weaver-bootstrap  # CLI + registry
cargo make weaver-smoke      # Version check + telemetry span
cargo make test-integration  # Full integration (Docker + weaver feature)

export WEAVER_ALLOW_SKIP=1   # Temporary skip when Docker unavailable
```

### Weaver Live-Check

**Feature:** `weaver` | **Purpose:** Validate OTEL spans/metrics against semantic conventions in real-time

```rust
use chicago_tdd_tools::observability::weaver::*;

fixture_test!(test_weaver, fixture, {
    let weaver = fixture.weaver_instance();
    send_otel_span("http.request", attributes);
    let result = weaver.validate_span("http.request");
    assert_ok!(result);
});
```

### OTEL Validation

**Feature:** `otel` | **Purpose:** Validate OTEL instrumentation without Weaver

```rust
use chicago_tdd_tools::observability::otel::*;

test!(test_otel, {
    let span = create_test_span("my.operation");
    assert_eq!(span.name(), "my.operation");
    assert!(span.has_attribute("service.name"));
});
```

---

## Common Tasks

### Making Changes

```bash
# Edit code
cargo make fmt && cargo make check && cargo make test-unit && cargo make lint
cargo make pre-commit  # Or run all at once
git add . && git commit -m "feat: description"
cargo make ci-local    # Before push (optional but recommended)
```

### Adding Features (TDD)

```bash
git checkout -b feature/name
# Write failing test → Implement minimal code → Refactor
cargo make pre-commit && cargo make test-all
git add . && git commit -m "feat: description" && git push -u origin feature/name
```

### Fixing CI

```bash
cargo make ci-local  # Simulate CI
# Fix issues (see CI Failure Recovery section)
git commit --amend --no-edit
```

### Adding Dependencies

```bash
# Add to Cargo.toml with justification comment
# Update feature flags if optional
cargo make test --all-features && cargo make test --no-default-features
cargo make audit
# Document in CLAUDE.md if significant
```

### Preparing Release

```bash
# 1. Update version in Cargo.toml + proc_macros/Cargo.toml
# 2. Update CHANGELOG.md + docs/releases/RELEASE_NOTES_*.md
cargo make release-validate  # Comprehensive checks
cargo make release           # Full pipeline
git tag -a v1.x.x -m "Release v1.x.x" && git push origin v1.x.x
```

---

## Documentation Structure

### Hierarchy

**Quick (80%):** `README.md`, `docs/getting-started/QUICK_GUIDE.md`
**Getting Started:** `GETTING_STARTED.md`, `USER_GUIDE.md`
**Reference:** `API_REFERENCE.md`, `ARCHITECTURE.md`, `SLA_REFERENCE.md`
**Patterns:** `cookbook/src/` (20 Alexander-style patterns: 5 testing, 5 architecture, 10 design)
**Process:** `SPR_GUIDE.md`, `FMEA_TESTS_BUILD_ACTIONS.md`, `CODE_REVIEW_CHECKLIST.md`, `TEST_ISOLATION_GUIDE.md`
**Analysis:** `docs/analysis/` (root cause), `docs/research/` (innovation)
**Releases:** `CHANGELOG.md`, `RELEASE_NOTES_*.md`

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

## AI Assistant Guidelines

### DO

1. Use `cargo make` exclusively (never raw `cargo`)
2. Run `cargo make pre-commit` before suggesting commits
3. Follow Poka-Yoke (prevent errors at compile time)
4. Use `?`/`match`/`if let` (never `unwrap`/`expect`)
5. Add tests for features (TDD: test first)
6. Use alert macros for logging (never `println!`)
7. Document significant patterns in `cookbook/`
8. Check FMEA when modifying CI/build
9. Follow AAA pattern in tests
10. Apply 80/20 thinking (second idea = sweet spot)

### DON'T

1. Use `.unwrap()`, `.expect()`, `panic!()`, `todo!()`, `unimplemented!()` in production
2. Use `println!`/`eprintln!` for logging
3. Skip tests when adding features
4. Ignore clippy warnings (fix or justify with `#[allow]`)
5. Add dependencies without justification
6. Commit without `cargo make pre-commit`
7. Modify CI without understanding FMEA
8. Break backward compatibility without discussion
9. Use alphabetical organization (use capability groups)
10. Guess at performance (measure with `cargo make test-timings`)

### Code Review Checklist

✅ Tests added | ✅ No unwrap/expect | ✅ Result/? error handling | ✅ Structured logging (alert/log)
✅ AAA pattern | ✅ No clippy warnings | ✅ Docs updated | ✅ `cargo make pre-commit` passes
✅ Backward compatible | ✅ FMEA considerations for critical paths

See `docs/process/CODE_REVIEW_CHECKLIST.md` for complete checklist.

### Common Pitfalls

1. **cargo instead of cargo make** → Proc-macro errors, missing timeouts → Always `cargo make`
2. **unwrap/expect** → CI failure → Use `?`/`match`/`if let` + `cargo make install-hooks`
3. **Ignoring clippy** → CI failure → `cargo make lint` + `cargo make pre-commit`
4. **Skipping tests** → Coverage regression → TDD approach (test first)
5. **Breaking compatibility** → User breakage → Re-export at crate root, deprecate gradually

---

## Quick Reference

```bash
# Essential
cargo make pre-commit                              # ALWAYS before commit
cargo make check && cargo make test-unit           # Fast dev cycle
cargo make ci-local                                # Simulate CI
cargo make test-all                                # Full suite (Docker)
cargo make release-validate                        # Release checks
```

**Quality is the default. Prevention beats detection.**

---

**Version:** 1.1.0 | **Updated:** 2025-11-14 | **Team:** KNHK | **License:** MIT
