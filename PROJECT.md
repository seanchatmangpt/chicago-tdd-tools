# Project: Chicago TDD Tools - Governance Refactoring

## Architecture
The Refactored Governance sub-system reduces duplicate verification logic between compile-time const checking and runtime validation, dries up macro definitions, and clarifies data flows.

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

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | Exploration & Analysis | Investigate duplicate logic in `src/core/governance/` and compile-fail tests | None | DONE |
| 2 | Lineage Logic Unification | Unify compile-time and runtime lineage checks into a shared `const fn` in `laws.rs` | M1 | PLANNED |
| 3 | Macro De-duplication | Dry up assertion macros and scan token munching into helper macros/functions | M2 | PLANNED |
| 4 | Visual Documentation | Create `docs/governance_architecture.md` with Mermaid diagrams | M1 | PLANNED |
| 5 | Verification & Testing | Build and run unit tests, compile-fail tests, clippy and formatting checks | M2, M3, M4 | PLANNED |
| 6 | OCEL 2.0 Integration | Implement automatic OCEL generation from test execution and self-teaching loop | M5 | PLANNED |

## Interface Contracts
...
- `on_test_started(name: &str)` - Lifecycle hook for test start.
- `on_test_completed(name: &str, passed: bool)` - Lifecycle hook for test completion.
- `OcelCollector` - DiagnosticSink that produces OCEL 2.0 logs.
- `verify_lineage(nodes: &[LineageNode]) -> Result<(), LineageValidationError>` - Shared validation algorithm.
- `verify_lineage_const(nodes: &[LineageNode]) -> bool` - Panic wrapper for compile-time.
- `verify_lineage_runtime(nodes: &[LineageNode]) -> Result<(), String>` - Diagnostic emitter for runtime.
- `__emit_gov_diag!` - Internal macro to build and emit standard diagnostics.

## Code Layout
- `src/core/governance/laws.rs` - Contains unified verification logic, helpers, and macro-backing validation.
- `src/core/governance/mod.rs` - Macro declarations and token scanning macros.
- `docs/governance_architecture.md` - Design and diagram documentation.
- `tests/governance_tests.rs` - Unit and integration tests.
