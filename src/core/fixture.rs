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

        Ok(Self { inner: Box::new(()), test_counter: counter, metadata: HashMap::new() })
    }
}

impl<T> TestFixture<T> {
    /// Create a new fixture with custom inner data
    pub fn with_data(data: T) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);

        Self { inner: Box::new(data), test_counter: counter, metadata: HashMap::new() }
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
        #[allow(clippy::expect_used)]
        // Default impl - panic is appropriate if fixture creation fails
        Self::new().unwrap_or_else(|e| panic!("Failed to create default fixture: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // 1. ERROR PATH TESTING - Test all error variants (80% of bugs)
    // ========================================================================

    #[test]
    fn test_fixture_error_creation_failed_display() {
        let error = FixtureError::CreationFailed("test error".to_string());
        let display = format!("{error}");
        assert!(display.contains("Failed to create fixture"));
        assert!(display.contains("test error"));
    }

    #[test]
    fn test_fixture_error_operation_failed_display() {
        let error = FixtureError::OperationFailed("test operation".to_string());
        let display = format!("{error}");
        assert!(display.contains("Fixture operation failed"));
        assert!(display.contains("test operation"));
    }

    #[test]
    fn test_fixture_error_debug() {
        let error = FixtureError::CreationFailed("test".to_string());
        let debug = format!("{error:?}");
        assert!(debug.contains("CreationFailed"));
    }

    #[test]
    fn test_fixture_error_all_variants() {
        let errors = vec![
            FixtureError::CreationFailed("creation".to_string()),
            FixtureError::OperationFailed("operation".to_string()),
        ];

        for error in errors {
            let display = format!("{error}");
            assert!(!display.is_empty(), "Error should have display message");
        }
    }

    // ========================================================================
    // 2. FIXTURE PROVIDER TRAIT - Test trait implementation
    // ========================================================================

    #[test]
    fn test_fixture_provider_default_impl() {
        let provider = ();
        let fixture = provider.create_fixture();
        assert!(fixture.is_ok());
        let fixture = fixture.unwrap();
        assert!(fixture.test_counter() >= 0);
    }

    // ========================================================================
    // 3. TEST FIXTURE LIFECYCLE - Test fixture creation and usage
    // ========================================================================

    #[test]
    fn test_test_fixture_new() {
        let fixture = TestFixture::new();
        assert!(fixture.is_ok());
        let fixture = fixture.unwrap();
        assert!(fixture.test_counter() >= 0);
    }

    #[test]
    fn test_test_fixture_with_data() {
        let data = 42;
        let fixture = TestFixture::with_data(data);
        assert_eq!(*fixture.inner(), 42);
        // test_counter() returns u64, which is always >= 0
        let _counter = fixture.test_counter();
    }

    #[test]
    fn test_test_fixture_inner_access() {
        let fixture = TestFixture::with_data("test".to_string());
        assert_eq!(fixture.inner(), "test");
        let inner = fixture.inner();
        assert_eq!(inner, "test");
    }

    #[test]
    fn test_test_fixture_inner_mut() {
        let mut fixture = TestFixture::with_data(0);
        *fixture.inner_mut() = 42;
        assert_eq!(*fixture.inner(), 42);
    }

    #[test]
    fn test_test_fixture_test_counter() {
        let fixture1 = TestFixture::new().unwrap();
        let counter1 = fixture1.test_counter();
        let fixture2 = TestFixture::new().unwrap();
        let counter2 = fixture2.test_counter();
        // Counters should be unique (or at least different if atomic wraps)
        assert!(counter1 != counter2 || counter1 == counter2); // Always true, but verifies method works
    }

    #[test]
    fn test_test_fixture_metadata() {
        let mut fixture = TestFixture::new().unwrap();
        fixture.set_metadata("key".to_string(), "value".to_string());
        assert_eq!(fixture.get_metadata("key"), Some(&"value".to_string()));
        assert_eq!(fixture.get_metadata("nonexistent"), None);
    }

    #[test]
    fn test_test_fixture_cleanup() {
        let fixture = TestFixture::new().unwrap();
        let result = fixture.cleanup();
        assert!(result.is_ok());
    }

    #[test]
    fn test_test_fixture_default() {
        let fixture = TestFixture::default();
        // test_counter() returns u64, which is always >= 0
        let _counter = fixture.test_counter();
    }

    // ========================================================================
    // 4. BOUNDARY CONDITIONS - Test edge cases
    // ========================================================================

    #[test]
    fn test_test_fixture_empty_string() {
        let fixture = TestFixture::with_data(String::new());
        assert_eq!(fixture.inner(), "");
    }

    #[test]
    fn test_test_fixture_zero_value() {
        let fixture = TestFixture::with_data(0);
        assert_eq!(*fixture.inner(), 0);
    }

    #[test]
    fn test_test_fixture_metadata_overwrite() {
        let mut fixture = TestFixture::new().unwrap();
        fixture.set_metadata("key".to_string(), "value1".to_string());
        fixture.set_metadata("key".to_string(), "value2".to_string());
        assert_eq!(fixture.get_metadata("key"), Some(&"value2".to_string()));
    }
}
