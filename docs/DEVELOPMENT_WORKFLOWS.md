# Development Workflows: Standards and Commands

This document provides a comprehensive guide to the development standards (`.cursorrules`) and workflow commands (`.cursor/commands`) used in the Chicago TDD Tools project. It explains how these tools work together to ensure elite-quality Rust development with Lean Six Sigma principles.

## Table of Contents

1. [Overview](#overview)
2. [Development Standards (`.cursorrules`)](#development-standards-cursorrules)
3. [Workflow Commands (`.cursor/commands`)](#workflow-commands-cursorcommands)
4. [Integration Patterns](#integration-patterns)
5. [Quick Reference](#quick-reference)

---

## Overview

The Chicago TDD Tools project uses two complementary systems to ensure elite-quality development:

1. **`.cursorrules`** - Core development standards and principles that guide all development work
2. **`.cursor/commands`** - Workflow commands that implement Lean Six Sigma methodologies for systematic problem-solving and improvement

Together, these systems ensure:
- **Type-first design** - Compile-time guarantees through Rust's type system
- **Zero-cost abstractions** - Performance without compromise
- **Systematic problem-solving** - Data-driven improvement processes
- **Quality-first 80/20** - Maximum value with quality standards maintained
- **DfLSS alignment** - Design for Lean Six Sigma (efficiency + quality)

---

## Development Standards (`.cursorrules`)

The `.cursorrules` file defines the core principles, non-negotiables, and patterns that all development work must follow.

### Core Principles

**Never trust text, only test results.** All code is production-ready with error handling. Focus on 80% value paths. Types encode invariants; compiler enforces correctness. Zero-cost abstractions are free; trait objects have cost. References over owned values; stack over heap. Ownership is explicit; lifetimes prevent use-after-free. APIs guide correct usage through types.

### Critical Non-Negotiables

#### Build System
- **Always use `cargo make` commands**, never direct `cargo` commands
- Cargo-make handles proc-macro crates, includes timeouts, ensures consistency
- Example: `cargo make test` (not `cargo test`)

#### Git Hooks
- **NEVER use `--no-verify` flag**
- All git hooks (pre-commit, pre-push, etc.) MUST run to enforce quality gates
- If hooks fail, fix the issues - do not bypass them
- Hooks are mandatory quality checks that prevent defects from entering the codebase

#### Timeout SLA
Every CLI command MUST have timeout wrapper to prevent freezing:
- Quick checks: `timeout 5s`
- Compilation: `timeout 10s`
- Unit tests: `timeout 1s`
- Integration: `timeout 30s`
- Long ops: `timeout 60s`

Timeouts indicate issues early.

#### Behavior Verification
- Tests verify observable outputs/state changes, not just function existence
- No tests that only check `assert_ok!()` without verifying behavior
- Tests check state changes, outputs, execution order, actual effects

#### Chicago TDD
- State-based testing (verify outputs, not implementation)
- Real collaborators (use real objects, minimize mocks)
- Behavior verification (verify what code does, not how)
- AAA pattern required (Arrange-Act-Assert)

### Elite Rust Mindset

#### Type-First Thinking
- Types encode invariants; compiler as design tool
- Use types to make invalid states unrepresentable
- PhantomData for type-level state machines
- Const generics over runtime values
- Ask: "What can I express in types?" before "What values do I need?"

#### Zero-Cost Awareness
- Generics monomorphize (zero-cost)
- Const generics are zero-cost
- Macros expand efficiently
- References are zero-cost
- Trait objects have dynamic dispatch cost
- Heap allocation has cost
- Ask: "Is this abstraction zero-cost?"

#### Performance Intuition
- References over owned values
- Stack over heap
- Cache locality matters
- Minimize allocations
- Optimize hot paths (20% that matters)
- Ask: "What's the performance characteristic?"

#### Memory Safety
- Ownership is explicit
- Borrowing enables zero-cost
- Lifetimes prevent use-after-free
- Rc/Arc for shared ownership
- Encapsulate unsafe in safe APIs
- Ask: "What are the ownership semantics?"

#### API Design
- Type-safe by default (errors impossible through types)
- Ergonomic interfaces (easy to use correctly, hard to misuse)
- Composable design
- Self-documenting types
- Explicit error handling (Result types, not panics)
- Ask: "How can I make misuse impossible?"

### 80/20 Thinking: Go the Extra Mile

#### Idea Generation
Always generate 3 ideas:
1. **First idea**: Solve immediate problem
2. **Second idea**: Go bigger (solve 80% of related problems with 20% effort) while maintaining quality standards
3. **Third idea**: Maximum value (type-level solutions, compile-time guarantees) with quality first

**Second idea is usually sweet spot** (80% more value, reasonable effort, quality maintained).

#### Quality-First 80/20
- Value includes quality, consistency, and maintainability - these are not optional
- Quality work may require more effort, but it's still high value because it prevents defects, maintains consistency, and improves maintainability
- Consistency (e.g., Rust in Rust project) is high value, not "extra effort"

#### DfLSS Alignment
- Design for Lean Six Sigma - addresses both efficiency (Lean waste elimination) AND quality (Six Sigma defect prevention) from the start
- Prevent defects AND waste rather than fixing them later
- Maintain consistency
- Quality and efficiency are foundational value, not optional
- DfLSS is superior to DFSS because it addresses both waste elimination and quality prevention simultaneously

### Completion Workflow (Mandatory)

1. **Run Tests Immediately**: `cargo make test` before completion claims
2. **Rich Todos**: For each failing test, include: test name, error message, file/line, root cause, proposed fix, status. Batch create 5-10+ related todos in single call
3. **Fix Systematically**: Read failure message → Identify root cause → Fix issue → Run specific test → Verify fix → Update todo status → Remove when fixed
4. **Re-Run All Tests**: `cargo make test` to verify all fixes worked
5. **Verify Completion**: All tests pass, no compilation errors, no test failures, all failing tests fixed and removed from todos, no pending test-related todos. Never mark complete without running tests first

**Quick Feedback**: `cargo make check` (~1s), `cargo make test-unit` (~1s), `cargo make test test_name` (single test). Full validation: `cargo make test`, `cargo make pre-commit`.

### Prohibited Patterns

- Placeholders, TODOs (except documented future enhancements)
- Unhandled errors (unwrap/expect/panics in production)
- Stubs, simulated behavior
- Claims without verification
- Meaningless tests (only assert_ok without behavior check)
- Direct cargo commands
- Type system misuse
- Unnecessary allocations
- Runtime checks when compile-time possible
- `print!` and `println!` macros in library code (use `log!` macros or alert macros instead)
- **NEVER use `--no-verify` flag with git commands**

### Required Patterns

- Real library integrations
- Error handling (Result<T,E>)
- Feature gating
- Test verification
- Behavior verification (observable outputs/state)
- Chicago TDD principles
- Type-first design
- Zero-cost abstractions
- Performance awareness
- Ergonomic APIs
- Structured logging (use `log!` macros or alert macros for library code)

---

## Workflow Commands (`.cursor/commands`)

The `.cursor/commands` directory contains slash commands that implement comprehensive development workflows based on Lean Six Sigma, Design for Lean Six Sigma (DfLSS), and expert engineering practices.

### Command Categories

#### Quality & Process Improvement

**`/kaizen-improvement`**: Continuous improvement workflow - Make small, incremental improvements rather than big rewrites. Implements Plan-Do-Check-Act (PDCA) cycle for sustainable, low-risk improvements.

**Use when**: Making incremental improvements, refactoring, code quality improvements

**`/root-cause-analysis`**: 5 Whys root cause analysis - Find underlying causes of problems, not just symptoms. Implements systematic problem analysis with prevention measures.

**Use when**: Debugging issues, investigating failures, preventing problem recurrence

**Key Features**:
- 5 Whys technique for deep analysis
- Root cause verification
- Prevention measure implementation
- DfLSS vs DFSS distinction (critical methodology difference)

**`/poka-yoke-design`**: Error prevention through design - Make invalid states unrepresentable using Rust's type system. Implement compile-time guarantees and zero-cost abstractions.

**Use when**: Preventing errors at compile time, adding type safety, enforcing invariants

**`/eliminate-muda`**: Waste elimination - Identify and eliminate the 8 types of waste (Muda) in code and process. Focus on value-adding activities.

**Use when**: Optimizing code, removing dead code, eliminating unnecessary complexity

**`/eliminate-mura`**: Standardization - Eliminate unevenness (Mura) through consistent patterns and standards. Standardize processes for predictable outcomes.

**Use when**: Standardizing code patterns, enforcing consistency, reducing variability

#### Problem Solving

**`/dmaic-problem-solving`**: Define-Measure-Analyze-Improve-Control workflow for systematic problem solving. Implements data-driven improvement process.

**Use when**: Solving complex problems, systematic improvement, data-driven decisions

**Workflow**:
1. **Define** - Clearly define problem, scope, success criteria
2. **Measure** - Collect baseline data about the problem
3. **Analyze** - Identify root causes using data (often uses 5 Whys)
4. **Improve** - Implement solution that addresses root cause
5. **Control** - Prevent problem from returning (tests, controls, monitoring)

**`/dmedi-design-process`**: Define-Measure-Explore-Develop-Implement workflow for new designs. Implements Design for Lean Six Sigma (DfLSS) methodology.

**Use when**: Designing new features, creating new systems, greenfield development

**`/triz-problem-solving`**: Theory of Inventive Problem Solving - Systematic innovation patterns for technical contradictions and inventive solutions.

**Use when**: Solving contradictions, finding innovative solutions, technical challenges

**`/fmea`**: Failure Mode and Effects Analysis - Proactive risk assessment and prevention. Identify potential failures before they occur.

**Use when**: Risk assessment, new feature development, critical path analysis

#### Design & Planning

**`/robust-design`**: Design systems robust to variation and noise. Implements Taguchi methods for resilient designs.

**Use when**: Designing reliable systems, handling edge cases, building resilient code

**`/concept-selection`**: Systematic concept evaluation - Pugh matrix for objective concept selection based on criteria weighting.

**Use when**: Choosing between design alternatives, evaluating trade-offs, decision making

**`/voice-of-customer-qfd`**: Quality Function Deployment - Translate customer needs into technical requirements using House of Quality.

**Use when**: Requirements gathering, feature prioritization, customer-focused design

#### Development Practices

**`/80-20-fill-gaps`**: Pareto principle application - Focus on the 20% that delivers 80% of value. Fill critical gaps systematically.

**Use when**: Prioritizing work, identifying critical gaps, maximizing value delivery

**`/expert-testing-patterns`**: Expert testing patterns - Comprehensive testing strategies including AAA pattern, real collaborators, behavior verification.

**Use when**: Writing tests, improving test quality, test-driven development

**Key Patterns**:
- Error path testing (80% of bugs)
- Boundary condition testing
- Resource cleanup testing
- Concurrency testing
- Real dependencies (not mocks)

**`/verify-tests`**: Systematic test verification - Ensure tests are correct, complete, and valuable. Validate test quality.

**Use when**: Reviewing tests, ensuring test coverage, validating test effectiveness

**Workflow**:
1. Run test suite
2. Analyze results (categorize failures)
3. Fix failures systematically
4. Re-run tests
5. Verify completion

#### Process Control

**`/andon-signals`**: Visual management - Recognize and respond to problem signals. Stop and fix issues immediately.

**Use when**: Monitoring build/test failures, detecting quality issues, rapid response

**`/gemba-walk`**: Go to the source - Observe actual code and systems where work happens. Understand reality vs. assumptions.

**Use when**: Understanding systems, investigating issues, learning codebase

#### Release Management

**`/release-preparation`**: Comprehensive release readiness checklist - Ensure code is ready for production deployment.

**Use when**: Preparing for releases, pre-deployment verification, quality gates

#### Git Workflow

**`/acp`**: Add, Commit, Push - Multi-step workflow for staging changes, validating code quality, committing, and pushing.

**Use when**: Committing changes, ensuring quality before commit

**Workflow**:
1. Pre-validation checkpoint (`cargo make pre-commit`)
2. Fix issues (if any)
3. Re-validation checkpoint
4. Stage changes (`git add -A`)
5. Generate commit message
6. Commit changes
7. Push to remote

**CRITICAL**: Pre-commit checks MUST pass before committing. Never use `--no-verify` flag.

### Command Execution Pattern

**CRITICAL**: Commands that require action (DMAIC, root cause analysis, etc.) must:

1. **Create 10+ item todo list** - Not documents/reports
2. **Execute todos** - Implement solutions and controls, not document them
3. **Verify fixes** - Test that solutions work
4. **Complete todos** - Mark todos as done as work completes

**Principle**: Execute solutions and controls, don't document them. Todos track progress, solutions fix problems.

---

## Integration Patterns

The development standards and workflow commands are designed to work together seamlessly. Here are common integration patterns:

### Problem → Analysis → Solution

**Pattern**: `/andon-signals` → `/root-cause-analysis` → `/kaizen-improvement`

**Example**:
1. Andon signal: Test failure detected
2. Root cause analysis: Use 5 Whys to find underlying cause
3. Kaizen improvement: Implement small, incremental fix

### New Feature Development

**Pattern**: `/voice-of-customer-qfd` → `/dmedi-design-process` → `/expert-testing-patterns`

**Example**:
1. Voice of customer: Gather requirements and prioritize
2. DMEDI design: Design feature using DfLSS methodology
3. Expert testing: Implement comprehensive test coverage

### Quality Improvement

**Pattern**: `/verify-tests` → `/root-cause-analysis` → `/poka-yoke-design`

**Example**:
1. Verify tests: Identify test failures
2. Root cause analysis: Find why tests are failing
3. Poka-yoke design: Use type system to prevent errors at compile time

### Code Quality

**Pattern**: `/gemba-walk` → `/eliminate-muda` → `/eliminate-mura`

**Example**:
1. Gemba walk: Observe actual code and understand reality
2. Eliminate muda: Remove waste (dead code, unnecessary complexity)
3. Eliminate mura: Standardize patterns for consistency

### Systematic Problem Solving

**Pattern**: `/dmaic-problem-solving` (uses `/root-cause-analysis` in Analyze step)

**Example**:
1. Define: Problem statement, scope, success criteria
2. Measure: Collect baseline data
3. Analyze: Use 5 Whys (root cause analysis) to find root cause
4. Improve: Implement solution addressing root cause
5. Control: Add tests, establish controls, monitor

### Test-Driven Development

**Pattern**: `/expert-testing-patterns` → `/verify-tests` → `/poka-yoke-design`

**Example**:
1. Expert testing: Write comprehensive tests using expert patterns
2. Verify tests: Ensure tests are correct and complete
3. Poka-yoke design: Use type system to prevent errors that tests catch

---

## Quick Reference

### Essential Commands

**Before Committing**:
```bash
cargo make pre-commit  # Validate before commit
```

**Testing**:
```bash
cargo make test              # Run all tests
cargo make test-unit         # Quick unit tests (~1s)
cargo make test test_name    # Single test
cargo make check             # Quick compilation check (~1s)
```

**Quality Checks**:
```bash
cargo make fmt               # Format code
cargo make lint              # Lint code
cargo make pre-commit        # Full pre-commit validation
```

### Workflow Command Quick Guide

**Problem Solving**:
- `/root-cause-analysis` - Find root causes (5 Whys)
- `/dmaic-problem-solving` - Systematic problem solving
- `/triz-problem-solving` - Innovative solutions

**Quality Improvement**:
- `/kaizen-improvement` - Incremental improvements
- `/eliminate-muda` - Remove waste
- `/eliminate-mura` - Standardize patterns

**Design**:
- `/poka-yoke-design` - Error prevention through types
- `/dmedi-design-process` - New feature design (DfLSS)
- `/robust-design` - Resilient system design

**Testing**:
- `/expert-testing-patterns` - Comprehensive test patterns
- `/verify-tests` - Test verification workflow

**Process Control**:
- `/andon-signals` - Respond to problems
- `/gemba-walk` - Understand reality
- `/acp` - Git workflow with validation

### Standards Quick Reference

**Build System**: Always use `cargo make`, never direct `cargo`

**Git Hooks**: NEVER use `--no-verify` flag

**Timeouts**: All CLI commands must have timeout wrappers

**Tests**: Must verify behavior, not just existence

**Completion**: Run `cargo make test` before marking complete

**80/20**: Always generate 3 ideas, second idea is usually sweet spot

**DfLSS**: Design for Lean Six Sigma (efficiency + quality), not just DFSS (quality only)

### Key Principles

1. **Never trust text, only test results**
2. **Types encode invariants** - Compiler enforces correctness
3. **Zero-cost abstractions** - Generics, const generics, macros, references
4. **References over owned** - Stack over heap
5. **Ownership is explicit** - Lifetimes prevent use-after-free
6. **APIs guide correct usage** - Type-safe by default
7. **80/20 thinking** - Focus on 20% that delivers 80% value
8. **Quality-first** - Quality, consistency, maintainability are not optional
9. **DfLSS alignment** - Address efficiency AND quality from start

---

## Documentation References

- **[Getting Started Guide](getting-started/GETTING_STARTED.md)** - Quick start with verified examples
- **[User Guide](getting-started/USER_GUIDE.md)** - Comprehensive testing guide
- **[API Reference](reference/API_REFERENCE.md)** - Complete API documentation
- **[Architecture](reference/ARCHITECTURE.md)** - Design principles
- **[Chicago TDD Standards](../.cursor/rules/chicago-tdd-standards.mdc)** - Detailed testing standards
- **[Commands README](../.cursor/commands/README.md)** - Complete command reference

---

## Summary

The development standards (`.cursorrules`) and workflow commands (`.cursor/commands`) work together to ensure elite-quality Rust development:

- **Standards** define what excellence looks like (type-first, zero-cost, quality-first)
- **Commands** provide systematic workflows for achieving excellence (DMAIC, 5 Whys, DfLSS)

Together, they ensure:
- Compile-time guarantees through types
- Zero-cost abstractions for performance
- Systematic problem-solving through data
- Quality-first 80/20 thinking
- DfLSS alignment (efficiency + quality)

**Remember**: Never trust text, only test results. Execute solutions, don't just document them. Quality, consistency, and maintainability are foundational value, not optional.

