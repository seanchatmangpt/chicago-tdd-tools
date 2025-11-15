//! Concurrency Testing Example
//!
//! Demonstrates concurrency testing using loom for Chicago TDD.
//! Loom performs model checking to explore all possible thread interleavings.

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
