//! Macros Examples
//!
//! Demonstrates test macros with AAA pattern enforcement, including procedural macros.

use chicago_tdd_tools::prelude::*;
use chicago_tdd_tools::{tdd_test, fixture};

// Note: Macros expand to test functions, so examples are shown in test module

#[cfg(test)]
mod tests {
    use super::*;

    /// Example: Synchronous test macro
    test!(test_sync_macro, {
        // Arrange: Set up test data
        let input = 5;
        let expected = 10;

        // Act: Execute feature
        let result = input * 2;

        // Assert: Verify behavior
        assert_eq!(result, expected);
    });

    /// Example: Async test macro
    async_test!(test_async_macro, {
        // Arrange: Set up test data
        let expected = 10;

        // Act: Execute async operation
        let result = async {
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            5 * 2
        }.await;

        // Assert: Verify behavior
        assert_eq!(result, expected);
    });

    /// Example: Fixture test macro
    fixture_test!(test_fixture_macro, fixture, {
        // Arrange: Use provided fixture
        let counter = fixture.test_counter();
        fixture.set_metadata("test_key".to_string(), "test_value".to_string());

        // Act: Execute test operation
        let metadata = fixture.get_metadata("test_key");

        // Assert: Verify behavior
        assert_eq!(metadata, Some(&"test_value".to_string()));
        assert!(counter >= 0);
    });

    /// Example: Performance test macro
    performance_test!(test_performance_macro, {
        // Arrange: Set up test data
        let input = vec![1, 2, 3];

        // Act: Execute hot path operation and measure ticks
        let (result, ticks) = measure_ticks(|| {
            input.iter().sum::<i32>()
        });

        // Assert: Verify performance constraint
        assert_within_tick_budget!(ticks, "Hot path operation");
        assert_eq!(result, 6);
    });

    /// Example: Async test with timeout
    async_test_with_timeout!(test_async_timeout, 5, {
        // Arrange: Set up test data
        let expected = 10;

        // Act: Execute async operation with timeout
        let result = async {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            5 * 2
        }.await;

        // Assert: Verify behavior
        assert_eq!(result, expected);
    });

    /// Example: Fixture test with timeout
    fixture_test_with_timeout!(test_fixture_timeout, fixture, 5, {
        // Arrange: Use provided fixture
        let counter = fixture.test_counter();

        // Act: Execute async operation
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Assert: Verify behavior
        assert!(counter >= 0);
    });

    // ========================================================================
    // Procedural Macros Examples
    // ========================================================================

    /// Example: #[tdd_test] procedural macro for synchronous tests
    #[tdd_test]
    fn test_tdd_test_procedural_macro() {
        // Arrange: Set up test data
        let input = 42;
        let expected = 84;

        // Act: Execute feature
        let result = input * 2;

        // Assert: Verify behavior
        assert_eq!(result, expected);
    }

    /// Example: #[tdd_test] procedural macro for async tests
    #[tdd_test]
    async fn test_tdd_test_async_procedural_macro() {
        // Arrange: Set up test data
        let expected = 100;

        // Act: Execute async operation
        let result = async {
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            50 * 2
        }.await;

        // Assert: Verify behavior
        assert_eq!(result, expected);
    }

    // TODO: #[fixture] procedural macro examples disabled due to compilation issues
    // The #[fixture] macro doesn't currently support standalone fn definitions
    // See: https://github.com/seanchatmangpt/chicago-tdd-tools/issues/TBD

    /* DISABLED - Compilation error with #[fixture] macro
    /// Example: #[fixture] procedural macro for automatic fixture setup
    #[fixture]
    fn test_fixture_procedural_macro() {
        // Arrange: Fixture automatically created by procedural macro
        let mut fixture = chicago_tdd_tools::fixture::TestFixture::new()
            .unwrap_or_else(|e| panic!("Failed to create test fixture: {}", e));
        let counter = fixture.test_counter();
        fixture.set_metadata("procedural_key".to_string(), "procedural_value".to_string());

        // Act: Execute test operation
        let metadata = fixture.get_metadata("procedural_key");

        // Assert: Verify behavior
        assert_eq!(metadata, Some(&"procedural_value".to_string()));
        assert!(counter >= 0);
    }

    /// Example: #[fixture] procedural macro for async tests
    #[fixture]
    async fn test_fixture_async_procedural_macro() {
        // Arrange: Fixture automatically created by procedural macro
        let mut fixture = chicago_tdd_tools::fixture::TestFixture::new()
            .unwrap_or_else(|e| panic!("Failed to create test fixture: {}", e));
        let counter = fixture.test_counter();

        // Act: Execute async operation
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;

        // Assert: Verify behavior
        assert!(counter >= 0);
    }
    */
}

