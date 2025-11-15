# Release Plan: v1.3.0 - "Enhanced Ergonomics"

**Version:** 1.3.0 (Minor Release)
**Target Date:** 2025-12-15
**Status:** Planning
**Theme:** Developer Experience Enhancement

---

## Executive Summary

Version 1.3.0 focuses on **developer experience enhancements** through expanded assertion capabilities, improved fixture ergonomics, and richer testing utilities. This release delivers immediate productivity gains while maintaining our core principles of **Poka-Yoke design** (error prevention at compile time) and **type-first thinking**.

**Key Value Proposition:** Enable developers to write more expressive, maintainable tests with less boilerplate while preserving backward compatibility.

---

## Release Objectives

### Primary Goals

1. **Assertion Expansion Pack** - Fill ergonomic gaps in assertion library
2. **Fixture Introspection** - Enable fixture debugging and inspection
3. **Builder Enhancements** - Reduce boilerplate in test data creation
4. **Snapshot Testing Improvements** - Inline snapshots and redaction hooks
5. **Mutation Testing Extensions** - More comprehensive mutation coverage
6. **Integration Testing Polish** - Enhanced testcontainers capabilities

### Success Metrics

- **Developer Satisfaction:** Reduced boilerplate in 80% of common test scenarios
- **Test Clarity:** Improved assertion failure messages with actionable context
- **Backward Compatibility:** 100% compatibility with v1.2.0 code
- **Coverage Maintenance:** Maintain 85%+ line coverage
- **Documentation:** All new features documented with cookbook examples

---

## Feature Breakdown

### 1. Assertion Expansion Pack (High Priority)

**Rationale:** Assertions are the most frequently used API surface. Ergonomic improvements here have multiplicative impact.

#### 1.1 Collection Assertions

```rust
// New macros (to be implemented)
assert_contains!(collection, item);
assert_not_contains!(collection, item);
assert_subset!(subset, superset);
assert_superset!(superset, subset);
```

**Implementation Details:**
- Macro-based for zero-cost abstraction
- Generic over `IntoIterator` for broad applicability
- Detailed diff output on failure
- Location: `src/core/macros/assert.rs`

**Impact:** 80% of collection testing scenarios simplified

#### 1.2 JSON Assertions

```rust
// New macro (to be implemented)
assert_json_eq!(actual_json, expected_json);
assert_json_eq!(actual_json, expected_json, "Custom message");
```

**Implementation Details:**
- Built on `serde_json` (already a dependency)
- Semantic equality (ignores key order, whitespace)
- Pretty-printed diff output on failure
- Location: `src/core/macros/assert.rs`

**Impact:** Eliminates manual JSON comparison boilerplate

#### 1.3 Approximate Equality

```rust
// New macro (to be implemented)
assert_approx_eq!(actual, expected, epsilon);
assert_approx_eq!(actual, expected, epsilon, "Custom message");
```

**Implementation Details:**
- Generic over `f32`, `f64`
- Configurable epsilon (tolerance)
- Clear failure messages with actual difference
- Location: `src/core/macros/assert.rs`

**Impact:** Eliminates floating-point comparison issues

#### 1.4 Pattern Matching Assertions

```rust
// New macro (to be implemented)
assert_matches_pattern!(text, r"^\d{3}-\d{4}$");  // Regex
assert_matches_glob!(path, "src/**/*.rs");         // Glob
```

**Implementation Details:**
- Regex: Use standard library `regex` crate (optional feature)
- Glob: Use `globset` crate (optional feature)
- Feature flags: `pattern-assertions`, `glob-assertions`
- Location: `src/core/macros/assert.rs`

**Impact:** Simplifies string/path validation in tests

---

### 2. Fixture Introspection API (Medium Priority)

**Rationale:** Debugging complex test fixtures requires visibility into fixture state and metadata.

#### 2.1 Fixture Metadata

```rust
// Enhancement to existing fixture system
#[derive(Fixture)]
struct MyFixture {
    data: String,
    #[fixture(metadata)]
    metadata: FixtureMetadata,
}

// Access metadata
fixture.metadata().created_at();
fixture.metadata().snapshot();  // State snapshot for debugging
```

**Implementation Details:**
- `FixtureMetadata` tracks creation time, snapshot history
- Opt-in via `#[fixture(metadata)]` attribute
- Zero-cost when not used
- Location: `src/core/fixture.rs`, `proc_macros/src/lib.rs`

**Impact:** Simplifies debugging complex fixture interactions

#### 2.2 Scoped Metadata

```rust
// New API (to be implemented)
fixture.with_scoped_metadata("test_phase", "arrange", || {
    // Metadata automatically expires at end of scope
    setup_complex_state();
});
```

**Implementation Details:**
- RAII-based scope management
- Automatic cleanup via `Drop` trait
- Stack-based metadata storage (no heap allocations)
- Location: `src/core/fixture.rs`

**Impact:** Reduces metadata cleanup boilerplate

---

### 3. Builder System Enhancements (Medium Priority)

**Rationale:** Test data builders are heavily used. Small ergonomic improvements yield large productivity gains.

#### 3.1 Builder Presets

```rust
// New API (to be implemented)
TestDataBuilder::preset("valid_order")
    .with_var("customer_id", "12345");

// Define presets
TestDataBuilder::register_preset("valid_order", |builder| {
    builder
        .with_var("order_id", "ORD-001")
        .with_var("amount", "100.00")
        .with_var("status", "pending")
});
```

**Implementation Details:**
- Registry of named presets (lazy_static HashMap)
- Composable: presets can build on other presets
- Type-safe: compiler validates preset names (via const generics)
- Location: `src/core/builders.rs`

**Impact:** Eliminates repeated builder setup in test suites

#### 3.2 Auto-Derived Fake Data

```rust
// Enhancement to existing #[derive(TestBuilder)]
#[derive(TestBuilder, FakeBuilder)]
struct User {
    email: String,
    age: u32,
}

// Automatically generates:
UserBuilder::new()
    .with_fake_email()    // Generates realistic email
    .with_fake_age()      // Generates realistic age
    .build();
```

**Implementation Details:**
- Requires `fake-data` feature flag (already exists)
- Leverages `fake` crate integration
- Derives from field types (String → fake::internet::email)
- Location: `proc_macros/src/lib.rs`

**Impact:** Reduces magic numbers and unrealistic test data

#### 3.3 Builder Validation Hooks

```rust
// New API (to be implemented)
TestDataBuilder::new()
    .with_validation(|data| {
        if data.get("amount").unwrap().parse::<f64>()? < 0.0 {
            Err("Amount must be positive")
        } else {
            Ok(())
        }
    })
    .build()?;  // Returns Result instead of panicking
```

**Implementation Details:**
- Optional validation closure
- Executes before `build()` completes
- Poka-Yoke: Catches invalid test data at build time
- Location: `src/core/builders.rs`

**Impact:** Prevents invalid test data from reaching test execution

---

### 4. Snapshot Testing Improvements (Medium Priority)

**Rationale:** Snapshot testing is powerful but current workflow has friction points (external file management).

#### 4.1 Inline Snapshot Mode

```rust
// New macro (to be implemented)
assert_snapshot_inline!(actual, @"expected_inline_value");
```

**Implementation Details:**
- Snapshots stored directly in source code (like Rust's `insta` crate)
- Requires `snapshot-testing` feature flag
- Integration with existing `insta` dependency
- Location: `src/testing/snapshot.rs`

**Impact:** Faster snapshot creation, better version control diffs

#### 4.2 Snapshot Redaction Hooks

```rust
// New API (to be implemented)
SnapshotAssert::new(data)
    .with_redaction(|value| {
        // Redact timestamps, UUIDs, etc.
        value.replace_regex(r"\d{4}-\d{2}-\d{2}", "[DATE]")
    })
    .assert_matches("snapshot_name");
```

**Implementation Details:**
- Pre-snapshot transformation pipeline
- Common redactions built-in (UUIDs, timestamps, IPs)
- Location: `src/testing/snapshot.rs`

**Impact:** Eliminates flaky snapshots due to dynamic values

#### 4.3 Snapshot Profiles

```rust
// New API (to be implemented)
SnapshotConfig::profile("strict");      // Exact match
SnapshotConfig::profile("pretty");      // Formatted, stable order
SnapshotConfig::profile("compact");     // Minimal whitespace
SnapshotConfig::profile("diff-only");   // Show only differences
```

**Implementation Details:**
- Profile configuration via environment variable or API
- Profiles control formatting, ordering, diff display
- Location: `src/testing/snapshot.rs`

**Impact:** Adapts snapshot behavior to different testing contexts

---

### 5. Mutation Testing Extensions (Low Priority)

**Rationale:** More mutation operators increase test coverage discovery.

#### 5.1 New Mutation Operators

```rust
// New operators (to be implemented in mutation framework)
- SwapValues(key1, key2)          // Swap two values in state
- RemoveRandomKey                  // Remove random key from map
- ToggleBoolean(key)               // Flip boolean values
- NumericDelta(key, offset)        // Add/subtract from numeric values
- StringCase(key)                  // Change string case
```

**Implementation Details:**
- Extend existing mutation framework (`src/testing/mutation.rs`)
- Configurable operator weighting
- Deterministic random selection (seeded RNG)
- Location: `src/testing/mutation.rs`

**Impact:** Discovers gaps in test coverage for edge cases

---

### 6. Integration Testing Polish (Medium Priority)

**Rationale:** Testcontainers integration is production-ready but missing common use cases.

#### 6.1 Enhanced Wait Conditions

```rust
// Enhancement to existing testcontainers module
container
    .wait_for_log_line("Server started", Duration::from_secs(30))?
    .wait_for_tcp_port(5432, Duration::from_secs(10))?
    .wait_for_command_exit("pg_isready", Duration::from_secs(5))?;
```

**Implementation Details:**
- Extends existing `WaitCondition` enum
- Timeout-based polling with exponential backoff
- Location: `src/integration/testcontainers/wait.rs` (partially exists)

**Impact:** Eliminates flaky integration tests due to timing issues

#### 6.2 Service Helpers

```rust
// New API (to be implemented)
let postgres = PostgresContainer::default()
    .with_version("16")
    .with_user("test_user")
    .with_password("test_password")
    .with_database("test_db")
    .start()?;

let connection_string = postgres.connection_string();
```

**Implementation Details:**
- Common service wrappers (Postgres, Redis, Kafka, RabbitMQ)
- Pre-configured health checks and port mappings
- Requires `testcontainers` feature flag
- Location: `src/integration/testcontainers/services/` (new module)

**Impact:** Reduces integration test boilerplate by 60%

#### 6.3 Reusable Containers

```rust
// New API (to be implemented)
#[fixture(reusable)]
fn shared_postgres() -> PostgresContainer {
    PostgresContainer::default().start().unwrap()
}

// Reused across all tests in module
fixture_test!(test_query_1, fixture, { /* ... */ });
fixture_test!(test_query_2, fixture, { /* ... */ });
```

**Implementation Details:**
- Container lifecycle managed at module level
- Single container per test module (amortized startup cost)
- Automatic cleanup at module exit
- Location: `src/core/fixture.rs`, `proc_macros/src/lib.rs`

**Impact:** Reduces integration test suite runtime by 40-60%

---

### 7. CLI Testing Enhancements (Low Priority)

**Rationale:** CLI testing is already comprehensive via `trycmd`, but environment management is manual.

#### 7.1 Environment Variable Helpers

```rust
// New API (to be implemented)
CliTest::new("my_binary")
    .with_env("DATABASE_URL", "postgres://...")
    .with_env_from_file(".env.test")
    .with_clean_env()  // Start with empty environment
    .run()?;
```

**Implementation Details:**
- Extends existing `CliTest` API (`src/testing/cli.rs`)
- Scoped environment variables (reset after test)
- Requires `cli-testing` feature flag
- Location: `src/testing/cli.rs`

**Impact:** Simplifies CLI testing with environment dependencies

#### 7.2 Separate stderr Capture

```rust
// New API (to be implemented)
let result = CliTest::new("my_binary")
    .capture_stderr_separately()
    .run()?;

assert_contains!(result.stdout, "Success");
assert_contains!(result.stderr, "Warning");
```

**Implementation Details:**
- Independent stdout/stderr assertions
- Maintains backward compatibility (combined output still default)
- Location: `src/testing/cli.rs`

**Impact:** Better assertions for CLI output validation

---

## Implementation Timeline

### Phase 1: Assertions (Weeks 1-2)
- Collection assertions (`assert_contains!`, `assert_subset!`, etc.)
- JSON assertions (`assert_json_eq!`)
- Approximate equality (`assert_approx_eq!`)
- Pattern matching assertions (optional features)

**Deliverable:** Expanded assertion macro suite

### Phase 2: Fixtures & Builders (Weeks 3-4)
- Fixture introspection (metadata, snapshots)
- Scoped metadata
- Builder presets
- Auto-derived fake data

**Deliverable:** Enhanced fixture and builder APIs

### Phase 3: Snapshot Testing (Week 5)
- Inline snapshot mode
- Redaction hooks
- Snapshot profiles

**Deliverable:** Improved snapshot testing workflow

### Phase 4: Integration & Mutation (Week 6)
- Enhanced testcontainers wait conditions
- Service helpers (Postgres, Redis)
- Reusable containers
- New mutation operators

**Deliverable:** Production-ready integration testing utilities

### Phase 5: CLI & Documentation (Week 7)
- CLI environment helpers ✅ (Documented in CHANGELOG)
- Separate stderr capture ✅ (Documented in CHANGELOG)
- **Playground CLI Enhancements** ✅ (Completed)
  - clap-noun-verb v3.7.1 upgrade with enhanced #[arg(...)] attributes
  - Multi-format output support (JSON, YAML, TOML, Table, TSV)
  - `--format` flag added to all stat() and list() commands
  - New `format_utils` module with OutputFormat enum
  - Applied consistently across all 6+ CLI modules
- Cookbook examples for all new features
- Updated user guides

**Deliverable:** Complete documentation suite

### Phase 6: Testing & Release (Week 8)
- Comprehensive test coverage (maintain 85%+)
- Cargo make ci-local validation
- Release validation
- GitHub release and crates.io publish

**Deliverable:** v1.3.0 released to crates.io

---

## Risk Management

### Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|---------|-----------|
| Backward compatibility breakage | Low | High | Comprehensive integration tests, deprecation warnings |
| Feature creep | Medium | Medium | Strict scope adherence, deferred features documented |
| Test coverage regression | Low | High | CI enforcement, coverage checks in pre-commit |
| Performance regression | Low | Medium | Benchmark suite, performance tests for new features |
| Proc-macro compilation issues | Medium | High | Incremental development, extensive compile-fail tests |

### Process Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|---------|-----------|
| Timeline slippage | Medium | Low | Buffer in Week 8, can defer low-priority features |
| Documentation lag | Medium | Medium | Documentation written alongside implementation |
| CI/CD failures | Low | Medium | Pre-commit hooks, cargo make ci-local simulation |

---

## Testing Strategy

### Unit Testing
- **Target:** 90%+ coverage for new code
- **Approach:** TDD - tests written before implementation
- **Tooling:** `cargo make test-unit`

### Integration Testing
- **Target:** All testcontainers enhancements validated with Docker
- **Approach:** Real service dependencies (Postgres, Redis)
- **Tooling:** `cargo make test-integration`

### Compile-Fail Testing
- **Target:** All proc-macro enhancements validated
- **Approach:** `trybuild` for invalid code rejection
- **Tooling:** `tests/compile-fail/`

### Property-Based Testing
- **Target:** New assertion macros validated against random inputs
- **Approach:** `proptest` for edge case discovery
- **Tooling:** `cargo make test-property`

### Mutation Testing
- **Target:** New mutation operators validated
- **Approach:** Self-testing with new operators
- **Tooling:** `cargo make test-mutation`

---

## Documentation Plan

### User-Facing Documentation

1. **Quick Guide Update** (`docs/getting-started/QUICK_GUIDE.md`)
   - Add examples for new assertions
   - 5-minute quick start for new features

2. **User Guide Update** (`docs/reference/USER_GUIDE.md`)
   - Comprehensive coverage of all new APIs
   - Migration examples from v1.2.0

3. **API Reference Update** (`docs/reference/API_REFERENCE.md`)
   - Complete API documentation for new modules
   - Type signatures and examples

4. **Cookbook Additions** (`cookbook/src/`)
   - **Pattern 21:** "Assertion Ergonomics" - When to use which assertion
   - **Pattern 22:** "Fixture Introspection" - Debugging complex fixtures
   - **Pattern 23:** "Builder Presets" - Reusable test data patterns
   - **Pattern 24:** "Snapshot Redaction" - Managing dynamic values in snapshots
   - **Pattern 25:** "Integration Test Performance" - Reusable containers strategy

5. **Release Notes** (`docs/releases/RELEASE_NOTES_v1.3.0.md`)
   - Comprehensive changelog
   - Migration guide
   - Examples for all new features

### Internal Documentation

1. **Architecture Update** (`docs/reference/ARCHITECTURE.md`)
   - New module structure
   - Design decisions and rationale

2. **CLAUDE.md Update**
   - New feature flags
   - Updated build commands
   - New assertion macros reference

---

## Feature Flags

### New Feature Flags (v1.3.0)

```toml
# Pattern assertions (optional)
pattern-assertions = ["dep:regex"]
glob-assertions = ["dep:globset"]

# Integration service helpers (optional)
testcontainers-services = ["testcontainers", "dep:postgres", "dep:redis-rs"]
```

### Updated Feature Bundles

```toml
# testing-extras: Most common advanced testing features
testing-extras = [
  "property-testing",
  "snapshot-testing",
  "fake-data",
  "pattern-assertions",  # NEW
]

# integration-full: Full integration testing with observability
integration-full = [
  "testcontainers",
  "testcontainers-services",  # NEW
  "weaver",
]
```

---

## Backward Compatibility Guarantees

### API Stability

- **100% backward compatible** - All v1.2.0 code compiles without changes
- **Deprecation warnings** - Any deprecated APIs marked with `#[deprecated]` and alternative suggested
- **Re-exports maintained** - All crate-root re-exports continue working

### Breaking Changes

**None.** This is a minor release adhering to semantic versioning.

---

## Dependencies

### New Dependencies

| Crate | Version | Purpose | Optional |
|-------|---------|---------|----------|
| `regex` | ^1.10 | Pattern assertions | Yes (`pattern-assertions`) |
| `globset` | ^0.4 | Glob assertions | Yes (`glob-assertions`) |
| `postgres` | ^0.19 | Postgres testcontainer helper | Yes (`testcontainers-services`) |
| `redis-rs` | ^0.23 | Redis testcontainer helper | Yes (`testcontainers-services`) |

**Rationale:** All new dependencies are optional and behind feature flags. Zero impact on default installation.

---

## Performance Considerations

### Zero-Cost Abstractions

- **Assertion macros:** Compile-time expansion, no runtime overhead
- **Fixture metadata:** Opt-in only, zero cost when unused
- **Builder presets:** Static dispatch via generics

### Expected Performance Impact

- **Assertion macros:** 0% overhead (macro expansion)
- **Fixture introspection:** <1% overhead when enabled (metadata tracking)
- **Builder presets:** 0% overhead (static dispatch)
- **Reusable containers:** 40-60% faster integration test suite

**Measurement:** Benchmark suite extended to cover new features (`cargo make test-timings`)

---

## Success Criteria

### Must-Have (Release Blockers)

- ✅ All new features documented in API reference
- ✅ Cookbook examples for 5 new patterns
- ✅ 85%+ line coverage maintained
- ✅ Zero clippy warnings
- ✅ All CI checks passing (lint, test, unwrap-check)
- ✅ Backward compatibility validated (all examples compile)
- ✅ Release notes complete with migration guide

### Nice-to-Have (Deferred if Time-Constrained)

- 🎯 Pattern assertions (`pattern-assertions` feature)
- 🎯 Glob assertions (`glob-assertions` feature)
- 🎯 Service helpers beyond Postgres/Redis
- 🎯 CLI testing enhancements (separate stderr capture)

---

## Post-Release Activities

### Week 9: Monitoring & Feedback

- Monitor GitHub issues for bug reports
- Collect community feedback on new features
- Identify quick-win improvements for v1.3.1 patch

### Week 10-12: Blog Posts & Promotion

- Technical blog post: "Enhanced Ergonomics in Chicago TDD Tools v1.3.0"
- Tutorial series: "Advanced Testing Patterns with chicago-tdd-tools"
- Social media promotion (Twitter, Reddit, Hacker News)

### Week 13+: Next Release Planning

- Gather feedback for v1.4.0 features
- Prioritize community-requested enhancements
- Begin design phase for next minor release

---

## Deferred Features (Future Releases)

The following features were considered but deferred to maintain focused scope:

### Deferred to v1.4.0+

1. **Async Fixture Composition** - Complex async fixture dependencies (needs design iteration)
2. **Custom Test Reporters** - JUnit XML, JSON output (moderate complexity)
3. **Test Parallelization Utilities** - Parallel test execution helpers (needs careful design)
4. **Advanced Mutation Strategies** - Semantic mutation operators (research phase)
5. **JTBD Scenario Hierarchy** - Nested scenarios for workflow testing (design iteration needed)

---

## Appendices

### Appendix A: Assertion Matrix

| Assertion Type | Current (v1.2.0) | New (v1.3.0) |
|---------------|------------------|--------------|
| Result assertions | `assert_ok!`, `assert_err!`, `assert_fail!` | - |
| Equality | `assert_eq_msg!`, `assert_eq_enhanced!` | `assert_approx_eq!`, `assert_json_eq!` |
| Range | `assert_in_range!` | - |
| Performance | `assert_within_tick_budget!` | - |
| Collections | - | `assert_contains!`, `assert_subset!`, `assert_superset!` |
| Patterns | - | `assert_matches_pattern!`, `assert_matches_glob!` |
| Guards | `assert_guard_constraint!` | - |

### Appendix B: Builder API Evolution

```rust
// v1.2.0
TestDataBuilder::new()
    .with_var("order_id", "ORD-001")
    .with_var("amount", "100.00")
    .with_var("status", "pending")
    .build();

// v1.3.0 (with presets)
TestDataBuilder::preset("valid_order")
    .with_var("customer_id", "12345")  // Override specific fields
    .build();

// v1.3.0 (with validation)
TestDataBuilder::preset("valid_order")
    .with_validation(validate_positive_amount)
    .build()?;  // Returns Result
```

### Appendix C: Snapshot Workflow Comparison

```rust
// v1.2.0 (external snapshots)
assert_snapshot!("snapshot_name", data);
// Requires: cargo make snapshot-review
// Requires: cargo make snapshot-accept

// v1.3.0 (inline snapshots)
assert_snapshot_inline!(data, @r#"
{
  "id": 123,
  "name": "John"
}
"#);
// Snapshots stored in source code
// Better version control diffs
```

---

## Approval & Sign-Off

**Release Manager:** [To Be Assigned]
**Technical Lead:** [To Be Assigned]
**Documentation Lead:** [To Be Assigned]

**Approved:** [Pending]
**Date:** [Pending]

---

**Version:** 1.0
**Last Updated:** 2025-11-15
**Status:** Draft
**Next Review:** 2025-11-22
