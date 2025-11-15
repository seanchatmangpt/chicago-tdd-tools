# Chicago TDD Tools

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

**Rust testing framework enforcing Chicago-style TDD (Classicist Test-Driven Development) through compile-time guarantees.**

If it compiles, correctness follows. Type system encodes invariants. Quality is the default, not an afterthought.

---

## Why Chicago TDD?

Chicago-style TDD (Classicist approach) focuses on **behavior verification** using **real collaborators** instead of mocks. This framework enforces that philosophy through Rust's type system:

- **Type-First Design**: Compiler prevents invalid test states before runtime. State machines encoded at type level—if your test compiles, the AAA (Arrange-Act-Assert) pattern is enforced.
- **Error Prevention (Poka-Yoke)**: Mistakes caught at compile time, not in CI. No `.unwrap()` in production code. No `panic!()`. Git hooks prevent them from being committed.
- **Zero-Cost Abstractions**: All safety guarantees compiled away—performance equals unsafe manual code.
- **80/20 Focus**: Framework solves 80% of testing problems with 20% extra effort via generics, const generics, and macros.

**Result**: Tests that actually verify behavior. Bugs prevented before code review. Production panic rate: ~zero.

---

## Quick Start (Choose Your Path)

### 1️⃣ First Time User? → 5-Minute Setup

```bash
# Install cargo-make (required)
cargo install cargo-make

# Create test file: tests/my_first_test.rs
mkdir -p tests
cat > tests/my_first_test.rs << 'EOF'
use chicago_tdd_tools::prelude::*;

test!(test_addition, {
    // Arrange
    let x = 5;
    let y = 3;
    // Act
    let result = x + y;
    // Assert
    assert_eq!(result, 8);
});

async_test!(test_async_example, {
    let result = async { 5 + 3 }.await;
    assert_eq!(result, 8);
});

fixture_test!(test_with_fixture, fixture, {
    let counter = fixture.test_counter();
    assert!(counter >= 0);
});
EOF

# Run tests
cargo make test
```

**✓ Installation complete** when `cargo make test` shows 3 passing tests.

---

### 2️⃣ Just Want Examples? → Examples Directory

```bash
# Browse working examples (8 included)
ls examples/
# Output: basic_test.rs, property_testing.rs, mutation_testing.rs,
#         snapshot_testing.rs, concurrency_testing.rs, otel_weaver_testing.rs,
#         testcontainers_example.rs, cli_testing.rs

# Run a specific example
cargo make example-basic-test
```

---

### 3️⃣ Need Full Reference? → API Documentation

```bash
cargo make docs   # Generate and open Rustdoc
```

---

### 4️⃣ Using Docker/Containers? → Integration Guide

See [Integration Testing](#integration-testing) section below.

---

### 5️⃣ Testing Observability/OTEL? → Weaver Setup

See [Observability & Weaver](#observability--weaver) section below.

---

## Core Capabilities (With Real Examples)

### 1. Essential Testing Macros

**Synchronous tests** (no async runtime needed):

```rust
use chicago_tdd_tools::prelude::*;

test!(test_sync_behavior, {
    // Arrange: Set up test data
    let input = vec![1, 2, 3, 4, 5];

    // Act: Execute code under test
    let sum: i32 = input.iter().sum();

    // Assert: Verify behavior
    assert_eq!(sum, 15);
});
```

**Async tests** (1s default timeout):

```rust
async_test!(test_async_operation, {
    // Arrange
    let data = async { vec![1, 2, 3] }.await;

    // Act: Run async code
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    let result = data.len();

    // Assert
    assert_eq!(result, 3);
});
```

**Fixture-based tests** (automatic setup/teardown):

```rust
fixture_test!(test_with_fixture_isolation, fixture, {
    // Arrange: Fixture created automatically, isolated per test
    let counter = fixture.test_counter();

    // Act: Use fixture
    let incremented = counter + 1;

    // Assert
    assert!(incremented > counter);
    // Cleanup: Automatic on scope exit
});
```

**Performance tests** (tick budget validation):

```rust
performance_test!(test_performance_constraint, {
    // Arrange
    let work = || {
        let mut sum = 0;
        for i in 0..100 {
            sum += i;
        }
        sum
    };

    // Act
    let result = work();

    // Assert within tick budget (default: ≤8 ticks)
    assert_within_tick_budget!(result > 0);
});
```

---

### 2. Advanced Assertion Helpers

Result type assertions:

```rust
test!(test_result_assertions, {
    let success: Result<i32, String> = Ok(42);
    let failure: Result<i32, String> = Err("oops".to_string());

    // Assert success case
    assert_ok!(success);
    assert_eq!(success.unwrap_or(0), 42);

    // Assert error case with detailed messages
    assert_err!(failure);
    assert_in_range!(success.unwrap_or(0), 40, 50);

    // Custom assertion messages
    assert_eq_msg!(success.unwrap_or(0), 42, "Expected 42, got: {:?}", success);
});
```

---

### 3. Property-Based Testing

Generate random test data and verify properties hold **for all inputs**:

```rust
use chicago_tdd_tools::property::*;

test!(test_commutativity_property, {
    // Arrange: Create property test generator
    let mut generator = PropertyTestGenerator::<100, 5>::new().with_seed(42);

    // Generate random test data
    let test_data = generator.generate_test_data();

    // Act & Assert: Verify property holds for all generated data
    for _item in test_data {
        // Property: a + b == b + a (commutativity)
        let a = rand::random::<u32>();
        let b = rand::random::<u32>();
        assert_eq!(a + b, b + a, "Addition is commutative");
    }
});
```

**With proptest** (requires `property-testing` feature):

```rust
test!(test_distributivity_with_proptest, {
    use proptest::prelude::*;

    let strategy = ProptestStrategy::new().with_cases(100);

    // Test: a * (b + c) == (a * b) + (a * c)
    strategy.test(
        proptest::prelude::any::<(u32, u32, u32)>(),
        |(a, b, c)| a * (b + c) == (a * b) + (a * c)
    );
});
```

**When to use**: Edge cases are hard to imagine. Random generation finds them automatically.

---

### 4. Mutation Testing

Verify test quality by **intentionally breaking code** and checking tests catch it:

```rust
use chicago_tdd_tools::mutation::*;
use std::collections::HashMap;

test!(test_mutation_detection, {
    // Arrange: Create mutation tester with test data
    let mut data = HashMap::new();
    data.insert("key1", "value1");
    let mut tester = MutationTester::new(data);

    // Act: Apply mutation (remove a key)
    tester.apply_mutation(MutationOperator::RemoveKey("key1".to_string()));

    // Assert: Test should fail because we removed data
    let mutated = tester.current_data();
    assert!(mutated.is_empty(), "Mutation was not caught!");

    // Calculate mutation score
    let score = MutationScore::calculate(95, 100);  // 95 mutations caught of 100
    assert!(score.is_acceptable(), "Score: {}%", score.score());
});
```

**Mutation operators**: `RemoveKey`, `AddKey`, `ChangeValue`, `NegateCondition`

**Target**: ≥80% mutation score indicates thorough test coverage.

---

### 5. Snapshot Testing

Verify complex outputs (JSON, HTML, serialized data) don't change unexpectedly:

```rust
test!(test_snapshot_comparison, {
    use insta::assert_snapshot;  // Requires snapshot-testing feature

    let data = serde_json::json!({
        "user": "alice",
        "status": "active"
    });

    // Assert snapshot matches expected output
    assert_snapshot!(data.to_string());

    // Workflow:
    // 1. First run: creates snapshot file
    // 2. Second run: compares against snapshot
    // 3. Change output? Update snapshot with: cargo make snapshot-accept
});
```

**Snapshot management**:
```bash
cargo make snapshot-review    # Review changes
cargo make snapshot-accept    # Accept new snapshots
cargo make snapshot-reject    # Reject and revert
```

---

### 6. Concurrency Testing

Detect race conditions with deterministic thread-safe testing:

```rust
test!(test_concurrent_safety, {
    use chicago_tdd_tools::concurrency::*;

    // Arrange: Use loom for deterministic testing
    loom::model(|| {
        let data = std::sync::Arc::new(std::sync::Mutex::new(0));
        let data_clone = data.clone();

        // Act: Spawn thread accessing shared data
        let handle = loom::thread::spawn(move || {
            let mut guard = data_clone.lock().unwrap();
            *guard += 1;
        });

        // Verify no panics (loom exhaustively tests interleavings)
        handle.join().unwrap();
    });
});
```

---

### 7. CLI Testing

Test command-line interfaces like they're black boxes:

```rust
test!(test_cli_invocation, {
    use chicago_tdd_tools::testing::cli::*;

    // Arrange: Prepare CLI test
    let mut cli_test = CliTest::new("my-cli-tool");

    // Act: Run command
    let output = cli_test
        .arg("--help")
        .run()
        .expect("CLI should run");

    // Assert: Check output
    assert!(output.stdout.contains("Usage:"));
    assert_eq!(output.exit_code, 0);
});
```

---

## Integration Testing

### Docker + Testcontainers

Test with real services (Postgres, Redis, etc.) without manual Docker commands:

```rust
fixture_test!(test_with_postgres, fixture, {
    use chicago_tdd_tools::integration::testcontainers::*;

    // Arrange: Fixture automatically spins up Postgres container
    let container = fixture.postgres_container()
        .expect("Postgres container should start");

    // Get connection string
    let conn_string = container.connection_string();
    println!("Connected to: {}", conn_string);

    // Act: Execute query
    let result = container.execute_query(
        "SELECT count(*) FROM pg_tables;"
    ).await;

    // Assert: Verify result
    assert_ok!(result);

    // Cleanup: Container automatically stopped on fixture drop
});
```

**Enable with**:
```toml
[dev-dependencies]
chicago-tdd-tools = { path = "../chicago-tdd-tools", features = ["testcontainers"] }
```

**Run with**:
```bash
cargo make test-integration  # Requires Docker running
```

---

## Observability & Weaver

Test OpenTelemetry (OTEL) instrumentation and semantic convention compliance with **Weaver live-check**.

### 1. Bootstrap Weaver (First Time)

```bash
# Download Weaver CLI + semantic convention registry
cargo make weaver-bootstrap

# This creates:
# - target/<profile>/weaver (executable)
# - registry/ (semantic conventions)
```

### 2. Quick Smoke Test (No Docker Required)

```bash
# Verify Weaver works + send test span
cargo make weaver-smoke

# Output: Weaver version + telemetry validation
```

### 3. OTEL Span Validation

```rust
use chicago_tdd_tools::observability::otel::*;

test!(test_otel_span_validation, {
    // Arrange: Create OTEL span context
    let trace_id = TraceId::new(12345);
    let span_id = SpanId::new(67890);
    let context = SpanContext::new(trace_id, span_id);

    // Act: Create span with attributes
    let span = Span::new("http.request", context)
        .with_attribute("http.method", "GET")
        .with_attribute("http.url", "https://example.com");

    // Assert: Validate span structure
    assert_eq!(span.name(), "http.request");
    assert!(span.has_attribute("http.method"));
});
```

### 4. Weaver Live-Check (Full Validation)

Validates spans/metrics against OpenTelemetry semantic conventions in real-time:

```rust
fixture_test!(test_weaver_live_check, fixture, {
    use chicago_tdd_tools::observability::weaver::*;

    // Arrange: Create Weaver validator (requires weaver feature + bootstrap)
    let weaver = fixture.weaver_instance()
        .expect("Weaver should initialize");

    // Act: Send OTEL span
    let span = create_http_span("GET", "/api/users");
    send_otel_span(span.clone());

    // Assert: Validate against semantic conventions
    let result = weaver.validate_span("http.request", &span);
    assert_ok!(result, "Span should comply with semantic conventions");
});
```

**Enable with**:
```toml
[dev-dependencies]
chicago-tdd-tools = {
    path = "../chicago-tdd-tools",
    features = ["weaver", "otel"]  # otel auto-enabled with weaver
}
```

**Run integration tests**:
```bash
cargo make test-integration  # Includes Weaver tests
# Or skip if Docker unavailable:
WEAVER_ALLOW_SKIP=1 cargo make test-integration
```

---

## Build System (Important!)

**⚠️ Always use `cargo make`, never raw `cargo`:**

```bash
cargo make check              # Compilation check (fast)
cargo make test               # Unit tests only
cargo make test-unit          # Same as test
cargo make test-integration   # Integration tests (Docker, Weaver)
cargo make test-all           # Unit + integration
cargo make lint               # Clippy checks
cargo make fmt                # Code formatting
cargo make pre-commit         # fmt + lint + unit tests (always run before commit)
cargo make ci-local           # Simulate full CI pipeline
```

**Why mandatory?**
- Handles proc-macro crates correctly
- Enforces timeouts (prevents hanging)
- Consistent build environment
- Single source of truth for build process

**Essential for safety**:
```bash
cargo make install-hooks  # Install git hooks that prevent unwrap/expect in production
```

---

## Quality Standards (Poka-Yoke Enforcement)

### Compile-Time Prevention

**Type-level AAA enforcement**: If test compiles, AAA pattern is correct.

**Sealed traits**: Can't create invalid test states.

**Const assertions**: Size and alignment checked at compile time.

### Build-Time Prevention

**Git hooks**: Prevent `.unwrap()`, `.expect()`, `panic!()` from being committed.

**Clippy enforcement**: All warnings treated as errors (`-D warnings`).

**Timeout SLAs** enforced:
- Quick checks: 5s (fmt, check)
- Compilation: 5-30s depending on profile
- Lint: 300s (CI cold-start)
- Unit tests: 1s per test
- Integration tests: 30s with Docker
- Coverage: 30s

### Runtime Safety

**Result-based errors**: No panics in production code.

```rust
// ❌ Never in production
let value = result.unwrap();

// ✅ Always do this
let value = result?;  // Propagate errors
// OR
let value = match result {
    Ok(v) => v,
    Err(e) => { alert_warning!("Failed: {}", e); default_value }
};
```

**Alert macros** for structured logging:

```rust
alert_critical!("Database unreachable");   // 🚨 Must stop
alert_warning!("Retry attempt {}", n);     // ⚠️ Should stop
alert_info!("Processing {} items", count);  // ℹ️ Informational
alert_success!("Backup complete");          // ✅ Success
alert_debug!("State: {:?}", state);         // 🔍 Diagnostics
```

### Risk Reduction (FMEA)

| Risk | Original | Current | Mitigation |
|------|----------|---------|-----------|
| Production panics (unwrap/expect) | RPN 180 | RPN 36 | Git hooks, CI checks, lint deny |
| Tests pass locally, fail CI | RPN 105 | RPN 21 | Multi-OS, pre-commit simulation |
| Clippy warnings accumulate | RPN 112 | RPN 11 | CI enforcement, pre-commit |
| Flaky tests | RPN 120 | RPN 24 | Retry logic (3x), test isolation |
| Coverage regressions | RPN 336 | RPN 67 | Coverage tracking, Codecov |

---

## Feature Flags

**Core** (always available): `test!`, `async_test!`, `fixture_test!`, builders, assertions

**Enable as needed**:

```toml
[dev-dependencies]
chicago-tdd-tools = {
    path = "../chicago-tdd-tools",
    features = [
        "testing-extras",      # property-testing + snapshot-testing + fake data (most common)
        "otel",                # OpenTelemetry span/metric validation
        "weaver",              # Weaver semantic convention live-check (implies otel)
        "testcontainers",      # Docker container support
        "async",               # Async fixture providers (Rust 1.75+)
    ]
}
```

**Recommended bundles**:
- **80% use case**: `["testing-extras"]` (property + snapshot + fake data)
- **Full testing**: `["testing-extras", "testcontainers"]`
- **With observability**: `["testing-extras", "otel", "weaver"]`
- **Everything**: `["testing-extras", "otel", "weaver", "testcontainers", "async"]`

---

## Documentation Portal

### 📚 Learning Path (Start Here)

1. **[Getting Started](docs/getting-started/GETTING_STARTED.md)** - Installation, first test, troubleshooting
2. **[Quick Guide](docs/getting-started/QUICK_GUIDE.md)** - Essential patterns (80% of use cases, 15 min read)
3. **[User Guide](docs/getting-started/USER_GUIDE.md)** - Comprehensive usage (deep dive, 1 hour)

### 🔧 How-to Guides (Solve Specific Problems)

- **[Weaver Live-Check](docs/features/WEAVER_LIVE_CHECK.md)** - Full OTEL + Weaver setup
- **[CLI Testing Guide](docs/testing/cli-testing-guide.md)** - Test command-line tools
- **[Observability Testing](docs/observability/observability-testing-guide.md)** - OTEL testing patterns
- **[Timeout Enforcement](docs/features/TIMEOUT_ENFORCEMENT.md)** - Custom timeout SLAs

### 📖 Reference (Lookup Technical Details)

- **[API Reference](docs/reference/API_REFERENCE.md)** - Complete API documentation
- **[Architecture](docs/reference/ARCHITECTURE.md)** - Design principles, module organization
- **[SLA Reference](docs/reference/SLA_REFERENCE.md)** - Service level agreements, quality standards

### 🎓 Understanding (Deep Dives)

- **[SPR Guide](docs/process/SPR_GUIDE.md)** - Elite Rust standards, best practices
- **[Code Review Checklist](docs/process/CODE_REVIEW_CHECKLIST.md)** - What reviewers look for
- **[FMEA: Tests, Build, Actions](docs/process/FMEA_TESTS_BUILD_ACTIONS.md)** - Risk analysis, improvements
- **[Test Isolation Guide](docs/process/TEST_ISOLATION_GUIDE.md)** - Preventing test interdependencies
- **[Pattern Cookbook](../cookbook/src/README.md)** - Alexander-style patterns (20 documented)

### 🔍 Troubleshooting

**Problem**: "command not found: cargo-make"
- **Fix**: `cargo install cargo-make`

**Problem**: "cannot find macro 'test!'"
- **Fix**: Add `use chicago_tdd_tools::prelude::*;` to your test file

**Problem**: "feature 'X' is required for module Y"
- **Fix**: Enable feature in `Cargo.toml`: `features = ["feature-name"]`

**Problem**: Tests pass locally but fail in CI
- **Fix**: Run `cargo make ci-local` to simulate CI environment

**More help**: See [Getting Started - Troubleshooting](docs/getting-started/GETTING_STARTED.md#troubleshooting)

---

## Examples

8 complete, runnable examples are included. Browse them:

```bash
# List examples
ls examples/

# Run an example
cargo make example-property-testing
cargo make example-mutation-testing
cargo make example-snapshot-testing
# ...
```

**Example files**:
- `basic_test.rs` - Fixtures, builders, assertions
- `property_testing.rs` - Random test generation, properties
- `mutation_testing.rs` - Test quality validation
- `snapshot_testing.rs` - Output comparison
- `concurrency_testing.rs` - Thread safety with loom
- `otel_weaver_testing.rs` - Observability testing
- `testcontainers_example.rs` - Docker integration
- `cli_testing.rs` - Command-line testing

---

## Requirements

| Component | Minimum | Verify | Install |
|-----------|---------|--------|---------|
| Rust | 1.70 | `rustc --version` | [rustup](https://rustup.rs/) |
| Cargo | Latest stable | `cargo --version` | Included with Rust |
| cargo-make | Latest | `cargo make --version` | `cargo install cargo-make` |
| Tokio | 1.0+ | (add to Cargo.toml) | [tokio](https://tokio.rs) |
| Docker* | Latest | `docker ps` | [Docker Desktop](https://www.docker.com) |
| Rust 1.75+* | For async fixtures | `rustc --version` | `rustup update stable` |

\* Optional—only needed for specific features (Docker, async fixtures)

---

## Contributing & Community

- **Issues/Questions**: [GitHub Issues](https://github.com/seanchatmangpt/chicago-tdd-tools/issues)
- **Documentation Feedback**: Create issue with `[docs]` tag
- **Code Contributions**: Follow [Code Review Checklist](docs/process/CODE_REVIEW_CHECKLIST.md)

---

## License

MIT

---

## Quick Commands Reference

```bash
# Development
cargo make pre-commit      # Format + lint + test (before every commit)
cargo make ci-local        # Simulate full CI pipeline

# Testing
cargo make test            # Unit tests (fast)
cargo make test-all        # Unit + integration
cargo make test-property   # Property-based tests
cargo make test-mutation   # Mutation testing
cargo make test-snapshot   # Snapshot tests

# Observability
cargo make weaver-bootstrap  # Setup Weaver (once)
cargo make weaver-smoke      # Verify Weaver works
cargo make test-integration  # Full integration tests

# Code Quality
cargo make lint            # Clippy checks (strict)
cargo make fmt             # Code formatting
cargo make coverage        # Test coverage report
cargo make docs            # Generate & open Rustdoc

# Build
cargo make check           # Compilation check
cargo make build           # Debug binary
cargo make build-release   # Optimized binary
```

---

**Next Step**: Follow the [Quick Start](#quick-start-choose-your-path) path that matches your need, or jump to [Learning Path](#-learning-path-start-here) for structured learning.

**Questions?** See [Troubleshooting](#-troubleshooting) or check [Getting Started](docs/getting-started/GETTING_STARTED.md).
