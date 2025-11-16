# chicago-tdd-tools LaTeX Documentation

## Overview

This directory contains comprehensive LaTeX documentation for **chicago-tdd-tools**, a Rust
testing framework that enforces Chicago-style Test-Driven Development through compile-time
type-level verification.

## Document Perspective: Framework-First

**Important**: This documentation is written from the **chicago-tdd-tools perspective**, not
the Chatman Equation perspective. The framework is the primary subject.

- **What We're Documenting**: chicago-tdd-tools testing framework
- **Why We're Documenting It**: Shows how it prevents bugs through type-level poka-yoke
- **How We're Explaining It**: How the framework embodies Chatman Equation principles
- **For Whom**: Developers using the framework, researchers in testing, practitioners wanting deterministic tests

## Document Structure

### Main Document
- **chicago-tdd-tools-formalization.tex** — Root document that includes all chapters and appendices

### Chapters (currently 4 completed, 8 total planned)

1. ✅ **Chapter 1: Framework Overview**
   - What is chicago-tdd-tools?
   - Core principle: Poka-yoke design
   - The AAA pattern enforced via types
   - Framework statistics
   - Chicago-style vs. London-style TDD
   - Why Rust enables poka-yoke

2. ✅ **Chapter 2: Core Testing Primitives**
   - Test Fixtures: reusable context with RAII cleanup
   - Builders: fluent test data construction
   - Assertions: rich assertion helpers with custom messages
   - Test Macros: zero-boilerplate test definition
   - Alert Macros: structured logging
   - Configuration Loading: validated configuration

3. ✅ **Chapter 3: Type-Level Safety**
   - Type state pattern enforcing AAA at compile time
   - PhantomData and zero-sized types
   - Sealed traits for API safety and control
   - Generic fixtures with associated types
   - Const generics for compile-time validation
   - Chatman Constant: recursion depth ≤ 8
   - Error handling without unwrap/expect/panic
   - Proof that invalid test states are unrepresentable

4. 📋 **Chapter 4: Advanced Testing Techniques** (planned)
   - Property-based testing (proptest)
   - Mutation testing framework
   - Snapshot testing (insta)
   - Concurrency testing (loom)
   - CLI testing (trycmd)

5. 📋 **Chapter 5: Validation and Quality Assurance** (planned)
   - Coverage analysis
   - Guard constraints
   - Jobs To Be Done (JTBD) validation
   - Performance validation with RDTSC

6. 📋 **Chapter 6: Observability and Telemetry** (planned)
   - OpenTelemetry integration
   - Weaver live validation
   - Unified observability API
   - Span and metric validation

7. ✅ **Chapter 7: Realizing the Chatman Equation** (DONE)
   - The Chatman Equation in testing context
   - Property 1: Determinism (identical inputs → identical results)
   - Property 2: Idempotence (test(test(x)) = test(x))
   - Property 3: Type Preservation (types maintained through lifecycle)
   - Property 4: Boundedness (execution time is measurable and bounded)
   - Integration: how the framework realizes A = µ(O)
   - Validation: proving the equation holds
   - Complete picture: tests are deterministic by design

8. 📋 **Chapter 8: Practical Guide and Best Practices** (planned)
   - Setting up chicago-tdd-tools
   - Writing effective tests
   - Testing patterns from the cookbook
   - Advanced patterns
   - CI/CD integration

### Supporting Files

- **FRAMEWORK_FOCUSED_OUTLINE.md** — Detailed outline showing framework-first perspective
- **references.bib** — Bibliography (40+ citations)

## Key Concepts

### Poka-Yoke Design
The framework prevents bugs at compile time through the Rust type system:
- Invalid test states are unrepresentable
- Type system encodes test invariants
- Compiler rejects violations before code runs

### Type-Level AAA Pattern
```
TestState<Arrange> → act() → TestState<Act> → assert() → TestState<Assert>
```
Each phase is a different type. Compiler prevents invalid orderings.

### Chatman Equation Realization
```
TestResult = test(Fixture, TestData)
```
The framework ensures:
1. **Determinism**: Identical inputs always produce identical results
2. **Idempotence**: Running twice = running once
3. **Type Preservation**: Test data types maintained throughout
4. **Boundedness**: Execution time is measurable and bounded

## Building the Document

### Prerequisites

```bash
# Ubuntu/Debian
sudo apt-get install texlive texlive-latex-extra texlive-fonts-recommended

# macOS
brew install mactex

# Windows
# Download MiKTeX from https://miktex.org/
```

### Quick Build

```bash
# One-shot compilation (with warnings/errors if needed)
pdflatex -interaction=nonstopmode chicago-tdd-tools-formalization.tex

# Recommended: with bibliography
pdflatex chicago-tdd-tools-formalization.tex
bibtex chicago-tdd-tools-formalization
pdflatex chicago-tdd-tools-formalization.tex
pdflatex chicago-tdd-tools-formalization.tex

# Best: using latexmk (automated)
latexmk -pdf chicago-tdd-tools-formalization.tex
```

### Cleaning Up

```bash
# Remove auxiliary files
latexmk -c

# Remove all generated files
latexmk -C
```

## Document Features

### Mathematical Notation
- Formal definitions using LaTeX math mode
- Theorems with proofs
- Equations for framework properties

### Code Examples
All Rust code examples come from chicago-tdd-tools source or examples:
- Syntax highlighting for readability
- Line numbers for reference
- Comments explaining key concepts
- Cross-references to actual source files

### Tables and Figures
- Comparison tables (framework vs. traditional approaches)
- Module organization diagrams
- Feature flags and dependencies
- Performance characteristics

### Cross-References
- Automatic equation and theorem numbering
- Table of contents with structure
- Bibliography with hyperlinks
- Clickable references in PDF

## Content Highlights

### Chapter 1: Framework Overview
- 10,600 LOC across 59 Rust files
- 7 capability-based module groups
- 15+ feature flags
- 30+ exported macros
- Enforces AAA pattern at compile time

### Chapter 2: Core Primitives
- **Fixtures**: Automatic cleanup via Rust's Drop trait
- **Builders**: Fluent API with `#[derive(TestBuilder)]`
- **Assertions**: `assert_ok!`, `assert_err!`, `assert_in_range!`, etc.
- **Macros**: `test!`, `async_test!`, `fixture_test!`, `performance_test!`

### Chapter 3: Type-Level Safety
- **Type State Pattern**: Zero-cost AAA enforcement
- **Sealed Traits**: API safety and extensibility control
- **Const Generics**: Compile-time bounds validation
- **Error Handling**: Compiler enforces `?` instead of `.unwrap()`
- **Proof**: Invalid test states are unrepresentable

### Chapter 7: Realizing the Chatman Equation
- Shows how testing framework embodies the equation
- Proves determinism through property testing
- Validates idempotence via multiple runs
- Type system proves type preservation
- RDTSC measurement proves boundedness

## Usage Guides

### For Framework Users
1. Read Chapter 1 for overview
2. Use Chapters 2-3 as reference for core features
3. Read Chapter 7 to understand deterministic testing properties
4. Chapter 8 (when complete) has best practices

### For Researchers
1. Study Chapter 3 for type-level verification techniques
2. Chapter 7 shows empirical validation of Chatman Equation
3. References section has 40+ citations for further reading
4. Mathematical proofs demonstrate compile-time guarantees

### For Contributors
1. Chapter 1 explains module organization
2. Chapter 2 documents core APIs
3. Chapter 3 shows design principles
4. Chapter 7 explains the underlying theory

## Customization

The document is organized for easy modification:

```
chicago-tdd-tools-formalization.tex (root)
├── chapters/
│   ├── 01-framework-overview.tex
│   ├── 02-core-primitives.tex
│   ├── 03-type-level-safety.tex
│   ├── 07-chatman-equation-realization.tex
│   ├── (04-08 planned)
│
├── references.bib
└── README.md (this file)
```

To include only specific chapters, edit the root document and comment out unwanted chapters:

```latex
% chicago-tdd-tools-formalization.tex
\include{chapters/01-framework-overview}
\include{chapters/02-core-primitives}
\include{chapters/03-type-level-safety}
% \include{chapters/04-advanced-testing}  % Skip this
\include{chapters/07-chatman-equation-realization}
```

## Related Documentation

- **README.md** (main project) — Project overview and quick start
- **CLAUDE.md** — AI assistant guide (development instructions)
- **src/** — Rust source code (matches documentation examples)
- **examples/** — 11 working examples showcasing framework features
- **cookbook/** — Alexander-style pattern language (20 patterns)

## Key Metrics

| Metric | Value |
|--------|-------|
| **Chapters Completed** | 4 / 8 |
| **Pages (estimated)** | ~100+ (when complete) |
| **Code Examples** | 20+ |
| **Theorems/Properties** | 10+ |
| **Tables** | 30+ |
| **Equations** | 50+ |
| **Bibliography Entries** | 40+ |
| **Total LaTeX Lines** | ~2,500 (so far) |

## Compilation Tips

### If you get font errors:
```bash
# Install missing fonts
sudo apt-get install fonts-liberation
# Or use pdflatex instead of xelatex
```

### If bibliography is wrong:
```bash
# Ensure bibtex sees the .bib file
rm chicago-tdd-tools-formalization.bbl
bibtex chicago-tdd-tools-formalization
pdflatex chicago-tdd-tools-formalization.tex
```

### For incremental builds:
```bash
# Watch file and rebuild automatically
latexmk -pdf -pvc chicago-tdd-tools-formalization.tex
```

## Planned Chapters (Detailed Outlines)

See **FRAMEWORK_FOCUSED_OUTLINE.md** for detailed section-by-section outlines of all 8 chapters
and 4 appendices.

## License

This documentation is part of chicago-tdd-tools (MIT License).

## Citation

If you reference this documentation in academic work:

```bibtex
@techreport{chicago-tdd-tools-2025,
  author = {KNHK Team},
  title = {chicago-tdd-tools: A Rust Framework for Type-Safe,
           Deterministic Testing},
  year = {2025},
  note = {Version 1.3.0},
  url = {https://github.com/seanchatmangpt/chicago-tdd-tools}
}
```

## Contributing

To contribute to the documentation:
1. Fork the repository
2. Create a feature branch
3. Make edits to .tex files
4. Build and verify with `latexmk -pdf`
5. Submit a pull request

## Feedback

Questions or suggestions about the documentation?
- Open an issue: https://github.com/seanchatmangpt/chicago-tdd-tools/issues
- Review the CLAUDE.md guide for development instructions

---

**Document Version**: 1.0 (Chapters 1-3, 7 complete)
**Last Updated**: November 16, 2025
**Framework Version**: chicago-tdd-tools 1.3.0
**Rust Edition**: 2021
