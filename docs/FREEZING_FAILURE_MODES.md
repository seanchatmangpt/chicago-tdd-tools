# Freezing Failure Modes - SPR

Critical issue: Commands that freeze indefinitely block agent progress. Must be prevented at all costs.

## Categories

**1. Test Execution Freezing**: Tests hang indefinitely. Causes: Infinite loops, deadlocks, waiting for resources, network calls without timeouts, blocking I/O, user input. Solutions: Multi-layer timeouts (test-level `#[ntest::timeout(1000)]`, async-level `tokio::time::timeout`, runner-level cargo-nextest, process-level `timeout 1s`). Integration tests use 30s timeout profile.

**2. Build Process Freezing**: Compilation hangs. Causes: Proc macro issues, circular dependencies, infinite recursion, compiler bugs, lock contention, disk I/O. Solutions: `cargo make check` wrapped with `timeout 5s`, `cargo make build` wrapped with `timeout 5s`, `cargo make build-release` wrapped with `timeout 30s`.

**3. Code Quality Tool Freezing**: Clippy/formatting hangs. Causes: Complex analysis, bugs, memory exhaustion, lock contention. Solutions: `cargo make lint` wrapped with `timeout 5s`, `cargo make fmt` wrapped with `timeout 5s`, incremental checks.

**4. Git Operation Freezing**: Git hooks/commands hang. Causes: Commands in hooks hanging, network operations, lock contention. Solutions: Individual checks in hooks have `timeout 5s`, hooks use `cargo make` (built-in timeouts), pre-commit checks staged files only.

**5. Network Operation Freezing**: HTTP/Docker operations hang. Causes: No timeout, network unavailability, DNS issues, firewall blocking. Solutions: `cargo make audit` wrapped with `timeout 15s`, testcontainers tests use 30s timeout, Docker health checks needed.

**6. File System Operation Freezing**: File I/O hangs. Causes: Disk full, permission issues, network file systems, lock contention. Solutions: File operations need timeouts, disk space checks, permission checks.

**7. Process Management Freezing**: Process spawn/wait hangs. Causes: Resource exhaustion, permission issues, system limits. Solutions: All commands wrapped with `timeout` command, process monitoring needed.

**8. Lock Contention Freezing**: Build artifact locks. Causes: Multiple cargo processes, stale lock files, file system locks. Solutions: Lock file monitoring needed, lock timeout needed, automatic cleanup needed.

**9. Interactive Command Freezing**: Commands waiting for input. Causes: Prompts, password prompts, confirmation prompts. Solutions: All commands use non-interactive flags, `cargo make` commands are non-interactive, detect interactive prompts needed.

**10. Resource Exhaustion Freezing**: Memory/CPU exhaustion. Causes: Memory leaks, large data structures, infinite loops, CPU-bound operations. Solutions: Timeouts prevent infinite loops, memory limits needed, CPU limits needed.

**11. External Dependency Freezing**: External tools hang. Causes: Tool bugs, waiting for resources, network issues. Solutions: External tool calls wrapped with timeouts, health checks needed, retry logic needed.

## Universal Solutions

**Defense in Depth**: Process-level timeouts (all `cargo make` tasks), application-level timeouts (tests, runner), network-level timeouts (HTTP clients, DNS), resource-level timeouts (file operations, process operations).

**Timeout Standards**: Fast operations (<5s): Check/Format/Lint 5s, Unit tests 1s. Medium operations (5-30s): Build 5s, Build release 30s, Integration tests 30s, Coverage 30s. Slow operations (>30s): Documentation 20s, Audit 15s, Network operations 15s.

**Non-Interactive Mode**: All commands MUST be non-interactive. `cargo make` commands are non-interactive. Git hooks are non-interactive.

## Implementation Status

**Implemented**: Test timeouts (multi-layer), build timeouts, formatting timeouts, linting timeouts, git hook timeouts, network operation timeouts, overall timeout wrappers, timeout verification, all git commands wrapped, all find commands wrapped, all grep commands wrapped.

**Needs Implementation** (Lower Priority): File operation timeouts, process spawn timeouts, lock operation timeouts, memory limits, CPU limits, health checks for external tools, retry logic for network operations, interactive prompt detection.

## Prevention Checklist

Before running any command: Command has explicit timeout (ALL commands now have timeouts), command is non-interactive (all commands use non-interactive flags), no network operations without timeouts (all network ops have timeouts), git commands have timeouts (all git commands wrapped with 5s timeout), find commands have timeouts (all find commands wrapped with 10s timeout), grep commands have timeouts (all grep commands wrapped with 5s timeout), git hooks have overall timeout wrappers (pre-commit: 30s, pre-push: 120s), timeout command verification (hooks and Makefile.toml verify timeout exists).

**Lower Priority**: File operations have timeouts, process operations have timeouts, locks have timeouts, resource limits are set, retry logic is implemented, health checks are performed.

## Monitoring and Detection

**Detection**: Timeout violations (commands exceeding SLA), process monitoring (processes running longer than expected), resource monitoring (unusual resource usage), lock monitoring (lock files not released).

**Response**: Immediate (kill hanging process), short-term (add timeout to prevent recurrence), long-term (root cause analysis and fix).

## Critical Rules

1. NEVER run a command without a timeout
2. NEVER run an interactive command
3. ALWAYS use `cargo make` (which has timeouts)
4. ALWAYS verify commands are non-interactive
5. ALWAYS monitor for hanging processes

## Summary

**Key Associations**: Freezing = Blocking = Failure. Timeouts = Prevention = SLA. Defense in Depth = Multiple Layers = Reliability. Non-Interactive = Automation = Consistency.

**Pattern**: All freezing modes follow same pattern: Symptoms → Causes → Solutions → Prevention. Universal solution: Timeouts at all layers. Critical rule: Every command must have timeout SLA.
