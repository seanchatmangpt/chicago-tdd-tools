# Weaver Live-Check - SPR

Pinnacle of Chicago TDD methodology. Embodies all four Chicago TDD principles in single powerful tool.

## Why Pinnacle

**State-Based Testing**: Validates actual telemetry state against schema. Checks real telemetry data, not mocked. Verifies observable state (conformance), not how telemetry is generated.

**Real Collaborators**: Uses real OTLP streams, not mocked telemetry. Validates against real semantic convention registry. Processes actual telemetry from real applications.

**Behavior Verification**: Verifies what telemetry does (conforms to schema), not how it's generated. Checks observable behavior (structure, types, values), not implementation details. Validates actual effects (conformance), not internal mechanisms.

**AAA Pattern**: Natural Arrange-Act-Assert structure. Arrange: Start Weaver with registry, configure OTLP endpoint. Act: Send telemetry to Weaver endpoint. Assert: Verify validation results (schema conformance).

## Chicago TDD Philosophy

**"Never Trust the Text, Only Trust Test Results"**: Documentation can be wrong. Comments can be outdated. Claims can be false. Only test results show the truth.

**Weaver Embodiment**: Schema is source of truth, not documentation. Live validation shows actual conformance, not theoretical claims. Test results show actual conformance, not claims.

## How Weaver Embodies Principles

**State-Based Testing**: Validates actual telemetry state (spans, metrics, attributes) against semantic convention registry. Checks real telemetry data. Verifies observable state (conformance), not how generated.

**Real Collaborators**: Uses real OTLP streams. Validates against real semantic convention registry. Processes actual telemetry from real applications. Mocks hide real issues. Real telemetry reveals actual problems.

**Behavior Verification**: Verifies what telemetry does (conforms to schema), not how generated. Checks observable behavior (structure, types, values). Validates actual effects (conformance). Tests remain valid even if implementation changes.

**AAA Pattern**: Arrange (start Weaver with registry, configure endpoint), Act (send telemetry to endpoint), Assert (verify validation results).

## Integration with Chicago TDD Tools

**Testcontainers Verification**: Uses testcontainers to verify Weaver integration. Real Weaver Docker container, not mocks. Weaver binary exists and works. Weaver can execute commands correctly. Framework uses own principles to test Weaver integration.

**Automatic Weaver Installation**: Automatically downloads and installs Weaver CLI when referenced. Build script downloads during compilation. Runtime download if not found. Binary resolution checks multiple locations (PATH, target/, vendors/).

**Chicago TDD Integration**: `WeaverValidator::new(registry_path).start()?` for setup. `send_test_span_to_weaver(&endpoint, "http.request")?` for sending telemetry. `validator.stop()?` for cleanup. Weaver validates actual telemetry state against schema. State-based testing with real collaborators.

## Operational Workflow (80/20)

1. **Bootstrap prerequisites**
   ```bash
   # Downloads weaver binary + semantic convention registry
   cargo make weaver-bootstrap
   ```
2. **Run smoke validation**
   ```bash
   # Verifies weaver --version and sends a test span without Docker
   cargo make weaver-smoke
   ```
3. **Execute integration (Docker required)**
   ```bash
   cargo make test-integration    # Runs testcontainers + Weaver live-check suite
   ```

> **Explicit skip only**: Set `WEAVER_ALLOW_SKIP=1` to bypass Weaver tests temporarily (e.g., constrained CI runners). Without the flag, missing prerequisites panic to enforce dogfooding discipline.

## Why This is the Pinnacle

**Real-World Validation**: Validates actual telemetry from real applications, not simulated data. Test real behavior, not mocks.

**Observable Behavior**: Validates observable behavior (telemetry conforms to schema), not implementation details. Verify what code does, not how.

**State-Based Testing**: Validates actual telemetry state against schema, not theoretical claims. Verify outputs and state, not implementation.

**Working Capability**: Validates working capability (real OTLP streams, real schema validation), not just unit tests. Test working capabilities, not just functions.

## Summary

**Key Associations**: Weaver = Pinnacle = Chicago TDD. Schema = Source of Truth = Validation. Real Telemetry = Real Collaborators = State-Based. Observable Behavior = Behavior Verification = AAA Pattern.

**Pattern**: Weaver live-check embodies all four Chicago TDD principles. Validates real telemetry (not mocks). Verifies observable behavior (schema conformance). Tests actual state (telemetry structure). Follows AAA pattern. Shows test results (not claims).

**Conclusion**: Weaver live-check is pinnacle of Chicago TDD because it validates working capabilities with real collaborators, verifying observable behavior through state-based testing.
