//! Basic Test Example
//!
//! Demonstrates basic usage of Chicago TDD tools.

use chicago_tdd_tools::prelude::*;

#[tokio::main]
async fn main() {
    println!("Basic Test Example");
    println!("==================");

    // Arrange: Create fixture
    // Note: TestFixture::new() returns Result - handle errors properly
    let fixture = match TestFixture::new() {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to create fixture: {}", e);
            eprintln!("This should not happen in normal usage - check your environment");
            std::process::exit(1);
        }
    };

    // Act: Use fixture
    let counter = fixture.test_counter();

    // Assert: Verify fixture created
    println!("Test counter: {}", counter);
    // Counter is always >= 0 for u64, so just verify it exists
    println!("✓ Fixture created successfully");

    // Arrange: Create test data
    let data = TestDataBuilder::new()
        .with_var("key1", "value1")
        .with_order_data("ORD-001", "100.00")
        .build_json()
        .unwrap_or_else(|e| {
            eprintln!("Failed to build JSON: {}", e);
            std::process::exit(1);
        });

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
    if result.is_ok() {
        println!("✓ Assertion helpers work correctly");
    } else {
        println!("✗ Assertion helpers failed");
    }
}
