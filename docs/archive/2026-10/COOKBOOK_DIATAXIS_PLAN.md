# Cookbook Diátaxis Restructuring Plan

**Goal**: Transform the 20-pattern cookbook from pure pattern language into a navigable Diátaxis framework that serves learners at every stage.

---

## Current State Analysis

### Existing Assets
- **20 Patterns** across 3 families:
  - Testing Patterns (5): AAA, Error Paths, Boundaries, Resource Cleanup, Real Collaborators
  - Architecture Patterns (5): Generic Base, Extension Layer, Composition, Single Source, Capability Groups
  - Design Patterns (10): Zero-Cost, Type Safety, Sealed Traits, Compile-Time, Type State, Fixture Lifecycle, Builder Test Data, Timeout Defense, Feature Gating, Macro Enforcement
- **Format**: Alexander-style (Context, Problem, Solution, Forces, Examples, Related Patterns)
- **Content Quality**: Strong, technical, theoretical foundation

### Weaknesses
- No learning path guidance
- No "decision matrix" for choosing patterns
- Limited practical, step-by-step implementation guides
- No content-type labeling
- Missing real-world examples and use cases
- Limited cross-references and navigation

---

## Diátaxis Framework Application

### How Diátaxis Maps to Cookbook

| Diátaxis Type | What It Is | Cookbook Application |
|---------------|-----------|----------------------|
| **Tutorials** | Learning-oriented, progressive | Pattern Learning Paths (3 sequences) |
| **How-to Guides** | Problem-oriented, practical | "Implement Pattern X" guides (1 per pattern) |
| **Reference** | Lookup-oriented, complete | Enhanced pattern documentation (existing + improvements) |
| **Explanations** | Understanding-oriented | Pattern philosophy, when/why (context expansion) |

---

## Implementation Plan

### Phase 1: Content Indicators & Quick References (2 hours)
**Goal**: Label all patterns and add lookup tables

**Tasks**:
- [ ] Add Diátaxis type indicator to each pattern (most are 🔧 HOW-TO)
- [ ] Create "Pattern at a Glance" quick reference for each pattern
- [ ] Add content-type indicators to pattern family READMEs
- [ ] Create Pattern Quick Reference Card (all 20 on one page)

**Deliverables**:
- 20 enhanced patterns with headers and quick refs
- 3 family READMEs with organization clarity
- 1 Quick Reference Card for all patterns

### Phase 2: Tutorial Learning Sequences (3 hours)
**Goal**: Create progressive learning paths through patterns

**Tasks**:
- [ ] Create "Learning Testing Patterns" (90-min tutorial path)
- [ ] Create "Learning Architecture Patterns" (60-min tutorial path)
- [ ] Create "Learning Design Patterns" (120-min tutorial path)
- [ ] Link patterns in dependency order (e.g., AAA → Error Paths → Real Collaborators)

**Structure for each tutorial**:
- Intro explaining why these patterns matter
- Progressive sequence of 5 patterns
- Checkpoint questions at each step
- Practice exercises

**Deliverables**:
- 3 comprehensive tutorial sequences
- ~60 minutes learning content per family
- Practice exercises and checkpoints

### Phase 3: Pattern Decision Matrix (2 hours)
**Goal**: Help users find the right pattern for their problem

**Tasks**:
- [ ] Create "Choosing Your Pattern" decision guide
- [ ] Build matrix: "I want to solve X, which pattern helps?"
- [ ] Create "Pattern by Problem" index
- [ ] Add "Pattern Decision Tree" for quick lookup

**Coverage**:
- Testing scenarios (What to test, how to structure, cleanup)
- Architecture scenarios (Organizing code, extending, reusing)
- Design scenarios (Type safety, performance, compile-time checks)

**Deliverables**:
- 1 comprehensive decision guide
- 3 decision matrices (one per family)
- ~40 problem-to-pattern mappings

### Phase 4: Navigation, Cross-References & Real-World Examples (2 hours)
**Goal**: Deep integration with code examples and navigation

**Tasks**:
- [ ] Add "Real-World Example" section to each pattern
- [ ] Create cross-references between related patterns
- [ ] Add "Next Pattern" suggestions after each one
- [ ] Link patterns to actual code in chicago-tdd-tools source
- [ ] Create pattern implementation checklist for each
- [ ] Update SUMMARY.md with new structure

**For each pattern**:
- Identify 1-2 real examples in the codebase
- Add "See This Pattern In Action" section
- Create implementation checklist
- Add links to related documentation

**Deliverables**:
- 20 enriched patterns with real-world examples
- Improved navigation structure
- Cross-reference map
- Implementation checklists

### Phase 5: Integration with Application Guide (1 hour)
**Goal**: Link cookbook to application guide seamlessly

**Tasks**:
- [ ] Create "Patterns in Practice" guide linking tutorials to patterns
- [ ] Add cookbook links to application guide tutorials
- [ ] Cross-reference CLI/Web tutorials to relevant patterns
- [ ] Create "Pattern → Feature" lookup

**Deliverables**:
- Integration guide
- Bidirectional links
- Pattern-to-tutorial mappings

---

## Content Expansion Strategy

### For Each Pattern: Add

1. **Pattern Header**
   ```
   > 🔧 **PATTERN NAME** | **Type** | Solve this problem
   ```

2. **Quick Glance Table**
   ```
   | Aspect | Details |
   | Problem | What this pattern solves |
   | Solution | Core idea in 1-2 sentences |
   | When To Use | Typical scenarios |
   | When NOT To Use | Anti-patterns |
   | Trade-offs | What you gain/lose |
   | Complexity | Low/Medium/High |
   | Real-World Example | Link to actual code |
   ```

3. **Implementation Checklist**
   - [ ] Step 1
   - [ ] Step 2
   - etc.

4. **Real-World Example Section**
   - From chicago-tdd-tools codebase
   - File path and line numbers
   - Brief explanation

5. **Common Mistakes Section**
   - What developers often get wrong
   - How to avoid them

6. **Related Patterns**
   - Improved with descriptions
   - "Use Pattern X before this"
   - "Combine with Pattern Y for"

---

## Expected Outcomes

### User Experience Improvements
- **New users**: Clear learning path, tutorials guide progression
- **Problem-solvers**: Decision matrices help find right pattern
- **Reference users**: Quick tables and implementation checklists
- **Practitioners**: Real-world examples show actual usage

### Content Metrics
- **Learning paths**: 3 sequential tutorials (270 min total)
- **Decision matrices**: 3 matrices covering 40+ scenarios
- **Code examples**: 20+ patterns with real-world usage
- **Cross-references**: All 20 patterns linked meaningfully
- **Navigation**: Multiple paths through the content

---

## Success Criteria

✅ All 20 patterns have content indicators
✅ All 20 patterns have quick reference tables
✅ 3 tutorial learning paths created
✅ Pattern decision matrix covers 40+ scenarios
✅ 20+ real-world code examples added
✅ Navigation between patterns improved
✅ Implementation checklists for each pattern
✅ Cookbook and application guide are cross-linked

---

## Timeline

**Phase 1**: 2 hours → Content indicators, quick refs
**Phase 2**: 3 hours → Learning sequences
**Phase 3**: 2 hours → Decision matrix
**Phase 4**: 2 hours → Navigation & real-world examples
**Phase 5**: 1 hour → Integration

**Total**: ~10 hours work

---

## Branch & Commits

**Branch**: `claude/mdbook-chicago-tdd-tools-01Finfw9y24Nc9LrsViqJ8kL` (same as application guide)

**Commits**:
1. docs(cookbook): Phase 1 - Content indicators and quick references
2. docs(cookbook): Phase 2 - Tutorial learning sequences
3. docs(cookbook): Phase 3 - Pattern decision matrix
4. docs(cookbook): Phase 4 - Navigation and real-world examples
5. docs(cookbook): Phase 5 - Integration with application guide
