# Perfect Weaver Live-Check Implementation Guide

## Overview

This document describes the perfect implementation of Weaver live-check testing, ensuring reliable, type-safe, and error-resistant telemetry validation. The implementation follows Chicago TDD principles, elite Rust developer standards, and poka-yoke (error prevention) design patterns.

## Architecture

![Architecture Diagram](weaver-perfect-architecture.puml)

The perfect Weaver live-check architecture consists of:

### Core Components

1. **`WeaverValidator`** - Main orchestrator managing Weaver process lifecycle
   - Uses poka-yoke types (`ValidRegistryPath`, `NonZeroPort`, `PositiveTimeout`)
   - Manages process state and readiness verification
   - Provides fail-fast error detection

2. **`WeaverLiveCheck`** - Low-level Weaver CLI integration
   - Spawns and manages Weaver process
   - Handles HTTP admin endpoint communication
   - Provides health check capabilities

3. **Poka-Yoke Types** - Type-level error prevention
   - `ValidRegistryPath` - Ensures registry path exists and is valid
   - `NonZeroPort` - Prevents invalid ports (port = 0)
   - `PositiveTimeout` - Prevents invalid timeouts (timeout = 0)
   - `RegistryVersion` - Validates registry version strings
   - `RegistryState` - Enforces validation state machine

### Key Design Principles

- **Type Safety**: Invalid states are unrepresentable at compile time
- **Fail-Fast**: Errors detected immediately with clear messages
- **Behavior Verification**: Actual process state checked, not just presence
- **Zero-Cost Abstractions**: Types compile away, no runtime overhead
- **Real Collaborators**: Uses actual Weaver process, not mocks

## Perfect Startup Flow

![Sequence Diagram](weaver-perfect-sequence.puml)

The perfect startup flow ensures Weaver is ready before proceeding:

### Step 1: Pre-Start Validation

**Registry Validation**:
- Validate registry before starting Weaver
- Catch validation errors early (fail-fast)
- Provide actionable error messages with fix instructions

**Binary Availability**:
- Check Weaver binary is available
- Attempt runtime download if missing
- Verify binary is executable

### Step 2: Process Spawn

**Spawn Process**:
- Spawn Weaver process with validated configuration
- Capture process handle immediately
- Handle spawn failures with clear errors

### Step 3: Immediate Exit Detection

**Check Process State**:
- Use `try_wait()` immediately after spawn
- Detect if process exited during startup
- Capture exit code and stderr for error messages

**Fail-Fast on Exit**:
- Return error immediately if process exited
- Include exit code, stderr, and fix instructions
- Prevent proceeding with dead process

### Step 4: Readiness Verification

**Wait for Ready**:
- Verify process is alive (`try_wait()` returns `None`)
- Verify admin port is listening (`TcpStream::connect()`)
- Use exponential backoff (100ms → 2000ms, total ~5s)

**Exponential Backoff**:
- Retry intervals: 100ms, 200ms, 400ms, 800ms, 1600ms, 2000ms
- Non-blocking checks (fast failure detection)
- Total timeout: 5 seconds (configurable via `WEAVER_STARTUP_TIMEOUT`)

**Success Criteria**:
- Process is running (`try_wait()` returns `None`)
- Admin port is listening (TCP connect succeeds)
- Both conditions met before timeout

## Process State Machine

![State Diagram](weaver-perfect-state.puml)

The perfect implementation manages Weaver process through clear states:

### States

1. **NotStarted** - Initial state, no process
2. **ValidatingRegistry** - Validating registry before start
3. **RegistryValid** - Registry validation passed
4. **Spawning** - Spawning Weaver process
5. **ProcessSpawned** - Process spawned, checking readiness
6. **CheckingReady** - Verifying process and port readiness
7. **Ready** - Process running and admin port listening
8. **Running** - Accepting and validating telemetry
9. **Stopping** - Shutting down process
10. **Stopped** - Process stopped successfully

### State Transitions

- **Success Path**: NotStarted → ValidatingRegistry → RegistryValid → Spawning → ProcessSpawned → CheckingReady → Ready → Running
- **Error Paths**: Any state can transition to error state with clear error message
- **Shutdown Path**: Running → Stopping → Stopped

## Readiness Check Flow

![Readiness Flow Diagram](weaver-perfect-readiness-flow.puml)

The perfect readiness check ensures Weaver is actually ready:

### Algorithm

1. **Initialize**: Set up exponential backoff intervals and timeout timer
2. **Loop**: For each backoff interval:
   - Check timeout exceeded → Return timeout error
   - Check process state (`try_wait()`) → Return error if exited
   - Check admin port (`TcpStream::connect_timeout()`) → Break if ready
   - Sleep for backoff interval
3. **Final Check**: Verify admin port is ready after all retries
4. **Success**: Return success if port is ready

### Key Features

- **Non-Blocking**: Uses `try_wait()` and `connect_timeout()` for fast checks
- **Exponential Backoff**: Reduces CPU usage while waiting
- **Fast Failure**: Detects process exit immediately
- **Clear Errors**: Provides actionable error messages

## Error Handling

![Error Handling Diagram](weaver-perfect-error-handling.puml)

Perfect error handling provides clear, actionable error messages:

### Error Categories

1. **Pre-Start Errors**:
   - `BinaryNotFound` - Weaver binary not found
   - `RegistryNotFound` - Registry path invalid or missing
   - Fix instructions: Run `cargo make weaver-bootstrap`, pin to known-good version

2. **Startup Errors**:
   - `ProcessStartFailed` - Process spawn failed
   - `ProcessStartFailed` - Process exited immediately (with exit code and stderr)
   - Fix instructions: Check registry validation, review stderr, verify binary

3. **Readiness Errors**:
   - `ProcessStartFailed` - Process exited during wait
   - `ProcessStartFailed` - Timeout exceeded (port not ready)
   - Fix instructions: Check Weaver logs, verify registry valid

4. **Runtime Errors**:
   - `ProcessNotRunning` - Process exited during operation
   - `ProcessStopFailed` - Failed to stop process gracefully
   - Fix instructions: Check process state, manually stop if needed

### Error Message Format

Perfect error messages include:

- **Clear Description**: What went wrong
- **Exit Code**: Process exit code (if applicable)
- **Stderr Output**: Captured stderr from process (if applicable)
- **Fix Instructions**: Actionable steps to resolve
- **Context**: Registry path, timeout values, etc.

Example:
```
🚨 Failed to start Weaver process: Process exited immediately (exit code: 1)
   ⚠️  STOP: Weaver cannot start
   💡 FIX: Check registry validation errors (run: weaver registry check -r registry)
   💡 FIX: Review stderr output above for detailed error messages
   💡 ROOT CAUSE FIX: Set WEAVER_REGISTRY_VERSION=<tag/branch/commit> to pin to known-good version
   📋 Exit code: 1
```

## Component Interactions

![Component Diagram](weaver-perfect-components.puml)

Perfect component interactions ensure clean separation of concerns:

### Layers

1. **Test Layer**: `WeaverTestFixture`, `weaver_test!` macro
2. **Observability Layer**: `ObservabilityTest`, `WeaverValidator`, `WeaverLiveCheck`
3. **Poka-Yoke Types**: Type-safe configuration values
4. **Process Management**: Process lifecycle and monitoring
5. **External Services**: Weaver binary, registry, network ports

### Interactions

- **Test Layer** → **Observability Layer**: Uses high-level API
- **Observability Layer** → **Poka-Yoke Types**: Validates configuration
- **Observability Layer** → **Process Management**: Manages process lifecycle
- **Process Management** → **External Services**: Spawns and monitors Weaver

## Perfect Lifecycle

![Lifecycle Diagram](weaver-perfect-lifecycle.puml)

The perfect lifecycle ensures reliable process management:

### Phases

1. **Initialization**:
   - Validate prerequisites (binary, registry)
   - Download binary if missing
   - Clone registry if missing

2. **Registry Validation**:
   - Check registry exists
   - Clone if missing
   - Validate schema before start

3. **Process Startup**:
   - Spawn process
   - Detect immediate exit
   - Wait for readiness

4. **Readiness Verification**:
   - Check process alive
   - Check port ready
   - Retry with backoff

5. **Running**:
   - Accept telemetry
   - Validate telemetry
   - Monitor process health

6. **Shutdown**:
   - Stop via admin port
   - Kill process if needed
   - Clean up resources

## Type Safety

![Type Diagram](weaver-perfect-types.puml)

Perfect type safety prevents entire classes of errors:

### Poka-Yoke Types

1. **`ValidRegistryPath`**:
   - **Invariant**: Path exists and is directory
   - **Prevention**: Cannot create invalid paths
   - **Usage**: Registry path validation

2. **`NonZeroPort`**:
   - **Invariant**: Port > 0
   - **Prevention**: Cannot create port = 0
   - **Usage**: OTLP and admin port configuration

3. **`PositiveTimeout`**:
   - **Invariant**: Timeout > 0
   - **Prevention**: Cannot create timeout = 0
   - **Usage**: Startup and operation timeouts

4. **`RegistryVersion`**:
   - **Invariant**: Version non-empty
   - **Prevention**: Cannot create empty version
   - **Usage**: Registry version pinning

5. **`RegistryState`**:
   - **Invariant**: Only `Validated` allows path access
   - **Prevention**: Cannot use unvalidated registry
   - **Usage**: Registry validation state machine

### Benefits

- **Compile-Time Safety**: Invalid states cannot be created
- **Explicit Error Handling**: `Option<T>` forces handling of invalid values
- **Self-Documenting**: Types encode invariants
- **Zero-Cost**: Types compile away, no runtime overhead

## Implementation Checklist

### Core Features

- [ ] Type-safe configuration (poka-yoke types)
- [ ] Registry validation before start
- [ ] Process exit detection immediately after spawn
- [ ] Readiness verification (process + port)
- [ ] Exponential backoff retry logic
- [ ] Clear error messages with fix instructions
- [ ] Process state monitoring (`is_running()` with `try_wait()`)
- [ ] Graceful shutdown (admin port + kill fallback)

### Error Handling

- [ ] Pre-start validation errors
- [ ] Process spawn errors
- [ ] Immediate exit detection
- [ ] Readiness timeout errors
- [ ] Runtime process exit errors
- [ ] Shutdown errors

### Testing

- [ ] Unit tests for each component
- [ ] Integration tests with real Weaver
- [ ] Error path tests (80% of bugs)
- [ ] Behavior verification tests
- [ ] Timeout scenario tests
- [ ] Process exit scenario tests

## Configuration

### Environment Variables

- `WEAVER_STARTUP_TIMEOUT` - Startup readiness timeout (default: 5 seconds)
- `WEAVER_REGISTRY_VERSION` - Pin registry to specific version
- `WEAVER_SKIP_REGISTRY_VALIDATION` - Skip validation (not recommended)

### Constants

- `DEFAULT_WEAVER_STARTUP_TIMEOUT_SECONDS` - Default timeout (5 seconds)
- `DEFAULT_OTLP_GRPC_PORT` - OTLP gRPC port (4317)
- `DEFAULT_ADMIN_PORT` - Admin HTTP port (4320)
- `LOCALHOST` - Localhost address ("127.0.0.1")

## Best Practices

### Do's ✅

- Use poka-yoke types for configuration
- Validate registry before starting Weaver
- Check process state immediately after spawn
- Wait for readiness before proceeding
- Provide clear error messages with fix instructions
- Use exponential backoff for retries
- Monitor process health during operation
- Clean up resources on shutdown

### Don'ts ❌

- Don't proceed without readiness verification
- Don't ignore process exit during startup
- Don't use magic numbers (use named constants)
- Don't skip error handling
- Don't use blocking waits (use timeouts)
- Don't proceed with dead process
- Don't modify upstream registry files
- Don't skip registry validation

## Success Criteria

A perfect Weaver live-check implementation:

1. **Reliability**: Always detects process failures immediately
2. **Type Safety**: Invalid states are unrepresentable
3. **Clear Errors**: Actionable error messages with fix instructions
4. **Fast Failure**: Errors detected early, not late
5. **Behavior Verification**: Actual state checked, not just presence
6. **Zero-Cost**: No runtime overhead from abstractions
7. **Testability**: Easy to test with unit and integration tests
8. **Maintainability**: Clear code following established patterns

## References

- [Kaizen Improvement Plan](../process/KAIZEN_WEAVER_READINESS.md) - Incremental improvements
- [Poka-Yoke Design](../process/POKA_YOKE_WEAVER_REGISTRY.md) - Type-level error prevention
- [Root Cause Analysis](../analysis/ROOT_CAUSE_ANALYSIS_WEAVER_REGISTRY.md) - Problem analysis
- [Chicago TDD Standards](../../.cursor/rules/chicago-tdd-standards.mdc) - Testing methodology
- [SPR Guide](../process/SPR_GUIDE.md) - Elite Rust developer standards

## Diagrams

All diagrams are available in PlantUML format:

- [Architecture](weaver-perfect-architecture.puml)
- [Sequence Flow](weaver-perfect-sequence.puml)
- [State Machine](weaver-perfect-state.puml)
- [Readiness Flow](weaver-perfect-readiness-flow.puml)
- [Error Handling](weaver-perfect-error-handling.puml)
- [Components](weaver-perfect-components.puml)
- [Lifecycle](weaver-perfect-lifecycle.puml)
- [Type Safety](weaver-perfect-types.puml)

Render diagrams using PlantUML or view in compatible tools.

## See Also

- **[Weaver Live Check](WEAVER_LIVE_CHECK.md)** - Weaver integration testing guide
- **[Registry Version Pinning](REGISTRY_VERSION_PINNING.md)** - Managing registry versions
- **[Upstream Issue Reporting](UPSTREAM_ISSUE_REPORTING.md)** - Reporting upstream issues
- **[Observability Testing Guide](../observability/observability-testing-guide.md)** - Comprehensive observability testing
- **[OTEL & Weaver Guide](../observability/otel-weaver-guide.md)** - OTEL and Weaver integration
