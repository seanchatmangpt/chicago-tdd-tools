# Dog Fooding - SPR

Using the testing framework to test itself. Every feature tested using the framework itself.

## Why This Matters

**Confidence**: If framework can test itself, it can test anything. Dogfooding proves framework works.

**Real-World Validation**: Using framework to test itself validates real scenarios, ergonomic APIs, practical patterns.

**Continuous Improvement**: Testing framework with itself reveals pain points, missing features, areas for improvement.

## How Framework Tests Itself

**Macros Test Macros**: `test!` tested using itself. `assert_ok!` tested using itself. AAA pattern enforced.

**Assertions Test Assertions**: Assertion macros tested using themselves. Error messages validated. Framework validates own behavior.

**Fixtures Test Fixtures**: `TestFixture` tested using itself. `fixture_test!` tested using itself. Automatic cleanup validated.

**Testcontainers Test Testcontainers**: Real Docker containers test the module. `assert_ok!` works with testcontainers Results. Real collaborators validated.

**Weaver Tests Weaver**: Real Weaver container tests integration. Testcontainers + Weaver integration validated. Complex integrations tested.

**Builders Test Builders**: Builder pattern tested using builders. Fluent API validated. Framework tests own patterns.

## Meta-Testing

**Macro Tests**: Tests verify macros expand correctly. Compilation success validates expansion.

**Framework Tests**: Tests verify framework components work. Default values validated. Configuration validated.

**Integration Tests**: Tests verify framework works with real dependencies. Docker, Weaver validated. Real-world scenarios tested.

**Self-Validation**: Framework validates own behavior. State verification. Behavior verification.

## Chicago TDD Principles in Self-Testing

**State-Based Testing**: Framework tests verify framework state (macros expand, fixtures work, builders build). Counter validation. Data validation.

**Real Collaborators**: Framework tests use real dependencies (Docker, Weaver, actual containers). No mocks. Real-world validation.

**Behavior Verification**: Framework tests verify what framework does (macros work, fixtures provide isolation, builders create data). Observable outputs verified.

**AAA Pattern**: All framework tests follow AAA pattern. Arrange-Act-Assert structure enforced.

## Benefits

**Confidence**: If framework can test itself, it can test anything. Dogfooding proves framework works.

**Real-World Validation**: Using framework to test itself validates real usage. If it works for testing framework, it works for testing other code.

**Continuous Improvement**: Self-testing reveals pain points. If feature hard to use in self-tests, it's hard to use in general.

**Documentation by Example**: Self-tests serve as examples. Users see how to use framework by looking at self-tests.

## Dogfooding Cycle

**1. Implement Feature**: Implement new feature (e.g., `test!` macro).

**2. Test with Framework**: Test feature using framework itself. `test!(test_new_feature, { /* test */ })`.

**3. Verify Behavior**: Verify feature works correctly (test passes).

**4. Use in Other Tests**: Use feature to test other parts of framework.

**5. Iterate**: If feature hard to use, improve based on self-testing experience.

## Summary

**Key Associations**: Dogfooding = Self-Testing = Confidence. Framework Tests Framework = Real-World Validation = Continuous Improvement. Meta-Testing = Self-Validation = Documentation by Example.

**Pattern**: All features tested using framework itself. Macros test macros. Assertions test assertions. Fixtures test fixtures. Testcontainers test testcontainers. Weaver tests weaver. Builders test builders.

**Conclusion**: Framework uses own principles to test itself, proving they work. If it can test itself, it can test anything.
