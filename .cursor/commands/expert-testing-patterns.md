# Expert-Level Testing Patterns

## Purpose

This command provides expert-level testing patterns that juniors commonly miss. These are the 80/20 critical test cases that catch real bugs in production.

## Documentation Reference

For complete testing documentation, see:
- **[Getting Started Guide](../../docs/GETTING_STARTED.md)** - Quick start with verified examples
- **[User Guide](../../docs/USER_GUIDE.md)** - Comprehensive testing guide with patterns
- **[API Reference](../../docs/API_REFERENCE.md)** - Complete API documentation
- **[Architecture](../../docs/ARCHITECTURE.md)** - Design principles

## Core Team Testing Priorities

The Rust core team emphasizes testing these areas that juniors often skip. See [User Guide - Best Practices](../../docs/USER_GUIDE.md#best-practices) for complete patterns.

### 1. Error Path Testing (Critical - 80% of bugs)

**Juniors test**: Happy path only  
**Experts test**: All error variants, error propagation, error recovery

See [User Guide - Best Practices](../../docs/USER_GUIDE.md#best-practices) for complete examples.

```rust
use chicago_tdd_tools::prelude::*;

// ❌ JUNIOR: Only tests happy path
#[test]
fn test_parse_number() {
    let result = parse_number("42");
    assert_eq!(result.unwrap(), 42);
}

// ✅ EXPERT: Tests all error paths
chicago_test!(test_parse_number_all_error_paths, {
    // Arrange: Test all error variants
    let test_cases = vec![
        ("", ParseError::EmptyInput),
        ("abc", ParseError::InvalidFormat),
        ("999999999999999999999", ParseError::Overflow),
        ("-0", ParseError::InvalidFormat), // Edge case
        (" 42 ", ParseError::InvalidFormat), // Whitespace
    ];
    
    // Act & Assert: Verify each error path
    for (input, expected_error) in test_cases {
        let result = parse_number(input);
        assert_err!(&result, format!("Should fail for input: {}", input));
        match result {
            Err(e) => assert_eq!(e, expected_error, "Error variant mismatch"),
            Ok(_) => panic!("Expected error for input: {}", input),
        }
    }
    
    // Also test error recovery
    let mut parser = NumberParser::new();
    assert_err!(&parser.parse("invalid"));
    // Parser should still be usable after error
    assert_ok!(&parser.parse("42"), "Parser should recover from error");
});
```

### 2. Boundary Condition Testing

**Juniors test**: Normal values  
**Experts test**: Empty, single item, max values, zero, negative

See [User Guide - Best Practices](../../docs/USER_GUIDE.md#best-practices) for complete examples.

```rust
use chicago_tdd_tools::prelude::*;

// ✅ EXPERT: Tests all boundary conditions
chicago_test!(test_collection_boundaries, {
    // Arrange: Test empty collection
    let empty: Vec<i32> = vec![];
    assert_eq!(process_collection(&empty).unwrap(), 0, "Empty collection should return 0");
    
    // Arrange: Test single item
    let single = vec![42];
    assert_eq!(process_collection(&single).unwrap(), 42, "Single item should work");
    
    // Arrange: Test max capacity (avoid OOM in test)
    let max_size = vec![0; usize::MAX / 8];
    let result = process_collection(&max_size);
    assert_ok!(&result, "Should handle large collections");
    
    // Arrange: Test zero values
    let zeros = vec![0; 100];
    assert_eq!(process_collection(&zeros).unwrap(), 0, "Zero values should work");
    
    // Arrange: Test negative values (if applicable)
    let negatives = vec![-1, -2, -3];
    let result = process_collection(&negatives);
    match result {
        Ok(v) => assert!(v < 0, "Negative sum should be negative"),
        Err(e) => assert!(matches!(e, ProcessingError::NegativeNotAllowed)),
    }
});
```

### 3. Resource Cleanup Testing

**Juniors test**: Happy path cleanup  
**Experts test**: Cleanup in error paths, double-drop safety, panic safety

See [User Guide - Best Practices](../../docs/USER_GUIDE.md#best-practices) for complete examples.

### 4. Concurrency and Race Condition Testing

**Juniors test**: Single-threaded only  
**Experts test**: Concurrent access, race conditions, Send/Sync bounds

See [User Guide - Best Practices](../../docs/USER_GUIDE.md#best-practices) for complete examples.

### 5. Property-Based Testing

**Juniors test**: Fixed inputs  
**Experts test**: Random inputs, property invariants

See [User Guide - Property-Based Testing](../../docs/USER_GUIDE.md#property-based-testing) for complete documentation.

```rust
use chicago_tdd_tools::prelude::*;

// ✅ EXPERT: Property-based testing
#[cfg(feature = "property-testing")]
chicago_test!(test_reverse_property, {
    use chicago_tdd_tools::property::PropertyTestGenerator;
    
    // Arrange: Create generator
    let mut generator = PropertyTestGenerator::<10, 3>::new()
        .with_seed(42);
    
    // Act & Assert: Test property for all generated inputs
    assert!(
        property_all_reverses_correctly(&mut generator, 1000),
        "Property: reverse(reverse(x)) == x for all inputs"
    );
});
```

### 6. Integration Testing with Real Collaborators

**Juniors test**: Mocked dependencies  
**Experts test**: Real dependencies, real interactions

See [User Guide - Testcontainers Integration](../../docs/USER_GUIDE.md#testcontainers-integration) for complete documentation.

```rust
use chicago_tdd_tools::prelude::*;

// ✅ EXPERT: Integration test with real dependencies
#[cfg(feature = "testcontainers")]
chicago_test!(test_integration_real_database, {
    use chicago_tdd_tools::testcontainers::*;
    
    // Arrange: Use real test database (not mock)
    let client = ContainerClient::new();
    let container = GenericContainer::new(
        client.client(),
        "postgres",
        "latest"
    ).unwrap();
    
    // Act: Execute real operations
    let port = container.get_host_port(5432).unwrap();
    
    // Assert: Verify real state changes
    assert!(port > 0, "Database port should be valid");
    
    // Container automatically cleaned up on drop
});
```

## Testing Checklist for Expert-Level Coverage

Before marking tests complete, verify expert-level coverage. See [User Guide - Best Practices](../../docs/USER_GUIDE.md#best-practices) for complete checklist.

- [ ] **Error paths**: All error variants tested (not just happy path)
- [ ] **Boundary conditions**: Empty, single, max, zero, negative tested
- [ ] **Resource cleanup**: Cleanup tested in error paths and panic paths
- [ ] **Concurrency**: Concurrent access patterns tested (if applicable)
- [ ] **Memory safety**: No leaks, use-after-free, double-free (use Miri: `cargo miri test`)
- [ ] **Property-based**: Key invariants tested with random inputs
- [ ] **Regression**: Previously fixed bugs have regression tests
- [ ] **Integration**: Real dependencies tested, not just mocks
- [ ] **Panic safety**: Panics don't corrupt state
- [ ] **Unsafe invariants**: Unsafe code invariants verified (if applicable)
- [ ] **Send/Sync bounds**: Concurrency safety verified
- [ ] **Drop behavior**: Drop implementations tested
- [ ] **Lifetime safety**: References don't outlive data (use Miri)

## Common Junior Mistakes to Avoid

See [User Guide - Anti-patterns](../../docs/USER_GUIDE.md#anti-patterns) for complete list.

1. ❌ **Only testing happy path** - Most bugs are in error paths (80% of bugs)
2. ❌ **Not testing boundary conditions** - Edge cases cause production bugs
3. ❌ **Not testing resource cleanup** - Leaks accumulate over time
4. ❌ **Not testing concurrency** - Race conditions are hard to reproduce
5. ❌ **Not testing with real dependencies** - Mocks hide integration issues
6. ❌ **Not testing panic safety** - Panics can corrupt state
7. ❌ **Not using property-based testing** - Fixed inputs miss edge cases
8. ❌ **Not testing regressions** - Bugs come back without regression tests

## Tools for Expert Testing

- **Miri**: Memory safety testing (`cargo miri test`)
- **Loom**: Concurrency testing (`cargo test --features loom`)
- **Property-based**: `chicago-tdd-tools` property testing features (see [User Guide](../../docs/USER_GUIDE.md#property-based-testing))
- **Fuzzing**: `cargo fuzz` for random input testing
- **Sanitizers**: AddressSanitizer, ThreadSanitizer
- **Testcontainers**: Real Docker containers (see [User Guide](../../docs/USER_GUIDE.md#testcontainers-integration))

## Example: Complete Expert Test Suite

See [User Guide - Best Practices](../../docs/USER_GUIDE.md#best-practices) for complete examples.

```rust
use chicago_tdd_tools::prelude::*;

mod expert_tests {
    use super::*;
    
    // 1. Error path testing
    chicago_test!(test_all_error_variants, { /* ... */ });
    
    // 2. Boundary conditions
    chicago_test!(test_boundary_conditions, { /* ... */ });
    
    // 3. Resource cleanup
    chicago_test!(test_resource_cleanup_all_paths, { /* ... */ });
    
    // 4. Concurrency
    chicago_test!(test_concurrent_access, { /* ... */ });
    
    // 5. Property-based
    #[cfg(feature = "property-testing")]
    chicago_test!(test_property_invariants, { /* ... */ });
    
    // 6. Regression
    chicago_test!(test_regression_bug_123, { /* ... */ });
    
    // 7. Integration
    #[cfg(feature = "testcontainers")]
    chicago_test!(test_integration_real_deps, { /* ... */ });
    
    // 8. Panic safety
    chicago_test!(test_panic_safety, { /* ... */ });
}
```

## Summary

Expert-level testing focuses on the **80/20 rule**: Test the 20% of cases that cause 80% of bugs:
- Error paths (not just happy path)
- Boundary conditions (not just normal values)
- Resource cleanup (not just normal execution)
- Concurrency (not just single-threaded)
- Real dependencies (not just mocks)

**Remember**: "Never trust the text, only trust test results" - especially for error paths and edge cases.

## Documentation

- **[Getting Started Guide](../../docs/GETTING_STARTED.md)** - Quick start with verified examples
- **[User Guide](../../docs/USER_GUIDE.md)** - Complete testing guide with patterns
- **[API Reference](../../docs/API_REFERENCE.md)** - Complete API documentation
- **[Architecture](../../docs/ARCHITECTURE.md)** - Design principles
