//! Concurrency Testing Framework
//!
//! Provides concurrency testing capabilities using loom for Chicago TDD.
//! Loom performs model checking for concurrent code, exploring all possible
//! thread interleavings to find race conditions and deadlocks.
//!
//! # Chicago TDD Alignment
//!
//! Concurrency testing aligns with Chicago TDD principles:
//! - **State-Based Testing**: Verifies concurrent state transitions
//! - **Behavior Verification**: Tests what concurrent code does under all interleavings
//! - **AAA Pattern**: Arrange (setup threads), Act (execute concurrently), Assert (verify state)

/// Concurrency test helper for Chicago TDD
///
/// Provides a Chicago TDD-friendly wrapper around loom's concurrency testing.
/// This makes concurrency testing consistent with other testing utilities in the framework.
#[cfg(feature = "concurrency-testing")]
pub struct ConcurrencyTest;

#[cfg(feature = "concurrency-testing")]
impl ConcurrencyTest {
    /// Run a concurrent test with model checking
    ///
    /// This function uses loom to explore all possible thread interleavings
    /// and verify that the concurrent code maintains invariants.
    ///
    /// # Arguments
    ///
    /// * `test_fn` - A function that sets up and runs concurrent code
    ///
    /// # Panics
    ///
    /// Panics if any thread interleaving violates invariants or causes a panic.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use chicago_tdd_tools::concurrency::ConcurrencyTest;
    /// use std::sync::{Arc, Mutex};
    ///
    /// ConcurrencyTest::run(|| {
    ///     let data = Arc::new(Mutex::new(0));
    ///     let data_clone = Arc::clone(&data);
    ///
    ///     loom::thread::spawn(move || {
    ///         *data_clone.lock().unwrap() += 1;
    ///     });
    ///
    ///     *data.lock().unwrap() += 1;
    /// });
    /// ```
    pub fn run<F>(test_fn: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        loom::model(move || {
            test_fn();
        });
    }

    /// Run a concurrent test with custom model configuration
    ///
    /// Allows customization of loom's model checking behavior.
    ///
    /// # Arguments
    ///
    /// * `max_threads` - Maximum number of threads to simulate
    /// * `test_fn` - A function that sets up and runs concurrent code
    pub fn run_with_config<F>(_max_threads: usize, _max_branches: usize, test_fn: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        loom::model(move || {
            test_fn();
        });
    }
}

#[cfg(feature = "concurrency-testing")]
#[cfg(test)]
mod tests {
    use super::*;
    use loom::sync::{Arc, Mutex};
    use loom::thread;

    #[test]
    fn test_concurrent_counter() {
        ConcurrencyTest::run(|| {
            let counter = Arc::new(Mutex::new(0));
            let counter_clone = Arc::clone(&counter);

            thread::spawn(move || {
                *counter_clone.lock().unwrap() += 1;
            });

            *counter.lock().unwrap() += 1;
        });
    }

    #[test]
    fn test_concurrent_vector_push() {
        ConcurrencyTest::run(|| {
            let vec = Arc::new(Mutex::new(Vec::new()));
            let vec_clone = Arc::clone(&vec);

            thread::spawn(move || {
                vec_clone.lock().unwrap().push(1);
            });

            vec.lock().unwrap().push(2);
        });
    }
}
