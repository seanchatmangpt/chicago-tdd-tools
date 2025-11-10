# Chicago TDD Tools

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

Testing framework for Chicago TDD (Classicist Test-Driven Development) in Rust. Enforces AAA pattern, provides fixtures, builders, and advanced testing capabilities.

## Quick Start

```toml
[dev-dependencies]
chicago-tdd-tools = { path = "../chicago-tdd-tools" }
tokio = { version = "1.0", features = ["rt", "macros"] }
```

```rust
use chicago_tdd_tools::prelude::*;

chicago_test!(test_example, {
    // Arrange
    let input = 5;
    // Act
    let result = input * 2;
    // Assert
    assert_eq!(result, 10);
});
```

```bash
cargo make test
```

## Features

**Core**: Fixtures, builders, assertions, macros, performance testing, guards, JTBD validation, coverage.

**Optional** (enable in `Cargo.toml`): `property-testing`, `mutation-testing`, `snapshot-testing`, `fake-data`, `concurrency-testing`, `parameterized-testing`, `cli-testing`, `testcontainers`, `otel`, `weaver`, `async`, `benchmarking`, `workflow-engine`.

## Build System

Use `cargo make` commands (not `cargo` directly):

```bash
cargo make check      # Compile check
cargo make test       # Run tests
cargo make lint       # Run clippy
cargo make pre-commit # Pre-commit validation
```

## Documentation

- **[Quick Guide](docs/QUICK_GUIDE.md)** - Essential patterns (80% of use cases)
- **[Getting Started](docs/GETTING_STARTED.md)** - Quick start guide
- **[User Guide](docs/USER_GUIDE.md)** - Complete usage guide
- **[API Reference](docs/API_REFERENCE.md)** - API documentation
- **[Architecture](docs/ARCHITECTURE.md)** - Design principles
- **[Examples](examples/)** - Working code examples

## Requirements

Rust 1.70+, Tokio (for async), cargo-make (for build system). Docker optional for `testcontainers` feature.

## License

MIT
