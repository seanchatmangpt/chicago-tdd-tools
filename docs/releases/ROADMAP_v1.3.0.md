# Roadmap: v1.3.0 - "Enhanced Ergonomics"

**Version:** 1.3.0
**Target Release:** 2025-12-15
**Theme:** Developer Experience Enhancement

---

## Release Timeline

```mermaid
gantt
    title Chicago TDD Tools v1.3.0 Release Timeline
    dateFormat  YYYY-MM-DD
    section Planning
    Requirements Analysis         :done, plan1, 2025-11-15, 3d
    Design Review                 :done, plan2, 2025-11-18, 2d
    Release Plan Approval         :active, plan3, 2025-11-20, 2d

    section Implementation
    Phase 1: Assertions           :impl1, 2025-11-22, 14d
    Phase 2: Fixtures & Builders  :impl2, 2025-12-06, 14d
    Phase 3: Snapshot Testing     :impl3, 2025-12-20, 7d
    Phase 4: Integration & Mutation :impl4, 2025-12-27, 7d
    Phase 5: CLI & Documentation  :impl5, 2026-01-03, 7d
    Phase 6: Testing & Release    :impl6, 2026-01-10, 7d

    section Release
    Release Candidate             :rc, 2026-01-15, 2d
    Final Release                 :milestone, release, 2026-01-17, 1d
```

---

## Feature Roadmap

```mermaid
mindmap
  root((v1.3.0<br/>Enhanced<br/>Ergonomics))
    Assertions
      Collection
        assert_contains!
        assert_subset!
        assert_superset!
      JSON
        assert_json_eq!
      Numeric
        assert_approx_eq!
      Patterns
        assert_matches_pattern!
        assert_matches_glob!
    Fixtures
      Introspection
        Metadata Tracking
        State Snapshots
      Scoped Metadata
        RAII Cleanup
        Stack Storage
    Builders
      Presets
        Named Configs
        Composable
      Auto Fake Data
        Type-Driven
        Realistic Values
      Validation Hooks
        Compile-Time
        Result-Based
    Snapshots
      Inline Mode
        In-Source
        Better Diffs
      Redaction
        UUID/Timestamp
        Regex-Based
      Profiles
        Strict/Pretty/Compact
    Integration
      Wait Conditions
        Log Line
        TCP Port
        Command Exit
      Service Helpers
        Postgres
        Redis
        Kafka
      Reusable Containers
        Module-Level
        Amortized Cost
    Mutation
      New Operators
        SwapValues
        ToggleBoolean
        NumericDelta
        StringCase
    CLI Testing
      Environment
        Scoped Vars
        .env Files
      Separate stderr
        Independent Capture
```

---

## Implementation Phases

```mermaid
flowchart TB
    Start([Start: v1.3.0]) --> Phase1[Phase 1:<br/>Assertions<br/>Weeks 1-2]
    Phase1 --> Phase2[Phase 2:<br/>Fixtures & Builders<br/>Weeks 3-4]
    Phase2 --> Phase3[Phase 3:<br/>Snapshot Testing<br/>Week 5]
    Phase3 --> Phase4[Phase 4:<br/>Integration & Mutation<br/>Week 6]
    Phase4 --> Phase5[Phase 5:<br/>CLI & Documentation<br/>Week 7]
    Phase5 --> Phase6[Phase 6:<br/>Testing & Release<br/>Week 8]
    Phase6 --> Release([Release: v1.3.0])

    Phase1 -.->|Collection Assertions| Deliverable1[assert_contains!<br/>assert_subset!<br/>assert_json_eq!<br/>assert_approx_eq!]
    Phase2 -.->|Fixture/Builder APIs| Deliverable2[Metadata<br/>Scoped Metadata<br/>Presets<br/>Validation]
    Phase3 -.->|Snapshot Workflow| Deliverable3[Inline Mode<br/>Redaction<br/>Profiles]
    Phase4 -.->|Integration Utils| Deliverable4[Wait Conditions<br/>Service Helpers<br/>Mutation Ops]
    Phase5 -.->|Documentation| Deliverable5[Cookbook<br/>Guides<br/>API Docs]
    Phase6 -.->|Quality| Deliverable6[85% Coverage<br/>CI Passing<br/>Release Notes]

    style Phase1 fill:#e1f5ff
    style Phase2 fill:#e1f5ff
    style Phase3 fill:#fff4e1
    style Phase4 fill:#fff4e1
    style Phase5 fill:#e8f5e9
    style Phase6 fill:#f3e5f5
    style Release fill:#c8e6c9
```

---

## Dependency Graph

```mermaid
graph LR
    subgraph Core Foundation
        A[Assertion Macros<br/>Phase 1] --> D[Documentation<br/>Phase 5]
        B[Fixture Introspection<br/>Phase 2] --> D
        C[Builder Enhancements<br/>Phase 2] --> D
    end

    subgraph Advanced Testing
        E[Snapshot Improvements<br/>Phase 3] --> D
        F[Mutation Operators<br/>Phase 4] --> D
        G[Integration Utils<br/>Phase 4] --> D
        H[CLI Testing<br/>Phase 5] --> D
    end

    subgraph Quality Assurance
        D --> I[Comprehensive Testing<br/>Phase 6]
        I --> J[Release Validation<br/>Phase 6]
        J --> K[v1.3.0 Release]
    end

    A -.->|Optional Deps| A1[regex crate<br/>pattern-assertions]
    A -.->|Optional Deps| A2[globset crate<br/>glob-assertions]
    G -.->|Optional Deps| G1[postgres crate<br/>testcontainers-services]
    G -.->|Optional Deps| G2[redis-rs crate<br/>testcontainers-services]

    style A fill:#4fc3f7
    style B fill:#4fc3f7
    style C fill:#4fc3f7
    style E fill:#ffb74d
    style F fill:#ffb74d
    style G fill:#ffb74d
    style H fill:#ffb74d
    style D fill:#81c784
    style I fill:#ba68c8
    style J fill:#ba68c8
    style K fill:#66bb6a
```

---

## Priority Matrix

```mermaid
quadrantChart
    title Feature Prioritization Matrix
    x-axis Low Complexity --> High Complexity
    y-axis Low Impact --> High Impact
    quadrant-1 Quick Wins
    quadrant-2 Strategic
    quadrant-3 Nice-to-Have
    quadrant-4 Deferred
    Collection Assertions: [0.2, 0.9]
    JSON Assertions: [0.3, 0.85]
    Approx Equality: [0.15, 0.75]
    Fixture Metadata: [0.4, 0.7]
    Builder Presets: [0.35, 0.8]
    Inline Snapshots: [0.45, 0.75]
    Service Helpers: [0.55, 0.85]
    Reusable Containers: [0.6, 0.9]
    Pattern Assertions: [0.5, 0.4]
    CLI Environment: [0.3, 0.35]
    Mutation Operators: [0.4, 0.45]
```

**Legend:**
- **Quadrant 1 (Quick Wins):** Collection Assertions, JSON Assertions, Approx Equality
- **Quadrant 2 (Strategic):** Builder Presets, Fixture Metadata, Inline Snapshots, Service Helpers, Reusable Containers
- **Quadrant 3 (Nice-to-Have):** CLI Environment, Mutation Operators, Pattern Assertions
- **Quadrant 4 (Deferred):** None in v1.3.0 scope

---

## Feature Capability Matrix

| Feature Area | v1.0.0 | v1.1.0 | v1.2.0 | v1.3.0 (Planned) | v1.4.0 (Future) |
|-------------|--------|--------|--------|------------------|-----------------|
| **Assertions** | ✅ Basic | ✅ Result/Range | ✅ Performance | ✅ Collections/JSON/Patterns | 🔮 Custom Reporters |
| **Fixtures** | ✅ Basic | ✅ Async | ✅ Dogfooding | ✅ Introspection/Scoped | 🔮 Composition |
| **Builders** | ✅ Basic | ✅ Generic | ✅ OTEL | ✅ Presets/Validation | 🔮 Auto-Derive |
| **Snapshots** | ❌ - | ✅ Basic | ✅ Insta | ✅ Inline/Redaction | 🔮 Semantic Diff |
| **Property** | ✅ Proptest | ✅ Arbitrary | ✅ Coverage | ✅ Shrink Viz | 🔮 Corpus Lib |
| **Mutation** | ✅ Basic | ✅ Framework | ✅ Coverage | ✅ More Operators | 🔮 Semantic Ops |
| **Integration** | ❌ - | ✅ Testcontainers | ✅ Wait Conditions | ✅ Service Helpers | 🔮 Cloud Services |
| **Observability** | ❌ - | ✅ OTEL/Weaver | ✅ Live Check | ✅ - | 🔮 Metrics |
| **CLI Testing** | ❌ - | ✅ Trycmd | ✅ Golden Files | ✅ Environment | 🔮 Parallel Exec |
| **Coverage** | ✅ 70% | ✅ 70% | ✅ 85% | ✅ 85% | ✅ 85% |

**Legend:**
- ✅ Available
- ❌ Not Available
- 🔮 Planned (Future)

---

## Risk Mitigation Timeline

```mermaid
gantt
    title Risk Mitigation Activities
    dateFormat  YYYY-MM-DD
    section Backward Compatibility
    Integration Tests (Existing Examples)  :risk1, 2025-11-22, 56d
    Deprecation Warning Review             :risk2, 2026-01-05, 5d

    section Test Coverage
    Unit Tests (90%+ New Code)             :risk3, 2025-11-22, 56d
    Coverage Enforcement (85%+)            :risk4, 2026-01-10, 7d

    section Performance
    Benchmark Suite Execution              :risk5, 2025-12-20, 14d
    Performance Regression Checks          :risk6, 2026-01-10, 7d

    section Proc-Macro Stability
    Compile-Fail Tests                     :risk7, 2025-12-06, 35d
    Incremental Compilation Tests          :risk8, 2026-01-05, 7d

    section CI/CD
    Pre-Commit Validation                  :risk9, 2025-11-22, 56d
    CI-Local Simulation                    :risk10, 2026-01-10, 7d
```

---

## Documentation Roadmap

```mermaid
flowchart LR
    subgraph Phase 1-2 Docs
        A1[Assertion API Docs] --> D1[API Reference]
        A2[Fixture API Docs] --> D1
        A3[Builder API Docs] --> D1
    end

    subgraph Phase 3-4 Docs
        B1[Snapshot API Docs] --> D2[Cookbook Patterns]
        B2[Integration API Docs] --> D2
        B3[Mutation API Docs] --> D2
    end

    subgraph Phase 5 Docs
        C1[Quick Guide Update] --> D3[User Documentation]
        C2[User Guide Update] --> D3
        C3[Migration Guide] --> D3
    end

    D1 --> E[Complete Documentation]
    D2 --> E
    D3 --> E
    E --> F[Release Notes]
    E --> G[Public Announcement]

    style A1 fill:#e1f5ff
    style A2 fill:#e1f5ff
    style A3 fill:#e1f5ff
    style B1 fill:#fff4e1
    style B2 fill:#fff4e1
    style B3 fill:#fff4e1
    style C1 fill:#e8f5e9
    style C2 fill:#e8f5e9
    style C3 fill:#e8f5e9
    style E fill:#c8e6c9
    style F fill:#ffccbc
    style G fill:#ffccbc
```

---

## Version History & Future

```mermaid
timeline
    title Chicago TDD Tools Version History
    2024-01-01 : v1.0.0 Initial Release
               : Core Testing Framework
               : Fixtures, Builders, Assertions
               : Property, Mutation Testing
    2025-11-10 : v1.1.0 Observability
               : Weaver Integration
               : OTEL Validation
               : Testcontainers Support
               : Module Reorganization
    2025-11-14 : v1.2.0 Coverage Enforcement
               : 85% Line Coverage Mandatory
               : CI/CD Blocking
               : Comprehensive Documentation
    2025-12-15 : v1.3.0 Enhanced Ergonomics (Planned)
               : Assertion Expansion Pack
               : Fixture Introspection
               : Builder Enhancements
               : Snapshot Improvements
    2026-Q1    : v1.4.0 Advanced Features (Future)
               : Async Fixture Composition
               : Custom Test Reporters
               : Test Parallelization
               : Advanced Mutation Strategies
    2026-Q2    : v1.5.0 Cloud Integration (Future)
               : Cloud Service Helpers
               : Distributed Testing
               : Remote Observability
```

---

## Success Metrics Dashboard

### Release Readiness Checklist

```
✅ Phase 1: Assertions (Weeks 1-2)
├── ✅ Collection assertions implemented
├── ✅ JSON assertions implemented
├── ✅ Approximate equality implemented
└── ✅ Pattern assertions implemented (optional features)

✅ Phase 2: Fixtures & Builders (Weeks 3-4)
├── ✅ Fixture introspection API
├── ✅ Scoped metadata
├── ✅ Builder presets
├── ✅ Auto-derived fake data
└── ✅ Builder validation hooks

✅ Phase 3: Snapshot Testing (Week 5)
├── ✅ Inline snapshot mode
├── ✅ Redaction hooks
└── ✅ Snapshot profiles

✅ Phase 4: Integration & Mutation (Week 6)
├── ✅ Enhanced wait conditions
├── ✅ Service helpers (Postgres, Redis)
├── ✅ Reusable containers
└── ✅ New mutation operators

✅ Phase 5: CLI & Documentation (Week 7)
├── ✅ CLI environment helpers
├── ✅ Separate stderr capture
├── ✅ Cookbook examples (5 patterns)
└── ✅ Updated user guides

✅ Phase 6: Testing & Release (Week 8)
├── ✅ 85%+ line coverage maintained
├── ✅ All CI checks passing
├── ✅ Backward compatibility validated
├── ✅ Release notes complete
└── ✅ Published to crates.io
```

### Quality Gates

| Gate | Threshold | Status |
|------|-----------|--------|
| Line Coverage | ≥ 85% | 🟡 Pending |
| New Code Coverage | ≥ 90% | 🟡 Pending |
| Clippy Warnings | 0 | 🟡 Pending |
| Documentation Coverage | 100% (public APIs) | 🟡 Pending |
| Backward Compatibility | 100% | 🟡 Pending |
| Performance Regression | < 5% | 🟡 Pending |

**Legend:**
- 🟢 Passed
- 🟡 Pending
- 🔴 Failed

---

## Stakeholder Communication Plan

```mermaid
flowchart TD
    Start([Release Planning Starts]) --> Internal[Internal Team Notification]
    Internal --> Design[Design Review Period]
    Design --> Approve{Approved?}
    Approve -->|No| Revise[Revise Plan]
    Revise --> Design
    Approve -->|Yes| Implement[Implementation Phase]
    Implement --> Weekly[Weekly Status Updates]
    Weekly --> Beta[Beta Release Announcement]
    Beta --> Community[Community Feedback Period]
    Community --> RC[Release Candidate]
    RC --> Final[Final Release]
    Final --> Announce[Public Announcement]
    Announce --> Blog[Blog Posts & Tutorials]
    Blog --> Monitor[Monitor Feedback]
    Monitor --> Iterate[Plan v1.4.0]

    style Start fill:#e8f5e9
    style Implement fill:#fff9c4
    style RC fill:#ffccbc
    style Final fill:#c8e6c9
    style Announce fill:#f8bbd0
```

---

## Post-Release Activities

### Week 9: Monitoring & Triage
- Monitor GitHub issues for bug reports
- Triage and prioritize quick-fix items
- Collect community feedback on new features

### Week 10-12: Content Creation
- Technical blog post: "Enhanced Ergonomics in Chicago TDD Tools v1.3.0"
- Tutorial series: "Mastering Assertions in v1.3.0"
- Video demos: Fixture introspection, Builder presets, Inline snapshots
- Social media promotion (Twitter, Reddit, Hacker News)

### Week 13+: Next Release Planning
- Gather feedback for v1.4.0 features
- Prioritize community-requested enhancements
- Begin design phase for advanced features

---

## Version Comparison

### v1.2.0 vs v1.3.0 Feature Matrix

| Capability | v1.2.0 | v1.3.0 | Delta |
|-----------|--------|--------|-------|
| Assertion Macros | 7 | 15 | +8 (114% increase) |
| Fixture APIs | 3 | 6 | +3 (100% increase) |
| Builder Features | 4 | 7 | +3 (75% increase) |
| Snapshot Modes | 1 | 4 | +3 (300% increase) |
| Mutation Operators | 5 | 10 | +5 (100% increase) |
| Integration Helpers | 2 | 8 | +6 (300% increase) |
| CLI Testing APIs | 3 | 5 | +2 (67% increase) |
| **Total Features** | **25** | **55** | **+30 (120% increase)** |

### Lines of Code Estimate

| Module | v1.2.0 | v1.3.0 (Est.) | Delta |
|--------|--------|---------------|-------|
| Core | ~2500 | ~3200 | +700 (+28%) |
| Testing | ~1800 | ~2200 | +400 (+22%) |
| Validation | ~1200 | ~1300 | +100 (+8%) |
| Observability | ~1500 | ~1500 | 0 (0%) |
| Integration | ~600 | ~1200 | +600 (+100%) |
| Proc Macros | ~400 | ~600 | +200 (+50%) |
| **Total** | **~8000** | **~10000** | **+2000 (+25%)** |

---

## Appendix: Future Roadmap (v1.4.0+)

### Planned for v1.4.0 (2026-Q1)

- **Async Fixture Composition** - Complex async fixture dependencies
- **Custom Test Reporters** - JUnit XML, JSON, TAP output formats
- **Test Parallelization** - Parallel test execution utilities
- **Advanced Mutation Strategies** - Semantic mutation operators
- **JTBD Scenario Hierarchy** - Nested scenarios for workflow testing

### Considered for v1.5.0 (2026-Q2)

- **Cloud Service Helpers** - AWS/GCP/Azure testcontainer integrations
- **Distributed Testing** - Multi-node test coordination
- **Remote Observability** - Distributed tracing across test boundaries
- **AI-Assisted Test Generation** - LLM-powered test case suggestions

### Long-Term Vision (v2.0.0+)

- **Type-Level Test Orchestration** - Compile-time test dependency resolution
- **Zero-Cost Test Fixtures** - Const-generic fixture optimization
- **Formal Verification Integration** - SMT solver integration for property proving
- **Quantum Testing Support** - Quantum circuit testing utilities (research phase)

---

**Version:** 1.0
**Prepared:** 2025-11-15
**Status:** Planning
**Next Update:** 2025-11-22 (Weekly)
