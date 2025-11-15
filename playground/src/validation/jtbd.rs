//! JTBD Examples
//!
//! Demonstrates Jobs To Be Done validation for real-world scenario testing, including advanced patterns.

use chicago_tdd_tools::prelude::*;
use chicago_tdd_tools::validation::jtbd::*;
use std::collections::HashMap;

/// Example: Basic JTBD scenario
pub fn example_jtbd_basic() {
    // Arrange: Create validator
    let mut validator = JtbdValidator::new();

    // Arrange: Register scenario
    validator.register_scenario(JtbdScenario {
        name: "Order Processing".to_string(),
        setup_context: Box::new(|| ExecutionContext::default()),
        execute: Box::new(|_ctx| {
            let mut vars = HashMap::new();
            vars.insert("order_id".to_string(), "ORD-001".to_string());
            ExecutionResult::ok(vars)
        }),
        validate_result: Box::new(|_ctx, result| {
            result.success && result.variables.contains_key("order_id")
        }),
        expected_behavior: "Process order and update state".to_string(),
    });

    // Act: Validate all scenarios
    let results = validator.validate_all();

    // Assert: Verify JTBD success
    assert_eq!(results.len(), 1);
    assert!(results[0].jtbd_success);
}

/// Example: JTBD scenario with index
pub fn example_jtbd_index() {
    // Arrange: Create validator
    let mut validator = JtbdValidator::new();

    // Arrange: Register scenario
    validator.register_scenario(JtbdScenario {
        name: "Test Scenario".to_string(),
        setup_context: Box::new(|| ExecutionContext::default()),
        execute: Box::new(|_ctx| ExecutionResult::ok(HashMap::new())),
        validate_result: Box::new(|_ctx, result| result.success),
        expected_behavior: "Test behavior".to_string(),
    });

    // Act: Validate using type-safe index
    let index = ScenarioIndex::new(0).unwrap();
    let result = validator.validate_scenario(index);

    // Assert: Verify result
    assert!(result.is_some());
    assert!(result.unwrap().jtbd_success);
}

/// Example: Advanced JTBD scenario with multiple validations
pub fn example_jtbd_advanced() {
    // Arrange: Create validator
    let mut validator = JtbdValidator::new();

    // Arrange: Register complex scenario with multiple validations
    validator.register_scenario(JtbdScenario {
        name: "Complex Order Processing".to_string(),
        setup_context: Box::new(|| {
            let mut ctx = ExecutionContext::default();
            ctx.variables.insert("customer_id".to_string(), "CUST-123".to_string());
            ctx.variables.insert("order_amount".to_string(), "100.00".to_string());
            ctx
        }),
        execute: Box::new(|ctx| {
            let mut vars = HashMap::new();
            // Extract from context
            let customer_id = ctx.variables.get("customer_id").cloned().unwrap_or_default();
            let amount = ctx.variables.get("order_amount").cloned().unwrap_or_default();

            // Process order
            vars.insert("order_id".to_string(), "ORD-001".to_string());
            vars.insert("customer_id".to_string(), customer_id);
            vars.insert("total_amount".to_string(), amount);
            vars.insert("status".to_string(), "processed".to_string());

            ExecutionResult::ok(vars)
        }),
        validate_result: Box::new(|ctx, result| {
            // Multiple validations: JTBD success + technical success + data validation
            let jtbd_ok = result.success;
            let has_order_id = result.variables.contains_key("order_id");
            let has_customer_id = result.variables.contains_key("customer_id");
            let customer_matches = result
                .variables
                .get("customer_id")
                .map(|id| id == ctx.variables.get("customer_id").unwrap_or(&String::new()))
                .unwrap_or(false);
            let status_correct =
                result.variables.get("status").map(|s| s == "processed").unwrap_or(false);

            jtbd_ok && has_order_id && has_customer_id && customer_matches && status_correct
        }),
        expected_behavior: "Process order with customer context and validate all fields"
            .to_string(),
    });

    // Act: Validate all scenarios
    let results = validator.validate_all();

    // Assert: Verify advanced JTBD success
    assert_eq!(results.len(), 1);
    assert!(results[0].jtbd_success);
    // Note: JtbdValidationResult has execution_success field, not success
    assert!(results[0].execution_success);
}

/// Example: Multiple JTBD scenarios
pub fn example_jtbd_multiple() {
    // Arrange: Create validator with multiple scenarios
    let mut validator = JtbdValidator::new();

    // Scenario 1: Order processing
    validator.register_scenario(JtbdScenario {
        name: "Order Processing".to_string(),
        setup_context: Box::new(|| ExecutionContext::default()),
        execute: Box::new(|_ctx| {
            let mut vars = HashMap::new();
            vars.insert("order_id".to_string(), "ORD-001".to_string());
            ExecutionResult::ok(vars)
        }),
        validate_result: Box::new(|_ctx, result| {
            result.success && result.variables.contains_key("order_id")
        }),
        expected_behavior: "Process order".to_string(),
    });

    // Scenario 2: Payment processing
    validator.register_scenario(JtbdScenario {
        name: "Payment Processing".to_string(),
        setup_context: Box::new(|| ExecutionContext::default()),
        execute: Box::new(|_ctx| {
            let mut vars = HashMap::new();
            vars.insert("payment_id".to_string(), "PAY-001".to_string());
            vars.insert("amount".to_string(), "100.00".to_string());
            ExecutionResult::ok(vars)
        }),
        validate_result: Box::new(|_ctx, result| {
            result.success
                && result.variables.contains_key("payment_id")
                && result.variables.contains_key("amount")
        }),
        expected_behavior: "Process payment".to_string(),
    });

    // Act: Validate all scenarios
    let results = validator.validate_all();

    // Assert: Verify all scenarios succeed
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.jtbd_success));
}

#[cfg(test)]
mod tests {
    use super::*;

    test!(test_jtbd_basic, {
        // Arrange-Act-Assert: Run example
        example_jtbd_basic();
    });

    test!(test_jtbd_index, {
        // Arrange-Act-Assert: Run example
        example_jtbd_index();
    });

    test!(test_jtbd_advanced, {
        // Arrange-Act-Assert: Run example
        example_jtbd_advanced();
    });

    test!(test_jtbd_multiple, {
        // Arrange-Act-Assert: Run example
        example_jtbd_multiple();
    });
}
