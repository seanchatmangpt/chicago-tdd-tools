//! Test Fixtures
//!
//! Provides reusable test fixtures with automatic cleanup and state management.
//! Uses Generic Associated Types (GATs) for flexible, type-safe fixture management.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;

/// Test fixture error
#[derive(Error, Debug)]
pub enum FixtureError {
    /// Failed to create fixture
    #[error("Failed to create fixture: {0}")]
    CreationFailed(String),
    /// Fixture operation failed
    #[error("Fixture operation failed: {0}")]
    OperationFailed(String),
}

/// Result type for fixture operations
pub type FixtureResult<T> = Result<T, FixtureError>;

/// Fixture provider trait using Generic Associated Types (GATs)
///
/// This trait allows for flexible fixture creation with type-safe lifetime management.
/// The `Fixture<'a>` associated type can reference data from the provider.
pub trait FixtureProvider {
    /// The fixture type with a lifetime parameter
    type Fixture<'a>: 'a
    where
        Self: 'a;
    /// Error type for fixture creation
    type Error: std::error::Error + Send + Sync + 'static;

    /// Create a fixture
    fn create_fixture(&self) -> Result<Self::Fixture<'_>, Self::Error>;
}

/// Generic test fixture with type parameter
///
/// This allows fixtures to wrap any type while maintaining type safety.
pub struct TestFixture<T: ?Sized = ()> {
    /// Inner fixture data
    inner: Box<T>,
    /// Unique test counter for isolation
    test_counter: u64,
    /// Test metadata
    metadata: HashMap<String, String>,
}

impl TestFixture<()> {
    /// Create a new test fixture with unique identifier
    pub fn new() -> FixtureResult<Self> {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);

        Ok(Self {
            inner: Box::new(()),
            test_counter: counter,
            metadata: HashMap::new(),
        })
    }
}

impl<T> TestFixture<T> {
    /// Create a new fixture with custom inner data
    pub fn with_data(data: T) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);

        Self {
            inner: Box::new(data),
            test_counter: counter,
            metadata: HashMap::new(),
        }
    }

    /// Get reference to inner data
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Get mutable reference to inner data
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Get test counter
    pub fn test_counter(&self) -> u64 {
        self.test_counter
    }

    /// Set metadata
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Get metadata
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// Cleanup fixture resources
    pub fn cleanup(&self) -> FixtureResult<()> {
        // Override in specific implementations
        Ok(())
    }
}

/// Default fixture provider implementation
impl FixtureProvider for () {
    type Fixture<'a> = TestFixture<()>;
    type Error = FixtureError;

    fn create_fixture(&self) -> Result<Self::Fixture<'_>, Self::Error> {
        TestFixture::new()
    }
}

impl Default for TestFixture<()> {
    fn default() -> Self {
        // Default implementation should not fail - use unwrap_or_else with panic
        Self::new().unwrap_or_else(|e| panic!("Failed to create default fixture: {}", e))
    }
}
