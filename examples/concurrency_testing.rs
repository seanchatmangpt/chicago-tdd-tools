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
    fn test_concurrent_counter() {
        // Arrange: Create shared counter
        ConcurrencyTest::run(|| {
            let counter = Arc::new(Mutex::new(0));
            let counter_clone = Arc::clone(&counter);

            // Act: Spawn thread to increment counter
            thread::spawn(move || {
                *counter_clone.lock().unwrap() += 1;
            });

            // Act: Main thread also increments counter
            *counter.lock().unwrap() += 1;

            // Assert: Counter should be 2 (both threads incremented)
            // Loom explores all interleavings to ensure this is always true
        });
    }

    #[test]
    fn test_concurrent_vector_push() {
        // Arrange: Create shared vector
        ConcurrencyTest::run(|| {
            let vec = Arc::new(Mutex::new(Vec::new()));
            let vec_clone = Arc::clone(&vec);

            // Act: Spawn thread to push value
            thread::spawn(move || {
                vec_clone.lock().unwrap().push(1);
            });

            // Act: Main thread also pushes value
            vec.lock().unwrap().push(2);

            // Assert: Vector should contain both values
            // Loom explores all interleavings to ensure this is always true
        });
    }

    #[test]
    fn test_concurrent_with_config() {
        // Arrange: Create shared data
        ConcurrencyTest::run_with_config(4, 1000, || {
            let data = Arc::new(Mutex::new(0));
            let data_clone = Arc::clone(&data);

            // Act: Spawn multiple threads
            thread::spawn(move || {
                *data_clone.lock().unwrap() += 1;
            });

            // Act: Main thread modifies data
            *data.lock().unwrap() += 1;

            // Assert: Data is modified correctly
            // Loom explores all interleavings with custom config
        });
    }
}
