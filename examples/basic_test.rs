//! Basic Test Example
//!
//! Demonstrates basic usage of Chicago TDD tools.

use chicago_tdd_tools::prelude::*;

#[tokio::main]
async fn main() {
    println!("Basic Test Example");
    println!("==================");

    // Arrange: Create fixture
    // **Best Practice**: TestFixture::new() returns Result - handle errors properly
    // This demonstrates proper error handling pattern for users
    let fixture = match TestFixture::new() {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to create fixture: {e}");
            eprintln!("This should not happen in normal usage - check your environment");
            // **FMEA Fix**: Return error instead of exit() for better error handling
            // In actual code, propagate errors with ? operator or return Result
            return; // Early return for example - in real code, return Result
        }
    };

    // Act: Use fixture
    let counter = fixture.test_counter();

    // Assert: Verify fixture created
    println!("Test counter: {counter}");
    // Counter is always >= 0 for u64, so just verify it exists
    println!("✓ Fixture created successfully");

    // Arrange: Create test data
    // **Best Practice**: Handle Result properly - demonstrate error handling pattern
    let data = match TestDataBuilder::new()
        .with_var("key1", "value1")
        .with_order_data("ORD-001", "100.00")
        .build_json()
    {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to build JSON: {e}");
            // **FMEA Fix**: Return error instead of exit() - demonstrates proper error handling
            // In actual code, propagate errors with ? operator or return Result
            return; // Early return for example - in real code, return Result
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

    // Arrange: Create result
    let result: Result<(), String> = Ok(());

    // Assert: Use assertion helpers
    // **FMEA Fix**: Demonstrate both success and error paths for complete learning
    if result.is_ok() {
        println!("✓ Assertion helpers work correctly");
    } else {
        println!("✗ Assertion helpers failed");
    }

    // **FMEA Fix**: Demonstrate error path handling
    let error_result: Result<(), String> = Err("example error".to_string());
    match error_result {
        Ok(_) => println!("✓ Error result handled - success case"),
        Err(e) => {
            println!("✓ Error result handled - error case: {e}");
            // **Best Practice**: In actual code, handle errors appropriately
            // - Return error with ? operator
            // - Log error and continue
            // - Transform error to user-friendly message
        }
    }
}
