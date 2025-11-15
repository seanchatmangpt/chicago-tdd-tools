# Chicago TDD Tools Playground

Comprehensive playground demonstrating all features of chicago-tdd-tools using a modern **noun-verb CLI architecture** powered by **clap-noun-verb 3.7.1**. This serves as both a validation suite and a reference implementation that showcases both the testing framework and best practices for building composable, type-safe CLI applications.

## Purpose

This playground serves three goals:
1. **Framework Validation**: Validates that all chicago-tdd-tools features work correctly for end users
2. **Reference Implementation**: Demonstrates best practices for building modern Rust CLIs using the noun-verb pattern
3. **Copyable Examples**: Provides examples that can be adapted for your own projects

## Architecture: Noun-Verb CLI Pattern

The playground demonstrates modern CLI design using **clap-noun-verb 3.7.1**, which enables:

### clap-noun-verb Features
- **Attribute Macros**: Zero-boilerplate `#[verb]` and `#[noun]` macros for command registration
- **Auto-Discovery**: Commands automatically registered at compile time via `linkme`
- **Type Inference**: Command arguments inferred from Rust function signatures
- **Auto-Serialization**: Return types automatically serialized to JSON (perfect for automation)
- **Multiple Formats**: JSON, YAML, TOML, Table, TSV output formats
- **Async Support**: Execute async operations from sync handlers with `run_async()`
- **Type Safety**: Compiler enforces command correctness at compile time
- **Composability**: Build complex CLIs by composing simple verb handlers

### Example: Noun-Verb Pattern
```
playg [NOUN] [VERB] [OPTIONS]
  playg core stat          # noun=core, verb=stat
  playg test exec          # noun=test, verb=exec
  playg gh check --fix     # noun=gh, verb=check
  playg obs otel           # noun=obs, verb=otel
```

## Features Demonstrated

### Framework Features (chicago-tdd-tools)

#### Core Features (Always Available)
- **Fixtures**: Test fixtures with state management and isolation
- **Async Fixtures**: Async fixture providers (requires `async` feature, Rust 1.75+)
- **Builders**: Fluent builders for test data
- **Assertions**: Comprehensive assertion utilities
- **Macros**: AAA pattern enforcement (`test!`, `async_test!`, `fixture_test!`, etc.)
- **State**: Type-level AAA pattern enforcement
- **Type Level**: Type-level programming with const generics
- **Const Assert**: Compile-time assertions
- **Alert**: Visual problem indicators

#### Testing Features (Optional)
- **Property Testing**: Property-based testing with const generics
- **Mutation Testing**: Test quality validation
- **Snapshot Testing**: Output comparison and regression testing
- **Concurrency Testing**: Deterministic thread model checking
- **CLI Testing**: Command-line tool testing with golden files
- **Generator**: Test code generation
- **Parameterized Testing**: Multiple inputs with rstest

#### Validation Features (Always Available)
- **Coverage**: Test coverage analysis and reporting
- **Guards**: Guard constraint enforcement (MAX_RUN_LEN ≤ 8, MAX_BATCH_SIZE)
- **JTBD**: Jobs To Be Done validation framework
- **Performance**: RDTSC benchmarking and tick measurement

#### Observability Features (Optional)
- **OTEL**: OpenTelemetry span/metric validation
- **Weaver**: Weaver live validation integration (requires `otel`)

#### Integration Features (Optional)
- **Testcontainers**: Docker container support for integration testing

## Usage

### CLI Commands: Noun-Verb Pattern with Type Inference

The playground provides a unified CLI using the `playg` command with noun-verb patterns. Each command automatically:
- **Infers arguments from Rust function signatures** (no enum boilerplate)
- **Serializes output to JSON by default** (perfect for scripting)
- **Supports multiple output formats** (JSON, YAML, TOML, Table, TSV)
- **Type-safe at compile time** (clap-noun-verb validates all arguments)

#### Core Features Commands
```bash
# Show status of all core features (returns JSON)
playg core stat

# Show with verbose output
playg core stat -v    # level 1 verbose
playg core stat -vv   # level 2 verbose (multiple -v flags)

# List available core examples
playg core list

# Execute one or more examples
playg core exec --names "fixtures"
playg core exec --names "fixtures builders assert"
```

#### Testing Features Commands
```bash
# Show testing features status
playg test stat

# List available test examples
playg test list

# Execute test examples
playg test exec --names "gen"
```

#### Validation Commands
```bash
# Show validation features status
playg valid stat

# Execute validation checks
playg valid exec --names "cov guard"
```

#### Observability Commands
```bash
# Show observability features
playg obs stat

# Run OTEL demo (if otel feature enabled)
playg obs otel

# Run Weaver demo (if weaver feature enabled)
playg obs weav
```

#### Integration Commands
```bash
# Show integration features
playg integ stat

# Run testcontainers demo (if testcontainers feature enabled)
playg integ contain
```

#### GitHub Actions Commands
```bash
# Show GitHub Actions status
playg gh stat

# List workflows (with format options)
playg gh list
playg gh list --format json
playg gh list --format paths

# Validate workflows
playg gh check
playg gh check --fix  # Auto-fix issues
playg gh check -vv    # Verbose mode

# Show recent workflow runs (requires 'gh' CLI)
playg gh runs --limit 5

# Open GitHub Actions in browser
playg gh open
```

### Output Formats

All commands support multiple output formats via the `--format` flag (where available):

```bash
# JSON (default) - Suitable for machines and scripting
playg core stat --format json

# YAML - Human-readable structured data
playg core stat --format yaml

# TOML - Configuration format
playg core stat --format toml

# Table - ASCII tables for terminals
playg core stat --format table

# TSV - Tab-separated values for spreadsheets
playg core stat --format tsv
```

### Type Inference in Action

clap-noun-verb infers command arguments from Rust function signatures:

```rust
#[verb]
fn stat(verbose: usize) -> Result<Status> {
    // verbose is automatically inferred as a count flag (-v, -vv, -vvv)
}

#[verb]
fn check(fix: bool, verbose: usize) -> Result<Vec<String>> {
    // fix is a boolean flag (--fix)
    // verbose is a count flag
}

#[verb]
fn list(format: Option<String>) -> Result<Vec<String>> {
    // format is an optional argument (--format <value>)
}
```

### JSON Output for Automation

All commands return JSON by default, perfect for scripts and automation:

```bash
$ playg core stat
{"features":["fixtures","async","builders",...], "examples":["fixtures","builders",...]}

$ playg gh stat
{"workflows":[{"name":"ci",...}], "total_workflows":5, "valid_workflows":5, "invalid_workflows":0}
```

Parse with standard tools:
```bash
# Extract field with jq
playg gh stat | jq '.valid_workflows'

# Convert to YAML
playg gh stat | jq -y 'to_entries | .[] | "\(.key): \(.value)"'
```

### Running Examples (Legacy)

```bash
# Run all examples (legacy method)
cargo run --bin playground

# Run specific example module (legacy method)
cargo run --example core::fixtures
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for specific feature
cargo test --features property-testing
cargo test --features snapshot-testing
cargo test --features testcontainers
```

### Feature Flags

All features are enabled by default in this playground. To test with specific features:

```bash
# Test with only core features
cargo test --no-default-features

# Test with specific feature groups
cargo test --features testing-extras
cargo test --features observability-full
cargo test --features integration-full
```

## Project Structure

```
playground/
├── Cargo.toml             # All features enabled
├── README.md              # This file (you are here)
├── GH_CLI_COMMANDS.md     # GitHub Actions CLI documentation
├── CLI_ARCHITECTURE.md    # Architecture guide for clap-noun-verb setup
├── EXTENDING_CLI.md       # Guide for adding new commands
├── PROJECT_CHARTER.md     # Project charter and goals
├── src/
│   ├── main.rs            # Entry point (calls clap_noun_verb::run())
│   ├── lib.rs             # Library exports
│   ├── cli/               # clap-noun-verb CLI commands (auto-discovered)
│   │   ├── mod.rs         # CLI module root (linkme registration)
│   │   ├── core.rs        # core noun commands
│   │   ├── test.rs        # test noun commands
│   │   ├── valid.rs       # validation noun commands
│   │   ├── obs.rs         # observability noun commands
│   │   ├── integ.rs       # integration noun commands
│   │   ├── gh.rs          # GitHub Actions noun commands
│   │   └── *.rs           # Additional noun modules (auto-discovered)
│   ├── core/              # Chicago-tdd-tools core feature examples
│   ├── testing/           # Testing feature examples
│   ├── validation/        # Validation feature examples
│   ├── observability/     # Observability feature examples
│   └── integration/       # Integration feature examples
└── tests/                 # Comprehensive test suite
```

### CLI Auto-Discovery Mechanism

The playground uses **clap-noun-verb's `linkme` integration** for automatic command discovery:

1. **Attribute Macros**: Each verb is decorated with `#[verb]` macro in `src/cli/*.rs`
2. **Auto-Registration**: The `linkme` crate automatically registers all verbs at compile time
3. **Module Registration**: `src/cli/mod.rs` explicitly imports CLI modules to trigger linkme
4. **Runtime Dispatch**: `clap_noun_verb::run()` in `main.rs` dispatches to the correct verb handler
5. **Zero Boilerplate**: No enums, match statements, or manual registration needed

## Examples by Category

### Core Features
- `src/core/fixtures.rs` - Test fixtures
- `src/core/async_fixtures.rs` - Async fixtures
- `src/core/builders.rs` - Test data builders
- `src/core/assertions.rs` - Assertion utilities
- `src/core/macros.rs` - Test macros
- `src/core/state.rs` - Type-level state
- `src/core/type_level.rs` - Type-level programming
- `src/core/const_assert.rs` - Compile-time assertions
- `src/core/alert.rs` - Alert helpers

### Testing Features
- `src/testing/property.rs` - Property-based testing
- `src/testing/mutation.rs` - Mutation testing
- `src/testing/snapshot.rs` - Snapshot testing
- `src/testing/concurrency.rs` - Concurrency testing
- `src/testing/cli.rs` - CLI testing
- `src/testing/generator.rs` - Test code generation
- `src/testing/parameterized.rs` - Parameterized testing

### Validation Features
- `src/validation/coverage.rs` - Coverage analysis
- `src/validation/guards.rs` - Guard constraints
- `src/validation/jtbd.rs` - JTBD validation
- `src/validation/performance.rs` - Performance validation

### Observability Features
- `src/observability/otel.rs` - OTEL validation
- `src/observability/weaver.rs` - Weaver validation

### Integration Features
- `src/integration/testcontainers.rs` - Docker containers

## Requirements

### Required
- Rust 1.70+ (1.75+ for `async` feature)
- Cargo
- cargo-make (for build system)

### Optional
- Docker (for `testcontainers` feature)
- Weaver binary (for `weaver` feature)

## Success Criteria

✅ All features demonstrated with working examples  
✅ All tests pass (100% success rate)  
✅ All features compile and run correctly  
✅ Playground serves as reference for end users  
✅ Clear examples and usage instructions  

## Copying Examples

All examples in this playground are designed to be copied and adapted for your projects. Each example demonstrates:

1. **Arrange**: Set up test data and context
2. **Act**: Execute the feature under test
3. **Assert**: Verify behavior and outputs

Follow the AAA pattern in all examples for consistency with Chicago TDD principles.

## Contributing

### Adding New Testing Examples

When adding new examples to the feature demonstrations:

1. Follow the **AAA pattern** (Arrange-Act-Assert)
2. Include comprehensive comments
3. Add tests that validate the example works
4. Update the README with new examples
5. Ensure all features compile and tests pass

### Adding New CLI Commands

When adding new verb handlers to extend the CLI:

1. **Create a new file** in `src/cli/` (e.g., `src/cli/yourfeature.rs`)
2. **Use #[verb] macros** for zero-boilerplate command registration:
   ```rust
   use clap_noun_verb_macros::verb;
   use clap_noun_verb::Result;
   use serde::Serialize;

   #[derive(Serialize)]
   struct MyOutput {
       status: String,
       count: usize,
   }

   /// Your command documentation (shows in --help)
   #[verb]
   fn mycommand(input: String, verbose: usize) -> Result<MyOutput> {
       // input is a required argument (--input <value>)
       // verbose is a count flag (-v, -vv, -vvv)
       Ok(MyOutput {
           status: input,
           count: verbose,
       })
   }
   ```

3. **Import the module** in `src/cli/mod.rs`:
   ```rust
   pub mod yourfeature;  // Auto-discovered by linkme!
   ```

4. **Return serializable types** - Your return type must implement `Serialize` for JSON output
5. **Use `Result<T>`** where `T: Serialize` for consistent error handling
6. **Test with different formats**:
   ```bash
   cargo run --bin playg -- yourfeature mycommand --input "test" --format json
   cargo run --bin playg -- yourfeature mycommand --input "test" --format yaml
   ```

See `EXTENDING_CLI.md` for detailed examples and best practices.

## License

MIT (same as chicago-tdd-tools)

