//! # Operator Registry Example - Comprehensive Guide
//!
//! Demonstrates global operator registry with guard system and pattern registration.
//! The registry serves as single source of truth for workflow patterns with compile-time
//! and runtime validation.
//!
//! ## Tutorial: Getting Started
//!
//! This example walks through operator registry usage:
//!
//! 1. **Accessing Registry**: Use `OperatorRegistry::get()` to access global singleton
//! 2. **Querying Patterns**: Find patterns by guard type, category, or properties
//! 3. **Understanding Guards**: Learn about 5 guard types (Legality, Budget, Chronology, Causality, Recursion)
//! 4. **Pattern Properties**: Understand Chatman Equation properties (determinism, idempotence, type preservation, boundedness)
//!
//! ## Explanation: Concepts
//!
//! **Operator Registry**: Global singleton registry of all 12 YAWL workflow control patterns.
//! Each pattern is characterized by the four properties of the Chatman Equation:
//! - **Determinism**: f(x) = f(x) always
//! - **Idempotence**: f(f(x)) = f(x)
//! - **Type Preservation**: Types maintained through execution
//! - **Boundedness**: Execution time is measurable and bounded
//!
//! **Guard System**: 5 guard types ensure safe execution:
//! - **Legality**: Prevents invalid state transitions
//! - **Budget**: Prevents exceeding resource limits
//! - **Chronology**: Enforces proper temporal ordering
//! - **Causality**: Ensures causal dependencies respected
//! - **Recursion**: Bounds recursion depth (Chatman Constant = 8)
//!
//! **YAWL Patterns**: 12 registered patterns from YAWL workflow control patterns,
//! each with guard definitions and Chatman Equation properties.
//!
//! ## How-to: Common Tasks
//!
//! - Query patterns: See `example_query_patterns()`
//! - Find patterns by guard: See `example_patterns_by_guard()`
//! - Check pattern properties: See `example_pattern_properties()`
//! - Understand guards: See `example_guard_system()`
//!
//! ## Reference: Quick Lookup
//!
//! **Key Types**:
//! - `OperatorRegistry`: Global singleton registry
//! - `OperatorDescriptor`: Complete pattern specification
//! - `OperatorProperties`: Chatman Equation properties
//! - `GuardType`: Guard type enumeration
//!
//! **Key Methods**:
//! - `OperatorRegistry::get() -> &OperatorRegistry`
//! - `OperatorRegistry::get_operator(id: &str) -> Option<&OperatorDescriptor>`
//! - `OperatorRegistry::operators_with_guard(guard: GuardType) -> Vec<&OperatorDescriptor>`

use chicago_tdd_tools::operator_registry::{global_registry, GuardType};

/// Example: Querying patterns from registry
///
/// ## How-to: Query Patterns
///
/// Demonstrates accessing the global registry and querying patterns.
fn example_query_patterns() {
    println!("=== Example: Querying Patterns ===");

    // Arrange: Access global registry
    let registry = global_registry();

    // Act: Get specific operator
    if let Some(operator) = registry.get_operator("sequence_op") {
        println!("✓ Found operator: {}", operator.pattern_name);
        println!("  Pattern Number: {}", operator.pattern_number);
        println!("  Category: {}", operator.pattern_category);
        println!("  Max Latency: {}ns", operator.max_latency_ns);
    }

    // Query all operators
    let all_operators = registry.all_operators();
    println!("\n✓ Total operators in registry: {}", all_operators.len());

    // Display first few operators
    println!("\nFirst 5 operators:");
    for (i, op) in all_operators.iter().take(5).enumerate() {
        println!("  {}. {}: {} ({})", i + 1, op.hook_id, op.pattern_name, op.pattern_category);
    }
}

/// Example: Finding patterns by guard type
///
/// ## How-to: Patterns by Guard
///
/// Demonstrates finding patterns that require specific guard types.
fn example_patterns_by_guard() {
    println!("\n=== Example: Patterns by Guard Type ===");

    let registry = global_registry();

    // Find patterns requiring Budget guard
    println!("\n--- Patterns Requiring Budget Guard ---");
    let budget_patterns = registry.operators_with_guard(GuardType::Budget);
    println!("✓ Found {} patterns requiring Budget guard", budget_patterns.len());
    for op in budget_patterns.iter().take(3) {
        println!("  - {}: {}", op.pattern_name, op.pattern_category);
    }

    // Find patterns requiring Legality guard
    println!("\n--- Patterns Requiring Legality Guard ---");
    let legality_patterns = registry.operators_with_guard(GuardType::Legality);
    println!("✓ Found {} patterns requiring Legality guard", legality_patterns.len());
    for op in legality_patterns.iter().take(3) {
        println!("  - {}: {}", op.pattern_name, op.pattern_category);
    }

    // Find patterns requiring Recursion guard
    println!("\n--- Patterns Requiring Recursion Guard ---");
    let recursion_patterns = registry.operators_with_guard(GuardType::Recursion);
    println!("✓ Found {} patterns requiring Recursion guard", recursion_patterns.len());
    for op in recursion_patterns.iter().take(3) {
        println!("  - {}: {}", op.pattern_name, op.pattern_category);
    }
}

/// Example: Understanding pattern properties
///
/// ## How-to: Pattern Properties
///
/// Demonstrates Chatman Equation properties for patterns.
fn example_pattern_properties() {
    println!("\n=== Example: Pattern Properties (Chatman Equation) ===");

    let registry = global_registry();

    // Get sequence operator (deterministic, idempotent)
    if let Some(operator) = registry.get_operator("sequence_op") {
        println!("\n--- Sequence Operator Properties ---");
        println!("Pattern: {}", operator.pattern_name);
        println!("Properties:");
        println!("  Deterministic: {}", operator.properties.deterministic);
        println!("  Idempotent: {}", operator.properties.idempotent);
        println!("  Type Preserving: {}", operator.properties.type_preserving);
        println!("  Bounded: {}", operator.properties.bounded);

        // Explain properties
        if operator.properties.deterministic {
            println!("\n✓ Deterministic: Same inputs always produce same outputs");
        }
        if operator.properties.idempotent {
            println!("✓ Idempotent: Running twice equals running once");
        }
        if operator.properties.type_preserving {
            println!("✓ Type Preserving: Input types maintained through execution");
        }
        if operator.properties.bounded {
            println!("✓ Bounded: Execution time is measurable and bounded");
        }
    }

    // Get deferred choice (non-deterministic)
    if let Some(operator) = registry.get_operator("deferred_choice_op") {
        println!("\n--- Deferred Choice Operator Properties ---");
        println!("Pattern: {}", operator.pattern_name);
        println!("Properties:");
        println!("  Deterministic: {}", operator.properties.deterministic);
        println!("  Idempotent: {}", operator.properties.idempotent);
        println!("  Type Preserving: {}", operator.properties.type_preserving);
        println!("  Bounded: {}", operator.properties.bounded);

        if !operator.properties.deterministic {
            println!("\n⚠ Non-Deterministic: Output depends on external events");
        }
    }
}

/// Example: Understanding guard system
///
/// ## How-to: Guard System
///
/// Demonstrates the 5 guard types and their purposes.
fn example_guard_system() {
    println!("\n=== Example: Guard System ===");

    let registry = global_registry();

    // Explain each guard type
    println!("\n--- Guard Types ---");

    // Legality
    println!("\n1. Legality Guard");
    println!("   Purpose: Prevents invalid state transitions");
    let legality_patterns = registry.operators_with_guard(GuardType::Legality);
    println!("   Used by {} patterns", legality_patterns.len());
    println!("   Example: Exclusive Choice, Simple Merge");

    // Budget
    println!("\n2. Budget Guard");
    println!("   Purpose: Prevents exceeding resource limits");
    let budget_patterns = registry.operators_with_guard(GuardType::Budget);
    println!("   Used by {} patterns", budget_patterns.len());
    println!("   Example: Settlement ≤ Policy Limit");

    // Chronology
    println!("\n3. Chronology Guard");
    println!("   Purpose: Enforces proper temporal ordering");
    let chronology_patterns = registry.operators_with_guard(GuardType::Chronology);
    println!("   Used by {} patterns", chronology_patterns.len());
    println!("   Example: Claim Date ≤ Current Date");

    // Causality
    println!("\n4. Causality Guard");
    println!("   Purpose: Ensures causal dependencies respected");
    let causality_patterns = registry.operators_with_guard(GuardType::Causality);
    println!("   Used by {} patterns", causality_patterns.len());
    println!("   Example: Fraud Score Determines Entitlements");

    // Recursion
    println!("\n5. Recursion Guard");
    println!("   Purpose: Bounds recursion depth (Chatman Constant = 8)");
    let recursion_patterns = registry.operators_with_guard(GuardType::Recursion);
    println!("   Used by {} patterns", recursion_patterns.len());
    println!("   Example: Arbitrary Cycles, Loop Patterns");
}

/// Example: Pattern categories
///
/// ## How-to: Pattern Categories
///
/// Demonstrates different pattern categories in registry.
fn example_pattern_categories() {
    println!("\n=== Example: Pattern Categories ===");

    let registry = global_registry();
    let all_operators = registry.all_operators();

    // Group by category
    let mut categories: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for op in &all_operators {
        categories
            .entry(op.pattern_category.clone())
            .or_insert_with(Vec::new)
            .push(op.hook_id.clone());
    }

    println!("\nPattern Categories:");
    for (category, pattern_ids) in categories.iter() {
        println!("  {}: {} patterns", category, pattern_ids.len());
        for pattern_id in pattern_ids.iter().take(3) {
            if let Some(op) = registry.get_operator(pattern_id) {
                println!("    - {} ({})", op.pattern_name, op.pattern_number);
            }
        }
    }
}

/// Example: Pattern latency requirements
///
/// ## How-to: Latency Requirements
///
/// Demonstrates latency requirements for different patterns.
fn example_pattern_latency() {
    println!("\n=== Example: Pattern Latency Requirements ===");

    let registry = global_registry();

    // Find patterns with different latency requirements
    let all_operators = registry.all_operators();

    // Group by latency ranges
    let mut fast_patterns = Vec::new();
    let mut medium_patterns = Vec::new();
    let mut slow_patterns = Vec::new();

    for op in &all_operators {
        if op.max_latency_ns < 1_000_000_000 {
            // < 1 second
            fast_patterns.push(op);
        } else if op.max_latency_ns < 10_000_000_000 {
            // < 10 seconds
            medium_patterns.push(op);
        } else {
            // >= 10 seconds
            slow_patterns.push(op);
        }
    }

    println!("\nFast Patterns (< 1s): {}", fast_patterns.len());
    for op in fast_patterns.iter().take(3) {
        println!("  - {}: {}ns", op.pattern_name, op.max_latency_ns);
    }

    println!("\nMedium Patterns (1s - 10s): {}", medium_patterns.len());
    for op in medium_patterns.iter().take(3) {
        println!("  - {}: {}ns", op.pattern_name, op.max_latency_ns);
    }

    println!("\nSlow Patterns (>= 10s): {}", slow_patterns.len());
    for op in slow_patterns.iter().take(3) {
        println!("  - {}: {}ns", op.pattern_name, op.max_latency_ns);
    }
}

/// Example: Required guards for pattern
///
/// ## How-to: Required Guards
///
/// Demonstrates checking which guards are required for a pattern.
fn example_required_guards() {
    println!("\n=== Example: Required Guards for Patterns ===");

    let registry = global_registry();

    // Check guards for different patterns
    let patterns_to_check = vec!["sequence_op", "cycles_op", "deferred_choice_op"];

    for pattern_id in patterns_to_check {
        if let Some(operator) = registry.get_operator(pattern_id) {
            println!("\n--- {} ---", operator.pattern_name);
            println!("Required Guards: {}", operator.required_guards.len());
            for guard in &operator.required_guards {
                println!("  - {:?}", guard);
            }

            // Explain guard purposes
            if operator.required_guards.contains(&GuardType::Budget) {
                println!("  → Budget guard ensures resource limits respected");
            }
            if operator.required_guards.contains(&GuardType::Recursion) {
                println!("  → Recursion guard bounds recursion depth");
            }
            if operator.required_guards.contains(&GuardType::Causality) {
                println!("  → Causality guard ensures causal dependencies");
            }
        }
    }
}

fn main() {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║  Operator Registry - Pattern Registration & Guard System      ║");
    println!("║  Single Source of Truth for Workflow Patterns                 ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    // Run examples
    example_query_patterns();
    example_patterns_by_guard();
    example_pattern_properties();
    example_guard_system();
    example_pattern_categories();
    example_pattern_latency();
    example_required_guards();

    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║  All Examples Completed Successfully!                          ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
}

#[cfg(test)]
mod tests {
    use super::*;

    test!(test_registry_access, {
        // Arrange & Act
        let registry = global_registry();

        // Assert
        assert!(registry.all_operators().len() > 0);
    });

    test!(test_get_operator, {
        // Arrange
        let registry = global_registry();

        // Act
        let operator = registry.get_operator("sequence_op");

        // Assert
        assert!(operator.is_some());
        if let Some(op) = operator {
            assert_eq!(op.pattern_name, "Sequence");
        }
    });

    test!(test_patterns_with_guard, {
        // Arrange
        let registry = global_registry();

        // Act
        let budget_patterns = registry.operators_with_guard(GuardType::Budget);

        // Assert
        assert!(budget_patterns.len() > 0);
    });

    test!(test_pattern_properties, {
        // Arrange
        let registry = global_registry();

        // Act
        if let Some(operator) = registry.get_operator("sequence_op") {
            // Assert: Sequence should be deterministic
            assert!(operator.properties.deterministic);
            assert!(operator.properties.type_preserving);
            assert!(operator.properties.bounded);
        }
    });
}
