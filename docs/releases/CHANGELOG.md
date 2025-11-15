# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.2.0] - 2025-11-14

### Added
- **Coverage Enforcement**: Mandatory 85% line coverage enforcement (up from 70% warning in v1.1.0)
  - Production code target: 90%+ coverage
  - Test utilities target: 80%+ coverage
  - CI/CD enforcement as hard requirement (blocking merges)
  - Coverage strategy documented in `docs/coverage/v1.2.0-coverage-strategy.md`

### Changed
- **Coverage Threshold**: Increased from 70% (warning) to 85% (enforced)
  - 85% coverage catches 95% of bugs (Codecov data)
  - Aligns with industry best practices for testing frameworks
  - Achievable without excessive test maintenance burden
- **CI/CD**: Coverage checks now block merges if threshold not met
  - Hard requirement instead of warning-only
  - Supports Poka-Yoke design (error-proofing) through comprehensive error path testing

### Documentation
- Added coverage strategy documentation (`docs/coverage/v1.2.0-coverage-strategy.md`)
- Updated coverage enforcement guidelines
- Documented rationale for 85% threshold (coverage science, Chicago TDD context)

## [1.1.2] - 2025-11-14

### Fixed
- **Testcontainers**: Fixed compilation errors in testcontainers module
  - Added `Debug` derive for error types
  - Fixed string comparison logic in port tests
  - Improved Docker availability checking with timeout protection and retry logic
  - Enhanced error messages for Docker unavailability scenarios
- **Config Module**: Fixed config module test isolation
  - Changed working directory to isolate config tests
  - Added `tempfile` as dev-dependency for config module tests
  - Declared config module to resolve dead code warnings
- **Test Assertions**: Fixed `assert_that` usage to `assert_that_with_msg` for message arguments
  - Updated all test files to use correct assertion macro
  - Improved test error messages with descriptive assertions

### Enhanced
- **CI/CD Pipeline**: Comprehensive improvements to GitHub Actions workflows
  - FMEA improvements: OIDC authentication, path filters, artifact validation
  - Andon signals implementation for quality monitoring
  - Enhanced error handling and reporting
- **Testcontainers**: Improved Docker availability checking
  - Added timeout protection (5s) to prevent hanging when Docker is unavailable
  - Implemented retry logic with exponential backoff for parallel test execution
  - Better error messages with actionable guidance for Docker setup
- **Documentation**: Added comprehensive guides
  - Timeout enforcement documentation (`docs/features/TIMEOUT_ENFORCEMENT.md`)
  - Observability testing guide (`docs/observability/observability-testing-guide.md`)
  - OTEL/Weaver integration guide (`docs/observability/otel-weaver-guide.md`)
  - CLI testing guide (`docs/testing/cli-testing-guide.md`)
  - Coverage strategy documentation (`docs/coverage/v1.2.0-coverage-strategy.md`)

### Changed
- **Build System**: Improved build reliability
  - Enhanced timeout handling in testcontainers Docker checks
  - Better error messages for common failure scenarios
- **Code Quality**: Applied kaizen improvements
  - Extracted magic numbers to named constants (Docker check timeout, retry counts)
  - Improved error message consistency and clarity
  - Enhanced code documentation with kaizen improvement notes

### Documentation
- Added timeout enforcement guide explaining timeout SLAs and patterns
- Added comprehensive observability testing guides (OTEL and Weaver)
- Added CLI testing guide with examples
- Added coverage strategy documentation
- Updated code comments with kaizen improvement documentation

## [1.1.1] - 2025-11-14

### Enhanced
- **`test!` Macro**: Added `Result<(), E>` return type support
  - Tests can now return `Result<(), E>` and use the `?` operator for error propagation
  - Backward compatible: existing tests returning `()` continue to work unchanged
  - Automatic conversion between `()` and `Result<(), E>` return types
  - Improved ergonomics for testing fallible operations
- **`assert_fail!` Macro**: New convenience macro for testing error paths
  - Calls a function and asserts it returns `Err`, then returns the error value
  - Enables concise error path testing without intermediate variables
  - Works seamlessly with `test!` macro's new `Result` return type support
  - Example: `let error = assert_fail!(fallible_function());`

### Documentation
- Added examples demonstrating `Result` return type usage in `test!` macro
- Updated `async_test!` macro documentation to clarify Result return type support
- Added `assert_fail!` macro documentation and examples

## [1.1.0] - 2025-11-10

### Added
- **Weaver Integration**: OpenTelemetry live validation with Weaver (`weaver` feature)
  - `WeaverValidator` for lifecycle management (start/stop)
  - `send_test_span_to_weaver()` helper function for testing
  - Static schema validation via `validate_schema_static()`
  - Automatic Weaver binary download during build (when `weaver` feature enabled)
- **OTEL Validation**: OpenTelemetry span/metric validation (`otel` feature)
  - `SpanValidator` for span validation
  - `MetricValidator` for metric validation
  - Type-safe OTEL types (TraceId, SpanId, SpanContext, etc.)
- **Testcontainers Support**: Docker container integration testing (`testcontainers` feature)
  - Generic container support
  - Port mapping, environment variables, command execution
  - Wait conditions (HTTP health checks, log messages)
  - Automatic cleanup via `Drop` trait
- **Module Reorganization**: Modules organized into capability groups
  - `core/`: Core testing infrastructure
  - `testing/`: Advanced testing techniques
  - `validation/`: Quality & validation
  - `observability/`: Telemetry & observability
  - `integration/`: Integration testing
  - Backward compatibility maintained (all modules re-exported at crate root)
- **Dog Fooding**: Framework tests itself using its own tools
  - All framework tests use `chicago_test!` macros
  - Framework validates its own ergonomics through self-testing

### Changed
- Module organization: Modules moved into capability groups for better discoverability
- Build system: All commands use `cargo make` with timeout protection
- Documentation: Comprehensive documentation updates (README, guides, architecture)
- Test framework: All tests migrated to use `chicago_test!` macro
- Code quality: Extracted magic numbers to named constants for improved readability and maintainability
- Tooling: Retired the custom git hook installer; `cargo make` tasks (`pre-commit`, `check`, `lint`, `audit`) are now the canonical workflow

### Fixed
- Documentation: Updated outdated reports to reflect actual implementation status
- Test framework: Fixed test organization and consistency
- Dead code: Removed duplicate `andon.rs` module (393 lines) - `alert.rs` is the correct implementation

### Documentation
- Added comprehensive README with Chicago TDD principles and dog fooding
- Added architecture documentation
- Added user guides and quick start
- Added SPR (Sparse Priming Representation) methodology guide
- Added dog fooding documentation

## [1.0.0] - 2024-01-01

### Added
- Initial release
- Core testing framework
- Fixtures, builders, assertions
- Test macros
- Property-based testing
- Mutation testing


