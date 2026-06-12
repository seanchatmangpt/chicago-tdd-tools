# Agent Governance Architecture

This document describes the design and flow of the agent governance sub-system in Chicago TDD Tools. It details how laws are enforced at compile-time and runtime, and how diagnostics are routed through the global channel to various sinks.

## 1. Governance Loop Architecture

The governance loop verifies that code modifications, metadata, and runtime execution conform to safety and regulatory laws. It operates in two phases:
1. **Compile-Time Static Verification**: Scans token streams for forbidden constructs and validates lineage history using a `const fn`.
2. **Runtime Execution Verification**: Monitors artifact admission, sector validation stacks, and merge strategies during runtime execution.

```mermaid
graph TD
    %% Nodes
    DeveloperCode["Developer / Agent Code"]
    
    %% Compile-Time Path
    subgraph Compile-Time ["Compile-Time Enforcement"]
        Macros["Governance Macros<br>(assert_admitted!, assert_no_bypass!, etc.)"]
        TokenScanner["Recursive Token Scanner<br>(assert_admitted_scan!, assert_no_bypass_scan!)"]
        ConstLineage["verify_lineage (const fn)"]
        CompileError["std::compile_error! / panic!"]
    end

    %% Runtime Path
    subgraph Runtime ["Runtime Verification Loop"]
        ArtifactAdmission["Artifact Verification / Admission"]
        ValidateArtifact["validate_artifact()"]
        SectorStacks["Sector Stacks<br>(ProcessIntelligenceSector, etc.)"]
        MergeStrategy["Merge Strategy Resolver<br>(Strict, Lenient, Precedence, Custom)"]
        VerdictResult["Validation Result & Conflict Detection"]
    end

    %% Global State & Output
    Channel["Global Channel State<br>(sinks, queue, domain, run_id)"]

    %% Connections
    DeveloperCode -->|1. Macros Guard Code| Macros
    Macros -->|Token Parsing| TokenScanner
    Macros -->|Lineage Constraints| ConstLineage
    TokenScanner -->|Bypass/Empty Violation| CompileError
    ConstLineage -->|Constraint Violation| CompileError

    DeveloperCode -->|2. Artifact Lifecycle| ArtifactAdmission
    ArtifactAdmission -->|Validate| ValidateArtifact
    ValidateArtifact -->|Query| SectorStacks
    SectorStacks -->|Verdicts| MergeStrategy
    MergeStrategy -->|Aggregate Result| VerdictResult
    VerdictResult -->|Emit Diagnostic| Channel
```

## 2. Diagnostic Data Flow

Diagnostics are generated from macro checks, lineage failures, or schema validation conflicts. They are emitted to the global diagnostics channel which applies thread-safe synchronization, capacity limits, and routes the reports to registered sinks.

```mermaid
graph LR
    %% Data Sources
    subgraph Violations ["Violation Triggers"]
        MacroFail["Macro Assertion Failure<br>(Drift, Substrate, Bypass, etc.)"]
        RuntimeLineage["verify_lineage_runtime Failure"]
        RuntimeValidation["validate_artifact Failure / Conflict"]
    end

    %% Channel
    subgraph ChannelSync ["Global Diagnostics Channel"]
        EmitDiag["emit_diagnostic(Diagnostic)"]
        Queue["Global Queue<br>(Capacity Limited)"]
        CloseChan["close_channel()"]
        RunSummary["Compile RunSummary<br>(andon_count, p_admitted, etc.)"]
    end

    %% Sinks
    subgraph Sinks ["Diagnostic Sinks"]
        LspMapper["map_to_lsp()"]
        LspTypes["LSP Diagnostics"]
        CustomSinks["Registered DiagnosticSinks<br>(Loggers, Metrics, etc.)"]
    end

    %% Flow
    MacroFail -->|Emit Diagnostic| EmitDiag
    RuntimeLineage -->|Emit Diagnostic| EmitDiag
    RuntimeValidation -->|Emit Diagnostic| EmitDiag

    EmitDiag -->|Queue Enabled| Queue
    EmitDiag -->|Capacity = 0| CustomSinks

    Queue -->|On Close| CloseChan
    CloseChan -->|Summarize| RunSummary
    CloseChan -->|Drain to Sinks| CustomSinks
    CustomSinks -->|LSP Sink| LspMapper
    LspMapper --> LspTypes
```
