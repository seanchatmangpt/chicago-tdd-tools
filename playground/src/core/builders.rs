//! Builders Examples
//!
//! Demonstrates fluent builders for test data, including advanced builder types and derive macro.

use chicago_tdd_tools::prelude::*;
use chicago_tdd_tools::{GenericTestDataBuilder, ValidatedTestDataBuilder, TestBuilder};

/// Example: Basic builder
pub fn example_basic_builder() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange: Create builder
    let data = TestDataBuilder::new()
        .with_var("key1", "value1")
        .with_var("key2", "value2")
        .build_json()?;

    // Act: Access data
    let key1 = data["key1"].as_str();

    // Assert: Verify data built
    assert_eq!(key1, Some("value1"));
    Ok(())
}

/// Example: Builder with business helpers
pub fn example_business_builder() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange: Create builder with business data
    let data = TestDataBuilder::new()
        .with_order_data("ORD-001", "100.00")
        .with_customer_data("CUST-123")
        .build_json()?;

    // Act: Access business data
    let order_id = data["order_id"].as_str();
    let customer_id = data["customer_id"].as_str();

    // Assert: Verify business data
    assert_eq!(order_id, Some("ORD-001"));
    assert_eq!(customer_id, Some("CUST-123"));
    Ok(())
}

#[cfg(feature = "fake-data")]
/// Example: Builder with fake data
pub fn example_fake_data_builder() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange: Create builder with fake data
    let data = TestDataBuilder::new()
        .with_fake_email()
        .with_fake_name()
        .with_fake_uuid()
        .build_json()?;

    // Act: Access fake data
    let email = data["email"].as_str();
    let name = data["name"].as_str();
    let uuid = data["uuid"].as_str();

    // Assert: Verify fake data generated
    assert!(email.is_some());
    assert!(name.is_some());
    assert!(uuid.is_some());
    Ok(())
}

/// Example: Builder to HashMap
pub fn example_builder_hashmap() {
    // Arrange: Create builder
    let data = TestDataBuilder::new()
        .with_var("key1", "value1")
        .build();

    // Act: Access HashMap
    let value = data.get("key1");

    // Assert: Verify HashMap data
    assert_eq!(value, Some(&"value1".to_string()));
}

/// Example: GenericTestDataBuilder for generic key/value types
pub fn example_generic_builder() {
    // Arrange: Create generic builder
    let builder = GenericTestDataBuilder::<String, String>::new()
        .with_var("key1".to_string(), "value1".to_string())
        .with_var("key2", "value2"); // Works with &str too

    // Act: Build data
    let data = builder.build();

    // Assert: Verify generic builder works
    assert_eq!(data.get("key1"), Some(&"value1".to_string()));
    assert_eq!(data.get("key2"), Some(&"value2".to_string()));
}

#[cfg(feature = "otel")]
/// Example: ValidatedTestDataBuilder with type-level validation
pub fn example_validated_builder() {
    // Arrange: Create validated builder
    let builder = ValidatedTestDataBuilder::<()>::new()
        .with_var("key1", "value1")
        .with_var("key2", "value2");

    // Act: Build validated data
    let data = builder.build();

    // Assert: Verify validated builder works
    assert_eq!(data.get("key1"), Some(&"value1".to_string()));
    assert_eq!(data.get("key2"), Some(&"value2".to_string()));
}

/// Example struct for TestBuilder derive macro
#[derive(TestBuilder)]
pub struct OrderData {
    order_id: String,
    amount: String,
    customer_id: String,
}

/// Example: Using #[derive(TestBuilder)] for custom builders
pub fn example_derive_testbuilder() {
    // Arrange: Create builder using derive macro
    let builder = OrderDataBuilder::new()
        .with_order_id("ORD-001".to_string())
        .with_amount("100.00".to_string())
        .with_customer_id("CUST-123".to_string());

    // Act: Build order data
    let order = builder.build().unwrap();

    // Assert: Verify derived builder works
    assert_eq!(order.order_id, "ORD-001");
    assert_eq!(order.amount, "100.00");
    assert_eq!(order.customer_id, "CUST-123");
}

#[cfg(test)]
mod tests {
    use super::*;

    test!(test_basic_builder, {
        // Arrange-Act-Assert: Run example
        assert_ok!(example_basic_builder());
    });

    test!(test_business_builder, {
        // Arrange-Act-Assert: Run example
        assert_ok!(example_business_builder());
    });

    #[cfg(feature = "fake-data")]
    test!(test_fake_data_builder, {
        // Arrange-Act-Assert: Run example
        assert_ok!(example_fake_data_builder());
    });

    test!(test_builder_hashmap, {
        // Arrange-Act-Assert: Run example
        example_builder_hashmap();
    });

    test!(test_generic_builder, {
        // Arrange-Act-Assert: Run example
        example_generic_builder();
    });

    #[cfg(feature = "otel")]
    test!(test_validated_builder, {
        // Arrange-Act-Assert: Run example
        example_validated_builder();
    });

    test!(test_derive_testbuilder, {
        // Arrange-Act-Assert: Run example
        example_derive_testbuilder();
    });
}

