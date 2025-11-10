# User Guide - SPR

Complete guide to using Chicago TDD Tools, organized by user journey (beginner to advanced).

## Test Fixtures

Test fixtures: Reusable setup with automatic cleanup. RAII patterns. Isolated test state.

**When to Use**: Isolated test state, automatic cleanup, test metadata, integration tests. **Avoid**: Simple unit tests, no state needed, one-off data.

**Basic Usage**: `TestFixture::new()?` for basic fixture, `TestFixture::with_data(data)` for custom data, `fixture.test_counter() -> u64` for unique counter, `fixture.set_metadata(key, value)` / `fixture.get_metadata(key)` for metadata.

**Automatic Setup**: `chicago_fixture_test!(name, fixture, { /* AAA */ })` for auto setup/teardown. Prefer over manual creation.

**Patterns**: Shared setup (auto fixture), metadata tracking (test identification), isolation (unique counters).

**Anti-patterns**: Don't create fixtures unnecessarily for simple tests. Don't use fixtures when no state needed.

## Test Data Builders

Fluent builders for test data. JSON/HashMap output. Domain-specific helpers.

**When to Use**: Complex test data, many fields, fluent readable creation, reusable patterns. **Avoid**: Simple data (use literals), one-off structures, no JSON needed.

**Basic Builder**: `TestDataBuilder::new().with_var(key, value).build_json()` or `.build()` for HashMap.

**Business Helpers**: `with_order_data(order_id, amount)`, `with_customer_data(customer_id)`, `with_approval_data(request_id, amount)`. Create domain-specific extensions.

**Patterns**: Reusable test data (helper functions), JSON for APIs, HashMap for internal use.

**Anti-patterns**: Don't use builders for simple literals. Don't convert to JSON unnecessarily.

## Macros

Test macros: AAA pattern enforcement. Zero-boilerplate tests. Assertion macros.

**When to Use**: All tests (enforces AAA), async tests, fixture tests, performance tests. **Always use**: Reduces boilerplate, enforces patterns.

**Test Macros**: `chicago_test!(name, { /* AAA */ })` (synchronous), `chicago_async_test!(name, { /* AAA */ })` (async), `chicago_fixture_test!(name, fixture, { /* AAA */ })` (auto fixture), `chicago_performance_test!(name, { /* AAA */ })` (tick budget).

**Assertion Macros**: `assert_ok!(result)` / `assert_ok!(result, message)`, `assert_err!(result)` / `assert_err!(result, message)`, `assert_within_tick_budget!(ticks)` / `assert_within_tick_budget!(ticks, message)`, `assert_in_range!(value, min, max)` / `assert_in_range!(value, min, max, message)`, `assert_eq_msg!(actual, expected, message)`, `assert_guard_constraint!(condition, constraint_name)`.

**Patterns**: AAA structure (Arrange-Act-Assert), macro usage (all tests), assertion macros (better messages).

**Anti-patterns**: Don't skip AAA comments. Don't use raw `#[test]` when macros available. Don't skip assertion macros.

## Assertions

Assertion helpers: Result assertions, predicates with HRTB, range assertions.

**When to Use**: Result validation, predicate testing, range validation, custom messages. **Always use**: Better error messages than standard assertions.

**Result Assertions**: `assert_success<T, E>(&Result<T, E>)`, `assert_error<T, E>(&Result<T, E>)`.

**Predicate Assertions**: `assert_that<T, F>(value, predicate)` (HRTB), `assert_that_with_msg<T, F>(value, predicate, msg)`.

**Range Assertions**: `assert_in_range<T>(value, min, max, msg)`, `assert_eq_with_msg<T>(actual, expected, msg)`.

**Patterns**: Result validation (assert_success/assert_error), predicate testing (assert_that), range validation (assert_in_range).

## Property-Based Testing

Property-based testing: Const generics. Reproducible with seeds. Invariant testing.

**When to Use**: Invariant testing, edge case discovery, random test data generation. **Avoid**: Deterministic tests, simple unit tests.

**Basic Usage**: `PropertyTestGenerator::<MAX_ITEMS, MAX_DEPTH>::new().with_seed(seed).generate_test_data()`. Use `property_all_data_valid(generator, num_tests)` for validation.

**Custom Properties**: Create custom property functions for domain invariants. Use fixed seeds for reproducibility. Increase iterations for thorough testing.

**Patterns**: Invariant testing (reverse twice is identity), edge case discovery (random data), reproducible tests (fixed seeds).

**Anti-patterns**: Don't use for deterministic tests. Don't skip seed setting. Don't use insufficient iterations.

## Mutation Testing

Mutation testing: Quality validation. Operators and scores. Test quality metrics.

**When to Use**: Test quality validation, CI/CD pipelines, finding weak tests. **Avoid**: Development loop (too slow), simple tests.

**Basic Usage**: `MutationTester::new(data).apply_mutation(operator).test_mutation_detection(test_fn)`. Use `MutationScore::calculate(caught, total).is_acceptable()` (≥80%).

**Operators**: `MutationOperator::RemoveKey(key)`, `MutationOperator::AddKey(key, value)`, `MutationOperator::ChangeValue(key, value)`.

**Patterns**: Quality validation (CI/CD), score tracking (≥80% acceptable), operator testing (all operators).

**Anti-patterns**: Don't use in development loop. Don't accept scores <80%. Don't skip operator testing.

## Snapshot Testing

Snapshot testing: Output capture and comparison. Review workflow. Multiple formats (JSON/YAML/TOML).

**When to Use**: Complex data structures, output stability, regression testing, API response validation. **Avoid**: Simple unit tests, frequently changing outputs.

**Basic Usage**: `SnapshotAssert::assert_matches(&value, "snapshot_name")` for Display, `SnapshotAssert::assert_debug_matches(&value, "snapshot_name")` for Debug, `SnapshotAssert::assert_json_matches(&json_value, "snapshot_name")` for JSON.

**Review Workflow**: Run tests → `cargo make snapshot-review` → Accept/reject changes → Commit snapshots. Use `cargo make snapshot-accept` to accept all, `cargo make snapshot-reject` to reject all.

**Custom Settings**: `SnapshotAssert::with_settings(|settings| { settings.set_snapshot_path("custom"); }, || { /* test */ })` for custom paths.

**Patterns**: Output stability (snapshot comparison), regression testing (capture outputs), review workflow (cargo insta review), multiple formats (JSON/YAML/TOML).

**Anti-patterns**: Don't use for frequently changing outputs. Don't skip review workflow. Don't commit snapshots without review.

## Performance Testing

Performance testing: RDTSC benchmarking. Tick budget (≤8 ticks = 2ns). Hot path validation.

**When to Use**: Hot path validation, performance-critical code, tick budget enforcement. **Avoid**: Non-critical paths, development loop.

**Tick Measurement**: `TickCounter::start().elapsed_ticks()`, `measure_ticks(|| operation()) -> (result, ticks)`. Use `HOT_PATH_TICK_BUDGET = 8`.

**Validation**: `assert_within_tick_budget!(ticks)` for hot path validation. RDTSC on x86_64, falls back to `std::time::Instant` on other platforms.

**Patterns**: Hot path validation (tick budget), performance measurement (measure_ticks), platform-specific (RDTSC fallback).

**Anti-patterns**: Don't use for non-critical paths. Don't ignore platform differences. Don't skip tick budget validation.

## Guards and Constraints

Guard constraints: Chatman Constant (≤8). Batch size limits. Input validation.

**When to Use**: Input validation, constraint enforcement, guard validation. **Always use**: Prevents invalid data at ingress.

**Guard Validation**: `GuardValidator::new().validate_run_length(length)?`, `validate_batch_size(size)?`. Use `MAX_RUN_LEN = 8`, `MAX_BATCH_SIZE = 1000`.

**Patterns**: Input validation (guard constraints), constraint enforcement (MAX_RUN_LEN), guard validation (all ingress points).

## JTBD Validation

JTBD validation: Scenario validation. Real-world testing. Purpose validation.

**When to Use**: Real-world scenario testing, purpose validation, end-to-end validation. **Avoid**: Simple unit tests.

**Basic Usage**: `JtbdValidator::new().register_scenario(scenario).validate_all()`. Use `JtbdScenario` with setup_context, validate_result, expected_behavior.

**Patterns**: Scenario validation (real-world), purpose validation (JTBD), end-to-end validation (complete workflows).

## Testcontainers Integration

Testcontainers: Docker container support. Port mapping. Command execution. Auto-cleanup.

**When to Use**: Integration testing, Docker services, real dependencies. **Avoid**: Unit tests, mocked dependencies.

**Basic Usage**: `GenericContainer::new(client.client(), image, tag)?`, `container.get_host_port(port)?`, `container.exec(cmd, args)?`. Use `ContainerClient::new()` for client.

**Command Execution**: `container.exec(cmd, args) -> ExecResult` (stdout, stderr, exit_code). Use for service containers that stay running.

**Wait Conditions**: `GenericContainer::with_wait_for(client, image, tag, WaitFor::http(path, port))?` for HTTP services. Verify observable behavior (HTTP responses).

**Patterns**: Integration testing (real containers), port mapping (get_host_port), command execution (exec), wait conditions (HTTP services).

**Anti-patterns**: Don't use for unit tests. Don't skip cleanup. Don't use containers that exit immediately.

## OTEL/Weaver Integration

OTEL/Weaver: Span/metric validation. Schema conformance. Live validation.

**When to Use**: OTEL validation, schema conformance, live validation. **Requires**: `otel` feature, `weaver` feature (requires otel).

**OTEL Validation**: `SpanValidator::new().with_required_attributes(attrs).validate(span)?`, `MetricValidator::new().validate(metric)?`.

**Weaver Validation**: `WeaverValidator::new(registry_path).start()?` / `stop()?`. Use `with_config(registry_path, otlp_grpc_port, admin_port)` for custom config.

**Patterns**: Span validation (required attributes), metric validation (schema conformance), live validation (weaver).

## Best Practices

**AAA Pattern**: Arrange-Act-Assert structure required. Use AAA comments. Verify observable outputs.

**Use Macros**: Always use `chicago_test!`, `chicago_async_test!`, `chicago_fixture_test!`. Never use raw `#[test]`.

**Real Collaborators**: Use real objects, minimize mocks. Use testcontainers for integration tests.

**State Verification**: Verify outputs and state, not implementation. Verify observable behavior.

**Behavior Verification**: Tests verify what code does, not how. Verify state changes, outputs, execution order.

## Common Patterns

**Test Isolation**: `chicago_fixture_test!` provides unique fixtures. Tests don't interfere.

**Reusable Test Data**: Helper functions for common test data. `fn create_test_order() -> Value`.

**Error Handling**: Use `?` operator in async tests. Use `assert_ok!` / `assert_err!` for Result validation.

## Anti-patterns

**Don't Skip AAA Comments**: Always use Arrange-Act-Assert structure with comments.

**Don't Use Fixtures Unnecessarily**: Only use fixtures when state/isolation needed.

**Don't Test Implementation Details**: Verify outputs and state, not internal counters/state.

**Don't Use Mocks When Real Available**: Use real collaborators (testcontainers, real dependencies).

**Don't Skip Behavior Verification**: Tests must verify observable outputs, not just function existence.

## Troubleshooting

**TestFixture::new() fails**: Ensure tokio runtime available for async tests.

**Property-based tests don't compile**: Enable `property-testing` feature flag.

**Testcontainers tests fail**: Ensure Docker running and `testcontainers` feature enabled.

**Performance tests fail on non-x86_64**: RDTSC is x86_64-specific; falls back to `std::time::Instant`.

## Summary

**Key Associations**: Fixtures = RAII = Auto-cleanup. Builders = Fluent = JSON/HashMap. Macros = AAA = Zero-boilerplate. Property = Const generics = Reproducible. Mutation = Quality = Operators. Performance = RDTSC = Tick budget. Guards = Constraints = Chatman Constant. Testcontainers = Docker = Auto-cleanup. OTEL/Weaver = Validation = Schema.

**Pattern**: All features follow consistent patterns: when to use, basic usage, common patterns, anti-patterns. Use macros for all tests. Verify observable outputs. Use real collaborators.

## Next Steps

- **[Quick Guide](QUICK_GUIDE.md)** - Essential patterns (80% of use cases)
- **[API Reference](API_REFERENCE.md)** - Complete API documentation
- **[Getting Started](GETTING_STARTED.md)** - Quick start guide
- **[Architecture](ARCHITECTURE.md)** - Design principles and patterns
- **[Examples](../examples/)** - Working code examples
