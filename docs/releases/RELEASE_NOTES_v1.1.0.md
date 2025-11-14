# Release Notes: v1.1.0

## Summary

v1.1.0 adds Weaver integration for OpenTelemetry live validation, OTEL validation capabilities, testcontainers support, and comprehensive module reorganization. All features are production-ready with full test coverage.

## New Features

### Weaver Integration (`weaver` feature)

OpenTelemetry live validation with Weaver for schema validation and telemetry verification.

**Key capabilities**:
- `WeaverValidator`: Lifecycle management for Weaver live-check
- `send_test_span_to_weaver()`: Helper function for sending test telemetry
- Static schema validation via `validate_schema_static()`
- Automatic Weaver binary download during build

**Usage**:
```rust
use chicago_tdd_tools::observability::weaver::WeaverValidator;

let validator = WeaverValidator::new()
    .with_registry_path("./registry")
    .start()?;

// Use validator for live-check validation
let endpoint = validator.otlp_endpoint();
// ... send telemetry to endpoint ...

validator.stop()?;
```

### OTEL Validation (`otel` feature)

OpenTelemetry span and metric validation with type-safe types.

**Key capabilities**:
- `SpanValidator`: Validate OpenTelemetry spans
- `MetricValidator`: Validate OpenTelemetry metrics
- Type-safe OTEL types (TraceId, SpanId, SpanContext, etc.)

### Testcontainers Support (`testcontainers` feature)

Docker container integration testing with automatic cleanup.

**Key capabilities**:
- Generic container support
- Port mapping, environment variables, command execution
- Wait conditions (HTTP health checks, log messages)
- Automatic cleanup via `Drop` trait

### Module Reorganization

Modules organized into capability groups for better discoverability:
- `core/`: Core testing infrastructure
- `testing/`: Advanced testing techniques
- `validation/`: Quality & validation
- `observability/`: Telemetry & observability
- `integration/`: Integration testing

**Backward compatibility**: All modules re-exported at crate root. Existing code continues to work.

### Dog Fooding

Framework tests itself using its own tools, validating framework ergonomics through self-testing.

## Improvements

- Module organization: Better discoverability with capability groups
- Build system: All commands use `cargo make` with timeout protection
- Documentation: Comprehensive updates (README, guides, architecture)
- Test framework: All tests migrated to use `chicago_test!` macro
- Code quality: Extracted magic numbers to named constants (test exit codes, benchmark warmup iterations, percentile calculations, property test cases) for improved readability and maintainability
- Dead code elimination: Removed duplicate `andon.rs` module (393 lines) - `alert.rs` is the correct implementation
- Tooling: Retired the git hook installer script; developers run `cargo make pre-commit`, `cargo make check`, `cargo make lint`, and related tasks directly

## Bug Fixes

- Documentation: Updated outdated reports to reflect actual implementation status
- Test framework: Fixed test organization and consistency
- Dead code: Removed duplicate `andon.rs` module - `alert.rs` is the correct implementation

## Breaking Changes

None. This is a minor release with backward compatibility maintained.

## Migration Guide

No migration needed. All existing code continues to work. New features are opt-in via feature flags.

## Requirements

- Rust 1.70+ (Edition 2021)
- `cargo-make` for build system
- Docker (optional, for `testcontainers` feature)
- Weaver binary (automatically downloaded when `weaver` feature enabled)

## Documentation

- [Quick Guide](docs/QUICK_GUIDE.md)
- [Getting Started](docs/GETTING_STARTED.md)
- [User Guide](docs/USER_GUIDE.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Dog Fooding](docs/DOG_FOODING.md)

## Test Results

- **Total tests**: 257
- **Passed**: 257 (100%)
- **Skipped**: 10 (testcontainers - excluded from unit tests by design for speed, run via `cargo make test-integration`)
- **Integration tests**: Run separately via `cargo make test-integration` when Docker is available

## Known Issues

- **Test organization**: `test-unit` excludes integration tests by design for fast iteration. Use `cargo make test-integration` to run integration tests.
- **Testcontainers tests**: Run via `cargo make test-integration` task, not included in `test-unit` for speed.


