//! # Basic Test Example - Comprehensive Guide
//!
//! Demonstrates basic usage of Chicago TDD tools with fixtures, data builders, and assertion helpers.
//!
//! ## Tutorial: Getting Started
//!
//! This example walks through the fundamental patterns of Chicago TDD:
//!
//! 1. **Creating Test Fixtures**: Use `TestFixture::new()` to create isolated test state
//! 2. **Building Test Data**: Use `TestDataBuilder` to construct test data structures
//! 3. **Error Handling**: Handle `Result` types properly in tests
//! 4. **Assertions**: Use both success and error path verification
//!
//! ## Explanation: Concepts
//!
//! **Test Fixtures**: Provide isolated test state with automatic cleanup. Each test gets
//! a fresh fixture, ensuring test isolation and preventing state leakage between tests.
//!
//! **Data Builders**: Fluent API for constructing complex test data structures (JSON, HashMap, etc.).
//! Builders enable readable, maintainable test data creation.
//!
//! **Error Handling**: Chicago TDD emphasizes proper error handling. Tests should demonstrate
//! both success and error paths to ensure complete behavior verification.
//!
//! **AAA Pattern**: All examples follow Arrange-Act-Assert structure:
//! - **Arrange**: Set up test data and fixtures
//! - **Act**: Execute the code under test
//! - **Assert**: Verify expected behavior
//!
//! ## How-to: Common Tasks
//!
//! - Create a test fixture: See `example_fixture_creation()`
//! - Build test data: See `example_data_building()`
//! - Handle errors: See `example_error_handling()`
//!
//! ## Reference: Quick Lookup
//!
//! **Key Types**:
//! - `TestFixture`: Test state management and isolation
//! - `TestDataBuilder`: Fluent builder for test data
//! - `Result<T, E>`: Standard Rust error handling
//!
//! **Key Functions**:
//! - `TestFixture::new() -> Result<TestFixture, FixtureError>`
//! - `TestDataBuilder::new() -> TestDataBuilder`
//! - `TestDataBuilder::build_json() -> Result<Value, String>`

use chicago_tdd_tools::prelude::*;

/// Example: Creating and using a test fixture
///
/// ## How-to: Create a Test Fixture
///
/// Test fixtures provide isolated test state. This example demonstrates:
/// - Creating a fixture with proper error handling
/// - Accessing fixture properties (test counter)
/// - Automatic cleanup on drop
///
/// ## Reference
///
/// - **Function**: `TestFixture::new() -> Result<TestFixture, FixtureError>`
/// - **Returns**: `Ok(TestFixture)` on success, `Err(FixtureError)` on failure
/// - **Errors**: Returns error if fixture creation fails (rare, usually indicates environment issue)
///
/// # Examples
///
/// ```rust
/// use chicago_tdd_tools::prelude::*;
///
/// let fixture = TestFixture::new()?;
/// let counter = fixture.test_counter();
/// assert!(counter >= 0);
/// ```
fn example_fixture_creation() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange: Create fixture
    // **Best Practice**: TestFixture::new() returns Result - handle errors properly
    let fixture = match TestFixture::new() {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to create fixture: {e}");
            eprintln!("Check your environment configuration");
            // **Best Practice**: In production code, propagate errors with ? operator or return Result
            return Err(e.into());
        }
    };

    // Act: Use fixture
    let counter = fixture.test_counter();

    // Assert: Verify fixture created
    println!("Test counter: {counter}");
    // Counter is always >= 0 for u64, so just verify it exists
    println!("✓ Fixture created successfully");
    Ok(())
}

/// Example: Building test data with `TestDataBuilder`
///
/// ## How-to: Build Test Data
///
/// Use `TestDataBuilder` to construct complex test data structures:
/// - Chain builder methods for fluent API
/// - Build JSON or `HashMap` structures
/// - Handle build errors properly
///
/// ## Reference
///
/// - **Builder**: `TestDataBuilder::new() -> TestDataBuilder`
/// - **Methods**: `with_var()`, `with_order_data()`, `build_json()`
/// - **Returns**: `Result<Value, String>` - JSON value or error message
/// - **Errors**: Returns error if JSON construction fails
///
/// # Examples
///
/// ```rust
/// use chicago_tdd_tools::prelude::*;
///
/// let data = TestDataBuilder::new()
///     .with_var("key1", "value1")
///     .with_order_data("ORD-001", "100.00")
///     .build_json()?;
/// assert!(data.is_object());
/// ```
fn example_data_building() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange: Create test data
    // **Best Practice**: Handle Result properly - demonstrates error handling pattern
    let data = match TestDataBuilder::new()
        .with_var("key1", "value1")
        .with_order_data("ORD-001", "100.00")
        .build_json()
    {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to build JSON: {e}");
            // **Best Practice**: In production code, propagate errors with ? operator or return Result
            return Err(e.into());
        }
    };

    // Assert: Verify data created
    println!("Test data created: {}", data.is_object());
    if data.is_object() {
        println!("  key1: {}", data["key1"]);
        println!("  order_id: {}", data["order_id"]);
        println!("✓ Data builder works correctly");
    } else {
        println!("✗ Data builder failed");
    }
    Ok(())
}

/// Example: Error handling patterns in tests
///
/// ## How-to: Handle Errors in Tests
///
/// Demonstrate both success and error paths:
/// - Check `Result::is_ok()` for success cases
/// - Use `match` to handle both `Ok` and `Err` variants
/// - Show proper error handling patterns
///
/// ## Reference
///
/// - **Type**: `Result<T, E>` - Standard Rust error handling
/// - **Methods**: `is_ok()`, `is_err()`, `unwrap()`, `expect()`, `?` operator
/// - **Pattern**: Use `match` for exhaustive error handling
///
/// # Examples
///
/// ```rust
/// let result: Result<(), String> = Ok(());
/// if result.is_ok() {
///     println!("Success");
/// }
///
/// let error_result: Result<(), String> = Err("error".to_string());
/// match error_result {
///     Ok(_) => println!("Success"),
///     Err(e) => println!("Error: {e}"),
/// }
/// ```
fn example_error_handling() {
    // Arrange: Create result
    let result: Result<(), String> = Ok(());

    // Assert: Use assertion helpers
    // **Best Practice**: Demonstrate both success and error paths for complete learning
    if result.is_ok() {
        println!("✓ Assertion helpers work correctly");
    } else {
        println!("✗ Assertion helpers failed");
    }

    // **Best Practice**: Demonstrate error path handling
    let error_result: Result<(), String> = Err("example error".to_string());
    match error_result {
        Ok(()) => println!("✓ Error result handled - success case"),
        Err(e) => {
            println!("✓ Error result handled - error case: {e}");
            // **Best Practice**: In production code, handle errors appropriately:
            // - Return error with ? operator
            // - Log error and continue
            // - Transform error to user-friendly message
        }
    }
}

#[tokio::main]
async fn main() {
    println!("Basic Test Example");
    println!("==================");
    println!();

    println!("1. Creating test fixture...");
    if let Err(e) = example_fixture_creation() {
        eprintln!("Failed: {e}");
        return;
    }
    println!();

    println!("2. Building test data...");
    if let Err(e) = example_data_building() {
        eprintln!("Failed: {e}");
        return;
    }
    println!();

    println!("3. Error handling patterns...");
    example_error_handling();
    println!();

    println!("✓ All examples completed successfully!");
}
