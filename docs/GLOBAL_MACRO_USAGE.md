# Using Chicago TDD Macros Globally

## Overview

The `chicago-tdd-tools` crate provides macros that can be used across the entire codebase to enforce Chicago TDD principles and reduce boilerplate.

## Quick Start

### 1. Add Dependency

Ensure `chicago-tdd-tools` is in your `Cargo.toml`:

```toml
[dependencies]
chicago-tdd-tools = { path = "../chicago-tdd-tools", version = "1.0.0" }
```

### 2. Import Macros

```rust
// Option 1: Import macros explicitly
use chicago_tdd_tools::{chicago_test, chicago_async_test, assert_ok, assert_err, assert_eq_msg};

// Option 2: Import prelude (includes helpers, but macros are auto-exported)
use chicago_tdd_tools::prelude::*;
// Macros are automatically available via #[macro_use]
```

### 3. Use Macros in Tests

```rust
// Synchronous test with AAA pattern
chicago_test!(test_basic_feature, {
    // Arrange: Set up test data
    let input = 5;
    
    // Act: Execute feature
    let result = input * 2;
    
    // Assert: Verify behavior
    assert_eq!(result, 10);
});

// Async test with AAA pattern
chicago_async_test!(test_async_feature, {
    // Arrange: Set up test data
    let fixture = TestFixture::new().unwrap();
    
    // Act: Execute async operation
    let result = async_function().await;
    
    // Assert: Verify behavior
    assert_ok!(&result, "Operation should succeed");
});

// Test with automatic fixture setup
chicago_fixture_test!(test_with_fixture, fixture, {
    // Arrange: Use provided fixture
    let counter = fixture.test_counter();
    
    // Act: Execute test
    let result = counter + 1;
    
    // Assert: Verify behavior
    assert!(result > 0);
});
```

## Available Macros

### Test Macros

- `chicago_test!` - Synchronous test with AAA pattern enforcement
- `chicago_async_test!` - Async test with AAA pattern enforcement
- `chicago_fixture_test!` - Async test with automatic fixture setup/teardown
- `chicago_performance_test!` - Performance test with tick budget validation

### Assertion Macros

- `assert_ok!` - Assert Result is Ok with detailed error messages
- `assert_err!` - Assert Result is Err with detailed error messages
- `assert_eq_msg!` - Assert equality with custom message
- `assert_in_range!` - Assert value is within range
- `assert_within_tick_budget!` - Validate performance constraints (≤8 ticks)
- `assert_guard_constraint!` - Validate guard constraints

## Refactoring Examples

### Before

```rust
#[tokio::test]
async fn test_workflow_registration() -> WorkflowResult<()> {
    // Arrange
    let state_store = StateStore::new("./test_db")?;
    let engine = WorkflowEngine::new(state_store);
    
    // Act
    let result = engine.register_workflow(spec).await;
    
    // Assert
    assert!(result.is_ok());
    assert_eq_with_msg(&result.unwrap(), &expected, "Should register workflow");
    Ok(())
}
```

### After

```rust
use chicago_tdd_tools::{chicago_async_test, assert_ok, assert_eq_msg};

chicago_async_test!(test_workflow_registration, {
    // Arrange
    let state_store = StateStore::new("./test_db").unwrap();
    let engine = WorkflowEngine::new(state_store);
    
    // Act
    let result = engine.register_workflow(spec).await;
    
    // Assert
    assert_ok!(&result, "Workflow registration should succeed");
    let spec_id = result.unwrap();
    assert_eq_msg!(&spec_id, &expected, "Should register workflow");
});
```

## Benefits

1. **Reduced Boilerplate** - Less code per test
2. **Consistent Patterns** - All tests follow AAA structure
3. **Better Error Messages** - Assertion macros provide context
4. **Performance Validation** - Built-in tick budget checking
5. **Guard Constraint Validation** - Easy validation of constraints like max_run_len ≤ 8

## Migration Guide

1. Replace `#[tokio::test]` with `chicago_async_test!`
2. Replace `#[test]` with `chicago_test!`
3. Replace `assert!(result.is_ok())` with `assert_ok!(&result)`
4. Replace `assert!(result.is_err())` with `assert_err!(&result)`
5. Replace `assert_eq_with_msg()` with `assert_eq_msg!()`

## Files Refactored

- ✅ `rust/chicago-tdd-tools/src/macros.rs` - Tests refactored
- ✅ `rust/knhk-workflow-engine/tests/chicago_tdd_tools_integration.rs` - Using macros
- ✅ `rust/knhk-workflow-engine/tests/chicago_tdd_framework_self_test.rs` - Using macros

## Next Steps

To use macros globally across the codebase:

1. Import macros in test files: `use chicago_tdd_tools::{chicago_test, assert_ok, ...};`
2. Replace test attributes with macro calls
3. Replace assertion helpers with macro versions
4. Run tests to verify everything works

The macros are production-ready and fully tested!




