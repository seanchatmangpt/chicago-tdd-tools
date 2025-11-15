//! v1.3.0 Advanced Mutation Testing Operators Examples
//!
//! Demonstrates the 5 new mutation operators added in v1.3.0:
//! - SwapValues: Swap two values in state
//! - ToggleBoolean: Flip boolean values
//! - NumericDelta: Add/subtract from numeric values
//! - StringCase: Change string case (upper/lower)
//! - RemoveRandomKey: Remove random key from map

use chicago_tdd_tools::prelude::*;
use chicago_tdd_tools::testing::mutation::{MutationTester, MutationOperator, MutationScore, CaseMode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AdvancedMutationResult {
    pub swap_values_tested: usize,
    pub toggle_boolean_tested: usize,
    pub numeric_delta_tested: usize,
    pub string_case_tested: usize,
    pub remove_random_key_tested: usize,
    pub mutations_caught: usize,
}

pub fn run() -> crate::Result<AdvancedMutationResult> {
    let mut swap_values_tested = 0;
    let mut toggle_boolean_tested = 0;
    let mut numeric_delta_tested = 0;
    let mut string_case_tested = 0;
    let mut remove_random_key_tested = 0;
    let mut mutations_caught = 0;

    // ========================================================================
    // 1. SWAPVALUES MUTATION - Swap two values in state
    // ========================================================================
    {
        // Example 1a: Order processing - swap source/destination
        let mut order_data = HashMap::new();
        order_data.insert("from_warehouse".to_string(), "warehouse_a".to_string());
        order_data.insert("to_warehouse".to_string(), "warehouse_b".to_string());
        order_data.insert("quantity".to_string(), "100".to_string());

        let mut mutation_tester = MutationTester::new(order_data.clone());

        // Apply mutation: Swap from/to warehouses
        let mutated = mutation_tester.apply_mutation(MutationOperator::SwapValues(
            "from_warehouse".to_string(),
            "to_warehouse".to_string(),
        ));

        // Test should catch this mutation
        assert_ne!(
            mutated.get("from_warehouse"),
            order_data.get("from_warehouse"),
            "Mutation should have swapped warehouse assignments"
        );
        mutations_caught += 1;
        swap_values_tested += 1;

        // Example 1b: User permissions - swap admin/user roles
        let mut user_perms = HashMap::new();
        user_perms.insert("granted_role".to_string(), "user".to_string());
        user_perms.insert("target_role".to_string(), "admin".to_string());

        let mut perm_tester = MutationTester::new(user_perms.clone());
        let mutated_perms = perm_tester.apply_mutation(MutationOperator::SwapValues(
            "granted_role".to_string(),
            "target_role".to_string(),
        ));

        // Critical: tests must verify role permissions correctly
        assert_eq!(
            mutated_perms.get("granted_role"),
            Some(&"admin".to_string()),
            "Granted role should be swapped"
        );
        mutations_caught += 1;
        swap_values_tested += 1;
    }

    // ========================================================================
    // 2. TOGGLEBOOLEAN MUTATION - Flip boolean values
    // ========================================================================
    {
        // Example 2a: Account status toggle
        let mut account_data = HashMap::new();
        account_data.insert("is_active".to_string(), "true".to_string());
        account_data.insert("is_verified".to_string(), "true".to_string());
        account_data.insert("username".to_string(), "alice".to_string());

        let mut account_tester = MutationTester::new(account_data.clone());

        // Mutation: Toggle is_active from true to false
        let mutated = account_tester.apply_mutation(MutationOperator::ToggleBoolean(
            "is_active".to_string(),
            CaseMode::Upper, // Flag for toggle direction (not used for boolean)
        ));

        // Test must catch that account is no longer active
        assert_ne!(
            mutated.get("is_active"),
            account_data.get("is_active"),
            "Boolean should be toggled"
        );
        mutations_caught += 1;
        toggle_boolean_tested += 1;

        // Example 2b: Payment processing flags
        let mut payment_flags = HashMap::new();
        payment_flags.insert("is_approved".to_string(), "true".to_string());
        payment_flags.insert("is_processed".to_string(), "false".to_string());
        payment_flags.insert("amount".to_string(), "99.99".to_string());

        let mut payment_tester = MutationTester::new(payment_flags.clone());
        let mutated_payment = payment_tester.apply_mutation(MutationOperator::ToggleBoolean(
            "is_approved".to_string(),
            CaseMode::Upper,
        ));

        // Critical: approval flag affects payment processing
        assert_eq!(
            mutated_payment.get("is_approved"),
            Some(&"false".to_string()),
            "Approval flag should be toggled"
        );
        mutations_caught += 1;
        toggle_boolean_tested += 1;
    }

    // ========================================================================
    // 3. NUMERICDELTA MUTATION - Add/subtract from numeric values
    // ========================================================================
    {
        // Example 3a: Inventory quantity mutation
        let mut inventory = HashMap::new();
        inventory.insert("product_id".to_string(), "PROD-001".to_string());
        inventory.insert("quantity".to_string(), "100".to_string());
        inventory.insert("min_stock".to_string(), "10".to_string());

        let mut inventory_tester = MutationTester::new(inventory.clone());

        // Mutation: Reduce quantity by 50
        let mutated = inventory_tester.apply_mutation(MutationOperator::NumericDelta(
            "quantity".to_string(),
            -50,
        ));

        // Test must verify the mutation was caught
        assert_eq!(
            mutated.get("quantity"),
            Some(&"50".to_string()),
            "Quantity should be reduced by delta"
        );
        mutations_caught += 1;
        numeric_delta_tested += 1;

        // Example 3b: Price calculation mutation
        let mut pricing = HashMap::new();
        pricing.insert("base_price".to_string(), "100".to_string());
        pricing.insert("tax_percentage".to_string(), "8".to_string());
        pricing.insert("discount".to_string(), "0".to_string());

        let mut price_tester = MutationTester::new(pricing.clone());

        // Mutation: Change tax percentage
        let mutated_price = price_tester.apply_mutation(MutationOperator::NumericDelta(
            "tax_percentage".to_string(),
            2, // Increase tax by 2%
        ));

        assert_eq!(
            mutated_price.get("tax_percentage"),
            Some(&"10".to_string()),
            "Tax percentage should increase"
        );
        mutations_caught += 1;
        numeric_delta_tested += 1;

        // Example 3c: Score/rating mutation
        let mut rating = HashMap::new();
        rating.insert("user_id".to_string(), "user_123".to_string());
        rating.insert("score".to_string(), "85".to_string());
        rating.insert("max_score".to_string(), "100".to_string());

        let mut rating_tester = MutationTester::new(rating.clone());

        // Mutation: Decrease score significantly
        let mutated_rating = rating_tester.apply_mutation(MutationOperator::NumericDelta(
            "score".to_string(),
            -20,
        ));

        assert_eq!(
            mutated_rating.get("score"),
            Some(&"65".to_string()),
            "Score should be reduced by delta"
        );
        mutations_caught += 1;
        numeric_delta_tested += 1;
    }

    // ========================================================================
    // 4. STRINGCASE MUTATION - Change string case (upper/lower)
    // ========================================================================
    {
        // Example 4a: Status code case mutation
        let mut status_data = HashMap::new();
        status_data.insert("order_status".to_string(), "pending".to_string());
        status_data.insert("payment_status".to_string(), "approved".to_string());
        status_data.insert("order_id".to_string(), "ORD-001".to_string());

        let mut status_tester = MutationTester::new(status_data.clone());

        // Mutation: Change status to uppercase
        let mutated = status_tester.apply_mutation(MutationOperator::StringCase(
            "order_status".to_string(),
            CaseMode::Upper,
        ));

        // Tests must validate that status case is significant
        assert_eq!(
            mutated.get("order_status"),
            Some(&"PENDING".to_string()),
            "Status should be uppercase"
        );
        mutations_caught += 1;
        string_case_tested += 1;

        // Example 4b: Enum-like string mutation
        let mut env_config = HashMap::new();
        env_config.insert("environment".to_string(), "production".to_string());
        env_config.insert("log_level".to_string(), "info".to_string());

        let mut env_tester = MutationTester::new(env_config.clone());

        // Mutation: Change environment to uppercase
        let mutated_env = env_tester.apply_mutation(MutationOperator::StringCase(
            "environment".to_string(),
            CaseMode::Upper,
        ));

        assert_eq!(
            mutated_env.get("environment"),
            Some(&"PRODUCTION".to_string()),
            "Environment should change case"
        );
        mutations_caught += 1;
        string_case_tested += 1;

        // Example 4c: API endpoint case mutation
        let mut endpoint_config = HashMap::new();
        endpoint_config.insert("method".to_string(), "POST".to_string());
        endpoint_config.insert("path".to_string(), "/api/users".to_string());

        let mut endpoint_tester = MutationTester::new(endpoint_config.clone());

        // Mutation: Change method to lowercase (would break HTTP semantics)
        let mutated_endpoint = endpoint_tester.apply_mutation(MutationOperator::StringCase(
            "method".to_string(),
            CaseMode::Lower,
        ));

        assert_eq!(
            mutated_endpoint.get("method"),
            Some(&"post".to_string()),
            "HTTP method should be lowercased (mutation)"
        );
        mutations_caught += 1;
        string_case_tested += 1;
    }

    // ========================================================================
    // 5. REMOVERANDOMKEY MUTATION - Remove random key from state
    // ========================================================================
    {
        // Example 5a: Required field removal
        let mut user_data = HashMap::new();
        user_data.insert("user_id".to_string(), "user_123".to_string());
        user_data.insert("email".to_string(), "alice@example.com".to_string());
        user_data.insert("username".to_string(), "alice".to_string());
        user_data.insert("phone".to_string(), "+1-555-0123".to_string());

        let mut user_tester = MutationTester::new(user_data.clone());

        // Mutation: Remove a random key
        let mutated = user_tester.apply_mutation(MutationOperator::RemoveRandomKey);

        // Test must verify data integrity
        assert!(
            mutated.len() < user_data.len(),
            "A key should have been removed"
        );
        mutations_caught += 1;
        remove_random_key_tested += 1;

        // Example 5b: Configuration removal
        let mut config = HashMap::new();
        config.insert("database_url".to_string(), "postgres://localhost".to_string());
        config.insert("api_key".to_string(), "secret_key_123".to_string());
        config.insert("timeout".to_string(), "30".to_string());
        config.insert("retries".to_string(), "3".to_string());

        let mut config_tester = MutationTester::new(config.clone());

        // Mutation: Remove random config key
        let mutated_config = config_tester.apply_mutation(MutationOperator::RemoveRandomKey);

        // Configuration must be complete for proper operation
        assert!(
            mutated_config.len() < config.len(),
            "Configuration key should be removed"
        );
        mutations_caught += 1;
        remove_random_key_tested += 1;
    }

    // ========================================================================
    // 6. COMPREHENSIVE MUTATION SCORE CALCULATION
    // ========================================================================
    {
        // Simulate mutation testing across all operators
        // In real scenario, you'd run mutation operators against your test suite
        let total_mutations = 100;
        let mutations_detected = 95;

        let score = MutationScore::calculate(mutations_detected, total_mutations);

        // Verify score is good (>80% is excellent)
        assert!(
            score.score() >= 80.0,
            "Mutation score should be above 80% for good test quality"
        );
    }

    Ok(AdvancedMutationResult {
        swap_values_tested,
        toggle_boolean_tested,
        numeric_delta_tested,
        string_case_tested,
        remove_random_key_tested,
        mutations_caught,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advanced_mutations() {
        let result = run().expect("Advanced mutations should run");
        assert!(result.swap_values_tested > 0);
        assert!(result.toggle_boolean_tested > 0);
        assert!(result.numeric_delta_tested > 0);
        assert!(result.string_case_tested > 0);
        assert!(result.remove_random_key_tested > 0);
        assert!(result.mutations_caught > 0);
    }
}
