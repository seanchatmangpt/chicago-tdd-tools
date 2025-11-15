# Concurrency Testing

Test thread-safe code with deterministic thread ordering using loom.

## Why Concurrency Testing?

Normal tests run threads in random order - race conditions may not appear:

```rust
// ❌ This might pass or fail randomly
test!(test_race_condition, {
    let data = Arc::new(Mutex::new(0));
    let data_clone = data.clone();

    thread::spawn(move || {
        *data_clone.lock().unwrap() += 1;
    });

    thread::sleep(Duration::from_millis(1));
    let result = *data.lock().unwrap();
    assert_eq!(result, 1);  // Might fail if thread hasn't run yet
});
```

Loom testing explores **all possible interleavings**:

```rust
// ✅ This tests all possible thread orderings
test!(test_with_loom, {
    loom::model(|| {
        let data = Arc::new(Mutex::new(0));
        let data_clone = data.clone();

        thread::spawn(move || {
            *data_clone.lock().unwrap() += 1;
        });

        let result = *data.lock().unwrap();
        assert_eq!(result, 1);  // Tests all interleavings
    });
});
```

## Basic Loom Testing

### Simple Loom Model

```rust
use chicago_tdd_tools::concurrency::*;
use std::sync::{Arc, Mutex};

test!(test_basic_loom, {
    loom::model(|| {
        let data = Arc::new(Mutex::new(0));
        let value = *data.lock().unwrap();
        assert_eq!(value, 0);
    });
});
```

### Two Threads

```rust
test!(test_two_threads, {
    loom::model(|| {
        let data = Arc::new(Mutex::new(0));

        let data_clone = data.clone();
        thread::spawn(move || {
            *data_clone.lock().unwrap() += 1;
        });

        let result = *data.lock().unwrap();
        // Loom tests both possible interleavings:
        // 1. Main thread reads first (0)
        // 2. Worker thread increments first (1)
    });
});
```

## Real-World Example: Counter

```rust
test!(test_concurrent_counter, {
    loom::model(|| {
        let counter = Arc::new(Mutex::new(0));

        let mut handles = vec![];

        // Spawn 3 threads
        for _ in 0..3 {
            let counter = counter.clone();
            let handle = thread::spawn(move || {
                *counter.lock().unwrap() += 1;
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // All threads should have incremented
        assert_eq!(*counter.lock().unwrap(), 3);
    });
});
```

## Real-World Example: Channel Communication

```rust
test!(test_channel_communication, {
    loom::model(|| {
        let (tx, rx) = std::sync::mpsc::channel();

        thread::spawn(move || {
            tx.send(42).unwrap();
        });

        let value = rx.recv().unwrap();
        assert_eq!(value, 42);
    });
});
```

## Common Concurrency Patterns

### Mutex Protection

```rust
test!(test_mutex_safety, {
    loom::model(|| {
        let data = Arc::new(Mutex::new(vec![]));

        let data_clone = data.clone();
        thread::spawn(move || {
            data_clone.lock().unwrap().push(1);
        });

        data.lock().unwrap().push(2);
        let result = data.lock().unwrap();
        assert_eq!(result.len(), 2);
    });
});
```

### RwLock (Reader-Writer Lock)

```rust
test!(test_rwlock, {
    loom::model(|| {
        let data = Arc::new(RwLock::new(0));

        let data_clone = data.clone();
        thread::spawn(move || {
            *data_clone.write().unwrap() = 42;
        });

        let value = *data.read().unwrap();
        assert_eq!(value, 42);
    });
});
```

### Atomic Operations

```rust
test!(test_atomic, {
    loom::model(|| {
        use std::sync::atomic::{AtomicU32, Ordering};

        let counter = Arc::new(AtomicU32::new(0));

        let counter_clone = counter.clone();
        thread::spawn(move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        let value = counter.load(Ordering::SeqCst);
        // Value might be 0 or 1 depending on scheduling
    });
});
```

## Detecting Race Conditions

### Race Condition Example

```rust
test!(test_detects_race_condition, {
    loom::model(|| {
        let data = Arc::new(Cell::new(0));  // ❌ Not thread-safe!

        let data_clone = data.clone();
        thread::spawn(move || {
            data_clone.set(data_clone.get() + 1);
        });

        // This will fail with loom!
        // Cell doesn't provide synchronization
    });
});
```

Loom detects this because `Cell` isn't thread-safe.

### Use Mutex Instead

```rust
test!(test_thread_safe, {
    loom::model(|| {
        let data = Arc::new(Mutex::new(0));  // ✅ Thread-safe

        let data_clone = data.clone();
        thread::spawn(move || {
            *data_clone.lock().unwrap() += 1;
        });

        let result = *data.lock().unwrap();
        // Now safe for all interleavings
    });
});
```

## Testing for Deadlocks

Loom can detect potential deadlocks:

```rust
test!(test_deadlock_detection, {
    loom::model(|| {
        let lock1 = Arc::new(Mutex::new(0));
        let lock2 = Arc::new(Mutex::new(0));

        let (lock1_clone, lock2_clone) = (lock1.clone(), lock2.clone());
        thread::spawn(move || {
            // Thread 1: Lock in order lock1, lock2
            let _g1 = lock1_clone.lock().unwrap();
            let _g2 = lock2_clone.lock().unwrap();
        });

        // Main thread: Lock in opposite order lock2, lock1
        // Loom will explore both interleavings
        // Can detect potential deadlock!
    });
});
```

## Best Practices

✅ **Do:**
- Test small, focused scenarios
- Use appropriate synchronization primitives
- Test with few threads (2-3 typical)
- Verify all possible interleavings
- Use Loom for critical concurrent code

❌ **Don't:**
- Test large thread pools (explodes combinations)
- Mix blocking I/O with Loom (I/O not deterministic)
- Over-test (Loom is slow, only use for critical code)
- Assume one test covers all cases

## Performance

Loom explores all interleavings - it's **slow**:

- Simple model (2 threads): 10ms - 100ms
- Complex model (3 threads): 100ms - 1s
- Many threads: Can be very slow

**Recommendation**:
- Only use Loom for critical synchronization
- Test with 2-3 threads, not more
- Use normal tests for non-concurrent code

## Limitations

Loom only works with:
- Loom-aware primitives (`loom::sync`)
- Thread creation (`loom::thread`)
- Standard Rust types it instruments

Cannot test:
- Real time (time is controlled)
- System I/O (returns dummy values)
- External libraries (unless they use loom)

## Combining with Other Techniques

### Concurrency + Property-Based

```rust
test!(test_concurrent_property, {
    loom::model(|| {
        let counter = Arc::new(Mutex::new(0));

        for i in 0..5 {
            let counter = counter.clone();
            thread::spawn(move || {
                *counter.lock().unwrap() += i;
            });
        }

        // Verify invariant holds
        let sum: u32 = (0..5).sum();
        assert!(*counter.lock().unwrap() <= sum);
    });
});
```

## Real-World Integration Example

```rust
test!(test_thread_pool_safety, {
    loom::model(|| {
        let task_queue = Arc::new(Mutex::new(vec![]));
        let result_queue = Arc::new(Mutex::new(vec![]));

        // Producer
        {
            let queue = task_queue.clone();
            thread::spawn(move || {
                queue.lock().unwrap().push("task1");
            });
        }

        // Consumer
        {
            let task_queue = task_queue.clone();
            let result_queue = result_queue.clone();
            thread::spawn(move || {
                if let Some(task) = task_queue.lock().unwrap().pop() {
                    result_queue.lock().unwrap().push(format!("done: {}", task));
                }
            });
        }

        // Verify result eventually
        // (Loom explores all orderings)
    });
});
```

## Troubleshooting

### "Too many interleavings"

Reduce complexity:
- Use fewer threads
- Smaller critical sections
- Simpler synchronization patterns

### "This synchronization is not supported"

Use only Loom-supported primitives:
- `loom::sync::Mutex`
- `loom::sync::RwLock`
- `std::sync::atomic`
- `std::sync::mpsc`

### Test Still Hangs/Deadlocks

Loom doesn't catch all deadlocks. Use timeouts:

```rust
#[test]
#[timeout = "5s"]  // Add timeout
fn test_with_timeout() {
    loom::model(|| {
        // Test code
    });
}
```

## Next Steps

Learn the "Go the Extra Mile" pattern: [Go the Extra Mile](../guides/extra-mile.md)

---

## Summary

Concurrency testing with Loom:
- ✅ Tests all possible thread interleavings
- ✅ Detects race conditions
- ✅ Verifies synchronization correctness
- ✅ Finds potential deadlocks

Use for critical concurrent code to ensure thread safety.

