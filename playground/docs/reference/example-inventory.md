# Example Inventory

**Complete list** of all examples in the playground.

## Core Examples (Always Available)

| Name | File | Purpose | Learn | Run |
|------|------|---------|-------|-----|
| **fixtures** | `src/core/fixtures.rs` | Automatic setup/teardown | Fixture lifecycle | `core exec --names "fixtures"` |
| **builders** | `src/core/builders.rs` | Fluent test data building | Builder pattern | `core exec --names "builders"` |
| **assertions** | `src/core/assertions.rs` | Clear assertion helpers | Assertion patterns | `core exec --names "assertions"` |
| **macros** | `src/core/macros.rs` | Test macros (test!, async_test!, fixture_test!) | AAA enforcement | `core exec --names "macros"` |
| **state** | `src/core/state.rs` | Type-level state machines | Compile-time correctness | `core exec --names "state"` |
| **const** | `src/core/const_assert.rs` | Compile-time assertions | Const assertions | `core exec --names "const"` |
| **alert** | `src/core/alert.rs` | Visual alert macros | Structured logging | `core exec --names "alert"` |

## Testing Examples (Optional Features)

| Name | Feature | File | Purpose | Run |
|------|---------|------|---------|-----|
| **prop** | `property-testing` | `src/testing/property.rs` | Property-based testing | `test exec --names "prop"` |
| **mut** | `mutation-testing` | `src/testing/mutation.rs` | Test quality validation | `test exec --names "mut"` |
| **snap** | `snapshot-testing` | `src/testing/snapshot.rs` | Output regression prevention | `test exec --names "snap"` |
| **conc** | `concurrency-testing` | `src/testing/concurrency.rs` | Race condition detection | `test exec --names "conc"` |
| **cli** | `cli-testing` | `src/testing/cli.rs` | Command-line tool testing | `test exec --names "cli"` |
| **gen** | `cli-testing` | `src/testing/generator.rs` | Test code generation | `test exec --names "gen"` |

## Validation Examples (Optional Features)

| Name | Feature | File | Purpose | Run |
|------|---------|------|---------|-----|
| **cov** | `coverage` | `src/validation/coverage.rs` | Test coverage analysis | `valid exec --names "cov"` |
| **guard** | `guards` | `src/validation/guards.rs` | Constraint enforcement | `valid exec --names "guard"` |
| **jtbd** | `jtbd` | `src/validation/jtbd.rs` | Feature completeness | `valid exec --names "jtbd"` |
| **perf** | `benchmarking` | `src/validation/performance.rs` | Performance validation | `valid exec --names "perf"` |

## Observability Examples (Optional Features)

| Name | Feature | File | Purpose | Run |
|------|---------|------|---------|-----|
| **otel** | `otel` | `src/observability/otel.rs` | OTEL span validation | `obs otel` |
| **weav** | `weaver` | `src/observability/weaver.rs` | Semantic convention validation | `obs weav` |
| **bootstrap** | `weaver` | CLI | Download Weaver CLI | `obs bootstrap` |
| **smoke** | `weaver` | CLI | Test Weaver installation | `obs smoke` |

## Integration Examples (Optional Features)

| Name | Feature | File | Purpose | Run |
|------|---------|------|---------|-----|
| **contain** | `testcontainers` | `src/integration/testcontainers.rs` | Docker container testing | `integ contain` |

## Quick Reference by Use Case

### "I want to learn TDD basics"

1. `core exec --names "fixtures"` - Learn setup/teardown
2. `core exec --names "builders"` - Learn test data building
3. `core exec --names "assertions"` - Learn clear assertions
4. `core exec --names "macros"` - Learn AAA pattern

### "I want advanced testing"

1. `test exec --names "prop"` - Property-based testing
2. `test exec --names "mut"` - Test quality validation
3. `test exec --names "snap"` - Regression prevention
4. `test exec --names "conc"` - Thread safety

### "I want to validate quality"

1. `valid exec --names "cov"` - Coverage analysis
2. `valid exec --names "guard"` - Constraints
3. `valid exec --names "jtbd"` - Completeness
4. `valid exec --names "perf"` - Performance

### "I want observability"

1. `obs otel` - OTEL basics
2. `obs bootstrap` - Setup Weaver
3. `obs weav` - Semantic conventions

### "I want integration testing"

1. `integ contain` - Docker/testcontainers

## Examples by Learning Path

### Beginner Path (5-10 minutes)

```bash
cargo run -- core stat
cargo run -- core exec --names "fixtures"
cargo run -- core exec --names "builders"
```

### Intermediate Path (30-45 minutes)

```bash
cargo run -- core exec --names "fixtures builders assertions macros"
cargo run --all-features -- test exec --names "prop"
cargo run --all-features -- test exec --names "snap"
```

### Advanced Path (1-2 hours)

```bash
# All core
cargo run -- core exec --names "fixtures builders assertions macros state const alert"

# All testing
cargo run --all-features -- test exec --names "prop mut snap conc cli gen"

# All validation
cargo run --all-features -- valid exec --names "cov guard jtbd perf"

# Observability
cargo run --features otel -- obs otel
cargo run --features weaver -- obs weav

# Integration
cargo run --features testcontainers -- integ contain
```

## Examples by Time Commitment

### 5 Minutes
- `core exec --names "fixtures"`

### 15 Minutes
- `core exec --names "fixtures builders"`

### 30 Minutes
- `core exec --names "fixtures builders assertions macros"`

### 1 Hour
- `core exec --names "fixtures builders assertions macros state const alert"`
- `test exec --names "prop snap"`

### 2+ Hours
- All examples with `--all-features`

## Dependencies Between Examples

### Prerequisites for Each

| Example | Requires | Reason |
|---------|----------|--------|
| builders | fixtures | Often used together |
| assertions | fixtures | Testing assertions |
| macros | none | Foundational |
| state | macros | Type-level building |
| property | none | Independent testing |
| mutation | none | Tests other tests |
| snapshot | none | Output testing |
| concurrency | none | Thread testing |
| coverage | none | Metrics |
| OTEL | none | Observability |
| Weaver | OTEL | Extended OTEL |
| testcontainers | none | Integration |

## Recommended Example Combinations

### Minimum Viable Testing

```bash
cargo run -- core exec --names "fixtures builders assertions"
```

### Standard Testing

```bash
cargo run --features testing-extras -- core exec --names "fixtures builders assertions"
cargo run --features testing-extras -- test exec --names "prop snap"
```

### Comprehensive Testing

```bash
cargo run --all-features -- core exec --names "fixtures builders assertions macros state"
cargo run --all-features -- test exec --names "prop mut snap conc"
cargo run --all-features -- valid exec --names "cov guard jtbd perf"
```

### Production-Grade

```bash
# All of the above plus:
cargo run --features otel -- obs otel
cargo run --features weaver -- obs weav
cargo run --features testcontainers -- integ contain
```

## Example Statistics

| Category | Total | Required | Optional |
|----------|-------|----------|----------|
| Core | 7 | 7 | 0 |
| Testing | 6 | 0 | 6 |
| Validation | 4 | 0 | 4 |
| Observability | 4 | 0 | 4 |
| Integration | 1 | 0 | 1 |
| **Total** | **22** | **7** | **15** |

## Feature Coverage

Each example demonstrates:
- ✅ Arrange-Act-Assert pattern
- ✅ Clear example structure
- ✅ Practical use cases
- ✅ Error handling
- ✅ Comments and documentation

## Next Steps

- **See CLI commands** → [CLI Command Reference](cli-commands.md)
- **Understand features** → [Feature Matrix](feature-matrix.md)
- **Learn how to add examples** → [How to Add Examples](../how-to/adding-examples.md)

---

Explore all examples with `cargo run -- <category> list`
