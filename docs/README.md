# Chicago TDD Tools Documentation

Complete documentation index organized using [Diátaxis framework](https://diataxis.fr/): **Tutorials** (learn), **How-to Guides** (accomplish), **Reference** (lookup), **Explanation** (understand).

## Quick Start

**New to Chicago TDD Tools?** Start with **[Getting Started](getting-started/GETTING_STARTED.md)** → **[Quick Guide](getting-started/QUICK_GUIDE.md)** → **[User Guide](getting-started/USER_GUIDE.md)**

---

## Tutorials (Learning-Oriented)

Step-by-step guides to learn Chicago TDD Tools:

- **[Getting Started](getting-started/GETTING_STARTED.md)** - Installation, first test, troubleshooting
- **[Quick Guide](getting-started/QUICK_GUIDE.md)** - Essential patterns (80% of use cases)
- **[User Guide](getting-started/USER_GUIDE.md)** - Comprehensive usage guide

---

## How-to Guides (Problem-Oriented)

Goal-focused guides to accomplish specific tasks:

### Core Features
- **[Weaver Live Check](features/WEAVER_LIVE_CHECK.md)** - Weaver integration testing
- **[Perfect Weaver Live Check](features/PERFECT_WEAVER_LIVE_CHECK.md)** - Perfect implementation with diagrams
- **[Timeout Enforcement](features/TIMEOUT_ENFORCEMENT.md)** - Timeout handling patterns
- **[Registry Version Pinning](features/REGISTRY_VERSION_PINNING.md)** - Managing registry versions
- **[Upstream Issue Reporting](features/UPSTREAM_ISSUE_REPORTING.md)** - Reporting upstream issues

### Testing Guides
- **[CLI Testing](testing/cli-testing-guide.md)** - Command-line testing patterns
- **[Observability Testing](observability/observability-testing-guide.md)** - Observability testing guide
- **[OTEL Weaver Guide](observability/otel-weaver-guide.md)** - OpenTelemetry + Weaver integration

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

### Analysis & Research
- **[Root Cause Analyses](analysis/)** - Clippy warnings, effectiveness, Weaver registry
- **[Research](research/)** - Innovative testing tools, problems solved
- **[Coverage Strategy](coverage/v1.2.0-coverage-strategy.md)** - Test coverage approach

---

## Additional Resources

### Releases
- **[CHANGELOG](releases/CHANGELOG.md)** - Complete version history
- **[Release Notes v1.2.0](releases/RELEASE_NOTES_v1.2.0.md)** - Current release (v1.2.0)
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
├── releases/            # Release documentation
└── diagrams/            # PlantUML diagrams
```

## Quick Navigation by Topic

**Testing**: [Quick Guide](getting-started/QUICK_GUIDE.md) | [User Guide](getting-started/USER_GUIDE.md) | [CLI Testing](testing/cli-testing-guide.md)  
**Architecture**: [Architecture](reference/ARCHITECTURE.md) | [API Reference](reference/API_REFERENCE.md)  
**Quality**: [SPR Guide](process/SPR_GUIDE.md) | [Code Review](process/CODE_REVIEW_CHECKLIST.md) | [SLA](reference/SLA_REFERENCE.md)  
**Weaver**: [Weaver Live Check](features/WEAVER_LIVE_CHECK.md) | [Diagrams](diagrams/)  
**Releases**: [CHANGELOG](releases/CHANGELOG.md) | [Release Notes](releases/)

---

## Contributing to Documentation

**Placement**: Use Diátaxis categories (Tutorials → `getting-started/`, How-to → `features/`/`testing/`, Reference → `reference/`, Explanation → `process/`/`analysis/`)  
**Standards**: Follow [Documentation Style Guide](process/DOCUMENTATION_STYLE_GUIDE.md)  
**Principles**: Concise, actionable, complete, navigable, current
