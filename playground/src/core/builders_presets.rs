//! v1.3.0 Builder Presets and Validation Hooks Examples
//!
//! Demonstrates builder presets for reusable test data and validation hooks for ensuring data validity.

use chicago_tdd_tools::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BuilderPresetsResult {
    pub presets_registered: usize,
    pub presets_used: usize,
    pub validations_performed: usize,
    pub fake_data_examples: usize,
}

pub fn run() -> crate::Result<BuilderPresetsResult> {
    let mut presets_registered = 0;
    let mut presets_used = 0;
    let mut validations_performed = 0;
    let mut fake_data_examples = 0;

    // ========================================================================
    // 1. BUILDER PRESETS - Reusable test data configurations
    // ========================================================================
    {
        // Example 1a: Register a "valid_user" preset
        TestDataBuilder::register_preset("valid_user", |builder| {
            builder
                .with_var("username", "alice")
                .with_var("email", "alice@example.com")
                .with_var("age", "25")
                .with_var("is_active", "true")
        })
        .expect("Failed to register valid_user preset");
        presets_registered += 1;

        // Example 1b: Use the preset
        let user = TestDataBuilder::preset("valid_user")?
            .build();
        presets_used += 1;

        // Verify preset worked
        assert_eq!(user.get("username"), Some(&"alice".to_string()));
        assert_eq!(user.get("email"), Some(&"alice@example.com".to_string()));

        // Example 1c: Override specific fields in preset
        let admin_user = TestDataBuilder::preset("valid_user")?
            .with_var("role", "admin")
            .with_var("username", "admin_alice")
            .build();
        presets_used += 1;

        assert_eq!(admin_user.get("username"), Some(&"admin_alice".to_string()));
        assert_eq!(admin_user.get("role"), Some(&"admin".to_string()));
        // But email still comes from preset
        assert_eq!(admin_user.get("email"), Some(&"alice@example.com".to_string()));
    }

    // ========================================================================
    // 2. BUILDER VALIDATION HOOKS - Ensure data validity at build time
    // ========================================================================
    {
        // Example 2a: Simple validation hook for email format
        let valid_user = TestDataBuilder::new()
            .with_var("username", "bob")
            .with_var("email", "bob@example.com")
            .with_var("age", "30")
            .with_validation(|data| {
                // Validate email contains @
                if let Some(email) = data.get("email") {
                    if !email.contains('@') {
                        return Err("Email must contain @".to_string());
                    }
                }
                // Validate age is numeric and positive
                if let Some(age_str) = data.get("age") {
                    match age_str.parse::<u32>() {
                        Ok(age) if age >= 18 && age <= 150 => Ok(()),
                        Ok(_) => Err("Age must be between 18 and 150".to_string()),
                        Err(_) => Err("Age must be numeric".to_string()),
                    }
                } else {
                    Err("Age is required".to_string())
                }
            })
            .try_build()
            .expect("Valid user should build successfully");
        validations_performed += 1;

        assert_eq!(valid_user.get("username"), Some(&"bob".to_string()));
        assert_eq!(valid_user.get("email"), Some(&"bob@example.com".to_string()));

        // Example 2b: Complex validation with multiple rules
        let order = TestDataBuilder::new()
            .with_var("order_id", "ORD-001")
            .with_var("amount", "150.00")
            .with_var("quantity", "3")
            .with_var("status", "pending")
            .with_validation(|data| {
                // Rule 1: Amount must be positive
                if let Some(amount_str) = data.get("amount") {
                    match amount_str.parse::<f64>() {
                        Ok(amount) if amount > 0.0 => {},
                        _ => return Err("Amount must be positive".to_string()),
                    }
                }

                // Rule 2: Quantity must be positive integer
                if let Some(qty_str) = data.get("quantity") {
                    match qty_str.parse::<u32>() {
                        Ok(qty) if qty > 0 => {},
                        _ => return Err("Quantity must be positive integer".to_string()),
                    }
                }

                // Rule 3: Status must be valid
                if let Some(status) = data.get("status") {
                    let valid_statuses = ["pending", "processing", "completed", "cancelled"];
                    if !valid_statuses.contains(&status.as_str()) {
                        return Err("Invalid order status".to_string());
                    }
                }

                Ok(())
            })
            .try_build()
            .expect("Valid order should build");
        validations_performed += 1;

        assert_eq!(order.get("order_id"), Some(&"ORD-001".to_string()));
        assert_eq!(order.get("amount"), Some(&"150.00".to_string()));
    }

    // ========================================================================
    // 3. BUILDER PRESETS WITH VALIDATION HOOKS - Combined approach
    // ========================================================================
    {
        // Register a "valid_product" preset
        TestDataBuilder::register_preset("valid_product", |builder| {
            builder
                .with_var("product_id", "PROD-001")
                .with_var("name", "Widget")
                .with_var("price", "29.99")
                .with_var("stock_quantity", "100")
                .with_var("category", "widgets")
        })
        .expect("Failed to register preset");
        presets_registered += 1;

        // Use preset with validation
        let product = TestDataBuilder::preset("valid_product")?
            .with_validation(|data| {
                // Validate price is positive
                if let Some(price_str) = data.get("price") {
                    match price_str.parse::<f64>() {
                        Ok(price) if price > 0.0 => {},
                        _ => return Err("Price must be positive".to_string()),
                    }
                }

                // Validate stock is non-negative
                if let Some(stock_str) = data.get("stock_quantity") {
                    match stock_str.parse::<u32>() {
                        Ok(_) => {},
                        Err(_) => return Err("Stock must be numeric".to_string()),
                    }
                }

                Ok(())
            })
            .try_build()
            .expect("Product should build");
        presets_used += 1;
        validations_performed += 1;

        assert_eq!(product.get("name"), Some(&"Widget".to_string()));
        assert_eq!(product.get("category"), Some(&"widgets".to_string()));
    }

    // ========================================================================
    // 4. AUTO-DERIVED FAKE DATA - Type-driven realistic test data
    // ========================================================================
    {
        // Example 4a: Using test data builder to simulate fake data generation
        let user_with_fake = TestDataBuilder::new()
            .with_var("id", "12345")
            .with_var("username", "test_user_001")
            .with_var("email", "user+001@example.com")
            .with_var("timestamp", "2025-11-15T10:30:00Z")
            .build();
        fake_data_examples += 1;

        assert!(user_with_fake.contains_key("email"));
        assert!(user_with_fake.get("email").unwrap().contains('@'));

        // Example 4b: Fake data with UUIDs
        let user_with_uuid = TestDataBuilder::new()
            .with_var("user_id", "550e8400-e29b-41d4-a716-446655440000")
            .with_var("session_id", "f47ac10b-58cc-4372-a567-0e02b2c3d479")
            .with_var("created_at", "2025-11-15")
            .build();
        fake_data_examples += 1;

        assert!(user_with_uuid.get("user_id").unwrap().contains('-'));
        assert!(user_with_uuid.get("session_id").unwrap().contains('-'));

        // Example 4c: Fake data with realistic values
        let realistic_user = TestDataBuilder::new()
            .with_var("first_name", "Alice")
            .with_var("last_name", "Johnson")
            .with_var("email", "alice.johnson@company.com")
            .with_var("phone", "+1-555-0123")
            .with_var("age", "28")
            .with_var("company", "Acme Corp")
            .build();
        fake_data_examples += 1;

        assert!(realistic_user.get("email").unwrap().contains('@'));
        assert!(realistic_user.get("phone").unwrap().contains('-'));
    }

    // ========================================================================
    // 5. COMPOSABLE PRESETS - Presets built on other presets
    // ========================================================================
    {
        // Base preset
        TestDataBuilder::register_preset("base_user", |builder| {
            builder
                .with_var("is_active", "true")
                .with_var("created_at", "2025-01-01")
        })
        .expect("Failed to register base_user");
        presets_registered += 1;

        // Composite preset - would extend base_user in real implementation
        TestDataBuilder::register_preset("admin_user", |builder| {
            builder
                .with_var("is_active", "true")
                .with_var("role", "admin")
                .with_var("permissions", "all")
                .with_var("created_at", "2025-01-01")
        })
        .expect("Failed to register admin_user");
        presets_registered += 1;

        let admin = TestDataBuilder::preset("admin_user")?
            .build();
        presets_used += 1;

        assert_eq!(admin.get("role"), Some(&"admin".to_string()));
        assert_eq!(admin.get("permissions"), Some(&"all".to_string()));
    }

    Ok(BuilderPresetsResult {
        presets_registered,
        presets_used,
        validations_performed,
        fake_data_examples,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_presets() {
        let result = run().expect("Builder presets should run");
        assert!(result.presets_registered > 0);
        assert!(result.presets_used > 0);
        assert!(result.validations_performed > 0);
        assert!(result.fake_data_examples > 0);
    }
}
