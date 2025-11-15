//! v1.3.0 New Assertion Macros Examples
//!
//! Demonstrates the 8 new assertion macros added in v1.3.0 for improved ergonomics.

use chicago_tdd_tools::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssertionV130Result {
    pub assertions_demonstrated: usize,
    pub collection_assertions: usize,
    pub json_assertions: usize,
    pub approximate_assertions: usize,
}

pub fn run() -> crate::Result<AssertionV130Result> {
    let mut assertions_demonstrated = 0;
    let mut collection_assertions = 0;
    let mut json_assertions = 0;
    let mut approximate_assertions = 0;

    // ========================================================================
    // 1. COLLECTION ASSERTIONS - assert_contains! and assert_not_contains!
    // ========================================================================
    {
        // Example 1a: Assert collection contains item
        let users = vec!["alice", "bob", "charlie"];

        assert_contains!(users, "bob");
        collection_assertions += 1;
        assertions_demonstrated += 1;

        // Example 1b: Assert collection does NOT contain item
        assert_not_contains!(users, "david");
        collection_assertions += 1;
        assertions_demonstrated += 1;

        // Example 1c: Works with iterators mapping
        let user_ids: Vec<u32> = vec![1, 2, 3, 4, 5];
        let id_iter = user_ids.iter().map(|id| *id);

        assert_contains!(id_iter, 3);
        collection_assertions += 1;
        assertions_demonstrated += 1;
    }

    // ========================================================================
    // 2. SUBSET/SUPERSET ASSERTIONS - assert_subset! and assert_superset!
    // ========================================================================
    {
        // Example 2a: Assert one collection is subset of another
        let allowed_roles = vec!["admin", "user", "guest"];
        let user_roles = vec!["user"];

        assert_subset!(user_roles, allowed_roles);
        collection_assertions += 1;
        assertions_demonstrated += 1;

        // Example 2b: Assert one collection is superset of another
        let required_features = vec!["login", "dashboard"];
        let implemented_features = vec!["login", "dashboard", "export", "settings"];

        assert_superset!(implemented_features, required_features);
        collection_assertions += 1;
        assertions_demonstrated += 1;
    }

    // ========================================================================
    // 3. JSON ASSERTIONS - assert_json_eq!
    // ========================================================================
    {
        // Example 3a: Semantic JSON comparison (ignores key order and whitespace)
        let actual = json!({
            "name": "Alice",
            "age": 30,
            "email": "alice@example.com"
        });

        let expected = json!({
            "email": "alice@example.com",
            "name": "Alice",
            "age": 30
        });

        // Passes despite different key order!
        assert_json_eq!(actual, expected);
        json_assertions += 1;
        assertions_demonstrated += 1;

        // Example 3b: JSON assertion with nested objects
        let api_response = json!({
            "status": "success",
            "data": {
                "id": 123,
                "username": "alice"
            }
        });

        let expected_response = json!({
            "status": "success",
            "data": {
                "username": "alice",
                "id": 123
            }
        });

        assert_json_eq!(api_response, expected_response);
        json_assertions += 1;
        assertions_demonstrated += 1;

        // Example 3c: JSON assertion with arrays
        let users_list = json!([
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"}
        ]);

        let expected_users = json!([
            {"name": "Alice", "id": 1},
            {"name": "Bob", "id": 2}
        ]);

        assert_json_eq!(users_list, expected_users);
        json_assertions += 1;
        assertions_demonstrated += 1;
    }

    // ========================================================================
    // 4. APPROXIMATE EQUALITY - assert_approx_eq!
    // ========================================================================
    {
        // Example 4a: Floating-point approximation with f64
        let pi_estimate = 3.14159265;
        let pi_expected = 3.14159;
        let epsilon = 0.001;

        assert_approx_eq!(pi_estimate, pi_expected, epsilon);
        approximate_assertions += 1;
        assertions_demonstrated += 1;

        // Example 4b: Floating-point approximation with f32
        let f32_value: f32 = 1.5;
        let f32_expected: f32 = 1.5;
        let f32_epsilon: f32 = 0.0001;

        assert_approx_eq!(f32_value, f32_expected, f32_epsilon);
        approximate_assertions += 1;
        assertions_demonstrated += 1;

        // Example 4c: Physics calculation approximation
        let calculated_speed = 9.81; // m/s^2
        let expected_gravity = 9.8;
        let tolerance = 0.1;

        assert_approx_eq!(calculated_speed, expected_gravity, tolerance);
        approximate_assertions += 1;
        assertions_demonstrated += 1;
    }

    // ========================================================================
    // 5. COMBINED V1.3.0 ASSERTIONS IN REALISTIC SCENARIO
    // ========================================================================
    {
        // Real-world e-commerce example using multiple v1.3.0 assertions
        let product_ids = vec![1, 2, 3, 4, 5];

        // Collection assertion: Product exists
        assert_contains!(product_ids, 3);
        collection_assertions += 1;
        assertions_demonstrated += 1;

        // JSON assertion: API response structure
        let product_response = json!({
            "id": 3,
            "name": "Widget",
            "price": 19.99,
            "in_stock": true
        });

        let expected_product = json!({
            "in_stock": true,
            "name": "Widget",
            "id": 3,
            "price": 19.99
        });

        assert_json_eq!(product_response, expected_product);
        json_assertions += 1;
        assertions_demonstrated += 1;

        // Approximate assertion: Price calculation with tax
        let product_price = 19.99;
        let tax_rate = 0.08;
        let calculated_total = product_price * (1.0 + tax_rate);
        let expected_total = 21.59;
        let price_tolerance = 0.01;

        assert_approx_eq!(calculated_total, expected_total, price_tolerance);
        approximate_assertions += 1;
        assertions_demonstrated += 1;

        // Subset assertion: Allowed payment methods
        let allowed_payments = vec!["credit_card", "debit_card", "paypal", "apple_pay"];
        let customer_payment_methods = vec!["credit_card", "paypal"];

        assert_subset!(customer_payment_methods, allowed_payments);
        collection_assertions += 1;
        assertions_demonstrated += 1;
    }

    Ok(AssertionV130Result {
        assertions_demonstrated,
        collection_assertions,
        json_assertions,
        approximate_assertions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v130_assertions() {
        let result = run().expect("v1.3.0 assertions should run");
        assert!(result.assertions_demonstrated > 0);
        assert!(result.collection_assertions > 0);
        assert!(result.json_assertions > 0);
        assert!(result.approximate_assertions > 0);
    }
}
