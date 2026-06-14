# Chicago TDD Tools Documentation

Complete documentation index organized using [Diátaxis framework](https://diataxis.fr/): **Tutorials** (learn), **How-to Guides** (accomplish), **Reference** (lookup), **Explanation** (understand).

## Quick Start

**New to Chicago TDD Tools?** Start with **[Getting Started](getting-started/GETTING_STARTED.md)** → **[Quick Guide](getting-started/QUICK_GUIDE.md)** → **[User Guide](getting-started/USER_GUIDE.md)**

---

## Tutorials (Learning-Oriented)

Step-by-step guides to learn Chicago TDD Tools:

### Getting Started
- **[Getting Started](getting-started/GETTING_STARTED.md)** - Installation, first test, troubleshooting
- **[Quick Guide](getting-started/QUICK_GUIDE.md)** - Essential patterns (80% of use cases)
- **[User Guide](getting-started/USER_GUIDE.md)** - Comprehensive usage guide

### Examples
- **[Examples Quick Reference](../EXAMPLES.md)** - All examples with quick commands
- **[Examples Documentation](../examples/README.md)** - Complete Diátaxis-organized examples
  - Tutorials: Basic Test, Macro Examples
  - How-To Guides: Property Testing, Snapshot Testing, Mutation Testing, Concurrency, CLI, Testcontainers, OTEL/Weaver
  - Explanation: Go the Extra Mile, Advanced Features

---

## How-to Guides (Problem-Oriented)

Goal-focused guides to accomplish specific tasks:

### Core Features
- **[OCEL 2.0 Process Mining](OCEL.md)** - Mine test runs as Object-Centric Event Logs (feature `ocel-generation`)
- **[Agent Governance Architecture](governance_architecture.md)** - Diagnostic channel, laws, and sector stacks (`core::governance`)
- **[Weaver Live Check](features/WEAVER_LIVE_CHECK.md)** - Weaver integration testing
- **[Perfect Weaver Live Check](features/PERFECT_WEAVER_LIVE_CHECK.md)** - Perfect implementation with diagrams
- **[Timeout Enforcement](features/TIMEOUT_ENFORCEMENT.md)** - Timeout handling patterns
- **[Registry Version Pinning](features/REGISTRY_VERSION_PINNING.md)** - Managing registry versions
- **[Upstream Issue Reporting](features/UPSTREAM_ISSUE_REPORTING.md)** - Reporting upstream issues

### Testing Guides
- **[CLI Testing](testing/cli-testing-guide.md)** - Command-line testing patterns
- **[Observability Testing](observability/observability-testing-guide.md)** - Observability testing guide
- **[OTEL Weaver Guide](observability/otel-weaver-guide.md)** - OpenTelemetry + Weaver integration
- **[Test Verification Results](testing/TEST_VERIFICATION_RESULTS.md)** - Results and fixes for test runs
- **[MdBook Verification Summary](testing/VERIFICATION_SUMMARY.md)** - Executable documentation checks

---

## Reference (Information-Oriented)

Complete technical reference documentation:

- **[API Reference](reference/API_REFERENCE.md)** - All modules, functions, types
- **[Architecture](reference/ARCHITECTURE.md)** - Design principles, patterns, decisions
- **[SLA Reference](reference/SLA_REFERENCE.md)** - Service level agreements, quality standards

---

## Explanation (Understanding-Oriented)

Background, context, and understanding of how/why things work:

### Development Standards
- **[Development Workflows](DEVELOPMENT_WORKFLOWS.md)** - Development standards (`.cursorrules`) and workflow commands
- **[SPR Guide](process/SPR_GUIDE.md)** - Elite Rust developer standards and best practices
- **[Coding Standards](process/CODING_STANDARDS.md)** - Consistent coding standards (eliminate Mura)
- **[Code Review Checklist](process/CODE_REVIEW_CHECKLIST.md)** - Code review guidelines
- **[Documentation Style Guide](process/DOCUMENTATION_STYLE_GUIDE.md)** - Documentation standards

### Quality & Process
- **[FMEA: Tests, Build, Actions](process/FMEA_TESTS_BUILD_ACTIONS.md)** - Failure Mode and Effects Analysis
- **[Test Isolation Guide](process/TEST_ISOLATION_GUIDE.md)** - Test isolation patterns and practices
- **[Poka-Yoke Weaver Registry](process/POKA_YOKE_WEAVER_REGISTRY.md)** - Type-level error prevention
- **[Kaizen Weaver Readiness](process/KAIZEN_WEAVER_READINESS.md)** - Continuous improvement for Weaver
- **[Dog Fooding](process/DOG_FOODING.md)** - Using Chicago TDD Tools to test itself
- **[CI/Unit Test Fixes Summary](process/CI_FIXES_SUMMARY.md)** - Final status of CI/Unit test fixes
- **[GitHub Actions Improvements Summary](process/GITHUB_ACTIONS_SUMMARY.md)** - Comprehensive CI/CD overhaul
- **[Kaizen Improvement Plan](process/KAIZEN_IMPROVEMENT_PLAN.md)** - Plan for error message consistency
- **[RDF Integration Summary](process/RDF_INTEGRATION_SUMMARY.md)** - Core data structures for semantic workflows

### Analysis & Research
- **[FMEA Executive Summary](analysis/FMEA_EXECUTIVE_SUMMARY.md)** - Unit test GitHub Actions FMEA
- **[FMEA: GitHub Actions](analysis/FMEA_GITHUB_ACTIONS.md)** - Workflow failure mode analysis
- **[FMEA: Root Cause Analysis Report](analysis/FMEA_ROOT_CAUSE_ANALYSIS.md)** - Risk mitigation and prevention details
- **[FMEA: Summary Table](analysis/FMEA_SUMMARY_TABLE.md)** - Matrix of failure modes and RPNs
- **[Weaver Timeout RCA](analysis/ROOT_CAUSE_ANALYSIS_WEAVER_TIMEOUT.md)** - Root Cause Analysis for Weaver integration timeouts
- **[Unit Test Failure RCA](analysis/UNIT_TEST_FAILURE_RCA.md)** - Root Cause Analysis for assertions module timeouts
- **[Mura (Unevenness) Inventory](analysis/MURA_INVENTORY.md)** - Standardizing code/docs inconsistencies
- **[Mura Inventory (Examples)](analysis/MURA_INVENTORY_EXAMPLES.md)** - Example folder inconsistencies analysis
- **[Similar Patterns Analysis](analysis/SIMILAR_PATTERNS_ANALYSIS.md)** - Inconsistencies post-macro import standardization
- **[Root Cause Analyses](analysis/)** - Clippy warnings, effectiveness, Weaver registry
- **[Research](research/)** - Innovative testing tools, problems solved
- **[Coverage Strategy](coverage/v1.2.0-coverage-strategy.md)** - Test coverage approach

### LaTeX Specification
- **[LaTeX Formalization Summary](latex/LATEX_FORMALIZATION_SUMMARY.md)** - Mathematical formalization of the Chatman Equation
- **[LaTeX Rewrite Summary](latex/LATEX_REWRITE_SUMMARY.md)** - Framework-first documentation rewrite

### Historical & Archives
- **[Final Completion Summary](archive/FINAL_SUMMARY.md)** - Theory to executable multi-sector spec
- **[Project Status](archive/PROJECT_STATUS.md)** - Progress and timeline overview
- **[Progress Update](archive/PROGRESS_UPDATE.md)** - Phase 2 completion and next steps
- **[PR Description](archive/PR_DESCRIPTION.md)** - PR overview of CI/CD pipeline overhaul
- **[Phase 1 Summary](archive/PHASE_1_SUMMARY.md)** - Spec Harness & Literate Verification
- **[Phase 2 Summary](archive/PHASE_2_SUMMARY.md)** - Core Ontology & Operator Registry
- **[Phase 3 Summary](archive/PHASE_3_SUMMARY.md)** - Paper as Self-Hosting RDF Instance
- **[Phase 4 Summary](archive/PHASE_4_SUMMARY.md)** - Sector-Grade Reference Stacks

---

## Additional Resources

### Releases
- **[CHANGELOG](releases/CHANGELOG.md)** - Complete version history
- **[Release Notes v26.6.121](releases/RELEASE_NOTES_v26.6.121.md)** - Current release (v26.6.121)
- **[GitHub Release v26.6.121](releases/GITHUB_RELEASE_v26.6.121.md)** - Highlights for v26.6.121
- **[Release Notes](releases/)** - Detailed release documentation

### Diagrams
- **[Weaver Diagrams](diagrams/)** - PlantUML diagrams (architecture, components, error handling, lifecycle, sequence, state, types)

### External
- **[Pattern Cookbook](../cookbook/src/README.md)** - Alexander-style pattern language
- **[Examples](../examples/)** - Working code examples

---

## Documentation Structure (Diátaxis)

```
docs/
├── README.md (this file)
├── getting-started/     # Tutorials (learn)
├── features/            # How-to Guides (accomplish)
├── testing/             # How-to Guides (accomplish)
├── observability/       # How-to Guides (accomplish)
├── reference/           # Reference (lookup)
├── process/             # Explanation (understand)
├── analysis/            # Explanation (understand)
├── research/            # Explanation (understand)
├── latex/               # LaTeX formalization docs
├── archive/             # Historical documents and milestones
├── releases/            # Release documentation
└── diagrams/            # PlantUML diagrams
```

## Quick Navigation by Topic

**Testing**: [Quick Guide](getting-started/QUICK_GUIDE.md) | [User Guide](getting-started/USER_GUIDE.md) | [CLI Testing](testing/cli-testing-guide.md)  
**Architecture**: [Architecture](reference/ARCHITECTURE.md) | [API Reference](reference/API_REFERENCE.md)  
**Quality**: [SPR Guide](process/SPR_GUIDE.md) | [Code Review](process/CODE_REVIEW_CHECKLIST.md) | [SLA](reference/SLA_REFERENCE.md)  
**Process Mining**: [OCEL 2.0](OCEL.md) | **Governance**: [Architecture](governance_architecture.md)  
**Weaver**: [Weaver Live Check](features/WEAVER_LIVE_CHECK.md) | [Diagrams](diagrams/)  
**Releases**: [CHANGELOG](releases/CHANGELOG.md) | [Release Notes](releases/)

---

## Contributing to Documentation

**Placement**: Use Diátaxis categories (Tutorials → `getting-started/`, How-to → `features/`/`testing/`, Reference → `reference/`, Explanation → `process/`/`analysis/`)  
**Standards**: Follow [Documentation Style Guide](process/DOCUMENTATION_STYLE_GUIDE.md)  
**Principles**: Concise, actionable, complete, navigable, current
