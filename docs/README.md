# Chicago TDD Tools Documentation

Complete documentation index for the Chicago TDD Tools testing framework.

## Quick Start

New to Chicago TDD Tools? Start here:

1. **[Getting Started](getting-started/GETTING_STARTED.md)** - Complete setup guide with troubleshooting
2. **[Quick Guide](getting-started/QUICK_GUIDE.md)** - Essential patterns (80% of use cases)
3. **[User Guide](getting-started/USER_GUIDE.md)** - Comprehensive usage guide

## Core Documentation

### Getting Started
Essential guides for new and existing users:

- **[Getting Started](getting-started/GETTING_STARTED.md)** - Complete setup guide with installation, first tests, and troubleshooting
- **[Quick Guide](getting-started/QUICK_GUIDE.md)** - Essential patterns covering 80% of common use cases
- **[User Guide](getting-started/USER_GUIDE.md)** - Comprehensive guide covering all features, patterns, and best practices

### Reference Documentation
Technical reference and architectural documentation:

- **[API Reference](reference/API_REFERENCE.md)** - Complete API documentation for all modules and functions
- **[Architecture](reference/ARCHITECTURE.md)** - Design principles, patterns, and architectural decisions
- **[SLA Reference](reference/SLA_REFERENCE.md)** - Service level agreements and quality standards

## Feature Documentation

Specialized features and integrations:

- **[Weaver Live Check](features/WEAVER_LIVE_CHECK.md)** - Weaver integration testing guide
- **[Perfect Weaver Live Check](features/PERFECT_WEAVER_LIVE_CHECK.md)** - Perfect implementation guide with PlantUML diagrams
- **[Timeout Enforcement](features/TIMEOUT_ENFORCEMENT.md)** - Timeout handling and enforcement patterns
- **[Registry Version Pinning](features/REGISTRY_VERSION_PINNING.md)** - Managing semantic convention registry versions
- **[Upstream Issue Reporting](features/UPSTREAM_ISSUE_REPORTING.md)** - Guidelines for reporting upstream issues

## Process Documentation

Development processes and quality standards:

- **[SPR Guide](process/SPR_GUIDE.md)** - Elite Rust developer standards and best practices
- **[Code Review Checklist](process/CODE_REVIEW_CHECKLIST.md)** - Comprehensive code review guidelines
- **[Poka-Yoke Weaver Registry](process/POKA_YOKE_WEAVER_REGISTRY.md)** - Type-level error prevention patterns
- **[Kaizen Weaver Readiness](process/KAIZEN_WEAVER_READINESS.md)** - Continuous improvement for Weaver integration
- **[Dog Fooding](process/DOG_FOODING.md)** - Using Chicago TDD Tools to test itself

## Analysis & Research

Root cause analyses and research documentation:

### Root Cause Analysis
- **[Clippy Warnings](analysis/ROOT_CAUSE_ANALYSIS_CLIPPY_WARNINGS.md)** - Detailed analysis of clippy warnings
- **[Clippy Warnings Summary](analysis/ROOT_CAUSE_ANALYSIS_CLIPPY_WARNINGS_SUMMARY.md)** - Summary of clippy warning fixes
- **[Effectiveness](analysis/ROOT_CAUSE_ANALYSIS_EFFECTIVENESS.md)** - Analysis of testing effectiveness
- **[Weaver Registry](analysis/ROOT_CAUSE_ANALYSIS_WEAVER_REGISTRY.md)** - Weaver registry integration analysis

### Research
- **[Innovative Testing Tools](research/INNOVATIVE_TESTING_TOOLS.md)** - Exploration of innovative testing approaches
- **[Problems Solved](research/PROBLEMS_SOLVED.md)** - Documentation of solved problems and solutions

## Release Documentation

Version history and release notes:

- **[CHANGELOG](releases/CHANGELOG.md)** - Complete changelog of all releases
- **[Release Notes v1.1.0](releases/RELEASE_NOTES_v1.1.0.md)** - Detailed release notes for version 1.1.0

## Diagrams

PlantUML diagrams for Weaver integration:

- **[Architecture](diagrams/weaver-perfect-architecture.puml)** - Overall architecture diagram
- **[Components](diagrams/weaver-perfect-components.puml)** - Component structure
- **[Error Handling](diagrams/weaver-perfect-error-handling.puml)** - Error handling flows
- **[Lifecycle](diagrams/weaver-perfect-lifecycle.puml)** - Component lifecycle
- **[Readiness Flow](diagrams/weaver-perfect-readiness-flow.puml)** - Readiness check flow
- **[Sequence](diagrams/weaver-perfect-sequence.puml)** - Sequence diagrams
- **[State](diagrams/weaver-perfect-state.puml)** - State machine diagrams
- **[Types](diagrams/weaver-perfect-types.puml)** - Type system diagrams

## External Resources

- **[Pattern Cookbook](../cookbook/src/README.md)** - Alexander-style pattern language for testing, architecture, and design
- **[Examples](../examples/)** - Working code examples

## Documentation Structure

```
docs/
├── README.md (this file)
├── getting-started/     # Essential guides for users
├── reference/           # Technical reference documentation
├── features/            # Feature-specific documentation
├── process/             # Development processes and standards
├── analysis/            # Root cause analyses
├── research/            # Research and innovation documentation
├── releases/            # Changelogs and release notes
└── diagrams/            # PlantUML diagrams
```

## Quick Navigation by Topic

### Testing
- [Quick Guide](getting-started/QUICK_GUIDE.md) - Test macros and patterns
- [User Guide](getting-started/USER_GUIDE.md) - Comprehensive testing guide
- [Pattern Cookbook](../cookbook/src/README.md) - Testing patterns

### Architecture
- [Architecture](reference/ARCHITECTURE.md) - Design principles
- [API Reference](reference/API_REFERENCE.md) - API structure
- [Pattern Cookbook](../cookbook/src/README.md) - Architectural patterns

### Quality & Process
- [SPR Guide](process/SPR_GUIDE.md) - Development standards
- [Code Review Checklist](process/CODE_REVIEW_CHECKLIST.md) - Review guidelines
- [SLA Reference](reference/SLA_REFERENCE.md) - Quality standards

### Weaver Integration
- [Weaver Live Check](features/WEAVER_LIVE_CHECK.md) - Integration guide
- [Perfect Weaver Live Check](features/PERFECT_WEAVER_LIVE_CHECK.md) - Perfect implementation
- [Diagrams](diagrams/) - Weaver architecture diagrams

### Releases
- [CHANGELOG](releases/CHANGELOG.md) - Version history
- [Release Notes](releases/) - Detailed release documentation

## Contributing to Documentation

When adding or updating documentation:

1. Place files in the appropriate subdirectory
2. Update this index (README.md)
3. Update cross-references in related documents
4. Follow the existing documentation style
5. Include code examples where applicable

## Documentation Standards

- **Concise**: Get to the point quickly
- **Actionable**: Provide clear, runnable examples
- **Complete**: Cover all relevant use cases
- **Navigable**: Include clear cross-references
- **Current**: Keep synchronized with code changes
