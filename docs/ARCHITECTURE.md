# Architecture - SPR

## Overview

Chicago TDD Tools: Generic testing framework base layer. Extensible for domain-specific needs. Reusable across projects. Maintains consistency, avoids duplication.

## Architecture

**Base Layer** (chicago-tdd-tools): Generic components (Fixtures, Builders, Macros, Assertions, Property, Mutation, Performance, Guards, JTBD, Testcontainers, OTEL/Weaver).

**Extension Layer** (domain-specific): Extends base components (Workflow Fixture, Workflow Builder, Domain-Specific).

**Relationship**: Extensions use base layer. Base layer has no domain dependencies.

## Design Principles

**Generic Base**: No domain dependencies. Extensible. Composable. Enables reuse across projects.

**Composition Over Duplication**: Extend TestFixture. Use or extend TestDataBuilder. Compose generic components. Avoids duplication, maintains flexibility.

**Single Source of Truth**: Generic components in one place. Domain projects import, don't duplicate. Ensures consistency, reduces maintenance.

**Zero-Cost Abstractions**: Macros expand efficiently. Compile-time validation. Efficient data structures. Performance critical, no overhead.

## Module Organization

Modules are organized into capability groups for better discoverability and maintainability:

**Core Testing Infrastructure** (`core/`): Foundational primitives that all tests use.
- `fixture`: Test fixtures (GATs, RAII, automatic cleanup)
- `builders`: Fluent builders for test data (JSON/HashMap)
- `assertions`: Assertion helpers (Result, predicate, range)
- `macros`: Test macros (AAA pattern, async, fixture, performance)
- `state`: Type-level AAA enforcement
- `const_assert`: Compile-time assertions

**Advanced Testing Techniques** (`testing/`): Specialized testing methodologies.
- `property`: Property-based testing (const generics, reproducible)
- `mutation`: Mutation testing (quality validation, operators, scores)
- `snapshot`: Snapshot testing (requires `snapshot-testing` feature)
- `concurrency`: Concurrency testing (requires `concurrency-testing` feature)
- `cli`: CLI testing (requires `cli-testing` feature)
- `generator`: Test code generation

**Quality & Validation** (`validation/`): Quality assurance and constraint validation.
- `coverage`: Test coverage analysis (tracking, reports)
- `guards`: Guard constraints (input validation, Chatman Constant ≤8, batch size limits)
- `jtbd`: Jobs To Be Done validation (scenario validation, real-world testing)
- `performance`: Performance validation (RDTSC, tick measurement, hot path budget)

**Telemetry & Observability** (`observability/`): Telemetry validation.
- `otel`: OTEL validation (span/metric validation, schema conformance)
- `weaver`: Weaver live validation (live validation, registry, OTLP)

**Integration Testing** (`integration/`): External system integration.
- `testcontainers`: Docker support (port mapping, exec, auto-cleanup)

**Backward Compatibility**: All modules are re-exported at the crate root. Existing code using `chicago_tdd_tools::fixture::*` continues to work. New code is encouraged to use capability group paths: `chicago_tdd_tools::core::fixture::*`

## Module Dependencies

Most modules have no dependencies (zero-cost). Optional features are feature-gated. Internal types avoid external dependencies.

**Dependency Graph**: lib.rs → core (fixture, builders, assertions, macros, state, const_assert), testing (property, mutation, snapshot, concurrency, cli, generator), validation (coverage, guards, jtbd, performance), observability (otel, weaver), integration (testcontainers). Most modules have no dependencies (zero-cost). Optional features are feature-gated. Internal types avoid external dependencies.

## Feature Flags

**default**: Core framework (no optional features). **property-testing**: Property-based testing. **mutation-testing**: Mutation testing. **testcontainers**: Docker support. **otel**: OTEL validation. **weaver**: Weaver live validation (requires otel).

**Rationale**: Users include only what they need. Reduces compile time, binary size.

## Extension Patterns

**Extend TestFixture**: Compose base fixture with domain fields. Delegate common operations. Pattern: `WorkflowTestFixture { base: TestFixture<()>, engine: WorkflowEngine }`.

**Use TestDataBuilder**: Use directly or extend with domain helpers. Pattern: `TestDataBuilder::new().with_var().build_json()` or extend with `with_workflow_data()`.

**Compose Components**: Use fixture + builder + assertions together. Pattern: `chicago_fixture_test!` with `TestDataBuilder` and `assert_ok!`.

## Type Safety

**GATs**: Flexible fixture creation with type-safe lifetimes. Pattern: `trait FixtureProvider { type Fixture<'a>; }`.

**Const Generics**: Compile-time configuration, zero runtime cost. Pattern: `PropertyTestGenerator<const MAX_ITEMS: usize>`.

**Type State**: Compile-time AAA enforcement. Pattern: `TestState<Phase>` with `PhantomData<Phase>`. Prevents wrong method order.

**HRTB**: Flexible predicates with any lifetime. Pattern: `F: for<'a> Fn(&'a T) -> bool`.

## Error Handling

All fallible operations return `Result<T, E>`. No `unwrap()` in production. Use `thiserror` for error types. Errors include context.

**Pattern**: `pub enum FixtureError { CreationFailed(String), OperationFailed(String) }`.

## Performance

**Zero-Cost**: Macros expand efficiently. Const generics = compile-time. Type state = zero-sized types.

**Efficient**: Atomic counters for fixtures (~1-2ns). Builders use HashMap, convert to JSON when needed. LCG for property testing.

**Hot Path**: RDTSC on x86_64. Tick budget enforced at compile time. Minimal allocations, reuse structures.

## Security

**No Unsafe**: Core framework has no unsafe (except RDTSC on x86_64, platform-specific). All operations memory-safe. Proper error handling prevents panics.

**Input Validation**: Public APIs validate inputs. Guard constraints prevent invalid data. Error messages don't leak sensitive info.

**Resource Cleanup**: RAII patterns. Drop traits handle cleanup. No leaks in error paths.

## Chicago TDD Alignment

State-based testing (verify outputs, not implementation). Real collaborators (actual dependencies, e.g., Docker). Behavior verification (verify what code does, not how). AAA pattern (enforced by macros and type state).

## Procedural Macros

**`#[chicago_test]`**: Zero-boilerplate tests with AAA validation. **`#[chicago_fixture]`**: Automatic fixture setup/teardown. **`#[derive(TestBuilder)]`**: Derive macro for fluent builders.

**Rationale**: Compile-time validation, reduce boilerplate.

## Thread Safety

Fixtures: Atomic counters, thread-safe. Builders: Not thread-safe (single-threaded tests). Property generators: Not thread-safe (single-threaded). Performance counters: Thread-safe (atomic operations).

**Note**: Test execution typically single-threaded, so thread safety less critical than production.

## Platform Support

x86_64: Full support including RDTSC. ARM/Other: RDTSC falls back to `std::time::Instant`. All platforms: Core functionality works.

## Summary

Generic, extensible testing framework base. Chicago TDD principles. Type-safe with zero-cost abstractions. Extensible for domain needs. Performance-optimized. Security-conscious.

**Key Associations**: Generic = Reusable = Extensible. Type-safe = Zero-cost = Performance. Composition = Flexibility = Consistency. Base layer = 80% value = Domain extensions = 20% effort.

**Design Philosophy**: 80% value with 20% complexity. Users extend for specific needs.
