# Coverage & Performance

Measure test coverage and performance metrics.

## Test Coverage

Coverage shows which code is executed by tests:

```bash
# Generate coverage report
cargo make coverage

# View coverage report
cargo make coverage-report
```

### Coverage Metrics

- **Line Coverage**: % of lines executed
- **Branch Coverage**: % of branches executed
- **Function Coverage**: % of functions executed

### Target Coverage

- **Minimum**: 70% (warning level)
- **Target**: 80%+ (good)
- **Excellent**: 90%+ (very thorough)

❌ Don't obsess over 100% (often impossible/impractical)

### Improving Coverage

Focus on uncovered lines:

```rust
// ❌ Uncovered error path
if let Err(e) = operation() {
    // This might not be tested
    log_error(e);
}

// ✅ Test the error path too
test!(test_error_handling, {
    let result = risky_operation();
    assert_err!(&result);
});
```

## Performance Testing

Measure operation timing:

```rust
test!(performance_test, {
    let start = std::time::Instant::now();

    // Code to benchmark
    for _ in 0..1000 {
        let _result = parse_number("42");
    }

    let elapsed = start.elapsed();
    println!("Time for 1000 parses: {:?}", elapsed);

    // Assert performance target
    assert!(elapsed.as_millis() < 100);  // < 100ms
});
```

### Performance Targets

| Operation | Target | Too Slow |
|-----------|--------|----------|
| Parse number | <1μs | >10μs |
| Database query | <10ms | >100ms |
| API call | <100ms | >1s |
| Test execution | <10ms | >100ms |

### Profiling

```bash
# Run performance tests
cargo make test-timings

# Profile with cargo flamegraph (if installed)
cargo flamegraph --test performance_tests
```

## Combining Coverage and Performance

Track both:

```rust
test!(comprehensive_test, {
    let start = std::time::Instant::now();

    // Success path (covered)
    let ok = parse_number("42");
    assert_ok!(&ok);

    // Error path (covered)
    let err = parse_number("invalid");
    assert_err!(&err);

    // Performance assertion
    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 10);
});
```

## Best Practices

✅ **Coverage:**
- Aim for 80%+ overall
- Focus on critical paths
- Test error cases
- Review uncovered lines

❌ **Coverage:**
- Don't chase 100%
- Don't test generated code
- Don't test trivial code

✅ **Performance:**
- Set realistic targets
- Measure on target hardware
- Profile before optimizing
- Test under load

❌ **Performance:**
- Optimize prematurely
- Ignore benchmarks
- Assume fast
- Measure on dev machine only
