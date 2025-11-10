# Architecture

High-level architecture and design principles for Chicago TDD Tools. This document explains the design decisions, extension patterns, and architectural choices.

## Overview

Chicago TDD Tools is a generic testing framework designed for reusability across projects. It provides a base layer of testing utilities that can be extended for domain-specific needs while maintaining consistency and avoiding duplication.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    chicago-tdd-tools                        │
│                  (Generic Base Layer)                       │
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   Fixtures   │  │   Builders   │  │   Macros     │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │  Assertions  │  │  Property    │  │  Mutation    │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │ Performance  │  │   Guards     │  │    JTBD     │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│                                                             │
│  ┌──────────────┐  ┌──────────────┐                        │
│  │Testcontainers│  │ OTEL/Weaver  │                        │
│  └──────────────┘  └──────────────┘                        │
└─────────────────────────────────────────────────────────────┘
                          ▲
                          │ uses
                          │
┌─────────────────────────────────────────────────────────────┐
│              Domain-Specific Extensions                     │
│         (e.g., knhk-workflow-engine)                        │
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │ Workflow     │  │ Workflow     │  │ Domain-      │     │
│  │ Fixture      │  │ Builder      │  │ Specific     │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

## Design Principles

### 1. Generic Base Layer

The core framework provides generic, reusable components:

- **No domain dependencies**: Core components don't depend on specific domains
- **Extensible**: Components can be extended for domain-specific needs
- **Composable**: Components work together seamlessly

**Rationale**: This design allows the framework to be used across different projects without coupling to any specific domain.

### 2. Composition Over Duplication

Domain-specific extensions use the generic base:

- Extend `TestFixture` for domain-specific fixtures
- Use `TestDataBuilder` directly or extend it
- Compose generic components for domain needs

**Rationale**: Avoids code duplication while maintaining flexibility for domain-specific needs.

### 3. Single Source of Truth

Generic components exist in one place:

- `TestDataBuilder` is only in `chicago-tdd-tools`
- Domain projects import and use, don't duplicate
- Consistent APIs across projects

**Rationale**: Ensures consistency and reduces maintenance burden.

### 4. Zero-Cost Abstractions

Macros expand to efficient code:

- No runtime overhead for macro usage
- Compile-time validation where possible
- Efficient data structures

**Rationale**: Performance is critical for testing frameworks; abstractions shouldn't add overhead.

## Module Organization

### Core Modules

**`fixture`** - Test fixtures with automatic cleanup
- Provides `TestFixture<T>` with Generic Associated Types (GATs)
- Automatic cleanup via RAII
- Metadata support for test context

**`builders`** - Fluent builders for test data
- `TestDataBuilder` for creating test data
- Business-specific helpers (orders, customers, approvals)
- JSON and HashMap output formats

**`assertions`** - Assertion helpers
- Result assertions (`assert_success`, `assert_error`)
- Predicate assertions with HRTB (`assert_that`)
- Range assertions (`assert_in_range`)

**`macros`** - Test macros for AAA pattern enforcement
- `chicago_test!` for synchronous tests
- `chicago_async_test!` for async tests
- `chicago_fixture_test!` for automatic fixture setup
- `chicago_performance_test!` for performance testing
- Assertion macros (`assert_ok!`, `assert_err!`, etc.)

**`state`** - Type-level programming for test state
- Type state pattern for AAA enforcement
- Compile-time phase tracking (Arrange → Act → Assert)
- Prevents calling methods in wrong order

### Advanced Testing Modules

**`property`** - Property-based testing framework
- `PropertyTestGenerator` with const generics
- Compile-time configuration (MAX_ITEMS, MAX_DEPTH)
- Reproducible with seed support

**`mutation`** - Mutation testing framework
- `MutationTester` for test quality validation
- `MutationOperator` enum for mutation types
- `MutationScore` for quality metrics

**`coverage`** - Test coverage analysis
- `CoverageReport` for coverage tracking
- Markdown report generation
- Coverage percentage calculation

**`generator`** - Test code generation
- `TestGenerator` for generating test code
- Compile-time array generation
- Const validation helpers

### Performance & Constraints Modules

**`performance`** - RDTSC benchmarking and tick measurement
- `TickCounter` for cycle-accurate measurement
- `measure_ticks()` helper function
- `HOT_PATH_TICK_BUDGET` constant (8 ticks)
- Platform-specific fallback (x86_64 vs others)

**`guards`** - Guard constraint enforcement
- `GuardValidator` for input validation
- `MAX_RUN_LEN` constant (Chatman Constant: ≤8)
- `MAX_BATCH_SIZE` constant
- Constraint validation at ingress points

### Validation Modules

**`jtbd`** - Jobs To Be Done validation framework
- `JtbdValidator` for scenario validation
- `JtbdScenario` for test scenarios
- Validates code accomplishes intended purpose
- Real-world scenario testing

**`otel`** - OpenTelemetry span/metric validation (optional)
- `SpanValidator` for span validation
- `MetricValidator` for metric validation
- Schema conformance checking
- Required attributes validation

**`weaver`** - Weaver live validation integration (optional)
- `WeaverValidator` for live validation
- Registry path configuration
- OTLP and admin port configuration
- Automatic report generation

### Integration Modules

**`testcontainers`** - Docker container support (optional)
- `ContainerClient` for Docker operations
- `GenericContainer` for any Docker image
- Port mapping and command execution
- Automatic cleanup via Drop trait

## Module Dependency Graph

```
lib.rs
├── fixture (no deps)
├── builders (serde_json)
├── assertions (no deps)
├── macros (fixture)
├── property (no deps)
├── mutation (no deps)
├── coverage (no deps)
├── generator (no deps)
├── performance (no deps)
├── guards (no deps)
├── jtbd (no deps)
├── state (no deps)
├── testcontainers (testcontainers crate, optional)
├── otel (internal otel_types, optional)
└── weaver (otel, internal weaver_types, optional)
```

**Key Observations**:
- Most modules have no dependencies (zero-cost abstractions)
- Optional features are properly feature-gated
- Internal types (`otel_types`, `weaver_types`) avoid external dependencies

## Feature Flags

The framework uses feature flags for optional capabilities:

- **`default`**: Core testing framework (no optional features)
- **`property-testing`**: Property-based testing
- **`mutation-testing`**: Mutation testing
- **`testcontainers`**: Docker container support
- **`otel`**: OTEL validation
- **`weaver`**: Weaver live validation (requires `otel`)

**Design Decision**: Feature flags allow users to include only what they need, reducing compile time and binary size.

## Extension Patterns

### Extending TestFixture

```rust
use chicago_tdd_tools::fixture::TestFixture;

pub struct WorkflowTestFixture {
    base: TestFixture<()>,
    // Domain-specific fields
    engine: WorkflowEngine,
}

impl WorkflowTestFixture {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            base: TestFixture::new()?,
            engine: WorkflowEngine::new()?,
        })
    }
    
    // Delegate to base fixture
    pub fn test_counter(&self) -> u64 {
        self.base.test_counter()
    }
    
    // Domain-specific methods
    pub fn workflow_engine(&self) -> &WorkflowEngine {
        &self.engine
    }
}
```

**Pattern**: Compose base fixture with domain-specific fields, delegate common operations.

### Using TestDataBuilder

```rust
use chicago_tdd_tools::builders::TestDataBuilder;

// Use directly
let data = TestDataBuilder::new()
    .with_var("key", "value")
    .build_json();

// Or extend for domain-specific helpers
impl TestDataBuilder {
    pub fn with_workflow_data(self, workflow_id: &str) -> Self {
        self.with_var("workflow_id", workflow_id)
    }
}
```

**Pattern**: Use builder directly or extend with domain-specific helpers.

### Composing Components

```rust
use chicago_tdd_tools::prelude::*;

chicago_fixture_test!(test_composed, fixture, {
    // Use fixture
    let counter = fixture.test_counter();
    
    // Use builder
    let data = TestDataBuilder::new()
        .with_var("counter", counter.to_string())
        .build_json();
    
    // Use assertions
    assert_ok!(&process_data(data), "Should process data");
});
```

**Pattern**: Compose multiple components for complex test scenarios.

## Type Safety

The framework uses Rust's type system for safety:

### Generic Associated Types (GATs)

```rust
pub trait FixtureProvider {
    type Fixture<'a>: 'a where Self: 'a;
    type Error: std::error::Error + Send + Sync + 'static;
    
    fn create_fixture(&self) -> Result<Self::Fixture<'_>, Self::Error>;
}
```

**Benefit**: Flexible fixture creation with type-safe lifetime management.

### Const Generics

```rust
pub struct PropertyTestGenerator<const MAX_ITEMS: usize = 10, const MAX_DEPTH: usize = 3> {
    // ...
}
```

**Benefit**: Compile-time configuration with zero runtime overhead.

### Type State Pattern

```rust
pub struct TestState<Phase> {
    _phase: std::marker::PhantomData<Phase>,
    // ...
}

impl TestState<Arrange> {
    pub fn act(self) -> TestState<Act> { /* ... */ }
}

impl TestState<Act> {
    pub fn assert(self) -> TestState<Assert> { /* ... */ }
}
```

**Benefit**: Compile-time AAA pattern enforcement - impossible to call methods in wrong order.

### Higher-Ranked Trait Bounds (HRTB)

```rust
pub fn assert_that<T, F>(value: &T, predicate: F)
where
    T: std::fmt::Debug,
    F: for<'a> Fn(&'a T) -> bool,
{
    // ...
}
```

**Benefit**: Flexible predicate functions that work with any lifetime.

## Error Handling

All fallible operations return `Result<T, E>`:

- **No `unwrap()` in production code paths**: All errors are properly handled
- **Proper error types**: Use `thiserror` for error types
- **Context in error messages**: Errors include context for debugging

**Example**:
```rust
pub enum FixtureError {
    #[error("Failed to create fixture: {0}")]
    CreationFailed(String),
    #[error("Fixture operation failed: {0}")]
    OperationFailed(String),
}
```

## Performance Characteristics

### Zero-Cost Abstractions

- **Macros**: Expand to efficient code, no runtime overhead
- **Const generics**: Compile-time configuration, zero runtime cost
- **Type state**: Zero-sized types, no runtime overhead

### Efficient Data Structures

- **Fixtures**: Use atomic counters for isolation (~1-2ns overhead)
- **Builders**: Use `HashMap<String, String>` internally, convert to JSON only when needed
- **Property testing**: Efficient LCG for random generation

### Hot Path Optimization

- **RDTSC**: Cycle-accurate measurement on x86_64
- **Tick budget**: Enforced at compile time where possible
- **Minimal allocations**: Reuse data structures where possible

## Security Considerations

### No Unsafe Code

- Core framework uses no `unsafe` code (except RDTSC on x86_64, which is platform-specific)
- All operations are memory-safe
- Proper error handling prevents panics

### Input Validation

- All public APIs validate inputs
- Guard constraints prevent invalid data
- Error messages don't leak sensitive information

### Resource Cleanup

- RAII patterns ensure cleanup
- Drop traits handle resource cleanup
- No resource leaks in error paths

## Chicago TDD Alignment

The framework enforces Chicago TDD principles:

1. **State-Based Testing**: Tests verify outputs and state, not implementation
2. **Real Collaborators**: Uses actual dependencies (e.g., Docker containers via testcontainers)
3. **Behavior Verification**: Tests verify what code does, not how
4. **AAA Pattern**: All tests follow Arrange-Act-Assert structure (enforced by macros and type state)

## Procedural Macros

The framework provides procedural macros in `proc_macros/`:

- **`#[chicago_test]`**: Zero-boilerplate tests with AAA validation
- **`#[chicago_fixture]`**: Automatic fixture setup/teardown
- **`#[derive(TestBuilder)]`**: Derive macro for fluent builders

**Design Decision**: Procedural macros provide compile-time validation and reduce boilerplate.

## Thread Safety

- **Fixtures**: Use atomic counters, thread-safe
- **Builders**: Not thread-safe (single-threaded use in tests)
- **Property generators**: Not thread-safe (single-threaded use)
- **Performance counters**: Thread-safe (atomic operations)

**Note**: Test execution is typically single-threaded, so thread safety is less critical than in production code.

## Platform Support

- **x86_64**: Full support including RDTSC
- **ARM/Other**: RDTSC falls back to `std::time::Instant`
- **All platforms**: Core functionality works on all platforms

## Summary

Chicago TDD Tools provides a generic, extensible base for testing Rust applications. It follows Chicago TDD principles, uses Rust's type system for safety, and provides optional features for advanced testing needs. Domain-specific projects can extend the base components while maintaining consistency and avoiding duplication.

**Key Strengths**:
- Generic and reusable
- Type-safe with zero-cost abstractions
- Extensible for domain-specific needs
- Performance-optimized
- Security-conscious
- Chicago TDD aligned

**Design Philosophy**: Provide 80% of value with 20% of complexity, allowing users to extend for their specific needs.
