# Coding Standards - Chicago TDD Tools

This document defines the coding standards enforced across the Chicago TDD Tools codebase to eliminate Mura (unevenness) and maintain consistency.

## Code Style Standards

### Formatting
- **Tool**: `cargo make fmt` (uses `rustfmt`)
- **Enforcement**: CI/CD pipeline fails if code is not formatted
- **Standard**: Rust standard formatting (enforced by rustfmt)
- **Action**: Run `cargo make fmt` before committing

### Naming Conventions
- **Functions**: `snake_case` (Rust convention)
- **Types**: `PascalCase` (Rust convention)
- **Constants**: `SCREAMING_SNAKE_CASE` (Rust convention)
- **Modules**: `snake_case` (Rust convention)
- **Enforcement**: Clippy checks naming conventions

### Import Organization
- **Order**: Alphabetical within groups
- **Groups**: std → external → local
- **Enforcement**: Clippy checks import organization

## Error Handling Standards

### Production Code
- **Pattern**: `Result<T, E>` with project-specific error types
- **Propagation**: Use `?` operator for error propagation
- **Prohibited**: `unwrap()`, `expect()` in production code
- **Enforcement**: Clippy denies `unwrap_used` and `expect_used` in production
- **Detection**: Pre-commit hooks + CI enforcement

### Test Code
- **Allowed**: `unwrap()`/`expect()` in test code (with justification if needed)
- **Preferred**: Use `assert_ok!()` and `assert_err!()` macros for better error messages
- **Pattern**: Test code can use `unwrap()` for test setup/teardown
- **After `assert_ok!()`**: Use descriptive `.expect()` messages that explain what is being unwrapped
  - **Pattern**: `assert_ok!()` verifies `Ok`, `.expect()` unwraps with context
  - **Example**: `result.expect("Exec result should be available after assert_ok verification")`
  - **Rationale**: More descriptive error messages help debug test failures

### Error Types
- **Library**: Use `thiserror` for error types
- **Pattern**: Custom error types per module/domain
- **Example**: `FixtureError`, `TestcontainersError`, `ConfigError`

## Documentation Standards

### Public APIs
- **Requirement**: All public functions, structs, enums, traits, types must have doc comments
- **Format**: Use `///` for item documentation
- **Structure**: 
  - Brief description
  - `# Arguments` section (if applicable)
  - `# Returns` section (if applicable)
  - `# Errors` section (if applicable)
  - `# Examples` section (for complex APIs)

### Modules
- **Requirement**: All modules must have module-level documentation
- **Format**: Use `//!` for module documentation
- **Content**: Explain module purpose, key concepts, usage patterns

### Examples
- **Requirement**: Public APIs should include usage examples
- **Format**: Code blocks in doc comments
- **Verification**: Examples should compile (use `rustdoc --test`)

## Testing Standards

### Macro Import Patterns
- **Root-level test modules**: Don't import macros - use directly (e.g., `assert_ok!(result)`)
  - **Rationale**: Macros exported with `#[macro_export]` are available at crate root automatically
  - **Example**: `tests/go_extra_mile_tests.rs` - uses `assert_ok!()` directly without import
- **Nested test modules**: Use macro wrappers that delegate to crate root
  - **Rationale**: Nested modules can't access crate root macros directly
  - **Example**: `tests/testcontainers/tests.rs` - uses `macro_rules! assert_ok { ... }` wrapper
- **Examples**: Use full path (e.g., `chicago_tdd_tools::assert_ok!(result)`)
  - **Rationale**: Examples are separate compilation units, need full path
  - **Example**: `examples/go_extra_mile.rs` - uses `chicago_tdd_tools::assert_ok!()`
- **Prohibited**: Importing macros with `use` causes "unused import" errors
- **Enforcement**: Code review checklist enforces import pattern

### Test Patterns
- **Macro**: Use `test!` macro consistently (not `#[test]`)
- **Organization**: AAA pattern (Arrange-Act-Assert)
- **Comments**: Include `// Arrange`, `// Act`, `// Assert` comments
- **Verification**: Verify behavior, not just function existence

### Test Coverage
- **Target**: Minimum 80% coverage per module
- **Measurement**: Use `cargo llvm-cov` for coverage
- **Enforcement**: CI tracks coverage, alerts on drops

### Test Isolation
- **Requirement**: Tests must be isolated (no shared mutable state)
- **Pattern**: Use `TestFixture` for test state
- **Resources**: Use RAII types (TempDir, NamedTempFile) for cleanup
- **Verification**: Code review checklist enforces isolation

## Pattern Standards

### Error Handling Pattern
- **Standard**: `Result<T, E>` with project error types
- **Reference**: `src/core/fixture.rs` (uses `Result<TestFixture, FixtureError>`)
- **Propagation**: Use `?` operator
- **Alternatives**: `if let`, `match` for conditional handling

### Validation Pattern
- **Type-level**: Use poka-yoke types when possible (compile-time validation)
- **Runtime**: Use guards and validated types when necessary
- **Reference**: `src/core/config/poka_yoke.rs` (type-level validation)
- **Pattern**: Type-level when possible, runtime when necessary

### Builder Pattern
- **Standard**: Fluent builder pattern for complex data structures
- **Reference**: `src/core/builders.rs` (TestDataBuilder, GenericTestDataBuilder)
- **Pattern**: Method chaining, `build()` method returns validated type

## Quality Standards

### Code Quality
- **Clippy**: All warnings treated as errors (`-D warnings`)
- **Enforcement**: CI fails on clippy warnings
- **Allows**: Use `#[allow(clippy::...)]` with justification comments
- **Pattern**: `#[allow(clippy::lint_name)] // Justification: why this is acceptable`

### Error Handling Quality
- **Production**: No `unwrap()`/`expect()` (enforced by clippy)
- **Test code**: `unwrap()`/`expect()` acceptable (documented exception)
- **Coverage**: All error paths must be tested

### Documentation Quality
- **Coverage**: 100% of public APIs must be documented
- **Style**: Consistent documentation style across modules
- **Examples**: Include usage examples for public APIs

## Automated Enforcement

### CI/CD Pipeline
- **Formatting**: `cargo make fmt` check (fails if not formatted)
- **Linting**: `cargo make lint` (fails on clippy warnings)
- **Tests**: `cargo make test` (fails on test failures)
- **Coverage**: Coverage tracking and alerts
- **unwrap/expect**: Automated check for production code

### Pre-commit Hooks
- **Installation**: `cargo make install-hooks`
- **Checks**: Format, lint, unwrap/expect detection
- **Enforcement**: Hooks prevent committing non-compliant code

### Code Review
- **Checklist**: See `docs/process/CODE_REVIEW_CHECKLIST.md`
- **Standards**: All standards must be verified in code review
- **Enforcement**: PR cannot merge if standards not met

## Reference Implementations

### Error Handling
- **Reference**: `src/core/fixture.rs`
- **Pattern**: `Result<TestFixture, FixtureError>`
- **Usage**: Proper error propagation with `?` operator

### Test Patterns
- **Reference**: `tests/go_extra_mile_tests.rs`
- **Pattern**: AAA pattern with `test!` macro
- **Usage**: Behavior verification, not just function existence

### Documentation
- **Reference**: `src/core/fixture.rs`
- **Pattern**: Comprehensive doc comments with examples
- **Usage**: All public APIs documented with examples

### Builder Pattern
- **Reference**: `src/core/builders.rs`
- **Pattern**: Fluent builder with validation
- **Usage**: Method chaining, validated output

## Consistency Maintenance

### Regular Audits
- **Frequency**: Weekly or monthly
- **Process**: Run consistency checks, identify new inconsistencies
- **Action**: Apply standards, update controls if needed

### Automated Checks
- **CI/CD**: Enforces standards automatically
- **Pre-commit**: Prevents non-compliant code from being committed
- **Code Review**: Verifies standards are met

### Documentation Updates
- **When**: When standards change or new patterns emerge
- **Where**: Update this document and related guides
- **Verification**: Ensure all documentation reflects current standards

## Summary

**Code Style**: Format with `cargo make fmt`, use Rust naming conventions
**Error Handling**: `Result<T, E>` in production, `unwrap()`/`expect()` only in tests
**Documentation**: 100% coverage for public APIs, consistent style
**Testing**: `test!` macro, AAA pattern, 80% minimum coverage
**Patterns**: Consistent error handling, validation, builder patterns
**Enforcement**: CI/CD, pre-commit hooks, code review checklist

**Key Principle**: Consistency is more important than perfection. Consistent, good code is easier to understand and maintain than perfect code in some places and poor code in others.

