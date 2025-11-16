# Getting Started

Get up and running with Chicago TDD Tools in 5 minutes. Verified, runnable examples.

**New to the framework?** Start here → [Installation](#installation) → [Your First Test](#your-first-test) → [Common Patterns](#common-patterns)

**Looking for specific information?**
- **Quick reference**: [Quick Guide](QUICK_GUIDE.md) - Essential patterns (80% of use cases)
- **Complete guide**: [User Guide](USER_GUIDE.md) - Comprehensive coverage
- **API details**: [API Reference](../reference/API_REFERENCE.md) - Complete API documentation
- **Architecture**: [Architecture](../reference/ARCHITECTURE.md) - Design principles

## Prerequisites

**Rust**: Edition 2021 (Rust 1.70+). **Rust 1.75+**: Required for `async` feature (async fixture providers). **Cargo**: Latest stable. **Tokio**: Required for async tests. **Docker**: Optional, for `testcontainers` feature.

**Verify**: `rustc --version` (1.70+, 1.75+ for async feature), `cargo --version` (latest stable).

## Installation

**Step 1**: Install `cargo-make` (required for build system):
```bash
cargo install cargo-make
```

**Step 2**: Add dependency to `Cargo.toml`:

**Complete `Cargo.toml` example**:
```toml
[package]
name = "my-project"
version = "0.1.0"
edition = "2021"  # Required: Edition 2021

[dev-dependencies]
chicago-tdd-tools = { path = "../chicago-tdd-tools" }  # Or use git URL when published
tokio = { version = "1.0", features = ["rt", "macros"] }
```

**Note**: For GitHub users, use `chicago-tdd-tools = "1.2.0"` when published to crates.io.

**Step 3**: Verify installation: `cargo make check`. If errors: verify path, Rust edition 2021, Tokio in dev-dependencies.

## Your First Test

**Create** `tests/my_first_test.rs`:
```rust
use chicago_tdd_tools::prelude::*;

test!(test_basic_example, {
    // Arrange: Set up test data
    let input = 5;
    // Act: Execute feature
    let result = input * 2;
    // Assert: Verify behavior
    assert_eq!(result, 10);
});
```

**Run**: `cargo make test test_basic_example`. **Verify**: Test passes. AAA pattern: Arrange (input), Act (result), Assert (verify).

## Common Patterns

### Synchronous Test
```rust
use chicago_tdd_tools::prelude::*;

test!(test_sync, {
    // Arrange
    let input = 5;
    // Act
    let result = input * 2;
    // Assert
    assert_eq!(result, 10);
});
```

### Async Test

**Requires**: Tokio runtime (included in dev-dependencies). No feature flags needed.

**File**: `tests/my_async_test.rs`:
```rust
use chicago_tdd_tools::prelude::*;

async_test!(test_async, {
    // Arrange
    let expected = 10;
    // Act
    let result = async { 5 * 2 }.await;
    // Assert
    assert_eq!(result, expected);
});
```

### Async Test with Custom Timeout
```rust
use chicago_tdd_tools::prelude::*;

async_test_with_timeout!(test_async_integration, 30, {
    // Arrange
    let expected = 10;
    // Act: Slow async operation
    let result = async { 5 * 2 }.await;
    // Assert
    assert_eq!(result, expected);
});
```

### Fixture Test

**Requires**: Tokio runtime (included in dev-dependencies). No feature flags needed.

**File**: `tests/my_fixture_test.rs`:
```rust
use chicago_tdd_tools::prelude::*;

fixture_test!(test_with_fixture, fixture, {
    // Arrange: Fixture automatically created
    let counter = fixture.test_counter();
    // Act
    let result = counter + 1;
    // Assert
    assert!(result > 0);
});
```

### Fixture Test with Custom Timeout
```rust
use chicago_tdd_tools::prelude::*;

fixture_test_with_timeout!(test_fixture_integration, fixture, 30, {
    // Arrange
    let counter = fixture.test_counter();
    // Act
    let result = counter + 1;
    // Assert
    assert!(result > 0);
});
```

### Performance Test

**Requires**: No feature flags. RDTSC on x86_64, falls back to `std::time::Instant` on other platforms.

**File**: `tests/my_performance_test.rs`:
```rust
use chicago_tdd_tools::prelude::*;

performance_test!(test_hot_path, {
    // Arrange
    let input = 5;
    // Act: Measure ticks
    let (result, ticks) = measure_ticks(|| input * 2);
    // Assert
    assert_within_tick_budget!(ticks);
    assert_eq!(result, 10);
});
```

### Parameterized Test

**Requires**: `parameterized-testing` feature flag.

**Enable in `Cargo.toml`**:
```toml
[dev-dependencies]
chicago-tdd-tools = { 
    path = "../chicago-tdd-tools",
    features = ["parameterized-testing"]
}
```

**File**: `tests/my_param_test.rs`:
```rust
use chicago_tdd_tools::prelude::*;

param_test! {
    #[case(1, 2, 3)]
    #[case(2, 3, 5)]
    #[case(3, 4, 7)]
    fn test_addition(a: i32, b: i32, expected: i32) {
        assert_eq!(a + b, expected);
    }
}
```

### Data Builder
```rust
use chicago_tdd_tools::prelude::*;

test!(test_data_builder, {
    // Arrange: Build test data
    let data = TestDataBuilder::new()
        .with_var("key", "value")
        .build_json();
    // Act & Assert
    assert!(data.contains("key"));
});
```

### Result Assertions
```rust
use chicago_tdd_tools::prelude::*;

test!(test_result_assertions, {
    // Arrange
    let ok_result: Result<i32, String> = Ok(42);
    let err_result: Result<i32, String> = Err("error".to_string());
    // Act & Assert
    assert_ok!(&ok_result);
    assert_err!(&err_result);
});
```

### Enhanced Assertions
```rust
use chicago_tdd_tools::prelude::*;

test!(test_enhanced_assertions, {
    // Arrange
    let value = 5;
    // Act & Assert
    assert_in_range!(value, 0, 10);
    assert_eq_msg!(value, 5, "Value should be 5");
    assert_eq_enhanced!(value, 5);
});
```

### Alert Macros
```rust
use chicago_tdd_tools::prelude::*;

test!(test_alerts, {
    // Arrange & Act: Emit alerts
    alert_info!("Test started");
    alert_success!("Test completed");
    // Assert: Alerts emitted (verify visually)
});
```

## Optional Features

### Individual Features

Enable individual features as needed:
- **`property-testing`**: Property-based testing with proptest (random test generation)
- **`snapshot-testing`**: Snapshot testing with insta (output comparison)
- **`fake-data`**: Fake data generation for realistic test data
- **`mutation-testing`**: Mutation testing for test quality validation
- **`concurrency-testing`**: Concurrency testing with loom (thread model checking)
- **`parameterized-testing`**: Parameterized tests with rstest
- **`cli-testing`**: CLI testing with trycmd (command execution testing)
- **`testcontainers`**: Docker container support (requires Docker running)
- **`otel`**: OpenTelemetry span/metric validation
- **`weaver`**: Weaver live validation (requires `otel`, auto-downloads Weaver CLI)
- **`async`**: Async fixture providers (async traits, Rust 1.75+)
- **`benchmarking`**: Criterion benchmarking support
- **`workflow-engine`**: Workflow-specific features
- **`logging`**: Standard log crate integration (enabled by default)

### Weaver Live-Check Setup (80/20 Quick Path)

1. **Bootstrap Weaver CLI + registry**
   ```bash
   cargo make weaver-bootstrap
   ```
   Downloads the Weaver binary to `target/<profile>/weaver` and clones the semantic convention registry into `registry/`.
2. **Run smoke validation**
   ```bash
   cargo make weaver-smoke
   ```
   Verifies `weaver --version` and sends a telemetry span via the library, ensuring live-check works without Docker.
3. **Run full integration (optional)**
   ```bash
   cargo make test-integration         # Requires Docker + weaver feature
   ```
   Executes the container-based Weaver suite. Tests fail fast if prerequisites are missing unless `WEAVER_ALLOW_SKIP=1` is set.

> **Skip intentionally?** Export `WEAVER_ALLOW_SKIP=1` to opt out temporarily. Without this flag, missing prerequisites panic to preserve dogfooding quality.

### Feature Groups (Recommended)

For better DX, use feature groups for common combinations:

**`testing-extras`**: Most common advanced testing features
```toml
[dev-dependencies]
chicago-tdd-tools = { 
    path = "../chicago-tdd-tools",
    features = ["testing-extras"]  # Enables property-testing, snapshot-testing, fake-data
}
```

**`testing-full`**: All testing features
```toml
[dev-dependencies]
chicago-tdd-tools = { 
    path = "../chicago-tdd-tools",
    features = ["testing-full"]  # All testing features
}
```

**`observability-full`**: Complete observability stack
```toml
[dev-dependencies]
chicago-tdd-tools = { 
    path = "../chicago-tdd-tools",
    features = ["observability-full"]  # otel + weaver
}
```

**`integration-full`**: Full integration testing
```toml
[dev-dependencies]
chicago-tdd-tools = { 
    path = "../chicago-tdd-tools",
    features = ["integration-full"]  # testcontainers + weaver
}
```

**See**: [README Features](../README.md#features) for complete feature documentation.

## Procedural Macros

### `#[tdd_test]` Attribute
```rust
use chicago_tdd_tools::tdd_test;

#[tdd_test]
fn my_test() {
    // Arrange
    let input = 5;
    // Act
    let result = input * 2;
    // Assert
    assert_eq!(result, 10);
}
```

### `#[fixture]` Attribute
```rust
use chicago_tdd_tools::fixture;

#[fixture]
fn my_test() {
    // Arrange: Fixture automatically created as `fixture`
    let counter = fixture.test_counter();
    // Act
    let result = counter + 1;
    // Assert
    assert!(result > 0);
}
```

### `#[derive(TestBuilder)]`
```rust
use chicago_tdd_tools::TestBuilder;

#[derive(TestBuilder)]
struct MyStruct {
    field1: String,
    field2: i32,
}

test!(test_builder, {
    // Arrange: Use generated builder
    let instance = MyStructBuilder::new()
        .with_field1("value".to_string())
        .with_field2(42)
        .build()
        .unwrap();
    // Act & Assert
    assert_eq!(instance.field1, "value");
    assert_eq!(instance.field2, 42);
});
```

## Async Fixtures (requires `async` feature, Rust 1.75+)

```rust
#[cfg(feature = "async")]
use chicago_tdd_tools::core::async_fixture::{AsyncFixtureManager, AsyncFixtureProvider};
use chicago_tdd_tools::core::fixture::FixtureError;

#[cfg(feature = "async")]
struct DatabaseFixture {
    connection: String,
}

#[cfg(feature = "async")]
struct DatabaseProvider;

#[cfg(feature = "async")]
impl chicago_tdd_tools::core::async_fixture::private::Sealed for DatabaseProvider {}

#[cfg(feature = "async")]
impl AsyncFixtureProvider for DatabaseProvider {
    type Fixture<'a> = DatabaseFixture;
    type Error = FixtureError;

    async fn create_fixture(&self) -> Result<Self::Fixture<'_>, Self::Error> {
        Ok(DatabaseFixture { connection: "connected".to_string() })
    }
}

#[cfg(feature = "async")]
async_test!(test_async_fixture, {
    // Arrange: Create async fixture manager
    let provider = DatabaseProvider;
    let manager = AsyncFixtureManager::new(provider);
    // Act: Setup async fixture
    let fixture = manager.setup().await?;
    // Assert: Verify fixture created
    assert_eq!(fixture.connection, "connected");
    Ok::<(), FixtureError>(())
});
```

## Verify Installation

**Run**: `cargo make test`. Runs all unit tests, integration tests, examples.

## Next Steps

- **[Quick Guide](QUICK_GUIDE.md)** - Essential patterns (80% of use cases)
- **[User Guide](USER_GUIDE.md)** - Comprehensive guide with all features
- **[API Reference](../reference/API_REFERENCE.md)** - Complete API documentation
- **[Architecture](../reference/ARCHITECTURE.md)** - Design principles and patterns
- **[Pattern Cookbook](../../cookbook/src/README.md)** - Alexander-style patterns for testing, architecture, and design
- **[Examples](../examples/)** - Working code examples (`cargo make test-examples`)

## Troubleshooting

### Common Compilation Errors

**Error**: `cannot find crate 'chicago_tdd_tools'`
- **Fix**: Verify path in `Cargo.toml` is correct
- **Check**: `[dev-dependencies]` section includes `chicago-tdd-tools = { path = "..." }`

**Error**: `edition 2021 required`
- **Fix**: Add `edition = "2021"` to `Cargo.toml`:
  ```toml
  [package]
  edition = "2021"
  ```

**Error**: `cannot find macro 'test!' in this scope`
- **Fix**: Add `use chicago_tdd_tools::prelude::*;` at top of test file
- **Alternative**: Use explicit import: `use chicago_tdd_tools::test;`

**Error**: `cannot find module 'observability'` or `cannot find module 'integration'`
- **Fix**: Enable required feature in `Cargo.toml`:
  ```toml
  chicago-tdd-tools = { features = ["otel", "weaver", "testcontainers"] }
  ```
- **Note**: Feature-gated modules require explicit feature flags

**Error**: `feature 'X' is required for module Y`
- **Fix**: Add feature to `Cargo.toml`:
  ```toml
  chicago-tdd-tools = { features = ["feature-name"] }
  ```

**Error**: `parameterized testing requires the 'parameterized-testing' feature`
- **Fix**: Enable `parameterized-testing` feature:
  ```toml
  chicago-tdd-tools = { features = ["parameterized-testing"] }
  ```

**Error**: `OTEL testing requires the 'otel' feature`
- **Fix**: Enable `otel` feature:
  ```toml
  chicago-tdd-tools = { features = ["otel"] }
  ```

**Error**: `Weaver testing requires the 'weaver' feature`
- **Fix**: Enable `weaver` feature (automatically enables `otel`):
  ```toml
  chicago-tdd-tools = { features = ["weaver"] }
  ```

### Weaver Testing Notes

- Integration checks require Docker plus `cargo make test-integration`.
- Set `WEAVER_ALLOW_SKIP=1` to opt out of Weaver tests temporarily (CI should leave it unset).
- Always re-run `cargo make weaver-smoke` after upgrading Weaver or OpenTelemetry dependencies.

### Common Runtime Errors

**Error**: `TestFixture::new()` panics
- **Fix**: Use `fixture_test!` macro for async tests, or ensure tokio runtime
- **Note**: `TestFixture::new()` returns `Result` - handle errors properly

**Error**: Property tests don't compile
- **Fix**: Enable `property-testing` feature in `Cargo.toml`

**Error**: Testcontainers tests fail
- **Fix**: Ensure Docker is running
- **Check**: Run `docker ps` to verify Docker is available

**Error**: Performance tests fail on non-x86_64
- **Fix**: RDTSC is x86_64-specific, automatically falls back to `std::time::Instant`
- **Note**: Tick budget still applies, fallback is transparent

**Error**: Async fixture tests fail
- **Fix**: Ensure Rust 1.75+ and `async` feature enabled
- **Check**: `rustc --version` should be 1.75+, `Cargo.toml` includes `features = ["async"]`

## Platform-Specific

**Linux**: Works out of box. RDTSC on x86_64. Docker for testcontainers. **macOS**: Works out of box. RDTSC on x86_64 (Intel). ARM uses `std::time::Instant` fallback. Docker Desktop for testcontainers. **Windows**: Works out of box. RDTSC on x86_64. Docker Desktop for testcontainers.

## Hyper-Advanced μ-Kernel Verification (v1.3+)

Chicago TDD Tools v1.3+ includes a **hyper-advanced μ-kernel verification substrate** that transforms the framework into a canonical verification substrate for **A = μ(O)** (Artifacts equal micro-operator of Observations).

### Quick Start: Hyper-Advanced Features

**All hyper-advanced features are core (no feature flags required).**

#### Track 1: Test Contracts as First-Class Types

Define compile-time test contracts with coverage analysis:

```rust
use chicago_tdd_tools::prelude::*;

// Define contract at compile time
const CONTRACT: TestContract = TestContract::hot_path(
    "test_critical_path",
    &["payments::process", "payments::validate"],
);

// Registry provides gap analysis
let registry = TestContractRegistry::new(&[CONTRACT]);
let uncovered = registry.uncovered_modules(&["payments::process", "payments::refund"]);
assert_eq!(uncovered, vec!["payments::refund"]);
```

#### Track 2: τ-Aware Thermal Testing

Enforce Chatman Constant (τ ≤ 8 ticks) for hot paths:

```rust
use chicago_tdd_tools::prelude::*;

// Hot Path: Critical operations must complete within τ ≤ 8 ticks
// For test environment, use relaxed config to account for measurement overhead
let relaxed_config = HotPathConfig {
    max_ticks: 1000,
    enforce_no_alloc: false,
    enforce_no_syscall: false,
};
let hot_test = HotPathTest::new(relaxed_config);

let result = hot_test.run(|| {
    // Critical business logic
    let mut sum = 0u64;
    for i in 1..=10 {
        sum += i;
    }
    sum
});

match result {
    Ok((value, ticks)) => {
        println!("Hot path succeeded: value={}, ticks={}", value, ticks);
        // In production, this would enforce τ ≤ 8
        assert!(ticks <= 1000);
    }
    Err(e) => panic!("Hot path failed: {:?}", e),
}
```

**Production Note**: Use `HotPathTest::default()` for strict τ ≤ 8 enforcement in production code.

#### Track 3: Effect-Typed Tests

Type-level effect constraints (network, storage, privileged, pure):

```rust
use chicago_tdd_tools::prelude::*;

// Pure function: no effects
let _pure_effects: Effects<Pure> = Effects::new();

// Network effects allowed
let _network_effects: Effects<NetworkRead> = Effects::new();

// Storage effects allowed
let _storage_effects: Effects<StorageWrite> = Effects::new();
```

#### Track 4: Type-Directed State Machine Testing

Compile-time valid transition enforcement:

```rust
use chicago_tdd_tools::prelude::*;

// Define states
struct Disconnected;
struct Connected;
struct Authenticated;

impl State for Disconnected {}
impl State for Connected {}
impl State for Authenticated {}

// Define valid transitions
struct Connect;
struct Authenticate;

impl Transition<Disconnected, Connected> for Connect {
    fn execute() -> Result<(), String> {
        Ok(())
    }
}

impl Transition<Connected, Authenticated> for Authenticate {
    fn execute() -> Result<(), String> {
        Ok(())
    }
}

// State machine enforces valid transitions at compile time
let sm: StateMachine<Disconnected> = StateMachine::new();
let sm = sm.transition::<Connected, Connect>().unwrap();
let sm = sm.transition::<Authenticated, Authenticate>().unwrap();
// let sm = sm.transition::<InvalidState>(); // Compile error!
```

#### Track 5: Proof-Carrying Test Receipts

Cryptographic provenance for test execution:

```rust
use chicago_tdd_tools::prelude::*;

// Create test receipt from contract
const CONTRACT: TestContract = TestContract::hot_path(
    "test_payment_processing",
    &["payments::process"],
);

let timing = TimingMeasurement::new(6, 1, "hot".to_string(), true, 8);
let mut receipt = TestReceipt::from_contract(&CONTRACT, timing, TestOutcome::Pass);

// Sign receipt for cryptographic provenance
receipt.sign();
assert!(receipt.signature.is_some());

// Add metadata for governance
receipt.add_metadata("deploy_env", "production");
receipt.add_metadata("ticket_id", "JIRA-1234");

// Store in registry for governance queries
let mut registry = TestReceiptRegistry::new();
registry.add_receipt(receipt);

// Governance queries
let tau_violations = registry.tau_violations();
let failed = registry.failed_receipts();

if tau_violations.is_empty() && failed.is_empty() {
    println!("✓ Deployment APPROVED: All constraints satisfied");
}
```

#### Track 6: Swarm-Native Test Orchestrator

Agent-driven test scheduling with priority/QoS:

```rust
use chicago_tdd_tools::prelude::*;

// Define contracts for orchestration
const CONTRACTS: &[TestContract] = &[
    TestContract::hot_path("test_critical_path", &["core::critical"]),
    TestContract::warm_path("test_business_logic", &["core::business"], &["no_panics"]),
];

let registry = TestContractRegistry::new(CONTRACTS);
let mut orchestrator = TestOrchestrator::new(registry.clone());

// Submit test plans with QoS
let plan = TestPlan {
    plan_id: "plan-1".to_string(),
    contracts: vec!["test_critical_path".to_string()],
    requester: "agent-1".to_string(),
    priority: 100,
    qos: QoSClass::Premium,
    resource_budget: ResourceBudget {
        max_cores: 1,
        max_memory_bytes: 1024 * 1024,
        max_wall_clock_seconds: 1,
        allow_network: false,
        allow_storage: false,
    },
    metadata: std::collections::HashMap::new(),
};

orchestrator.submit_plan(plan);

// Agent suggests tests for code changes
let suggested = orchestrator.suggest_tests_for_change(&["core::critical"]);
assert_eq!(suggested.len(), 1);
```

### Complete Hyper-Advanced Workflow

End-to-end workflow: Contract → Test → Receipt → Orchestration → Governance:

```rust
use chicago_tdd_tools::prelude::*;

// 1. Define test contract
const CONTRACT: TestContract = TestContract::hot_path(
    "test_production_workflow",
    &["workflow::checkout", "workflow::payment"],
);

// 2. Execute τ-aware test
let relaxed_config = HotPathConfig {
    max_ticks: 1000,
    enforce_no_alloc: false,
    enforce_no_syscall: false,
};
let hot_test = HotPathTest::new(relaxed_config);

let (value, ticks) = hot_test.run(|| 42).expect("Test should succeed");

// 3. Create proof-carrying receipt
let timing = TimingMeasurement::new(ticks, 1, "hot".to_string(), true, 8);
let mut receipt = TestReceipt::from_contract(&CONTRACT, timing, TestOutcome::Pass);
receipt.sign();
receipt.add_metadata("workflow", "checkout");

// 4. Store receipt for governance
let mut receipt_registry = TestReceiptRegistry::new();
receipt_registry.add_receipt(receipt);

// 5. Orchestrator suggests tests for changes
let contract_registry = TestContractRegistry::new(&[CONTRACT]);
let orchestrator = TestOrchestrator::new(contract_registry);
let suggested = orchestrator.suggest_tests_for_change(&["workflow::checkout"]);

// 6. Governance decision
let tau_violations = receipt_registry.tau_violations();
let all_passed = receipt_registry.failed_receipts().is_empty();

if tau_violations.is_empty() && all_passed {
    println!("✓ DEPLOYMENT APPROVED");
    println!("  - τ constraints satisfied");
    println!("  - All tests passed");
    println!("  - Cryptographic proof provided");
}
```

### Theory: A = μ(O)

**Canonical Equation:** A = μ(O)
- **A**: Artifacts (test receipts, contracts, proofs)
- **μ**: Micro-operator (μ-kernel verification substrate)
- **O**: Observations (timing, effects, state transitions)

The μ-kernel substrate transforms raw observations into cryptographically-signed artifacts that provide:
1. **Compile-time correctness**: Types prevent invalid states
2. **Runtime verification**: τ constraints enforce timing discipline
3. **Cryptographic provenance**: Receipts provide audit trail
4. **Agent-driven optimization**: Orchestrator suggests minimal test sets

### Next Steps: Hyper-Advanced

- **[Hyper-Advanced Guide](../features/HYPER_ADVANCED_MICROKERNEL.md)** - Complete documentation for all 6 tracks
- **[Hyper-Advanced Example](../../examples/hyper_advanced_microkernel.rs)** - Runnable demonstration
- **[Integration Tests](../../tests/hyper_advanced_integration.rs)** - Comprehensive test coverage

## Summary

**Key Associations**: Installation = Dependency + Verify. First Test = AAA Pattern = Success. Common Patterns = Macros + Builders + Assertions. Optional Features = Feature Flags = Docker. **Hyper-Advanced** = μ-Kernel Verification = A = μ(O).

**Pattern**: Add dependency → Create test → Run test → Verify success → Explore features → Adopt hyper-advanced μ-kernel substrate. All tests follow AAA pattern. Use macros for all tests. Verify observable outputs. Hyper-advanced features provide compile-time contracts, τ-aware testing, effect typing, state machines, receipts, and orchestration.
