# Pattern 4: Resource Cleanup

## Context

Your tests allocate resources – files, network ports, containers, telemetry backends – that must be released even when assertions fail.

## Problem

Forgetting to release resources causes nondeterministic failures, leaking containers, or state pollution between tests. Manual cleanup logic scatters across tests and is easy to miss.

## Solution

Use Chicago TDD Tools fixtures and the RAII guarantees they provide. Wrap resource management inside `fixture_test!` or `fixture_test_with_timeout!`, storing handles in the fixture. Allow Drop implementations and the framework cleanup to run automatically so every test returns to a known good state.

## Forces

- Determinism vs. speed: automated teardown must be reliable without slowing the hot path
- Simplicity vs. observability: cleanup should be invisible unless a failure occurs
- Isolation vs. reuse: fixtures create fresh state while sharing setup logic

## Examples

```rust
use chicago_tdd_tools::prelude::*;
use testcontainers::clients::Cli as DockerCli;

struct DockerFixture {
    docker: DockerCli,
}

fixture_test!(test_exec_container_command, fixture, {
    // Arrange
    let docker = DockerCli::default();
    let container = docker.run("alpine:3.19");

    // Act
    let result = container.exec("echo", &["ok"])?;

    // Assert
    assert_eq!(result.stdout, "ok\n");

    Ok::<(), testcontainers::Error>(())
});
```

The fixture ensures containers stop even if the assertion fails.

## Related Patterns

- Pattern 5: Real Collaborators
- Pattern 16: Fixture Lifecycle Management
- Pattern 18: Timeout Defense in Depth
