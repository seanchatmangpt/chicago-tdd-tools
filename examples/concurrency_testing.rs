//! # Concurrency Testing Example - Comprehensive Guide
//!
//! Demonstrates concurrency testing using `loom` for Chicago TDD. Loom performs model
//! checking to explore all possible thread interleavings and find concurrency bugs.
//!
//! ## Tutorial: Getting Started
//!
//! This example demonstrates concurrency testing:
//!
//! 1. **Basic Concurrency Test**: Use `ConcurrencyTest::run()` to test concurrent operations
//! 2. **Custom Configuration**: Use `ConcurrencyTest::run_with_config()` for custom settings
//! 3. **Thread Safety**: Verify shared data access is thread-safe
//!
//! **Run tests**: `cargo test --features concurrency-testing --example concurrency_testing`
//!
//! ## Explanation: Concepts
//!
//! **Model Checking**: Loom explores all possible thread interleavings systematically.
//! Instead of running tests multiple times hoping to hit a race condition, loom guarantees
//! all interleavings are tested.
//!
//! **Thread Interleavings**: Different orders in which threads execute. A race condition
//! may only occur in specific interleavings, making it hard to reproduce.
//!
//! **ConcurrencyTest**: Wrapper around `loom` that provides Chicago TDD-compatible API.
//! Runs tests with model checking to explore all interleavings.
//!
//! **Shared State**: Data accessed by multiple threads. Must be protected with synchronization
//! primitives (e.g., `Mutex`, `Arc`) to prevent data races.
//!
//! **Data Races**: Concurrent access to shared mutable state without synchronization.
//! Undefined behavior in Rust - model checking helps find these bugs.
//!
//! ## How-to: Common Tasks
//!
//! - Test concurrent counter: See `test_concurrent_counter`
//! - Test concurrent vector operations: See `test_concurrent_vector_push`
//! - Use custom configuration: See `test_concurrent_with_config`
//!
//! ## Reference: Quick Lookup
//!
//! **Key Types**:
//! - `ConcurrencyTest`: Concurrency test wrapper
//!
//! **Key Functions**:
//! - `ConcurrencyTest::run(test)` - Run test with default configuration
//! - `ConcurrencyTest::run_with_config(threads, preemptions, test)` - Run with custom config
//!
//! **Key Concepts**:
//! - **Model Checking**: Systematic exploration of all thread interleavings
//! - **Interleaving**: Order in which threads execute
//! - **Race Condition**: Bug that occurs in specific interleavings
//! - **Thread Safety**: Code that works correctly in all interleavings

#[cfg(feature = "concurrency-testing")]
#[allow(unused_imports)] // Example code - imports shown for demonstration
use chicago_tdd_tools::concurrency::ConcurrencyTest;
#[cfg(feature = "concurrency-testing")]
#[allow(unused_imports)] // Example code - imports shown for demonstration
use loom::thread;
#[cfg(feature = "concurrency-testing")]
#[allow(unused_imports)] // Example code - imports shown for demonstration
use std::sync::{Arc, Mutex};

#[cfg(feature = "concurrency-testing")]
fn main() {
    println!("Concurrency Testing Example");
    println!("Run: cargo test --features concurrency-testing --example concurrency_testing");
}

#[cfg(not(feature = "concurrency-testing"))]
fn main() {
    println!(
        "Concurrency testing feature not enabled. Enable with: --features concurrency-testing"
    );
}

#[cfg(feature = "concurrency-testing")]
#[cfg(test)]
mod tests {
    use super::*;

    // Example: Concurrent counter test
    //
    // ## How-to: Test Concurrent Operations
    //
    // Use `ConcurrencyTest::run()` to test concurrent operations. Loom explores all
    // possible thread interleavings to find race conditions.
    //
    // ## Reference
    //
    // - **Function**: `ConcurrencyTest::run(test)`
    // - **Parameters**: `test` - Closure containing concurrent test code
    // - **Behavior**: Explores all thread interleavings with model checking
    //
    // # Examples
    //
    // ```rust
    // use chicago_tdd_tools::concurrency::ConcurrencyTest;
    // use std::sync::{Arc, Mutex};
    // use loom::thread;
    //
    // ConcurrencyTest::run(|| {
    //     let counter = Arc::new(Mutex::new(0));
    //     let counter_clone = Arc::clone(&counter);
    //     let handle = thread::spawn(move || {
    //         *counter_clone.lock().unwrap() += 1;
    //     });
    //     *counter.lock().unwrap() += 1;
    //     handle.join().unwrap();
    //     assert_eq!(*counter.lock().unwrap(), 2);
    // });
    // ```
    #[test]
    #[allow(clippy::unwrap_used)] // Test code - Loom guarantees Mutex lock won't fail
    fn test_concurrent_counter() {
        // Arrange: Create shared counter
        ConcurrencyTest::run(|| {
            let counter = Arc::new(Mutex::new(0));
            let counter_clone = Arc::clone(&counter);

            // Act: Spawn thread to increment counter and join it
            let handle = thread::spawn(move || {
                *counter_clone.lock().unwrap() += 1;
            });

            // Act: Main thread also increments counter
            *counter.lock().unwrap() += 1;

            // Wait for spawned thread to complete
            handle.join().unwrap();

            // Assert: Verify behavior - Counter should be 2 (both threads incremented)
            // Loom explores all interleavings to ensure this is always true
            let final_value = *counter.lock().unwrap();
            assert_eq!(final_value, 2, "Counter should be 2 after both threads increment");
        });
    }

    // Example: Concurrent vector operations test
    //
    // ## How-to: Test Concurrent Vector Operations
    //
    // Test concurrent operations on shared vectors. Loom ensures all interleavings
    // are explored, catching race conditions that might only occur in specific orders.
    //
    // ## Reference
    //
    // - **Pattern**: Use `Arc<Mutex<Vec<T>>>` for shared mutable vectors
    // - **Thread Safety**: Mutex ensures only one thread accesses vector at a time
    // - **Model Checking**: Loom explores all interleavings automatically
    //
    // # Examples
    //
    // ```rust
    // use chicago_tdd_tools::concurrency::ConcurrencyTest;
    // use std::sync::{Arc, Mutex};
    // use loom::thread;
    //
    // ConcurrencyTest::run(|| {
    //     let vec = Arc::new(Mutex::new(Vec::new()));
    //     let vec_clone = Arc::clone(&vec);
    //     let handle = thread::spawn(move || {
    //         vec_clone.lock().unwrap().push(1);
    //     });
    //     vec.lock().unwrap().push(2);
    //     handle.join().unwrap();
    //     assert_eq!(vec.lock().unwrap().len(), 2);
    // });
    // ```
    #[test]
    #[allow(clippy::unwrap_used)] // Test code - Loom guarantees Mutex lock won't fail
    fn test_concurrent_vector_push() {
        // Arrange: Create shared vector
        ConcurrencyTest::run(|| {
            let vec = Arc::new(Mutex::new(Vec::new()));
            let vec_clone = Arc::clone(&vec);

            // Act: Spawn thread to push value and join it
            let handle = thread::spawn(move || {
                vec_clone.lock().unwrap().push(1);
            });

            // Act: Main thread also pushes value
            vec.lock().unwrap().push(2);

            // Wait for spawned thread to complete
            handle.join().unwrap();

            // Assert: Verify behavior - Vector should contain both values
            // Loom explores all interleavings to ensure this is always true
            let final_vec = vec.lock().unwrap();
            assert_eq!(final_vec.len(), 2, "Vector should contain 2 values");
            assert!(final_vec.contains(&1), "Vector should contain value 1");
            assert!(final_vec.contains(&2), "Vector should contain value 2");
        });
    }

    // Example: Concurrent test with custom configuration
    //
    // ## How-to: Use Custom Configuration
    //
    // Use `ConcurrencyTest::run_with_config()` to customize model checking parameters:
    // - `threads`: Maximum number of threads to simulate
    // - `preemptions`: Maximum number of preemptions to explore
    //
    // ## Reference
    //
    // - **Function**: `ConcurrencyTest::run_with_config(threads, preemptions, test)`
    // - **Parameters**:
    //   - `threads`: Maximum threads (e.g., 4)
    //   - `preemptions`: Maximum preemptions (e.g., 1000)
    //   - `test`: Closure containing concurrent test code
    // - **Behavior**: Explores interleavings with custom limits
    //
    // # Examples
    //
    // ```rust
    // use chicago_tdd_tools::concurrency::ConcurrencyTest;
    //
    // ConcurrencyTest::run_with_config(4, 1000, || {
    //     // Concurrent test code
    // });
    // ```
    #[test]
    #[allow(clippy::unwrap_used)] // Test code - Loom guarantees Mutex lock won't fail
    fn test_concurrent_with_config() {
        // Arrange: Create shared data
        ConcurrencyTest::run_with_config(4, 1000, || {
            let data = Arc::new(Mutex::new(0));
            let data_clone = Arc::clone(&data);

            // Act: Spawn thread and join it
            let handle = thread::spawn(move || {
                *data_clone.lock().unwrap() += 1;
            });

            // Act: Main thread modifies data
            *data.lock().unwrap() += 1;

            // Wait for spawned thread to complete
            handle.join().unwrap();

            // Assert: Verify behavior - Data should be 2 after both threads modify
            // Loom explores all interleavings with custom config
            let final_value = *data.lock().unwrap();
            assert_eq!(final_value, 2, "Data should be 2 after both threads modify");
        });
    }
}
