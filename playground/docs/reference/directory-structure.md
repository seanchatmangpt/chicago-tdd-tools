# Directory Structure Reference

**Complete breakdown** of the playground directory structure and file organization.

## Root Directory Layout

```
playground/
├── Cargo.toml              # Project manifest
├── Cargo.lock              # Dependency lock file
├── README.md               # Main project documentation
├── PROJECT_CHARTER.md      # Project goals and vision
├── GH_CLI_COMMANDS.md      # GitHub CLI integration notes
├── docs/                   # Diataxis-structured documentation
├── src/                    # Source code
├── tests/                  # Integration tests
└── .gitignore              # Git ignore rules
```

## Documentation Structure (`docs/`)

```
docs/
├── README.md               # Documentation hub
├── tutorials/              # Learning-oriented guides
│   ├── getting-started.md
│   ├── running-core-examples.md
│   ├── running-feature-examples.md
│   └── copying-examples.md
├── how-to/                 # Problem-oriented guides
│   ├── core-features.md
│   ├── testing-features.md
│   ├── validation-features.md
│   ├── observability-features.md
│   ├── integration-features.md
│   ├── feature-flags.md
│   ├── json-output.md
│   └── adding-examples.md
├── reference/              # Lookup-oriented reference
│   ├── cli-commands.md
│   ├── directory-structure.md (this file)
│   ├── example-inventory.md
│   ├── feature-matrix.md
│   ├── configuration.md
│   └── json-schema.md
└── explanation/            # Understanding-oriented deep dives
    ├── playground-philosophy.md
    ├── noun-verb-pattern.md
    ├── feature-organization.md
    ├── aaa-pattern.md
    ├── example-lifecycle.md
    └── testing-philosophy.md
```

## Source Code Structure (`src/`)

```
src/
├── lib.rs                  # Library root
├── main.rs                 # CLI entry point
├── cli/                    # CLI command handlers
│   ├── mod.rs
│   ├── core.rs             # Core feature commands
│   ├── test.rs             # Testing feature commands
│   ├── valid.rs            # Validation commands
│   ├── obs.rs              # Observability commands
│   ├── integ.rs            # Integration commands
│   └── gh.rs               # GitHub integration
├── core/                   # Core feature examples
│   ├── mod.rs
│   ├── fixtures.rs
│   ├── builders.rs
│   ├── assertions.rs
│   ├── macros.rs
│   ├── state.rs
│   ├── const_assert.rs
│   └── alert.rs
├── testing/                # Testing feature examples
│   ├── mod.rs
│   ├── property.rs         # Property-based testing
│   ├── mutation.rs         # Mutation testing
│   ├── snapshot.rs         # Snapshot testing
│   ├── concurrency.rs      # Concurrency testing
│   ├── cli.rs              # CLI testing
│   └── generator.rs        # Test generation
├── validation/             # Validation feature examples
│   ├── mod.rs
│   ├── coverage.rs
│   ├── guards.rs
│   ├── jtbd.rs
│   └── performance.rs
├── observability/          # Observability examples
│   ├── mod.rs
│   ├── otel.rs             # OpenTelemetry
│   └── weaver.rs           # Weaver validation
└── integration/            # Integration examples
    ├── mod.rs
    └── testcontainers.rs   # Docker containers
```

## Key Files Explained

### Cargo.toml

```toml
[package]
name = "chicago-tdd-tools-playground"
version = "1.0.0"

[dependencies]
chicago-tdd-tools = { path = "../", features = ["testing-extras"] }

[dev-dependencies]
# Testing framework dependencies

[features]
# Feature flags
default = ["all-features"]
all-features = ["testing-full", "observability-full", "integration-full"]
```

### src/main.rs

Entry point for CLI application. Routes commands to appropriate handlers in `src/cli/`.

### src/lib.rs

Library root. Re-exports all modules:

```rust
pub mod core;
pub mod testing;
pub mod validation;
pub mod observability;
pub mod integration;
pub mod cli;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
```

### src/cli/mod.rs

CLI command routing and argument parsing:

```rust
pub async fn run() -> Result<()> {
    // Parse arguments
    // Route to appropriate handler
}
```

## Feature-Specific Directories

### src/core/ - Core Features

Always available. No feature flags needed.

**Files:**
- `fixtures.rs` - Test fixture examples
- `builders.rs` - Builder pattern examples
- `assertions.rs` - Assertion helpers
- `macros.rs` - Test macros (test!, async_test!, etc.)
- `state.rs` - Type-level state machines
- `const_assert.rs` - Compile-time assertions
- `alert.rs` - Alert macros

**CLI access:** `cargo run -- core exec --names "fixtures"`

### src/testing/ - Advanced Testing

Optional features (property, mutation, snapshot, etc.)

**Files:**
- `property.rs` - Property-based testing
- `mutation.rs` - Mutation testing
- `snapshot.rs` - Snapshot testing
- `concurrency.rs` - Concurrency testing
- `cli.rs` - CLI testing
- `generator.rs` - Test code generation

**CLI access:** `cargo run --all-features -- test exec --names "prop"`

### src/validation/ - Validation Features

Code quality and constraint validation.

**Files:**
- `coverage.rs` - Test coverage
- `guards.rs` - Constraint guards
- `jtbd.rs` - Jobs to be done
- `performance.rs` - Performance testing

**CLI access:** `cargo run -- valid exec --names "cov"`

### src/observability/ - Observability Features

OTEL and Weaver integration.

**Files:**
- `otel.rs` - OpenTelemetry validation
- `weaver.rs` - Semantic conventions

**CLI access:** `cargo run --features otel -- obs otel`

### src/integration/ - Integration Features

Docker and container support.

**Files:**
- `testcontainers.rs` - Docker integration

**CLI access:** `cargo run --features testcontainers -- integ contain`

## Tests Structure (`tests/`)

```
tests/
├── integration_tests.rs    # Integration test suite
└── common.rs               # Shared test utilities
```

## Adding New Files

### To Add a Core Example

1. Create `src/core/new_example.rs`
2. Implement `pub fn run() -> Result<T>`
3. Add to `src/core/mod.rs`
4. Register in `src/cli/core.rs`
5. Test: `cargo run -- core exec --names "new_example"`

### To Add a Testing Example

1. Create `src/testing/new_example.rs`
2. Implement with feature gate: `#[cfg(feature = "...")]`
3. Add to `src/testing/mod.rs`
4. Register in `src/cli/test.rs`
5. Test: `cargo run --all-features -- test exec --names "new_example"`

### To Add Documentation

1. Create file in appropriate `docs/` subdirectory
2. Update `docs/README.md` with link
3. Link to related documents

## File Naming Conventions

- **Source files:** `snake_case.rs` (e.g., `property_testing.rs`)
- **Documentation files:** `kebab-case.md` (e.g., `cli-commands.md`)
- **Test files:** `test_*.rs` or `*_test.rs`

## Module Organization

### Example: core/mod.rs

```rust
pub mod fixtures;
pub mod builders;
pub mod assertions;
pub mod macros;
pub mod state;
pub mod const_assert;
pub mod alert;

pub use fixtures::*;
pub use builders::*;
// ... re-export all
```

### Example: cli/mod.rs

```rust
pub mod core;
pub mod test;
pub mod valid;
pub mod obs;
pub mod integ;

pub async fn run() -> Result<()> {
    // Main CLI entry
}
```

## Build Output

```
target/
├── debug/                  # Debug builds
│   └── playground          # Debug binary
└── release/                # Release builds
    └── playground          # Release binary
```

## Documentation Build Output

```
target/
└── doc/                    # Generated API docs (cargo doc)
    └── chicago_tdd_tools/
```

## Import Conventions

### In Examples

```rust
// Always include prelude
use chicago_tdd_tools::prelude::*;

// Feature-gated imports
#[cfg(feature = "property-testing")]
use proptest::prelude::*;

// Standard library
use std::time::Instant;
use std::collections::HashMap;

// Serialization
use serde::{Serialize, Deserialize};
```

### In CLI Handlers

```rust
use crate::Result;
use serde_json;
use std::error::Error;
```

## Next Steps

- **See all examples** → [Example Inventory](example-inventory.md)
- **View feature matrix** → [Feature Matrix](feature-matrix.md)
- **Add new examples** → [How to Add Examples](../how-to/adding-examples.md)

---

Understand the playground organization to extend it effectively.
