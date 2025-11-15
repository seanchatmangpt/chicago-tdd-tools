# Sparse Priming Representation (SPR) Guide - Chicago TDD Tools

## What is SPR?

Sparse Priming Representation (SPR) is a language technique optimized for advanced NLP, NLU, and NLG tasks with Large Language Models (LLMs). SPR distills information into compact, associative representations that effectively prime LLMs to understand and work with complex concepts.

## Theory

LLMs embed knowledge, abilities, and concepts in latent space. This latent space can be activated with the correct series of words as inputs, creating useful internal states. SPR uses concise, associative language to prime an LLM to think in specific ways, similar to how shorthand cues prime human minds.

## Methodology for This Project

### Core Principles

1. **Distill, Don't Dilute**: Capture essential concepts, associations, patterns, and metaphors in minimal words
2. **Complete Sentences**: Use full sentences, but make them compact and information-dense
3. **Associative Language**: Use associations, analogies, and metaphors that prime understanding
4. **LLM-First**: Write for another language model, not humans (though humans can still read it)
5. **80/20 Focus**: Preserve the 20% of content that provides 80% of value. Value includes quality, consistency, and maintainability - these are not optional.

### Format Standards

**Structure**:
- Use hierarchical headings (##, ###) to maintain organization
- Group related concepts together
- Use bullet points for lists of concepts/patterns
- Use associations (A = B = C) to link concepts

**Language Style**:
- Succinct statements and assertions
- Complete sentences in compact form
- Associations and analogies (e.g., "Types = invariants = compile-time guarantees")
- Direct, declarative statements
- Action-oriented when describing workflows

**Content Density**:
- Each sentence should convey maximum information
- Eliminate redundancy and filler words
- Preserve critical information (non-negotiables, workflows, patterns)
- Remove examples unless they're essential for understanding

### Project-Specific Patterns

**Rust/Chicago TDD Context**:
- Use type system associations (types = invariants = safety)
- Link performance concepts (zero-cost = generics = monomorphization)
- Connect testing principles (Chicago TDD = state-based = real collaborators)
- Associate workflows (test-first = behavior verification = observable outputs)

**Elite Developer Mindset**:
- Capture thinking patterns, not just rules
- Use questions as associations ("Ask: X" patterns)
- Link concepts to outcomes (type-first → compile-time guarantees)
- Connect patterns to benefits (80/20 → maximum value)

**Workflow Patterns**:
- Mandatory steps as numbered lists
- Critical requirements as bullet points
- Prohibited/Required patterns as concise lists
- Associations for quick reference

**Clippy Standards** (Root Cause Prevention):
- All warnings treated as errors (`-D warnings`) - prevents defects at compile time
- Use `#[allow(clippy::...)]` with justification comments when necessary
- CI/CD pipeline enforces clippy checks automatically - prevents accumulation of lint errors
- Pattern: `#[allow(clippy::lint_name)] // Justification: why this is acceptable`
- Common allows: `expect_used` (with mutex justification), `panic` (test helpers), `unwrap_used` (test code)
- Root cause fix: CI/CD pipeline prevents clippy errors from entering codebase
- **FMEA Fix**: Deny `unwrap_used` and `expect_used` in production code (RPN: 180 → 36)

**Error Handling Patterns** (FMEA: Production Panic Prevention):
- **Rule**: NEVER use `.unwrap()` or `.expect()` in production code (causes panics)
- **Why**: Panics crash the process, lose data, degrade user experience
- **Detection**: Pre-commit hooks + CI enforcement + clippy deny rules
- **Alternatives**:
  - Use `?` operator for error propagation: `let value = result?;`
  - Use `if let Ok(value) = result { ... }` for conditional handling
  - Use `match result { Ok(v) => ..., Err(e) => ... }` for explicit handling
  - Use `unwrap_or(default)` or `unwrap_or_else(|| default)` for fallback values
  - Use `unwrap_or_default()` for types implementing Default
- **In tests**: Use `assert_ok!(result)` or `assert_err!(result)` macros (better error messages)
- **Allowed**: Only in tests, examples, benches, with `#[allow(clippy::unwrap_used)]` + justification
- **Setup**: Run `cargo make install-hooks` to install pre-commit hooks

## SPR Consolidation Process

### Step 1: Identify Core Concepts

Extract the essential concepts, principles, and patterns from the source material. Ask:
- What are the non-negotiable requirements?
- What are the core principles?
- What associations link concepts together?
- What patterns repeat throughout?

### Step 2: Create Associations

Link related concepts using associations:
- Types = invariants = compile-time guarantees
- Zero-cost = generics = monomorphization
- Tests = observable outputs = behavior verification
- 80/20 = second idea = sweet spot = maximum value

### Step 3: Distill to Essentials

Remove:
- Redundant explanations
- Verbose examples (unless critical)
- Filler words and phrases
- Repetitive content

Preserve:
- Critical requirements
- Core principles
- Essential workflows
- Key associations
- Important patterns

### Step 4: Structure for Priming

Organize content to prime understanding:
- Start with core principles
- Group related concepts
- Use hierarchical structure
- End with summary associations

### Step 5: Verify Integrity

Check that:
- All critical information is preserved
- Key concepts are still present
- Workflows are complete
- Associations are clear
- Structure is maintained

## Example: .cursorrules Consolidation

**Before**: 661 lines with verbose explanations, examples, and repetition

**After**: 91 lines (86% reduction) with:
- Core principles distilled
- Associations linked (Types = invariants, etc.)
- Critical requirements preserved
- Workflows condensed
- Patterns captured

**Key Transformations**:
- Verbose explanations → Succinct assertions
- Multiple examples → Essential patterns
- Repetitive content → Associations
- Long sections → Compact statements
- Detailed workflows → Numbered steps

## Application Guidelines

### When to Apply SPR

Apply SPR to:
- Long documentation files (>200 lines)
- Files with redundancy
- Reference documentation
- Standards and guidelines
- Architecture documentation

Don't apply SPR to:
- Getting started guides (need examples)
- Tutorial content (need step-by-step detail)
- Code examples (need full context)

### Target Reduction

Aim for 70-80% reduction while preserving:
- All critical requirements
- Core principles
- Essential workflows
- Key associations
- Important patterns

### Quality Checks

After consolidation:
- Verify all critical information present
- Check associations are clear
- Ensure workflows are complete
- Test readability (can LLM understand it?)
- Validate structure is maintained

## SPR Format Template

```markdown
# Title - SPR

## Core Principles

[Succinct statements of core principles. Use associations where helpful.]

## Critical Requirements

[Non-negotiable requirements. Use bullet points. Link to outcomes.]

## Key Concepts

[Essential concepts with associations. Format: Concept = Association = Outcome]

## Patterns

[Important patterns. Use concise descriptions. Link to benefits.]

## Workflows

[Essential workflows as numbered steps. Remove verbose explanations.]

## Summary

[Key associations and patterns. Quick reference format.]
```

## Best Practices

1. **Start with Core**: Lead with the most important principles
2. **Use Associations**: Link concepts (A = B = C) for quick understanding
3. **Preserve Structure**: Maintain hierarchical organization
4. **Eliminate Redundancy**: Remove repeated information
5. **Focus on Priming**: Write to activate latent understanding in LLMs
6. **Verify Integrity**: Ensure all critical information survives consolidation
7. **Test Effectiveness**: Verify the SPR version primes understanding correctly

## Summary

SPR for this project means: Distill documentation to essential concepts, associations, and patterns. Use complete sentences but make them compact. Write for LLM priming while maintaining human readability. Preserve all critical information while achieving 70-80% reduction. Focus on the 20% of content that provides 80% of value.

**Key Associations**: SPR = Distillation = Priming = Associations = Compact = Complete. Documentation = Concepts + Patterns + Workflows = Essential Information. Consolidation = Reduction + Preservation = Efficiency + Effectiveness.

