//! Snapshot Testing Examples
//!
//! Demonstrates snapshot testing for output comparison and regression testing.

#[cfg(feature = "snapshot-testing")]
use chicago_tdd_tools::testing::snapshot::SnapshotAssert;
use chicago_tdd_tools::prelude::*;

#[cfg(feature = "snapshot-testing")]
/// Example: String snapshot
pub fn example_snapshot_string() {
    // Arrange: Create test data
    let data = "Hello, Chicago TDD!";

    // Act-Assert: Assert snapshot matches
    SnapshotAssert::assert_matches(&data, "test_snapshot_string");
}

#[cfg(feature = "snapshot-testing")]
/// Example: JSON snapshot
pub fn example_snapshot_json() {
    // Arrange: Create JSON test data
    let data = serde_json::json!({
        "name": "Chicago TDD Tools",
        "version": "1.0.0",
        "features": ["snapshot-testing", "property-testing"]
    });

    // Act-Assert: Assert JSON snapshot matches
    SnapshotAssert::assert_json_matches(&data, "test_snapshot_json");
}

#[cfg(feature = "snapshot-testing")]
/// Example: Debug snapshot
pub fn example_snapshot_debug() {
    // Arrange: Create complex data structure
    let data = vec![("key1", "value1"), ("key2", "value2")];

    // Act-Assert: Assert debug snapshot matches
    SnapshotAssert::assert_debug_matches(&data, "test_snapshot_debug");
}

#[cfg(feature = "snapshot-testing")]
/// Example: Snapshot with custom settings
pub fn example_snapshot_custom_settings() {
    // Arrange: Create test data
    let data = "test_with_custom_settings";

    // Act-Assert: Assert snapshot with custom settings
    SnapshotAssert::with_settings(
        |settings| {
            settings.set_snapshot_path("custom_snapshots");
        },
        || {
            SnapshotAssert::assert_matches(&data, "test_custom_path");
        },
    );
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "snapshot-testing")]
    use super::*;

    #[cfg(feature = "snapshot-testing")]
    test!(test_snapshot_string, {
        // Arrange-Act-Assert: Run example
        example_snapshot_string();
    });

    #[cfg(feature = "snapshot-testing")]
    test!(test_snapshot_json, {
        // Arrange-Act-Assert: Run example
        example_snapshot_json();
    });

    #[cfg(feature = "snapshot-testing")]
    test!(test_snapshot_debug, {
        // Arrange-Act-Assert: Run example
        example_snapshot_debug();
    });

    #[cfg(feature = "snapshot-testing")]
    test!(test_snapshot_custom_settings, {
        // Arrange-Act-Assert: Run example
        example_snapshot_custom_settings();
    });
}

