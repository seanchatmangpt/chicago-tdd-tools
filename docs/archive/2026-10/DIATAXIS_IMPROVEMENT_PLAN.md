# Diátaxis-Based Improvement Plan

## Current Assessment

The application guide is **87% complete** structurally but needs refinement for **optimal user experience** using Diátaxis principles.

### Current Issues (Ranked by Impact)

| Issue | Impact | Effort | Priority |
|-------|--------|--------|----------|
| Missing reference sections | HIGH | MEDIUM | 🔴 P1 |
| Blended how-to + reference | HIGH | MEDIUM | 🔴 P1 |
| No coherent learning path | HIGH | MEDIUM | 🔴 P1 |
| Incomplete tutorials | MEDIUM | HIGH | 🟡 P2 |
| Navigation ambiguity | MEDIUM | LOW | 🟡 P2 |
| Content duplication | LOW | MEDIUM | 🟢 P3 |

---

## Phase 1: Quick Wins (1-2 hours)

### 1.1 Add Content Type Indicators
Add clear markers to each page:

```markdown
# Fixtures & Test Setup  
📚 **Reference** | 🔧 **How-to** | 🎓 **Tutorial**
```

**Files to update**: All 23 pages in src/
**Impact**: Immediate clarity on document purpose

### 1.2 Create API Reference Checklist
Add tables to each component:

**fixtures.md** add:
```markdown
## Quick Reference

| Method | Parameters | Returns | Purpose |
|--------|-----------|---------|---------|
| new() | none | Result<TestFixture> | Create fixture |
| set_metadata() | key, value | () | Store test data |
| get_metadata() | key | Option<&String> | Retrieve data |
| capture_snapshot() | state | () | Save state |
| snapshots() | none | &[HashMap] | Get all snapshots |
```

**Files to update**:
- core/fixtures.md
- core/data-builders.md  
- advanced/mutation-testing.md
- guides/otel.md

**Impact**: Users can quickly look up APIs

### 1.3 Create "User Goal" Navigation
Add to introduction.md:

```markdown
## What Do You Want to Do?

**I want to learn the basics**
→ Go to: Getting Started (tutorial)

**I need to solve a specific problem**
→ Go to: How-to Guides (by problem)

**I need to find an API**
→ Go to: Reference Documentation

**I want to understand the philosophy**
→ Go to: Explanation & Design
```

**Impact**: Users immediately know where to go

---

## Phase 2: Structure Improvements (3-4 hours)

### 2.1 Create "Getting Started" Tutorial Section
New file: `tutorials/getting-started.md`

```markdown
# Getting Started: Your First Test

## 1. Create Your First Test (5 minutes)
[Step-by-step with actual code]

## 2. Use a Fixture for Test Isolation (5 minutes)
[Concrete example]

## 3. Build Test Data (5 minutes)
[Real scenario]

## 4. Write Effective Assertions (5 minutes)
[Common patterns]

## 5. Test Error Cases (5 minutes)
[Error path demonstration]
```

**Impact**: New users have a clear 25-minute path to competency

### 2.2 Reorganize Core Patterns
Split each into three parts:

**Pattern A: Fixtures**
- `tutorials/fixtures-tutorial.md` - Learn fixtures (TUTORIAL)
- `core/fixtures.md` - Using fixtures (HOW-TO)
- `reference/fixture-api.md` - Methods & types (REFERENCE)

Same for: Builders, Assertions, Error Paths

**Impact**: Users can choose their approach (learn vs solve vs lookup)

### 2.3 Create Comprehensive Reference Section
New directory: `reference/`

```
reference/
├── README.md                  # All APIs at a glance
├── fixture-api.md            # TestFixture complete API
├── builder-api.md            # TestDataBuilder complete API
├── assertion-api.md          # All assertion helpers
├── mutation-operators.md     # All MutationOperator variants
├── property-testing-api.md   # ProptestStrategy API
├── snapshot-api.md           # SnapshotAssert API
├── cli-api.md               # CLI testing API
├── concurrency-api.md       # Loom testing API
└── otel-api.md              # OTEL/Span/Metric types
```

**Impact**: Complete lookup reference (one source of truth)

---

## Phase 3: Content Completion (4-6 hours)

### 3.1 Complete Real-World Tutorials
Currently: Sketchy templates
Should be: Full runnable examples

**tutorials/cli-application-complete.md**
```markdown
# Tutorial: Build a Todo CLI

## Step 1: Project Setup (10 min)
- Create project
- Add chicago-tdd-tools
- Set up structure

## Step 2: First Command (20 min)
- Implement `todo list`
- Write tests
- Run tests

## Step 3: Add & Remove (20 min)
- Implement `todo add`
- Implement `todo remove`
- Full test coverage

## Step 4: Integration Tests (20 min)
- Test CLI e2e
- Test error paths
- Test edge cases

Result: Working CLI with 80% coverage
```

**Impact**: Users can build something real

### 3.2 Complete Web Service Tutorial
Similarly for `tutorials/web-service-complete.md`

### 3.3 Complete Docker Integration Tutorial
Similarly for `tutorials/docker-integration-complete.md`

---

## Phase 4: Navigation Polish (1-2 hours)

### 4.1 Add Section Headers
Each section starts with:

```markdown
# Advanced Testing Techniques

This section contains:
- **Tutorials** 🎓 - Learn each technique
- **How-to Guides** 🔧 - Solve specific problems
- **Reference** 📚 - Look up APIs
```

### 4.2 Create Decision Matrix
New file: `guides/choosing-techniques.md`

```markdown
# Choosing the Right Testing Technique

| Your Question | Technique | Section |
|--------------|-----------|---------|
| How do I test mathematical properties? | Property-Based | `tutorials/property-testing.md` (planned path, never landed) |
| How do I validate test quality? | Mutation Testing | `tutorials/mutation-testing.md` (planned path, never landed) |
| etc. | | |
```

### 4.3 Add "What's Next" Links
Each page ends with:

```markdown
## What's Next?

**Just learned about fixtures?**
→ `how-to/fixtures.md` (planned path, never landed)

**Want to see all fixture methods?**
→ `reference/fixture-api.md` (planned path, never landed)

**Ready for advanced topics?**
→ `advanced/README.md` (planned path, never landed)
```

---

## Estimated Timeline

| Phase | Effort | Timeline | Priority |
|-------|--------|----------|----------|
| Phase 1: Quick Wins | 2 hrs | **This week** | 🔴 P1 |
| Phase 2: Restructure | 4 hrs | **This month** | 🔴 P1 |
| Phase 3: Complete | 6 hrs | **This month** | 🟡 P2 |
| Phase 4: Polish | 2 hrs | **Next month** | 🟢 P3 |
| **Total** | **~14 hrs** | **~3-4 weeks** | |

---

## Quick Win Checklist (Do First)

- [ ] Add content type indicators to all 23 pages
- [ ] Add API reference tables to 5 key files
- [ ] Create user goal navigation section
- [ ] Add "What's Next" section to 10 critical pages

**Time: 2 hours → High Impact**

---

## Success Metrics

After improvements:

✅ Users can identify document type at a glance
✅ New users have 25-min getting-started path
✅ Users can complete at least one real tutorial
✅ Users can look up any API without scrolling tutorials
✅ Users know which section answers their question
✅ Navigation between related content is clear

---

## File Changes Summary

### New Files to Create (Phase 2-3)
- tutorials/getting-started.md
- tutorials/fixtures-tutorial.md
- tutorials/property-testing-tutorial.md
- tutorials/mutation-testing-tutorial.md
- tutorials/cli-application-complete.md
- tutorials/web-service-complete.md
- tutorials/docker-integration-complete.md
- reference/README.md
- reference/fixture-api.md
- reference/builder-api.md
- reference/assertion-api.md
- reference/mutation-operators.md (expand from current)
- guides/choosing-techniques.md

### Modified Files (Phase 1-2)
- All 23 existing files: Add content type indicators
- core/fixtures.md: Add quick reference section
- core/data-builders.md: Add quick reference section
- advanced/mutation-testing.md: Add quick reference section
- guides/otel.md: Add quick reference section
- introduction.md: Add "user goal" navigation

---

## Next Steps

1. **This week**: Complete Phase 1 (quick wins)
2. **Next week**: Complete Phase 2 (restructuring)
3. **Following week**: Complete Phase 3 (full tutorials)
4. **Polish**: Phase 4 as needed

---

## Supporting Diátaxis Principles

The plan follows Diátaxis by:

1. ✅ **Tutorials** - Clear learning paths for new users
2. ✅ **How-to** - Problem-focused guides separate from learning
3. ✅ **Reference** - Complete API lookup sections
4. ✅ **Explanation** - Conceptual understanding isolated
5. ✅ **Navigation** - Users know exactly where to go
6. ✅ **Progressive** - Scaffold from basics to advanced

