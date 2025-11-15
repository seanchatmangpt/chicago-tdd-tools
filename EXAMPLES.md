# Chicago TDD Tools - Examples Quick Reference

**Version:** 1.2.0 | **For complete documentation:** [examples/README.md](examples/README.md)

Quick reference for all examples organized by the [Diátaxis framework](https://diataxis.fr/).

---

## Start Here

**New to Chicago TDD Tools?** Start with these examples in order:

1. **[basic_test.rs](examples/basic_test.rs)** - Core patterns, fixtures, builders
   ```bash
   cargo run --example basic_test
   ```

2. **[macro_examples.rs](examples/macro_examples.rs)** - Test and assertion macros
   ```bash
   cargo test --example macro_examples
   ```

3. **Pick examples relevant to your use case** from the categories below

---

## By Category

### Tutorials (Learning-Oriented)

Learn fundamental concepts step-by-step:

| Example | Description | Command |
|---------|-------------|---------|
| [basic_test.rs](examples/basic_test.rs) | Core patterns: fixtures, builders, AAA | `cargo run --example basic_test` |
| [macro_examples.rs](examples/macro_examples.rs) | Test/assertion macros | `cargo test --example macro_examples` |

---

### How-To Guides (Task-Oriented)

Solve specific testing problems:

| Example | Description | Features | Command |
|---------|-------------|----------|---------|
| [property_testing.rs](examples/property_testing.rs) | Random test data, shrinking | `property-testing` | `cargo run --example property_testing --features property-testing` |
| [snapshot_testing.rs](examples/snapshot_testing.rs) | Output stability validation | `snapshot-testing` | `cargo test --features snapshot-testing --example snapshot_testing` |
| [mutation_testing.rs](examples/mutation_testing.rs) | Test quality measurement | None | `cargo run --example mutation_testing` |
| [concurrency_testing.rs](examples/concurrency_testing.rs) | Thread safety with loom | `concurrency-testing` | `cargo test --features concurrency-testing --example concurrency_testing` |
| [cli_testing.rs](examples/cli_testing.rs) | CLI app testing | `cli-testing` | `cargo test --features cli-testing --example cli_testing` |
| [testcontainers_example.rs](examples/testcontainers_example.rs) | Docker integration | `testcontainers` | `cargo run --example testcontainers_example --features testcontainers` |
| [otel_weaver_testing.rs](examples/otel_weaver_testing.rs) | Observability validation | `otel,weaver` | `cargo test --features otel,weaver --example otel_weaver_testing` |

---

### Explanation (Understanding-Oriented)

Understand concepts and philosophy:

| Example | Description | Command |
|---------|-------------|---------|
| [go_extra_mile.rs](examples/go_extra_mile.rs) | 1st/2nd/3rd idea progression, 80/20 thinking | `cargo run --example go_extra_mile --features otel,weaver` |
| [advanced_features.rs](examples/advanced_features.rs) | Type-level guarantees, zero-cost abstractions | `cargo run --example advanced_features` |

---

## By Use Case

### Unit Testing

1. **[basic_test.rs](examples/basic_test.rs)** - Core patterns
2. **[macro_examples.rs](examples/macro_examples.rs)** - Macros
3. **[property_testing.rs](examples/property_testing.rs)** - Property-based testing

### Integration Testing

1. **[testcontainers_example.rs](examples/testcontainers_example.rs)** - Docker containers
2. **[otel_weaver_testing.rs](examples/otel_weaver_testing.rs)** - Observability

### Advanced Testing

1. **[mutation_testing.rs](examples/mutation_testing.rs)** - Test quality
2. **[concurrency_testing.rs](examples/concurrency_testing.rs)** - Thread safety
3. **[snapshot_testing.rs](examples/snapshot_testing.rs)** - Output stability

### CLI Applications

1. **[cli_testing.rs](examples/cli_testing.rs)** - CLI testing patterns

---

## Quick Commands

### Run All Examples

```bash
# Run all runnable examples
cargo make test-examples

# Run all with all features
for example in basic_test advanced_features property_testing mutation_testing go_extra_mile testcontainers_example; do
    cargo run --example $example --all-features
done
```

### Run Specific Example

```bash
# No features required
cargo run --example basic_test
cargo run --example advanced_features
cargo run --example mutation_testing

# Feature-specific examples
cargo run --example property_testing --features property-testing
cargo test --features snapshot-testing --example snapshot_testing
cargo test --features concurrency-testing --example concurrency_testing
cargo test --features cli-testing --example cli_testing
cargo run --example testcontainers_example --features testcontainers
cargo test --features otel,weaver --example otel_weaver_testing
cargo run --example go_extra_mile --features otel,weaver
```

---

## Feature Requirements

| Feature | Examples | Install Command |
|---------|----------|-----------------|
| None | `basic_test`, `advanced_features`, `mutation_testing`, `macro_examples` | N/A |
| `property-testing` | `property_testing` | Add to `Cargo.toml` features |
| `snapshot-testing` | `snapshot_testing` | Add to `Cargo.toml` features |
| `concurrency-testing` | `concurrency_testing` | Add to `Cargo.toml` features |
| `cli-testing` | `cli_testing` | Add to `Cargo.toml` features |
| `testcontainers` | `testcontainers_example` | Requires Docker + feature |
| `otel` + `weaver` | `otel_weaver_testing`, `go_extra_mile` | Add to `Cargo.toml` features |

**Feature Bundles:**
- `testing-extras`: `property-testing` + `snapshot-testing` + `fake-data` (most common)
- `testing-full`: All testing features
- `observability-full`: `otel` + `weaver`

---

## Common Patterns

### AAA Pattern

All examples follow Arrange-Act-Assert:

```rust
test!(my_test, {
    // Arrange: Set up test data
    let input = 5;

    // Act: Execute code under test
    let result = input * 2;

    // Assert: Verify expected behavior
    assert_eq!(result, 10);
});
```

### Error Handling

```rust
// Use ? operator to propagate
let value = result?;

// Or use match for explicit handling
match result {
    Ok(value) => { /* success */ },
    Err(e) => { /* error */ },
}
```

### Fixture Usage

```rust
fixture_test!(my_test, fixture, {
    let data = fixture.test_data();
    // ... test code ...
});
```

---

## Troubleshooting

### Example Won't Run

**Error:** `no such command: example`
- **Fix:** Use `cargo run --example name` not `cargo example name`

**Error:** Feature not enabled
- **Fix:** Add `--features feature-name` to command
  ```bash
  cargo run --example property_testing --features property-testing
  ```

### Docker Examples Fail

**Error:** Cannot connect to Docker daemon
- **Fix:** Start Docker Desktop and verify with `docker ps`

### Weaver Examples Fail

**Error:** Weaver not available
- **Fix:** Bootstrap Weaver:
  ```bash
  cargo make weaver-bootstrap
  ```

---

## Learn More

- **[Complete Examples Documentation](examples/README.md)** - Full Diátaxis documentation
- **[Getting Started Guide](docs/getting-started/GETTING_STARTED.md)** - Setup and configuration
- **[API Reference](docs/reference/API_REFERENCE.md)** - Complete API docs
- **[Pattern Cookbook](cookbook/src/README.md)** - Alexander-style patterns

---

**Quality is the default. Prevention beats detection.**

*Version 1.2.0 | Updated 2025-11-15 | Team KNHK | License MIT*
