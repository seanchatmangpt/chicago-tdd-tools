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
- See [SPR Guide](docs/SPR_GUIDE.md) for clippy best practices

## Quick Start

**Step 1**: Install `cargo-make` (required for build system):
```bash
cargo install cargo-make
```

**Verify**: `cargo make --version` should show version. If you get "command not found", install cargo-make first.

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
chicago-tdd-tools = "1.1.0"  # Use version when published
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

**See [Quick Guide](docs/QUICK_GUIDE.md) for complete macro reference.**

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

**See [Getting Started](docs/GETTING_STARTED.md) for complete feature list** (property-testing, mutation-testing, testcontainers, otel, weaver, etc.)

## Weaver Live-Check Quick Start

Chicago TDD Tools dogfoods Weaver. Follow these steps to exercise live-check locally:

1. **Bootstrap prerequisites** (Weaver CLI + semantic convention registry):
   ```bash
   cargo make weaver-bootstrap
   ```
2. **Run the fast smoke test** (version check + telemetry span):
   ```bash
   cargo make weaver-smoke
   ```
3. **Run full integration when Docker is available**:
   ```bash
   cargo make test-integration        # Requires Docker + weaver feature
   ```

**Need to temporarily skip Weaver tests?** Set `WEAVER_ALLOW_SKIP=1` in your environment. Without that explicit opt-out, Weaver tests fail fast when prerequisites are missing—quality is the default.

## Build System

**Always use `cargo make` commands** (required for proc-macro crates and timeout enforcement):
```bash
cargo make check      # Compile check
cargo make test       # Run tests
cargo make lint       # Run clippy
```

**Why cargo-make?** The build system handles proc-macro crates correctly, includes timeouts to prevent hanging, and ensures consistency. Using `cargo test` directly may fail with proc-macro errors.

See [Build System Practices](.cursor/rules/build-system-practices.mdc) for details.

## Documentation

- **[Quick Guide](docs/QUICK_GUIDE.md)** - Essential patterns (80% of use cases)
- **[Getting Started](docs/GETTING_STARTED.md)** - Complete setup guide with troubleshooting
- **[User Guide](docs/USER_GUIDE.md)** - Comprehensive usage guide
- **[API Reference](docs/API_REFERENCE.md)** - Complete API documentation
- **[Architecture](docs/ARCHITECTURE.md)** - Design principles
- **[Pattern Cookbook](cookbook/src/README.md)** - Alexander-style pattern language (testing, architecture, design)

## Requirements

- **Rust**: Edition 2021 (Rust 1.70+)
- **cargo-make**: `cargo install cargo-make` (verify with `cargo make --version`)
- **Tokio**: Included in dev-dependencies

**Optional**: Docker (for `testcontainers` feature), Rust 1.75+ (for `async` feature)

## Troubleshooting

**"command not found: cargo-make"**: Install with `cargo install cargo-make`, then verify with `cargo make --version`.

**"cannot find crate 'chicago_tdd_tools'"**: Check that `Cargo.toml` has `edition = "2021"` in `[package]` section and path is correct.

**"cannot find macro 'test!'"**: Ensure you have `use chicago_tdd_tools::prelude::*;` at the top of your test file.

**See [Getting Started - Troubleshooting](docs/GETTING_STARTED.md#troubleshooting) for more help.**

## License

MIT
