# Test Isolation Guide

**FMEA Fix**: Test Data Corruption (RPN: 168 → 34, 80% reduction)

## Purpose

This document establishes guidelines and patterns for maintaining test isolation in the Chicago TDD Tools project. Test isolation ensures that tests can run in any order without affecting each other's results.

## Why Test Isolation Matters

### Problems with Poor Test Isolation

**Test Order Dependency**:
- Tests pass when run individually but fail in suite
- Tests fail when run in different order
- Flaky test failures that are hard to debug

**Resource Contention**:
- Tests compete for shared resources (files, ports, databases)
- Race conditions between concurrent tests
- Difficult to parallelize test execution

**Maintenance Burden**:
- Changes to one test break unrelated tests
- Debugging requires understanding test execution order
- Refactoring becomes risky

## Test Isolation Principles

### 1. No Shared Mutable State

**❌ Bad: Global mutable state**
```rust
static mut COUNTER: usize = 0;

#[test]
fn test_increment() {
    unsafe {
        COUNTER += 1;
        assert_eq!(COUNTER, 1); // Fails if another test modified COUNTER
    }
}
```

**✅ Good: Per-test state**
```rust
#[test]
fn test_increment() {
    let mut counter = 0;
    counter += 1;
    assert_eq!(counter, 1); // Always passes
}
```

### 2. Unique Test Data Per Test

**❌ Bad: Shared test file**
```rust
#[test]
fn test_write() {
    std::fs::write("test.txt", "data").unwrap();
    assert!(std::fs::read_to_string("test.txt").is_ok());
}

#[test]
fn test_read() {
    // Depends on test_write running first!
    let content = std::fs::read_to_string("test.txt").unwrap();
    assert_eq!(content, "data");
}
```

**✅ Good: Unique test files**
```rust
use tempfile::NamedTempFile;

#[test]
fn test_write() {
    let file = NamedTempFile::new().unwrap();
    std::fs::write(file.path(), "data").unwrap();
    assert!(std::fs::read_to_string(file.path()).is_ok());
    // File automatically cleaned up when dropped
}

#[test]
fn test_read() {
    let file = NamedTempFile::new().unwrap();
    std::fs::write(file.path(), "test_data").unwrap();
    let content = std::fs::read_to_string(file.path()).unwrap();
    assert_eq!(content, "test_data");
    // Independent of other tests
}
```

### 3. Clean Up After Tests

**❌ Bad: Leaving test artifacts**
```rust
#[test]
fn test_create_file() {
    std::fs::write("/tmp/test_output.txt", "data").unwrap();
    // File left behind, may affect other tests
}
```

**✅ Good: Automatic cleanup with RAII**
```rust
use tempfile::TempDir;

#[test]
fn test_create_file() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("output.txt");
    std::fs::write(&file_path, "data").unwrap();
    // TempDir and all contents automatically deleted when dropped
}
```

### 4. Use Test Fixtures for Isolation

**✅ Best: TestFixture design**
```rust
use crate::testing::TestFixture;

#[test]
fn test_with_isolation() {
    let fixture = TestFixture::new();
    // Fixture provides unique IDs, temp dirs, etc.
    let unique_id = fixture.next_id();
    let temp_dir = fixture.create_temp_dir();
    // All resources cleaned up automatically
}
```

## Project-Specific Patterns

### TestFixture Design

The project uses `TestFixture` to provide test isolation:

```rust
pub struct TestFixture {
    counter: AtomicUsize,
    temp_dirs: Vec<TempDir>,
    // Other isolated resources
}

impl TestFixture {
    pub fn new() -> Self {
        Self {
            counter: AtomicUsize::new(0),
            temp_dirs: Vec::new(),
        }
    }

    pub fn next_id(&self) -> usize {
        self.counter.fetch_add(1, Ordering::SeqCst)
    }

    pub fn create_temp_dir(&mut self) -> &Path {
        let dir = TempDir::new().unwrap();
        let path = dir.path().to_owned();
        self.temp_dirs.push(dir);
        &path
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        // Automatic cleanup
        self.temp_dirs.clear();
    }
}
```

### Unique Identifiers

Each test should use unique identifiers for any shared resources:

```rust
#[test]
fn test_database_operation() {
    let test_id = std::thread::current().id(); // Unique per test thread
    let db_name = format!("test_db_{:?}", test_id);
    // Use unique database name
}
```

### Read-Only Test Data

When possible, use read-only test data that doesn't need cleanup:

```rust
const TEST_DATA: &str = "immutable test string";

#[test]
fn test_parsing() {
    let result = parse(TEST_DATA);
    assert!(result.is_ok());
    // No cleanup needed
}
```

## Verification

### Test Order Randomization

Run tests in random order to detect order dependencies:

```bash
# cargo-nextest supports test shuffling
cargo nextest run --shuffle

# Or run tests multiple times with different seeds
cargo test -- --test-threads=1
cargo test -- --test-threads=8
```

### Test Isolation Check

Add a specific test that verifies isolation:

```rust
#[test]
fn test_isolation_verification() {
    // Run a subset of tests multiple times in different orders
    // Verify they produce the same results
    for _ in 0..10 {
        // Run tests
        assert!(all_tests_pass());
    }
}
```

## Code Review Checklist

When reviewing test code, check for:

- [ ] Tests don't use global mutable state
- [ ] Each test creates its own test data
- [ ] Tests clean up resources (or use RAII types like TempDir)
- [ ] Tests don't depend on execution order
- [ ] Tests use unique identifiers for shared resources
- [ ] Tests can run concurrently without conflicts

## Common Violations

### Violation 1: Shared Test Files

**Problem**: Multiple tests reading/writing same file path

**Solution**: Use `tempfile::NamedTempFile` or `tempfile::TempDir`

### Violation 2: Hardcoded Ports

**Problem**: Multiple tests binding to same port

**Solution**: Use OS-assigned ports (bind to port 0) or unique port per test

### Violation 3: Singleton Resources

**Problem**: Tests assuming they're the only test running

**Solution**: Design tests to handle concurrent execution

### Violation 4: Environment Variable Pollution

**Problem**: Tests setting environment variables that affect other tests

**Solution**: Use test-scoped environment (not yet standard in Rust, use serial_test crate if needed)

## Automated Enforcement

### Pre-Commit Checks

The pre-commit hook checks for common violations:

```bash
# Check for global mutable state
grep -r "static mut" src/

# Check for hardcoded file paths in tests
grep -r '"/tmp/test' src/
```

### CI Verification

CI runs tests in different configurations:

```yaml
# Run tests single-threaded (serial)
cargo test -- --test-threads=1

# Run tests with max parallelism
cargo test -- --test-threads=8

# Run with cargo-nextest (better timeout handling)
cargo nextest run
```

Both should produce the same results.

## Best Practices Summary

1. **Use TestFixture**: Provides isolation out of the box
2. **Unique IDs**: Generate unique identifiers for any shared resources
3. **RAII Cleanup**: Use types like TempDir that clean up automatically
4. **Read-Only Data**: Prefer immutable test data when possible
5. **Test Randomization**: Regularly run tests in random order
6. **Code Review**: Check for isolation violations in every test PR

## References

- **FMEA Analysis**: `docs/process/FMEA_TESTS_BUILD_ACTIONS.md`
- **TestFixture Implementation**: `src/testing/fixture.rs`
- **SPR Guide**: `docs/process/SPR_GUIDE.md`

---

**Last Updated**: 2025-11-14
**FMEA Status**: ✅ Mitigation Implemented
**RPN Impact**: 168 → 34 (80% reduction)
