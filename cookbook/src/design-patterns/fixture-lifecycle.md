# Pattern 16: Fixture Lifecycle Management

## Context

Complex tests require deterministic setup and teardown of shared state: databases, telemetry, temporary directories.

## Problem

Manual lifecycle logic is error-prone. Forgetting teardown causes cascading failures across tests. Async setup complicates matters further.

## Solution

Wrap lifecycle responsibilities in `TestFixture` or `AsyncFixtureManager`. Use the fixture to hold handles and expose helper methods. Let Drop and the manager `.teardown()` guarantee cleanup. For async resources, implement `AsyncFixtureProvider` and return strongly typed handles.

## Forces

- Determinism vs. flexibility: fixtures must isolate state yet allow custom behavior per test
- Async vs. sync complexity: asynchronous resources require explicit lifecycle boundaries
- Performance vs. safety: reuse is tempting, but fresh fixtures avoid hidden coupling

## Examples

```rust
struct DbProvider;

impl chicago_tdd_tools::core::async_fixture::private::Sealed for DbProvider {}

impl AsyncFixtureProvider for DbProvider {
    type Fixture<'a> = DatabaseHandle;
    type Error = DbError;

    fn create_fixture<'a>(&'a self) -> DbFuture<'a, DatabaseHandle> {
        Box::pin(async move { DatabaseHandle::connect().await })
    }
}

async_test!(test_query_latency, {
    let manager = AsyncFixtureManager::new(DbProvider);
    let handle = manager.setup().await?;
    // ...
    manager.teardown().await?;
    Ok(())
});
```

## Related Patterns

- Pattern 4: Resource Cleanup
- Pattern 12: Type Safety with GATs
- Pattern 18: Timeout Defense in Depth
