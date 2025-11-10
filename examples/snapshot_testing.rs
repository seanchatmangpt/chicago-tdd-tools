//! Snapshot Testing Example
//!
//! Demonstrates snapshot testing using insta for Chicago TDD.
//! Snapshot testing is ideal for testing complex data structures and ensuring output stability.

#[cfg(feature = "snapshot-testing")]
use chicago_tdd_tools::prelude::*;
#[cfg(feature = "snapshot-testing")]
use chicago_tdd_tools::snapshot::SnapshotAssert;

#[cfg(feature = "snapshot-testing")]
fn main() {
    println!("Snapshot Testing Example");
    println!("Run: cargo test --features snapshot-testing --example snapshot_testing");
}

#[cfg(not(feature = "snapshot-testing"))]
fn main() {
    println!("Snapshot testing feature not enabled. Enable with: --features snapshot-testing");
}

#[cfg(feature = "snapshot-testing")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_string() {
        // Arrange: Create test data
        let data = "Hello, Chicago TDD!";

        // Act: Assert snapshot matches
        // Assert: Verify snapshot (creates snapshot on first run, compares on subsequent runs)
        SnapshotAssert::assert_matches(&data, "test_snapshot_string");
    }

    #[test]
    fn test_snapshot_json() {
        // Arrange: Create JSON test data
        let data = serde_json::json!({
            "name": "Chicago TDD Tools",
            "version": "1.0.0",
            "features": ["snapshot-testing", "property-testing"]
        });

        // Act: Assert JSON snapshot matches
        // Assert: Verify JSON snapshot
        SnapshotAssert::assert_json_matches(&data, "test_snapshot_json");
    }

    #[test]
    fn test_snapshot_debug() {
        // Arrange: Create complex data structure
        let data = vec![("key1", "value1"), ("key2", "value2"), ("key3", "value3")];

        // Act: Assert debug snapshot matches
        // Assert: Verify debug representation snapshot
        SnapshotAssert::assert_debug_matches(&data, "test_snapshot_debug");
    }

    #[test]
    fn test_snapshot_with_settings() {
        // Arrange: Create test data
        let data = "test_with_custom_settings";

        // Act: Assert snapshot with custom settings
        // Assert: Verify snapshot with custom path
        SnapshotAssert::with_settings(
            |settings| {
                settings.set_snapshot_path("custom_snapshots");
            },
            || {
                SnapshotAssert::assert_matches(&data, "test_custom_path");
            },
        );
    }
}
