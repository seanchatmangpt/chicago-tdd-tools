//! Fixtures Examples
//!
//! Demonstrates test fixtures with state management and test isolation.

use chicago_tdd_tools::prelude::*;

/// Example: Basic fixture creation
pub fn example_basic_fixture() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange: Create fixture
    let fixture = TestFixture::new()?;

    // Act: Use fixture
    let counter = fixture.test_counter();

    // Assert: Verify fixture created
    assert!(counter >= 0);
    Ok(())
}

/// Example: Fixture with custom data
pub fn example_fixture_with_data() {
    // Arrange: Create fixture with custom data
    let data = vec![1, 2, 3];
    let fixture = TestFixture::with_data(data);

    // Act: Access inner data
    let inner = fixture.inner();

    // Assert: Verify data accessible
    assert_eq!(inner.len(), 3);
    assert_eq!(inner[0], 1);
}

/// Example: Fixture metadata
pub fn example_fixture_metadata() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange: Create fixture
    let mut fixture = TestFixture::new()?;

    // Act: Set and get metadata
    fixture.set_metadata("test_key".to_string(), "test_value".to_string());
    let metadata = fixture.get_metadata("test_key");

    // Assert: Verify metadata stored
    assert_eq!(metadata, Some(&"test_value".to_string()));
    Ok(())
}

/// Example: Fixture isolation (unique counters)
pub fn example_fixture_isolation() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange: Create multiple fixtures
    let fixture1 = TestFixture::new()?;
    let fixture2 = TestFixture::new()?;

    // Act: Get counters
    let counter1 = fixture1.test_counter();
    let counter2 = fixture2.test_counter();

    // Assert: Verify isolation (counters are different)
    assert_ne!(counter1, counter2);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    test!(test_basic_fixture, {
        // Arrange-Act-Assert: Run example
        assert_ok!(example_basic_fixture());
    });

    test!(test_fixture_with_data, {
        // Arrange-Act-Assert: Run example
        example_fixture_with_data();
    });

    test!(test_fixture_metadata, {
        // Arrange-Act-Assert: Run example
        assert_ok!(example_fixture_metadata());
    });

    test!(test_fixture_isolation, {
        // Arrange-Act-Assert: Run example
        assert_ok!(example_fixture_isolation());
    });
}

