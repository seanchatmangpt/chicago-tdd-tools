# Snapshot Testing

Snapshot testing captures output on the first run and compares it on subsequent runs to detect unintended changes.

## Why Snapshot Testing?

Perfect for testing output that's complex but stable:

```rust
// ❌ Hard to maintain
let report = generate_report();
assert_eq!(report, "Employee Report\nAlice: ...\nBob: ...\n");  // 100 lines!

// ✅ Easy to maintain with snapshots
assert_matches!(report, "employee_report");  // Stored in file
```

## How It Works

### First Run: Create Snapshot

```rust
test!(test_report, {
    let report = generate_report();
    assert_matches!(report, "report");  // Creates report.snap
    // report.snap contains the generated report
});
```

### Subsequent Runs: Compare

If output changes:
- Old: `Employee Report\nAlice: 50000\n`
- New: `Employee Report\nAlice: 60000\n`  (salary changed)

Test shows a **diff**:

```
- Alice: 50000
+ Alice: 60000
```

You review and decide:
- ✅ Accept change (intentional update)
- ❌ Reject change (bug introduced)

## Basic Usage

### String Snapshots

```rust
test!(test_string_snapshot, {
    let output = "Hello, World!";
    assert_matches!(output, "greeting");
});
```

File `greeting.snap` contains: `Hello, World!`

### JSON Snapshots

```rust
test!(test_json_snapshot, {
    let data = TestDataBuilder::new()
        .with_var("name", "Alice")
        .with_var("age", "30")
        .build_json()?;

    assert_json_matches!(data, "user_data");
});
```

File `user_data.snap` contains JSON:

```json
{
  "name": "Alice",
  "age": "30"
}
```

### Debug Snapshots

```rust
test!(test_debug_snapshot, {
    let user = User { id: 123, name: "Alice".to_string() };
    assert_debug_matches!(user, "user_debug");
});
```

File `user_debug.snap` contains:

```
User {
    id: 123,
    name: "Alice",
}
```

## Real-World Example: API Response

```rust
test!(test_api_response_snapshot, {
    let client = ApiClient::new();
    let response = client.get_users()?;

    // Snapshot the API response
    assert_json_matches!(response, "api_users_response");

    // If API adds fields, you'll see a diff
    // Review and accept if intentional
});
```

## Real-World Example: Report Generation

```rust
test!(test_report_snapshot, {
    let data = vec![
        Employee { name: "Alice".to_string(), salary: 50000 },
        Employee { name: "Bob".to_string(), salary: 60000 },
    ];

    let report = generate_report(&data)?;

    // Snapshot the entire report
    assert_matches!(report, "employee_report");

    // Changes to formatting or content show up immediately
});
```

## Workflow: Accepting Changes

When you intentionally change output:

### Step 1: Run Tests

```bash
cargo test --features snapshot-testing
# Tests fail with diff if snapshot changed
```

### Step 2: Review Diff

```
- Alice: 50000
+ Alice: 60000
```

### Step 3: Accept or Reject

```bash
# Review snapshot changes
cargo insta review

# Or programmatically
insta::assert_snapshot!(output);  // Accepts in CI if --accept-all
```

### Step 4: Commit Changes

```bash
git add snapshot.snap
git commit -m "Update snapshot for salary changes"
```

## Configuration

### Snapshot Paths

Control where snapshots are stored:

```rust
#[test]
fn test_with_custom_path() {
    let settings = insta::Settings::clone_current();
    settings.set_snapshot_dir("tests/snapshots");
    settings.bind(|| {
        insta::assert_snapshot!("my_test", "output");
    });
}
```

### Snapshot Cleanup

Remove old snapshots:

```bash
# Remove unused snapshots
insta::cleanup_unused_snapshots!();
```

## Snapshot Comparisons

### Inline Snapshots

Store snapshot in test file (useful for small outputs):

```rust
test!(test_inline_snapshot, {
    let result = simple_function();
    insta::assert_snapshot!("simple_function", @"expected output");
});
```

### File Snapshots

Store snapshot in separate file (better for large outputs):

```rust
test!(test_file_snapshot, {
    let result = large_report();
    assert_matches!(result, "large_report");  // Stored in file
});
```

## Best Practices

✅ **Do:**
- Use for stable, complex output
- Review diffs carefully
- Commit snapshot changes
- Version control snapshots
- Update when intentional changes occur

❌ **Don't:**
- Use for simple outputs (too much overhead)
- Use for non-deterministic output (timestamps, random data)
- Blindly accept all changes
- Skip reviewing diffs
- Use for performance data (it changes)

## When to Use Snapshots

✅ **Use for:**
- API responses
- Generated reports
- Formatted output
- Complex data structures
- UI/HTML output

❌ **Don't use for:**
- Simple assertions (`assert_eq!`)
- Non-deterministic output
- Performance metrics
- Timestamps

## Performance

Snapshots are fast:
- First run: Create snapshot (~1ms)
- Subsequent runs: Compare (~1ms)

No performance overhead.

## Troubleshooting

### Snapshot Not Updating

Check file permissions:

```bash
ls -la tests/snapshots/
# Should be readable/writable
```

### Snapshot Too Long

Break into multiple snapshots:

```rust
// ❌ One large snapshot
assert_matches!(entire_report, "report");

// ✅ Multiple focused snapshots
assert_matches!(report.header, "report_header");
assert_matches!(report.body, "report_body");
assert_matches!(report.footer, "report_footer");
```

### Non-Deterministic Output

Normalize data before snapshotting:

```rust
// ❌ Timestamps change every run
let output = format!("Time: {}", now());
assert_matches!(output, "output");

// ✅ Normalize timestamps
let output = "Time: [TIMESTAMP]";
assert_matches!(output, "output");
```

## Combining with Other Techniques

### Snapshots + Property-Based Testing

```rust
test!(test_snapshot_property, {
    let strategy = ProptestStrategy::new().with_cases(10);

    strategy.test(any::<u32>(), |num| {
        let formatted = format!("{}", num);
        let parsed: u32 = formatted.parse().unwrap();

        // Snapshot the first case
        if num == 1 {
            assert_matches!(formatted, "formatted_number");
        }

        num == parsed
    });
});
```

### Snapshots + Fixtures

```rust
test!(test_snapshot_fixture, {
    let fixture = TestFixture::new()?;
    let report = generate_report(&fixture)?;
    assert_matches!(report, "fixture_report");
});
```

## Next Steps

Learn CLI testing: [CLI Testing](cli-testing.md)

---

## Summary

Snapshot testing:
- ✅ Captures complex output
- ✅ Detects unintended changes
- ✅ Easy to review diffs
- ✅ Great for regression detection

Perfect for generated output and API responses.

