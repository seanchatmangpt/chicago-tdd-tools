# Operator Registry Example

**Category:** Reference  
**Level:** Intermediate  
**Prerequisites:** Understanding of workflow patterns, guard systems  
**Features Required:** None

---

## Overview

This example demonstrates the global operator registry with guard system and pattern registration. The registry serves as single source of truth for workflow patterns with compile-time and runtime validation.

**What you'll learn:**
- Accessing the global operator registry
- Querying patterns by guard type, category, or properties
- Understanding the 5 guard types and their purposes
- Checking Chatman Equation properties (determinism, idempotence, type preservation, boundedness)
- Finding patterns with specific requirements

---

## Quick Start

```bash
cargo run --example operator_registry
```

---

## Prerequisites

- Rust 1.70+ (Edition 2021)
- Chicago TDD Tools installed
- Understanding of workflow patterns

---

## Tutorial: Getting Started

### Step 1: Access Global Registry

Access the global singleton registry:

```rust
use chicago_tdd_tools::operator_registry::global_registry;

let registry = global_registry();
```

### Step 2: Query Patterns

Get specific operator or all operators:

```rust
// Get specific operator
if let Some(operator) = registry.get_operator("sequence_op") {
    println!("Pattern: {}", operator.pattern_name);
    println!("Category: {}", operator.pattern_category);
}

// Get all operators
let all_operators = registry.all_operators();
println!("Total operators: {}", all_operators.len());
```

### Step 3: Find Patterns by Guard

Find patterns requiring specific guard types:

```rust
// Find patterns requiring Budget guard
let budget_patterns = registry.operators_with_guard(GuardType::Budget);

// Find patterns requiring Recursion guard
let recursion_patterns = registry.operators_with_guard(GuardType::Recursion);
```

### Step 4: Check Pattern Properties

Verify Chatman Equation properties:

```rust
if let Some(operator) = registry.get_operator("sequence_op") {
    println!("Deterministic: {}", operator.properties.deterministic);
    println!("Idempotent: {}", operator.properties.idempotent);
    println!("Type Preserving: {}", operator.properties.type_preserving);
    println!("Bounded: {}", operator.properties.bounded);
}
```

---

## How-To: Common Tasks

### Query Patterns by Guard Type

```rust
let registry = global_registry();

// Budget guard
let budget_patterns = registry.operators_with_guard(GuardType::Budget);

// Legality guard
let legality_patterns = registry.operators_with_guard(GuardType::Legality);

// Recursion guard
let recursion_patterns = registry.operators_with_guard(GuardType::Recursion);
```

### Check Pattern Properties

```rust
if let Some(operator) = registry.get_operator("sequence_op") {
    // Check individual properties
    if operator.properties.deterministic {
        println!("Pattern is deterministic");
    }
    
    // Check all properties
    if operator.satisfies_all_properties() {
        println!("Pattern satisfies all Chatman Equation properties");
    }
}
```

### Find Patterns by Category

```rust
let registry = global_registry();
let all_operators = registry.all_operators();

// Filter by category
let basic_flow: Vec<_> = all_operators
    .iter()
    .filter(|op| op.pattern_category == "Basic Control Flow")
    .collect();
```

### Check Required Guards

```rust
if let Some(operator) = registry.get_operator("cycles_op") {
    println!("Required Guards:");
    for guard in &operator.required_guards {
        println!("  - {:?}", guard);
    }
}
```

### Check Latency Requirements

```rust
if let Some(operator) = registry.get_operator("sequence_op") {
    println!("Max Latency: {}ns", operator.max_latency_ns);
    println!("Max Latency: {:.2}ms", operator.max_latency_ms());
}
```

---

## Explanation: Concepts

### Operator Registry

**Global Singleton**: Single source of truth for all workflow patterns. Registry is initialized once and shared across all operations.

**12 YAWL Patterns**: Registered patterns from YAWL workflow control patterns, each with:
- Pattern number (1-43)
- Pattern name and category
- Chatman Equation properties
- Maximum latency requirements
- Required guard types

### Guard System

Five guard types ensure safe execution:

1. **Legality**: Prevents invalid state transitions
2. **Budget**: Prevents exceeding resource limits
3. **Chronology**: Enforces proper temporal ordering
4. **Causality**: Ensures causal dependencies respected
5. **Recursion**: Bounds recursion depth (Chatman Constant = 8)

### Chatman Equation Properties

Four properties characterize each pattern:

- **Determinism**: f(x) = f(x) always (same inputs → same outputs)
- **Idempotence**: f(f(x)) = f(x) (running twice equals running once)
- **Type Preservation**: Types maintained through execution
- **Boundedness**: Execution time is measurable and bounded

### Pattern Categories

Patterns organized by category:
- Basic Control Flow
- Advanced Branching
- Structural
- Multiple Instance
- State-Based
- Cancellation

---

## Reference: Quick Lookup

### global_registry()

Access global singleton registry:

```rust
pub fn global_registry() -> &'static OperatorRegistry
```

### OperatorRegistry

**Methods:**
```rust
pub fn get_operator(&self, hook_id: &str) -> Option<&OperatorDescriptor>
pub fn all_operators(&self) -> Vec<&OperatorDescriptor>
pub fn operators_with_guard(&self, guard: GuardType) -> Vec<&OperatorDescriptor>
pub fn count_by_category(&self) -> HashMap<String, usize>
pub fn count_deterministic(&self) -> usize
pub fn count_idempotent(&self) -> usize
pub fn count_type_preserving(&self) -> usize
pub fn count_bounded(&self) -> usize
pub fn operators_fully_deterministic(&self) -> Vec<&OperatorDescriptor>
```

### OperatorDescriptor

```rust
pub struct OperatorDescriptor {
    pub hook_id: String,
    pub pattern_number: u32,
    pub pattern_name: String,
    pub pattern_category: String,
    pub properties: OperatorProperties,
    pub max_latency_ns: i64,
    pub required_guards: Vec<GuardType>,
    pub slo: Option<String>,
}
```

**Methods:**
```rust
pub fn is_bounded(&self) -> bool
pub fn max_latency_ms(&self) -> f64
pub fn satisfies_all_properties(&self) -> bool
```

### OperatorProperties

```rust
pub struct OperatorProperties {
    pub deterministic: bool,
    pub idempotent: bool,
    pub type_preserving: bool,
    pub bounded: bool,
}
```

### GuardType

```rust
pub enum GuardType {
    Legality,
    Budget,
    Chronology,
    Causality,
    Recursion,
}
```

---

## Common Patterns

### Query All Patterns

```rust
let registry = global_registry();
let all_operators = registry.all_operators();

for operator in all_operators {
    println!("{}: {}", operator.pattern_name, operator.pattern_category);
}
```

### Find Patterns by Guard

```rust
let registry = global_registry();
let budget_patterns = registry.operators_with_guard(GuardType::Budget);

for pattern in budget_patterns {
    println!("{} requires Budget guard", pattern.pattern_name);
}
```

### Check Pattern Properties

```rust
if let Some(operator) = registry.get_operator("sequence_op") {
    if operator.properties.deterministic {
        println!("Pattern is deterministic");
    }
    
    if operator.satisfies_all_properties() {
        println!("Satisfies all Chatman Equation properties");
    }
}
```

### Count by Category

```rust
let registry = global_registry();
let counts = registry.count_by_category();

for (category, count) in counts {
    println!("{}: {} patterns", category, count);
}
```

---

## Troubleshooting

### Operator Not Found

**Error**: `get_operator()` returns `None`

**Solution**: Check operator ID:
```rust
// Valid IDs include:
// - "sequence_op"
// - "parallel_split_op"
// - "cycles_op"
// etc.
```

### No Patterns with Guard

**Error**: `operators_with_guard()` returns empty

**Solution**: Not all patterns require all guards. Check which guards are actually used:
```rust
let registry = global_registry();
let all_operators = registry.all_operators();

// Check which guards are actually used
for op in all_operators {
    if op.required_guards.contains(&GuardType::Budget) {
        println!("{} requires Budget guard", op.pattern_name);
    }
}
```

---

## Related Documentation

- **Operator Registry**: `src/operator_registry.rs`
- **Release Notes**: `docs/releases/RELEASE_NOTES_v1.4.0.md`

---

## See Also

- [Sector Stacks Workflows](sector_stacks_workflows.md) - Production-grade workflows
- [RDF Validation](rdf_validation.md) - RDF-driven validation
- [Fail-Fast Verification](fail_fast_verification.md) - 12-phase verification

---

**Quality is the default. Prevention beats detection.**

*Version 1.4.0 | Updated 2025-01-16 | Team KNHK | License MIT*

