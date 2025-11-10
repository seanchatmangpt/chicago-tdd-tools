# Expert-Level Testing Patterns - Multi-Step Workflow

## Purpose

This command guides agents through implementing expert-level testing patterns that catch 80% of production bugs. It breaks down complex testing scenarios into clear, sequential steps with examples and validation checkpoints.

## Workflow Overview

```
Step 1: Identify Test Type → Step 2: Choose Pattern → Step 3: Implement Test → Step 4: Verify Coverage → Step 5: Validate Quality
```

## Documentation Reference

For complete testing documentation, see:
- **[Getting Started Guide](../../docs/GETTING_STARTED.md)** - Quick start with verified examples
- **[User Guide](../../docs/USER_GUIDE.md)** - Comprehensive testing guide with patterns
- **[API Reference](../../docs/API_REFERENCE.md)** - Complete API documentation
- **[Architecture](../../docs/ARCHITECTURE.md)** - Design principles

## Core Principle: 80/20 Rule

**Expert testing focuses on the 20% of test cases that catch 80% of bugs**:
- Error paths (not just happy path)
- Boundary conditions (not just normal values)
- Resource cleanup (not just normal execution)
- Concurrency (not just single-threaded)
- Real dependencies (not just mocks)

## Step-by-Step Pattern Implementation

### Pattern 1: Error Path Testing (Critical - 80% of bugs)

#### Step 1.1: Identify Error Scenarios

**Action**: List all possible error conditions for the function/feature.

**Questions to ask**:
- What inputs cause errors?
- What error variants exist?
- Can errors be recovered from?
- Are errors properly propagated?

**Example**: For `parse_number(input: &str) -> Result<u32, ParseError>`
- Empty input → `ParseError::EmptyInput`
- Invalid format → `ParseError::InvalidFormat`
- Overflow → `ParseError::Overflow`
- Edge cases: `"-0"`, `" 42 "`, etc.

#### Step 1.2: Create Test Cases

**Action**: Create test cases for each error scenario.

```rust
use chicago_tdd_tools::prelude::*;

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
});
```

#### Step 1.3: Test Error Recovery

**Action**: Verify system can recover from errors.

```rust
chicago_test!(test_error_recovery, {
    // Arrange: Create parser
    let mut parser = NumberParser::new();
    
    // Act: Cause error
    assert_err!(&parser.parse("invalid"));
    
    // Assert: Parser should still be usable after error
    assert_ok!(&parser.parse("42"), "Parser should recover from error");
});
```

#### Step 1.4: Verify Coverage

**Checklist**:
- [ ] All error variants tested
- [ ] Error messages verified
- [ ] Error recovery tested
- [ ] Edge cases covered

**Reference**: See [User Guide - Best Practices](../../docs/USER_GUIDE.md#best-practices)

---

### Pattern 2: Boundary Condition Testing

#### Step 2.1: Identify Boundaries

**Action**: List all boundary conditions.

**Common boundaries**:
- Empty collections
- Single item
- Maximum size
- Zero values
- Negative values (if applicable)
- Minimum/maximum ranges

#### Step 2.2: Create Boundary Tests

**Action**: Test each boundary condition.

```rust
use chicago_tdd_tools::prelude::*;

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

#### Step 2.3: Verify Coverage

**Checklist**:
- [ ] Empty collection tested
- [ ] Single item tested
- [ ] Maximum size tested (safely)
- [ ] Zero values tested
- [ ] Negative values tested (if applicable)

---

### Pattern 3: Resource Cleanup Testing

#### Step 3.1: Identify Resources

**Action**: List all resources that need cleanup.

**Common resources**:
- File handles
- Network connections
- Database connections
- Memory allocations
- Locks/mutexes

#### Step 3.2: Test Normal Cleanup

**Action**: Verify resources are cleaned up in normal execution.

```rust
use chicago_tdd_tools::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

struct TestResource {
    id: usize,
}

impl Drop for TestResource {
    fn drop(&mut self) {
        DROP_COUNT.fetch_add(1, Ordering::SeqCst);
    }
}

chicago_test!(test_resource_cleanup_normal_path, {
    // Arrange: Reset counter
    DROP_COUNT.store(0, Ordering::SeqCst);
    
    // Act: Create and drop resource
    {
        let resource = TestResource { id: 1 };
        // Resource should drop here
    }
    
    // Assert: Verify cleanup
    assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 1, "Resource should be dropped");
});
```

#### Step 3.3: Test Error Path Cleanup

**Action**: Verify resources are cleaned up even when errors occur.

```rust
chicago_test!(test_resource_cleanup_error_path, {
    // Arrange: Reset counter
    DROP_COUNT.store(0, Ordering::SeqCst);
    
    // Act: Create resource, then error
    let result: Result<(), String> = (|| {
        let resource = TestResource { id: 2 };
        return Err("error".to_string()); // Error path
        // Resource should still drop
    })();
    
    // Assert: Verify cleanup happened
    assert_err!(&result);
    assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 1, "Resource should drop even in error path");
});
```

#### Step 3.4: Test Panic Safety

**Action**: Verify resources are cleaned up even on panic.

```rust
chicago_test!(test_resource_cleanup_panic_safety, {
    // Arrange: Reset counter
    DROP_COUNT.store(0, Ordering::SeqCst);
    
    // Act: Create resource, then panic
    let result = std::panic::catch_unwind(|| {
        let resource = TestResource { id: 3 };
        panic!("test panic");
        // Resource should still drop
    });
    
    // Assert: Verify cleanup happened
    assert!(result.is_err(), "Should catch panic");
    assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 1, "Resource should drop even on panic");
});
```

#### Step 3.5: Verify Coverage

**Checklist**:
- [ ] Normal cleanup tested
- [ ] Error path cleanup tested
- [ ] Panic safety tested
- [ ] Double-drop safety verified (if applicable)

---

### Pattern 4: Concurrency Testing

#### Step 4.1: Identify Concurrency Scenarios

**Action**: List concurrent access patterns.

**Common scenarios**:
- Multiple threads accessing shared state
- Race conditions
- Deadlocks
- Send/Sync bounds

#### Step 4.2: Test Concurrent Access

**Action**: Create concurrent test.

```rust
use chicago_tdd_tools::prelude::*;
use std::sync::Arc;
use std::thread;

chicago_test!(test_concurrent_access, {
    // Arrange: Shared state
    let counter = Arc::new(std::sync::Mutex::new(0));
    let mut handles = vec![];
    
    // Act: Spawn multiple threads
    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let mut value = counter.lock().unwrap();
                *value += 1;
            }
        });
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Assert: Verify final state
    let final_value = counter.lock().unwrap();
    assert_eq!(*final_value, 1000, "All increments should be applied");
});
```

#### Step 4.3: Test Send/Sync Bounds

**Action**: Verify concurrency safety.

```rust
chicago_test!(test_send_sync_bounds, {
    // Arrange: Type that should be Send + Sync
    let counter = Arc::new(std::sync::Mutex::new(0));
    
    // Assert: Verify Send + Sync bounds
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<Arc<Mutex<i32>>>();
    assert_sync::<Arc<Mutex<i32>>>();
});
```

#### Step 4.4: Verify Coverage

**Checklist**:
- [ ] Concurrent access tested
- [ ] Race conditions tested
- [ ] Send/Sync bounds verified
- [ ] Deadlock prevention tested (if applicable)

---

### Pattern 5: Property-Based Testing

#### Step 5.1: Identify Properties

**Action**: List invariants that should always hold.

**Common properties**:
- Reversibility: `reverse(reverse(x)) == x`
- Idempotency: `f(f(x)) == f(x)`
- Commutativity: `f(a, b) == f(b, a)`
- Associativity: `f(f(a, b), c) == f(a, f(b, c))`

#### Step 5.2: Implement Property Test

**Action**: Create property test function.

```rust
use chicago_tdd_tools::prelude::*;

#[cfg(feature = "property-testing")]
fn property_all_reverses_correctly(
    generator: &mut PropertyTestGenerator<10, 3>,
    iterations: usize,
) -> bool {
    for _ in 0..iterations {
        let data = generator.generate_test_data();
        // Test property: reverse(reverse(x)) == x
        let reversed_once: HashMap<_, _> = data.iter()
            .map(|(k, v)| (v.clone(), k.clone()))
            .collect();
        let reversed_twice: HashMap<_, _> = reversed_once.iter()
            .map(|(k, v)| (v.clone(), k.clone()))
            .collect();
        
        if data != reversed_twice {
            return false; // Property violated
        }
    }
    true // Property holds for all tested inputs
}

#[cfg(feature = "property-testing")]
chicago_test!(test_reverse_property, {
    // Arrange: Create generator
    let mut generator = PropertyTestGenerator::<10, 3>::new()
        .with_seed(42);
    
    // Act & Assert: Test property
    assert!(
        property_all_reverses_correctly(&mut generator, 1000),
        "Property: reverse(reverse(x)) == x for all inputs"
    );
});
```

#### Step 5.3: Verify Coverage

**Checklist**:
- [ ] Property function implemented
- [ ] Generator configured with seed
- [ ] Sufficient iterations (100+)
- [ ] Property violation detection works

**Reference**: See [User Guide - Property-Based Testing](../../docs/USER_GUIDE.md#property-based-testing)

---

### Pattern 6: Integration Testing with Real Collaborators

#### Step 6.1: Identify Dependencies

**Action**: List external dependencies.

**Common dependencies**:
- Databases
- APIs
- File systems
- Network services

#### Step 6.2: Set Up Real Dependencies

**Action**: Use testcontainers or real test services.

```rust
use chicago_tdd_tools::prelude::*;

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
    
    let port = container.get_host_port(5432).unwrap();
    
    // Act: Execute real operations
    let db = connect_to_database(port).unwrap();
    let user = create_user(&db, "test_user").await.unwrap();
    let retrieved = get_user(&db, user.id).await.unwrap();
    
    // Assert: Verify real state changes
    assert_eq!(retrieved.id, user.id);
    assert_eq!(retrieved.name, "test_user");
    
    // Verify: Database actually persisted data
    let count = count_users(&db).await.unwrap();
    assert_eq!(count, 1, "User should be persisted in database");
    
    // Cleanup: Automatic via Drop trait
});
```

#### Step 6.3: Verify Coverage

**Checklist**:
- [ ] Real dependencies used (not mocks)
- [ ] State changes verified
- [ ] Cleanup verified
- [ ] Integration points tested

**Reference**: See [User Guide - Testcontainers Integration](../../docs/USER_GUIDE.md#testcontainers-integration)

---

## Complete Expert Test Suite Template

### Step 7: Create Complete Test Suite

**Action**: Combine all patterns into comprehensive test suite.

```rust
use chicago_tdd_tools::prelude::*;

mod expert_tests {
    use super::*;
    
    // 1. Error path testing
    chicago_test!(test_all_error_variants, {
        // Implement error path tests
    });
    
    // 2. Boundary conditions
    chicago_test!(test_boundary_conditions, {
        // Implement boundary tests
    });
    
    // 3. Resource cleanup
    chicago_test!(test_resource_cleanup_all_paths, {
        // Implement cleanup tests
    });
    
    // 4. Concurrency
    chicago_test!(test_concurrent_access, {
        // Implement concurrency tests
    });
    
    // 5. Property-based
    #[cfg(feature = "property-testing")]
    chicago_test!(test_property_invariants, {
        // Implement property tests
    });
    
    // 6. Regression
    chicago_test!(test_regression_bug_123, {
        // Implement regression tests
    });
    
    // 7. Integration
    #[cfg(feature = "testcontainers")]
    chicago_test!(test_integration_real_deps, {
        // Implement integration tests
    });
    
    // 8. Panic safety
    chicago_test!(test_panic_safety, {
        // Implement panic safety tests
    });
}
```

### Step 8: Verify Expert Coverage

**Checklist**: Before marking tests complete, verify expert-level coverage:

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

---

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

---

## Tools for Expert Testing

- **Miri**: Memory safety testing (`cargo miri test`)
- **Loom**: Concurrency testing (`cargo test --features loom`)
- **Property-based**: `chicago-tdd-tools` property testing features (see [User Guide](../../docs/USER_GUIDE.md#property-based-testing))
- **Fuzzing**: `cargo fuzz` for random input testing
- **Sanitizers**: AddressSanitizer, ThreadSanitizer
- **Testcontainers**: Real Docker containers (see [User Guide](../../docs/USER_GUIDE.md#testcontainers-integration))

---

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
