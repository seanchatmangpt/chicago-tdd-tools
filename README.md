# Chicago TDD Tools

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

Testing framework for Chicago TDD (Classicist Test-Driven Development) in Rust. Enforces AAA pattern, provides fixtures, builders, and advanced testing capabilities.

## Development Workflow

**Pre-commit validation** (run before committing):
```bash
cargo make pre-commit  # Format, lint, test
```

**Important**: The CI/CD pipeline automatically enforces clippy checks on every commit/PR.
Always run `cargo make pre-commit` before committing to catch issues early.

**Clippy standards**:
- All clippy warnings are treated as errors (`-D warnings`)
- Use `#[allow(clippy::...)]` with justification comments when necessary
- CI/CD pipeline will fail if clippy finds any warnings/errors
- See [SPR Guide](docs/process/SPR_GUIDE.md) for clippy best practices

## Quick Start

**Step 1**: Install `cargo-make` (required for build system):
```bash
cargo install cargo-make
```

**Verify**: `cargo make --version` should show version. If you get "command not found", install cargo-make first.

**Step 1.5**: Install Git hooks (recommended for production code safety):
```bash
cargo make install-hooks
```

This installs pre-commit hooks that prevent `.unwrap()` and `.expect()` in production code, reducing production panics.

**Step 2**: Create a new Rust project (if you don't have one):
```bash
cargo new my-project
cd my-project
```

**Step 3**: Add dependency to `Cargo.toml`:

**For local development** (if framework is in parent directory):
```toml
[package]
name = "my-project"
version = "0.1.0"
edition = "2021"  # Required: Edition 2021

[dev-dependencies]
chicago-tdd-tools = { path = "../chicago-tdd-tools" }
tokio = { version = "1.0", features = ["rt", "macros"] }
```

**For GitHub users** (when framework is published to crates.io):
```toml
[package]
name = "my-project"
version = "0.1.0"
edition = "2021"  # Required: Edition 2021

[dev-dependencies]
chicago-tdd-tools = "1.1.2"  # Use version when published
tokio = { version = "1.0", features = ["rt", "macros"] }
```

**Step 4**: Create your first test file `tests/my_first_test.rs`:
```rust
use chicago_tdd_tools::prelude::*;  // prelude::* imports all common macros and types

test!(test_example, {
    // Arrange
    let input = 5;
    // Act
    let result = input * 2;
    // Assert
    assert_eq!(result, 10);
});

async_test!(test_async, {
    let result = async { 5 * 2 }.await;
    assert_eq!(result, 10);
});

fixture_test!(test_with_fixture, fixture, {
    let counter = fixture.test_counter();
    assert!(counter >= 0);
});
```

**Step 5**: Verify installation and run tests:
```bash
cargo make check      # Verify compilation
cargo make test       # Run tests
```

**Verify Installation**:
- ✅ Compilation succeeds: `cargo make check` completes without errors
- ✅ Tests run: `cargo make test` shows your tests passing
- ✅ Macros work: Test file compiles and runs successfully

**Note**: Always use `cargo make` commands (not `cargo test` directly). The build system handles proc-macro crates and includes timeouts.

## Core Macros

### Test Macros
- **`test!`**: Synchronous tests with AAA pattern
- **`async_test!`**: Async tests (1s timeout)
- **`fixture_test!`**: Async tests with automatic fixture setup

### Assertion Macros
- **`assert_ok!`**: Assert Result is Ok
- **`assert_err!`**: Assert Result is Err
- **`assert_in_range!`**: Assert value in range

**See [Quick Guide](docs/getting-started/QUICK_GUIDE.md) for complete macro reference.**

## Features

**Most Common**: Enable `testing-extras` for property-based testing, snapshot testing, and fake data:
```toml
[dev-dependencies]
chicago_tdd_tools = { 
    path = "../chicago-tdd-tools",  # Or use git URL when published
    features = ["testing-extras"]   # Enable common testing features
}
```

**When to use features**: Enable features only when you need them (e.g., `testcontainers` for Docker integration, `otel` for observability testing).

**See [Getting Started](docs/getting-started/GETTING_STARTED.md) for complete feature list** (property-testing, mutation-testing, testcontainers, otel, weaver, etc.)

## Weaver Live-Check Quick Start

Chicago TDD Tools dogfoods Weaver. Follow these steps to exercise live-check locally:

### Prerequisites
- **Weaver feature**: Enable `weaver` feature in `Cargo.toml` (automatically enables `otel`)
- **Docker** (for integration tests only): Required for `cargo make test-integration`
  - Verify Docker is running: `docker ps` should succeed
  - Install: [Docker Desktop](https://www.docker.com/products/docker-desktop) if not installed

### Setup Steps

1. **Bootstrap prerequisites** (Weaver CLI + semantic convention registry):
   ```bash
   cargo make weaver-bootstrap
   ```
   - Downloads Weaver binary to `target/<profile>/weaver`
   - Clones semantic convention registry to `registry/`
   - Takes ~30-60 seconds on first run

2. **Run the fast smoke test** (version check + telemetry span):
   ```bash
   cargo make weaver-smoke
   ```
   - Verifies Weaver CLI is installed and working
   - Sends a test telemetry span
   - Does not require Docker
   - Should complete in <5 seconds

3. **Run full integration when Docker is available**:
   ```bash
   cargo make test-integration        # Requires Docker + weaver feature
   ```
   - Runs container-based Weaver integration tests
   - Requires Docker daemon running
   - Requires `weaver` feature enabled
   - Tests fail fast if prerequisites missing (unless `WEAVER_ALLOW_SKIP=1` is set)

**Need to temporarily skip Weaver tests?** Set `WEAVER_ALLOW_SKIP=1` in your environment. Without that explicit opt-out, Weaver tests fail fast when prerequisites are missing—quality is the default.

## Build System

**Always use `cargo make` commands** (required for proc-macro crates and timeout enforcement):
```bash
cargo make check      # Compile check
cargo make test       # Run tests
cargo make lint       # Run clippy
```

**Why cargo-make?** The build system handles proc-macro crates correctly, includes timeouts to prevent hanging, and ensures consistency. Using `cargo test` directly may fail with proc-macro errors.

## Documentation

- **[Quick Guide](docs/getting-started/QUICK_GUIDE.md)** - Essential patterns (80% of use cases)
- **[Getting Started](docs/getting-started/GETTING_STARTED.md)** - Complete setup guide with troubleshooting
- **[User Guide](docs/getting-started/USER_GUIDE.md)** - Comprehensive usage guide
- **[API Reference](docs/reference/API_REFERENCE.md)** - Complete API documentation
- **[Architecture](docs/reference/ARCHITECTURE.md)** - Design principles
- **[Pattern Cookbook](cookbook/src/README.md)** - Alexander-style pattern language (testing, architecture, design)
- **[Documentation Index](docs/README.md)** - Complete documentation navigation

## Requirements

### Prerequisites Checklist

**Required**:
- **Rust**: Edition 2021 (Rust 1.70+)
  - **Verify**: Run `rustc --version` - should show 1.70.0 or higher
  - **Install**: Use [rustup](https://rustup.rs/) if not installed
- **Cargo**: Latest stable (comes with Rust)
  - **Verify**: Run `cargo --version` - should show latest stable
- **cargo-make**: Required for build system
  - **Install**: `cargo install cargo-make`
  - **Verify**: Run `cargo make --version` - should show version number
  - **If missing**: Install with `cargo install cargo-make`, then verify
- **Tokio**: Required for async tests (add to `dev-dependencies` in your `Cargo.toml`)

**Optional** (enable features as needed):
- **Docker**: Required for `testcontainers` feature
  - **Verify**: Run `docker ps` - should show Docker daemon running
  - **Install**: [Docker Desktop](https://www.docker.com/products/docker-desktop) for your platform
- **Rust 1.75+**: Required for `async` feature (async fixture providers)
  - **Verify**: Run `rustc --version` - should show 1.75.0 or higher
  - **Upgrade**: Run `rustup update stable` if needed

## Troubleshooting

### Common Setup Errors

**"command not found: cargo-make"**
- **Cause**: cargo-make not installed or not in PATH
- **Fix**: Install with `cargo install cargo-make`, then verify with `cargo make --version`
- **Verify**: Command should show version number, not "command not found"

**"cannot find crate 'chicago_tdd_tools'"**
- **Cause**: Path incorrect, edition missing, or dependency not added
- **Fix**: 
  1. Check `Cargo.toml` has `edition = "2021"` in `[package]` section
  2. Verify path in `[dev-dependencies]` is correct (e.g., `path = "../chicago-tdd-tools"`)
  3. Ensure dependency is in `[dev-dependencies]`, not `[dependencies]`
- **Verify**: Run `cargo make check` - should compile without errors

**"cannot find macro 'test!'"**
- **Cause**: Missing prelude import
- **Fix**: Add `use chicago_tdd_tools::prelude::*;` at the top of your test file
- **Alternative**: Use explicit import: `use chicago_tdd_tools::test;`
- **Verify**: Test file should compile after adding import

**"edition 2021 required"**
- **Cause**: `Cargo.toml` missing edition specification
- **Fix**: Add `edition = "2021"` to `[package]` section in `Cargo.toml`:
  ```toml
  [package]
  name = "my-project"
  version = "0.1.0"
  edition = "2021"  # Required
  ```

**"feature 'X' is required for module Y"**
- **Cause**: Feature flag not enabled for feature-gated module
- **Fix**: Enable required feature in `Cargo.toml`:
  ```toml
  chicago-tdd-tools = { 
      path = "../chicago-tdd-tools",
      features = ["feature-name"]  # e.g., "otel", "weaver", "testcontainers"
  }
  ```
- **Common features**: `testing-extras`, `testcontainers`, `otel`, `weaver`, `async`

**"cannot find module 'observability'" or "cannot find module 'integration'"**
- **Cause**: Feature-gated modules require explicit feature flags
- **Fix**: Enable required features: `features = ["otel", "weaver", "testcontainers"]`

**Tests pass locally but fail in CI**
- **Cause**: Environment differences (missing dependencies, different Rust version)
- **Fix**: 
  1. Verify Rust version matches: `rustc --version`
  2. Run `cargo make ci-local` to simulate CI environment
  3. Check all prerequisites are installed (cargo-make, Docker if needed)

**Docker/Testcontainers tests fail**
- **Cause**: Docker daemon not running or not available
- **Fix**: 
  1. Start Docker Desktop
  2. Verify with `docker ps` - should show running containers or empty list (not error)
  3. Ensure Docker is accessible: `docker info` should succeed

**Weaver tests fail**
- **Cause**: Weaver CLI not installed or registry not bootstrapped
- **Fix**: 
  1. Run `cargo make weaver-bootstrap` to install Weaver CLI and registry
  2. Verify with `cargo make weaver-smoke` - should pass
  3. If Docker required: Ensure Docker is running for integration tests

**See [Getting Started - Troubleshooting](docs/getting-started/GETTING_STARTED.md#troubleshooting) for more detailed help.**

## License

MIT
