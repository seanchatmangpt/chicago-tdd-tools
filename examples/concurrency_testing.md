# Concurrency Testing Example

**Category:** How-To Guide
**Level:** Advanced
**Prerequisites:** Understanding of concurrency and threading
**Features Required:** `concurrency-testing`

---

## Overview

This example demonstrates concurrency testing using `loom` for Chicago TDD. Loom performs model checking to explore all possible thread interleavings and find concurrency bugs.

**What you'll learn:**
- Using `ConcurrencyTest` to test concurrent operations
- Model checking with loom
- Testing thread safety
- Finding race conditions

---

## Quick Start

```bash
cargo test --features concurrency-testing --example concurrency_testing
```

---

## Prerequisites

- Rust 1.70+ (Edition 2021)
- Chicago TDD Tools with `concurrency-testing` feature
- Understanding of threading and synchronization

**Add to Cargo.toml:**
```toml
[dev-dependencies]
chicago-tdd-tools = { path = "../chicago-tdd-tools", features = ["concurrency-testing"] }
```

---

## Key Concepts

### Model Checking

Loom explores **all** possible thread interleavings systematically. Instead of running tests multiple times hoping to hit a race condition, loom guarantees all interleavings are tested.

**Traditional Testing:**
- Run test 1000 times
- Hope to hit the rare interleaving
- Race conditions may still slip through

**Model Checking:**
- Systematically explore all interleavings
- Guaranteed to find race conditions
- Deterministic results

### Thread Interleavings

Different orders in which threads execute. A race condition may only occur in specific interleavings.

**Example:**
```
Thread 1: counter += 1
Thread 2: counter += 1

Interleavings:
1. T1 reads, T1 writes, T2 reads, T2 writes → counter = 2 ✓
2. T1 reads, T2 reads, T1 writes, T2 writes → counter = 1 ✗ (race!)
```

---

## Code Examples

### Example 1: Concurrent Counter

```rust
use chicago_tdd_tools::concurrency::ConcurrencyTest;
use std::sync::{Arc, Mutex};
use loom::thread;

#[test]
fn test_concurrent_counter() {
    ConcurrencyTest::run(|| {
        // Arrange: Create shared counter
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = Arc::clone(&counter);

        // Act: Spawn thread and increment
        let handle = thread::spawn(move || {
            *counter_clone.lock().unwrap() += 1;
        });

        *counter.lock().unwrap() += 1;
        handle.join().unwrap();

        // Assert: Counter should be 2
        let final_value = *counter.lock().unwrap();
        assert_eq!(final_value, 2);
    });
}
```

**Key Points:**
- Loom explores all interleavings
- Test passes only if correct for all interleavings
- Race conditions caught automatically

### Example 2: Concurrent Vector Operations

```rust
use chicago_tdd_tools::concurrency::ConcurrencyTest;
use std::sync::{Arc, Mutex};
use loom::thread;

#[test]
fn test_concurrent_vector_push() {
    ConcurrencyTest::run(|| {
        // Arrange: Create shared vector
        let vec = Arc::new(Mutex::new(Vec::new()));
        let vec_clone = Arc::clone(&vec);

        // Act: Push from both threads
        let handle = thread::spawn(move || {
            vec_clone.lock().unwrap().push(1);
        });

        vec.lock().unwrap().push(2);
        handle.join().unwrap();

        // Assert: Vector should contain both values
        let final_vec = vec.lock().unwrap();
        assert_eq!(final_vec.len(), 2);
        assert!(final_vec.contains(&1));
        assert!(final_vec.contains(&2));
    });
}
```

### Example 3: Custom Configuration

```rust
use chicago_tdd_tools::concurrency::ConcurrencyTest;

#[test]
fn test_with_custom_config() {
    ConcurrencyTest::run_with_config(4, 1000, || {
        // Max 4 threads, max 1000 preemptions
        // ... concurrent test code ...
    });
}
```

---

## Common Patterns

### Pattern 1: Shared State Testing

```rust
ConcurrencyTest::run(|| {
    let shared_data = Arc::new(Mutex::new(initial_value));
    let clone = Arc::clone(&shared_data);

    let handle = thread::spawn(move || {
        // Modify shared data
    });

    // Modify from main thread
    handle.join().unwrap();

    // Verify final state
});
```

### Pattern 2: Channel Testing

```rust
ConcurrencyTest::run(|| {
    let (tx, rx) = loom::sync::mpsc::channel();

    thread::spawn(move || {
        tx.send(42).unwrap();
    });

    let value = rx.recv().unwrap();
    assert_eq!(value, 42);
});
```

### Pattern 3: Atomic Operations

```rust
ConcurrencyTest::run(|| {
    use loom::sync::atomic::{AtomicUsize, Ordering};

    let counter = Arc::new(AtomicUsize::new(0));
    let clone = Arc::clone(&counter);

    thread::spawn(move || {
        clone.fetch_add(1, Ordering::SeqCst);
    });

    counter.fetch_add(1, Ordering::SeqCst);
    assert_eq!(counter.load(Ordering::SeqCst), 2);
});
```

---

## Best Practices

### 1. Use Proper Synchronization

```rust
// ✓ Good: Using Mutex
let data = Arc::new(Mutex::new(vec![]));

// ✗ Bad: No synchronization
let data = Arc::new(vec![]);  // Race condition!
```

### 2. Test Small Units

```rust
// ✓ Good: Test specific operation
ConcurrencyTest::run(|| {
    test_counter_increment();
});

// ✗ Bad: Test entire system
ConcurrencyTest::run(|| {
    test_entire_application();  // Too many interleavings
});
```

### 3. Keep Tests Deterministic

```rust
// ✓ Good: Deterministic operations
counter.fetch_add(1, Ordering::SeqCst);

// ✗ Bad: Non-deterministic operations
std::thread::sleep(Duration::from_millis(10));  // Don't use real sleep
```

---

## Troubleshooting

### Error: "concurrency-testing feature required"

**Cause:** Feature not enabled

**Fix:**
```toml
[dev-dependencies]
chicago-tdd-tools = { path = "../chicago-tdd-tools", features = ["concurrency-testing"] }
```

### Test Hangs or Times Out

**Cause:** Too many interleavings to explore

**Fix:** Reduce scope or use custom config:
```rust
ConcurrencyTest::run_with_config(2, 100, || {
    // Fewer threads and preemptions
});
```

### Race Condition Found

**Cause:** Improper synchronization

**Fix:**
- Add Mutex around shared data
- Use atomic operations
- Use channels for communication

---

## Next Steps

After mastering concurrency testing, explore:

1. **[Property Testing](property_testing.md)** - Random test generation
2. **[Mutation Testing](mutation_testing.md)** - Test quality
3. **[Testcontainers](testcontainers_example.md)** - Integration testing

---

## Related Documentation

- [Examples README](README.md) - All examples overview
- [loom documentation](https://docs.rs/loom/) - Complete loom guide
- [API Reference](../docs/reference/API_REFERENCE.md) - Complete API documentation

---

## Reference

### Key Functions

- `ConcurrencyTest::run(test)` - Run test with default config
- `ConcurrencyTest::run_with_config(threads, preemptions, test)` - Custom config

### Loom Primitives

- `loom::thread::spawn` - Spawn thread
- `loom::sync::Mutex` - Mutex
- `loom::sync::Arc` - Arc
- `loom::sync::atomic` - Atomic types
- `loom::sync::mpsc` - Channels

### Configuration

- `threads`: Maximum number of threads to simulate
- `preemptions`: Maximum number of preemptions to explore

---

**Quality is the default. Prevention beats detection.**

*Example: concurrency_testing.rs | Version: 1.2.0 | Updated: 2025-11-15*
