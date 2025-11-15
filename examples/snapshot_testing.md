# Snapshot Testing Example

**Category:** How-To Guide
**Level:** Intermediate
**Prerequisites:** Understanding of testing concepts
**Features Required:** `snapshot-testing`

---

## Overview

This example demonstrates snapshot testing using `insta` for Chicago TDD. Snapshot testing is ideal for testing complex data structures and ensuring output stability.

**What you'll learn:**
- String snapshot testing
- JSON snapshot testing
- Debug representation snapshots
- Snapshot review workflow

---

## Quick Start

```bash
cargo test --features snapshot-testing --example snapshot_testing
```

---

## Prerequisites

- Rust 1.70+ (Edition 2021)
- Chicago TDD Tools with `snapshot-testing` feature
- `cargo-insta` CLI tool (recommended)

**Add to Cargo.toml:**
```toml
[dev-dependencies]
chicago-tdd-tools = { path = "../chicago-tdd-tools", features = ["snapshot-testing"] }
```

**Install cargo-insta:**
```bash
cargo install cargo-insta
```

---

## Key Concepts

### Snapshot Testing

Captures output on first run and compares it on subsequent runs. Ideal for:
- Complex data structures
- API responses
- Generated code
- Configuration files

### Snapshot Lifecycle

1. **First Run:** Creates `.snap` file
2. **Subsequent Runs:** Compares output to snapshot
3. **On Change:** Shows diff and creates `.snap.new` file
4. **Review:** Accept or reject changes with `cargo insta review`

---

## Code Examples

### Example 1: String Snapshots

```rust
use chicago_tdd_tools::snapshot::SnapshotAssert;

#[test]
fn test_snapshot_string() {
    // Arrange: Create test data
    let data = "Hello, Chicago TDD!";

    // Act & Assert: Create/verify snapshot
    SnapshotAssert::assert_matches(&data, "test_snapshot_string");
}
```

**Creates:** `snapshots/test_snapshot_string.snap`

### Example 2: JSON Snapshots

```rust
use chicago_tdd_tools::snapshot::SnapshotAssert;
use serde_json::json;

#[test]
fn test_snapshot_json() {
    // Arrange: Create JSON test data
    let data = json!({
        "name": "Chicago TDD Tools",
        "version": "1.0.0",
        "features": ["snapshot-testing", "property-testing"]
    });

    // Act & Assert: Create/verify JSON snapshot
    SnapshotAssert::assert_json_matches(&data, "test_snapshot_json");
}
```

**Creates:** `snapshots/test_snapshot_json.snap` with formatted JSON

### Example 3: Debug Snapshots

```rust
use chicago_tdd_tools::snapshot::SnapshotAssert;

#[test]
fn test_snapshot_debug() {
    // Arrange: Create complex data structure
    let data = vec![("key1", "value1"), ("key2", "value2"), ("key3", "value3")];

    // Act & Assert: Create/verify debug snapshot
    SnapshotAssert::assert_debug_matches(&data, "test_snapshot_debug");
}
```

### Example 4: Custom Settings

```rust
use chicago_tdd_tools::snapshot::SnapshotAssert;

#[test]
fn test_snapshot_with_settings() {
    let data = "test_with_custom_settings";

    // Configure snapshot path
    SnapshotAssert::with_settings(
        |settings| {
            settings.set_snapshot_path("custom_snapshots");
        },
        || {
            SnapshotAssert::assert_matches(&data, "test_custom_path");
        },
    );
}
```

---

## Workflow

### 1. Write Test

```rust
#[test]
fn test_api_response() {
    let response = json!({
        "status": "success",
        "data": {"id": 123, "name": "Test"}
    });
    SnapshotAssert::assert_json_matches(&response, "api_response");
}
```

### 2. Run Test (First Time)

```bash
cargo test test_api_response
```

Creates `snapshots/api_response.snap`:
```
{
  "status": "success",
  "data": {
    "id": 123,
    "name": "Test"
  }
}
```

### 3. Code Changes

Change code that produces different output.

### 4. Run Test Again

```bash
cargo test test_api_response
```

Test fails, shows diff, creates `.snap.new` file.

### 5. Review Changes

```bash
cargo insta review
```

**Options:**
- **Accept** - Update snapshot to new output
- **Reject** - Keep existing snapshot
- **Skip** - Review later

### 6. Accept/Reject All

```bash
cargo insta accept  # Accept all changes
cargo insta reject  # Reject all changes
```

---

## Common Patterns

### Pattern 1: API Response Testing

```rust
#[test]
fn test_user_api() {
    let user = create_user_response();
    SnapshotAssert::assert_json_matches(&user, "user_response");
}
```

### Pattern 2: Complex Data Structures

```rust
#[test]
fn test_config() {
    let config = load_configuration();
    SnapshotAssert::assert_debug_matches(&config, "config_structure");
}
```

### Pattern 3: Generated Code

```rust
#[test]
fn test_code_generation() {
    let generated_code = generate_rust_code();
    SnapshotAssert::assert_matches(&generated_code, "generated_code");
}
```

---

## Advanced Usage

### Multiple Snapshots in One Test

```rust
#[test]
fn test_multiple_stages() {
    let stage1 = process_stage_1();
    SnapshotAssert::assert_matches(&stage1, "stage1");

    let stage2 = process_stage_2(stage1);
    SnapshotAssert::assert_matches(&stage2, "stage2");
}
```

### Inline Snapshots

```rust
use insta::assert_snapshot;

#[test]
fn test_inline() {
    let output = "Hello, World!";
    assert_snapshot!(output, @"Hello, World!");
}
```

### Redactions

```rust
SnapshotAssert::with_settings(
    |settings| {
        settings.add_redaction(".timestamp", "[TIMESTAMP]");
    },
    || {
        // Snapshot test with redacted timestamp
    },
);
```

---

## Best Practices

### 1. Commit Snapshots

**Always commit `.snap` files to version control:**
```bash
git add snapshots/*.snap
git commit -m "Add snapshots for API tests"
```

### 2. Review Carefully

Review snapshot changes carefully - they represent your expected output.

### 3. Meaningful Names

Use descriptive snapshot names:
```rust
// ✓ Good
SnapshotAssert::assert_matches(&data, "user_profile_json");

// ✗ Bad
SnapshotAssert::assert_matches(&data, "test1");
```

### 4. Keep Snapshots Small

Large snapshots are hard to review. Consider testing subsets:
```rust
// Instead of entire response:
SnapshotAssert::assert_json_matches(&response.data.user, "user_data");
```

---

## Troubleshooting

### Error: "snapshot-testing feature required"

**Cause:** Feature not enabled

**Fix:**
```toml
[dev-dependencies]
chicago-tdd-tools = { path = "../chicago-tdd-tools", features = ["snapshot-testing"] }
```

### Error: "snapshot assertion failed"

**Cause:** Output changed from snapshot

**Fix:**
1. Review diff: `cargo insta review`
2. Accept if correct: Press 'a'
3. Reject if incorrect: Press 'r'

### Snapshots Not Created

**Cause:** Test might be failing before snapshot assertion

**Fix:**
- Check test output for errors
- Ensure code runs to snapshot assertion
- Verify snapshot directory exists

---

## Next Steps

After mastering snapshot testing, explore:

1. **[Property Testing](property_testing.md)** - Random test data generation
2. **[Mutation Testing](mutation_testing.md)** - Test quality validation
3. **[CLI Testing](cli_testing.md)** - Golden file testing

---

## Related Documentation

- [Examples README](README.md) - All examples overview
- [insta documentation](https://docs.rs/insta/) - Complete insta guide
- [API Reference](../docs/reference/API_REFERENCE.md) - Complete API documentation

---

## Reference

### Key Functions

- `SnapshotAssert::assert_matches(data, name)` - String snapshots
- `SnapshotAssert::assert_json_matches(data, name)` - JSON snapshots
- `SnapshotAssert::assert_debug_matches(data, name)` - Debug snapshots
- `SnapshotAssert::with_settings(config, test)` - Custom settings

### Cargo Insta Commands

- `cargo insta review` - Review pending snapshots
- `cargo insta accept` - Accept all changes
- `cargo insta reject` - Reject all changes
- `cargo insta test` - Run tests with insta

### File Structure

```
project/
├── tests/
│   └── snapshots/
│       ├── my_test.snap       # Committed snapshot
│       └── my_test.snap.new   # New pending snapshot
```

---

**Quality is the default. Prevention beats detection.**

*Example: snapshot_testing.rs | Version: 1.2.0 | Updated: 2025-11-15*
