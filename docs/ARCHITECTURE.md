# Chicago TDD Framework - Best Practice Architecture

**Date**: 2025-01-XX  
**Status**: ✅ **ARCHITECTURE DESIGNED**

---

## Architecture Principles

### 1. **Separation of Concerns**
- **Generic Base**: Reusable across all projects (`chicago-tdd-tools`)
- **Domain-Specific Extensions**: Workflow-specific functionality (`knhk-workflow-engine`)

### 2. **Composition Over Duplication**
- Workflow-specific components use generic base
- No code duplication
- Consistent APIs

### 3. **Backward Compatibility**
- Existing tests continue to work
- Gradual migration path
- No breaking changes

---

## Architecture Layers

```
┌─────────────────────────────────────────────────────────┐
│  chicago-tdd-tools (Generic Base)                      │
│  - TestFixture (base fixture)                         │
│  - TestDataBuilder (generic data builder)              │
│  - Assertion helpers (generic)                         │
│  - PropertyTestGenerator (generic)                     │
│  - MutationTester (generic)                            │
│  - CoverageAnalyzer (generic)                          │
└─────────────────────────────────────────────────────────┘
                        ▲
                        │ uses
                        │
┌─────────────────────────────────────────────────────────┐
│  knhk-workflow-engine/src/testing/                     │
│  - WorkflowTestFixture (uses TestFixture)              │
│  - WorkflowSpecBuilder (workflow-specific)             │
│  - TaskBuilder (workflow-specific)                     │
│  - Pattern helpers (workflow-specific)                 │
│  - Resource helpers (workflow-specific)                │
│  - WorkflowPropertyTester (uses PropertyTestGenerator) │
│  - WorkflowMutationTester (uses MutationTester)       │
└─────────────────────────────────────────────────────────┘
```

---

## Component Mapping

### Generic → Workflow-Specific

| Generic Component | Workflow-Specific Extension |
|-------------------|---------------------------|
| `TestFixture` | `WorkflowTestFixture` (contains engine, specs, cases) |
| `TestDataBuilder` | Same API, used directly |
| `PropertyTestGenerator` | `WorkflowPropertyTester` (generates WorkflowSpec) |
| `MutationTester` | Workflow-specific mutations (RemoveTask, etc.) |
| `CoverageAnalyzer` | Workflow-specific coverage (tasks, patterns) |

---

## Refactoring Plan

### Phase 1: Generic Base (`chicago-tdd-tools`)
1. ✅ Generic `TestFixture` with test counter and metadata
2. ✅ `TestDataBuilder` with `HashMap<String, String>` → JSON API
3. ✅ Generic assertion helpers
4. ✅ Generic property-based testing
5. ✅ Generic mutation testing
6. ✅ Generic coverage analysis

### Phase 2: Workflow Extensions (`knhk-workflow-engine`)
1. `WorkflowTestFixture` uses generic `TestFixture`
2. Keep workflow-specific builders (`WorkflowSpecBuilder`, `TaskBuilder`)
3. Keep workflow-specific helpers (patterns, resources, worklets)
4. `WorkflowPropertyTester` uses generic `PropertyTestGenerator`
5. Workflow-specific `MutationTester` uses generic base

### Phase 3: API Alignment
1. Ensure `TestDataBuilder` API matches exactly
2. Ensure assertion helpers are consistent
3. Update imports in tests
4. Verify backward compatibility

---

## Benefits

### ✅ Reusability
- Generic tools usable across projects
- No workflow dependencies in base

### ✅ Maintainability
- Single source of truth for generic components
- Workflow-specific code isolated

### ✅ Consistency
- Same APIs across projects
- Same patterns everywhere

### ✅ Extensibility
- Easy to add domain-specific extensions
- Generic base remains stable

---

**Last Updated**: 2025-01-XX  
**Status**: ✅ **ARCHITECTURE DESIGNED**

