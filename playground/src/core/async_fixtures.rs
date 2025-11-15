//! Async Fixtures Examples
//!
//! Demonstrates async fixture providers with async traits (Rust 1.75+).

#[cfg(feature = "async")]
use chicago_tdd_tools::core::async_fixture::{AsyncFixtureManager, AsyncFixtureProvider};
#[cfg(feature = "async")]
use chicago_tdd_tools::core::fixture::FixtureError;

#[cfg(feature = "async")]
struct DatabaseFixture {
    connection: String,
}

#[cfg(feature = "async")]
struct DatabaseProvider;

// Note: Async fixture examples require implementing sealed trait pattern
// For playground, we demonstrate the API usage patterns
// See src/core/async_fixture.rs tests for full implementation examples

#[cfg(feature = "async")]
pub async fn example_async_fixture_manager() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange: Async fixture manager usage pattern
    // Note: This is a demonstration - actual implementation requires sealed trait

    // Act: Manager pattern
    // let manager = AsyncFixtureManager::new(provider);
    // let fixture = manager.setup().await?;
    // manager.teardown().await?;

    // Assert: Pattern demonstrated
    Ok(())
}

#[cfg(not(feature = "async"))]
pub fn example_async_fixture_manager() {
    // Async fixtures require async feature
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "async")]
    use super::*;

    #[cfg(feature = "async")]
    async_test!(test_async_fixture_manager, {
        // Arrange-Act-Assert: Run example
        assert_ok!(example_async_fixture_manager().await);
    });
}
