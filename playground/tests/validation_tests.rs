//! Validation Features Test Suite

use chicago_tdd_tools::prelude::*;

pub fn run_validation_tests() {
    println!("  Testing validation features...");
    
    // Coverage
    test_coverage();
    
    // Guards
    test_guards();
    
    // JTBD
    test_jtbd();
    
    // Performance
    test_performance();
    
    println!("  ✓ Validation features validated");
}

fn test_coverage() {
    let mut report = CoverageReport::new();
    report.add_item("item1".to_string(), true);
    assert_eq!(report.total.get(), 1);
}

fn test_guards() {
    let validator = GuardValidator::new();
    assert!(validator.validate_run_len(5).is_ok());
}

fn test_jtbd() {
    use std::collections::HashMap;
    let mut validator = JtbdValidator::new();
    validator.register_scenario(JtbdScenario {
        name: "test".to_string(),
        setup_context: Box::new(|| ExecutionContext::default()),
        execute: Box::new(|_| ExecutionResult::ok(HashMap::new())),
        validate_result: Box::new(|_, _| true),
        expected_behavior: "test".to_string(),
    });
    let results = validator.validate_all();
    assert_eq!(results.len(), 1);
}

fn test_performance() {
    let counter = TickCounter::start();
    let _ticks = counter.elapsed_ticks();
}

