# How to Use Playground with Feature Flags

**Quick reference** for enabling and managing feature flags in the playground.

## What Are Feature Flags?

Feature flags control which capabilities are compiled. Some features are optional (require additional setup or dependencies).

## Quick Commands

```bash
# All features (default)
cargo run --all-features -- core stat

# Specific features
cargo run --features property-testing -- test stat

# No optional features (core only)
cargo run --no-default-features -- core stat

# Multiple features
cargo run --features "property-testing,snapshot-testing" -- test stat
```

## Feature Groups

### Core Features (Always Available)

No feature flag needed:

- Fixtures
- Builders
- Assertions
- Macros
- State machines
- Const assertions
- Alert macros

```bash
cargo run -- core stat
```

### Testing Features (Optional)

Require feature flags:

| Feature | Flag | Purpose |
|---------|------|---------|
| Property-based testing | `property-testing` | Random test generation |
| Mutation testing | `mutation-testing` | Test quality validation |
| Snapshot testing | `snapshot-testing` | Output regression prevention |
| Concurrency testing | `concurrency-testing` | Race condition detection |
| CLI testing | `cli-testing` | Command-line tool testing |
| Fake data | `fake-data` | Test data generation |

**Run with specific feature:**
```bash
cargo run --features property-testing -- test stat
```

**Run all testing features:**
```bash
cargo run --features testing-extras -- test stat
```

### Validation Features (Optional)

| Feature | Flag | Purpose |
|---------|------|---------|
| Coverage analysis | `coverage` | Test coverage reporting |
| Guard constraints | `guards` | Compile-time constraints |
| JTBD validation | `jtbd` | Feature completeness |
| Performance testing | `benchmarking` | Tick budget validation |

```bash
cargo run --features coverage -- valid stat
```

### Observability Features (Optional)

| Feature | Flag | Purpose |
|---------|------|---------|
| OTEL validation | `otel` | OpenTelemetry testing |
| Weaver validation | `weaver` | Semantic conventions (implies `otel`) |

```bash
cargo run --features otel -- obs stat
cargo run --features weaver -- obs stat
```

### Integration Features (Optional)

| Feature | Flag | Purpose |
|---------|------|---------|
| Docker support | `testcontainers` | Container integration testing |
| Async fixtures | `async` | Async fixture providers (Rust 1.75+) |

```bash
cargo run --features testcontainers -- integ stat
```

## Feature Bundles (Recommended)

### Bundle: Testing Extras

Most common combination:

```bash
cargo run --features testing-extras -- test stat
```

Includes:
- Property-based testing
- Snapshot testing
- Fake data generation

### Bundle: Testing Full

All testing features:

```bash
cargo run --features testing-full -- test stat
```

Includes:
- Property-based testing
- Mutation testing
- Snapshot testing
- Concurrency testing
- CLI testing
- Fake data generation

### Bundle: Observability Full

All observability features:

```bash
cargo run --features observability-full -- obs stat
```

Includes:
- OTEL validation
- Weaver validation

### Bundle: Integration Full

All integration features:

```bash
cargo run --features integration-full -- integ stat
```

Includes:
- Testcontainers
- Async fixtures (if Rust 1.75+)

## Common Feature Combinations

### Scenario 1: Learning Basics

```bash
# Just core features
cargo run --no-default-features -- core stat
```

Perfect for:
- Learning fixtures, builders, assertions
- Understanding macros
- No external dependencies

### Scenario 2: Standard Testing

```bash
# Core + testing extras
cargo run --features testing-extras -- core stat
cargo run --features testing-extras -- test stat
```

Perfect for:
- 80% of use cases
- Property, snapshot, fake data
- Comprehensive testing

### Scenario 3: Advanced Testing

```bash
# Core + all testing features
cargo run --features testing-full -- test stat
```

Perfect for:
- Advanced test quality
- Mutation testing
- Concurrency testing
- Comprehensive coverage

### Scenario 4: Full Stack

```bash
# All features
cargo run --all-features -- core stat
cargo run --all-features -- test stat
cargo run --all-features -- valid stat
cargo run --all-features -- obs stat
cargo run --all-features -- integ stat
```

Perfect for:
- Production-grade validation
- Observability testing
- Integration testing
- Everything enabled

## Enabling Features in Cargo.toml

```toml
[dev-dependencies]
# Just core features (always available)
chicago-tdd-tools = "1.1"

# Common: Testing extras
chicago-tdd-tools = { version = "1.1", features = ["testing-extras"] }

# Full testing
chicago-tdd-tools = { version = "1.1", features = ["testing-full"] }

# With observability
chicago-tdd-tools = { version = "1.1", features = ["testing-extras", "otel"] }

# Everything
chicago-tdd-tools = { version = "1.1", features = ["testing-full", "observability-full", "integration-full"] }

# Specific features
chicago-tdd-tools = { version = "1.1", features = ["property-testing", "snapshot-testing", "testcontainers"] }
```

## Testing Feature Combinations

### Test Across Feature Levels

```bash
# No optional features
cargo test --no-default-features

# Core features only (testing-extras excluded)
cargo test --features ""

# Testing extras
cargo test --features testing-extras

# All features
cargo test --all-features

# Specific features
cargo test --features "property-testing,snapshot-testing"
```

## Dependency Implications

| Feature | External Dependencies | When Needed |
|---------|----------------------|------------|
| `property-testing` | proptest | Random test generation |
| `mutation-testing` | (internal) | Test quality checks |
| `snapshot-testing` | insta | Output regression |
| `concurrency-testing` | loom | Race condition detection |
| `cli-testing` | assert_cmd | CLI testing |
| `fake-data` | fake, faker | Test data generation |
| `testcontainers` | testcontainers | Docker support |
| `otel` | opentelemetry | OTEL validation |
| `weaver` | opentelemetry | Semantic conventions |
| `async` | tokio | Async fixtures |

## Feature Flag Optimization

### Compile Time Reduction

If you don't need certain features:

```bash
# Fast: Core features only
cargo run --no-default-features -- core stat

# Medium: Testing extras
cargo run --features testing-extras -- test stat

# Slow: All features
cargo run --all-features -- test stat
```

### Conditional Compilation

In tests, you can conditionally compile:

```rust
#[cfg(feature = "property-testing")]
test!(test_property_based, {
    // Property test code
});

#[cfg(feature = "testcontainers")]
fixture_test!(test_with_docker, fixture, {
    // Docker test code
});
```

## Feature Flag Verification

### List Enabled Features

```bash
# Show which features are enabled
cargo tree --features testing-extras
```

### Check If Feature Is Available

```bash
# Try to run with specific feature
cargo run --features property-testing -- test stat
```

If it succeeds, feature is available.

## Troubleshooting

**Q: "Feature 'X' is required but not available"**
A: Enable the feature:
```bash
cargo run --features property-testing -- test stat
```

**Q: "Unknown feature"**
A: Check available features:
```bash
cargo run --all-features -- test stat
```

**Q: "Too slow to compile"**
A: Use fewer features:
```bash
cargo run --features testing-extras -- test stat  # Faster than --all-features
```

**Q: "Docker feature not working"**
A: Ensure Docker is installed and running:
```bash
docker ps
```

## Feature Recommendations by Role

### Test Writer

```bash
cargo run --features testing-extras -- test stat
```

Property-based, snapshot, fake data.

### Performance Engineer

```bash
cargo run --features "benchmarking" -- valid stat
```

Tick budget validation.

### DevOps/SRE

```bash
cargo run --features "otel,weaver" -- obs stat
```

Observability and semantic conventions.

### Full Stack Developer

```bash
cargo run --all-features -- core stat
```

Everything.

## Next Steps

- **Copy to your project** → [Copying Examples](../tutorials/copying-examples.md)
- **See all commands** → [CLI Command Reference](../reference/cli-commands.md)
- **Understand features** → [Feature Organization](../explanation/feature-organization.md)

---

Enable features strategically to balance functionality and compile time.
