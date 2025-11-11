//! Concurrency Testing Examples
//!
//! Demonstrates concurrency testing with loom for deterministic thread model checking.

#[cfg(feature = "concurrency-testing")]
use chicago_tdd_tools::testing::concurrency::ConcurrencyTest;
#[cfg(feature = "concurrency-testing")]
use chicago_tdd_tools::prelude::*;
#[cfg(feature = "concurrency-testing")]
use loom::thread;
#[cfg(feature = "concurrency-testing")]
use std::sync::{Arc, Mutex};

#[cfg(feature = "concurrency-testing")]
/// Example: Concurrent counter
pub fn example_concurrent_counter() {
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

#[cfg(feature = "concurrency-testing")]
/// Example: Concurrent vector push
pub fn example_concurrent_vector() {
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

#[cfg(feature = "concurrency-testing")]
/// Example: Concurrency test with custom config
pub fn example_concurrency_config() {
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

#[cfg(test)]
mod tests {
    #[cfg(feature = "concurrency-testing")]
    use super::*;

    #[cfg(feature = "concurrency-testing")]
    test!(test_concurrent_counter, {
        // Arrange-Act-Assert: Run example
        example_concurrent_counter();
    });

    #[cfg(feature = "concurrency-testing")]
    test!(test_concurrent_vector, {
        // Arrange-Act-Assert: Run example
        example_concurrent_vector();
    });

    #[cfg(feature = "concurrency-testing")]
    test!(test_concurrency_config, {
        // Arrange-Act-Assert: Run example
        example_concurrency_config();
    });
}

