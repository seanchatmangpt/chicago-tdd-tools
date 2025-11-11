# Pattern 18: Timeout Defense in Depth

## Context

Async tests interact with containers, networks, or external services. A hung future or stalled process could freeze the suite.

## Problem

Single-layer timeouts fail silently when they reside in the wrong place. For example, process-level timeouts kill the entire run without explaining which test stalled.

## Solution

Layer timeouts at three levels:

1. **Test-level** (`tokio::time::timeout` inside macros) – fails the specific test with a clear message.
2. **Runner-level** (`cargo-nextest` profiles) – applies SLA-based timeouts per profile.
3. **Process-level** (`timeout` wrapper in `Makefile.toml`) – stops catastrophic hangs.

Expose constants for standard timeouts (`DEFAULT_UNIT_TEST_TIMEOUT_SECONDS = 1`, `DEFAULT_INTEGRATION_TEST_TIMEOUT_SECONDS = 30`) and use `*_with_timeout!` macros for slow scenarios.

## Forces

- Resilience vs. noise: timeouts must be strict enough to catch hangs but lenient for expected latency
- Diagnostics vs. overhead: detailed error messages help triage without cluttering success paths
- Configurability vs. consistency: shared constants keep expectations aligned

## Examples

```toml
# .config/nextest.toml
[profile.default]
slow-timeout = { period = "1s", terminate-after = 1 }

[profile.integration]
slow-timeout = { period = "30s", terminate-after = 1 }
```

```rust
fixture_test_with_timeout!(test_container_warmup, fixture, DEFAULT_INTEGRATION_TEST_TIMEOUT_SECONDS, {
    // slow operation
    Ok(())
});
```

## Related Patterns

- Pattern 4: Resource Cleanup
- Pattern 9: Single Source of Truth
- Pattern 20: Macro Pattern Enforcement
