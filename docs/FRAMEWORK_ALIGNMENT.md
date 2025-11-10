# Chicago TDD Tools - Framework Alignment Analysis

**Date**: 2025-01-XX  
**Status**: ⚠️ **NEEDS ALIGNMENT WITH EXISTING FRAMEWORK**

---

## Issue Identified

The `chicago-tdd-tools` package was created without fully referencing the existing Chicago TDD micro framework in `knhk-workflow-engine/src/testing/chicago_tdd.rs`.

---

## Existing Framework Structure

### In `knhk-workflow-engine/src/testing/`:

1. **`chicago_tdd.rs`** (703 lines):
   - `WorkflowTestFixture` - Workflow-specific fixture with engine, specs, cases
   - `TestDataBuilder` - Builds `HashMap<String, String>`, converts to JSON
   - `WorkflowSpecBuilder` - Builds workflow specifications
   - `TaskBuilder` - Builds workflow tasks
   - Pattern helpers: `create_test_registry()`, `create_test_context()`, `assert_pattern_*()`
   - Resource helpers: `create_test_resource()`, `create_test_role()`, `create_test_capability()`
   - Worklet helpers: `create_test_worklet()`
   - Performance helpers: `PerformanceTestHelper`
   - Integration helpers: `IntegrationTestHelper`
   - Property tester: `WorkflowPropertyTester`

2. **`property.rs`**:
   - `PropertyTestGenerator` - Generates `WorkflowSpec` objects
   - Property functions: `property_all_workflows_registrable()`, etc.

3. **`mutation.rs`**:
   - `MutationTester` - Works with `WorkflowSpec`
   - `MutationOperator` - Workflow-specific mutations (RemoveTask, AddTask, etc.)

4. **`coverage.rs`**:
   - `CoverageAnalyzer` - Analyzes `WorkflowSpec` coverage
   - `CoverageReport` - Workflow-specific coverage report

---

## What I Created (Misaligned)

### In `chicago-tdd-tools/`:

1. **Generic `TestFixture`** - Not workflow-specific
2. **`TestDataBuilder`** - Builds JSON directly (different API)
3. **`PropertyTestGenerator`** - Generates `HashMap<String, String>` (not `WorkflowSpec`)
4. **`MutationTester`** - Works with `HashMap` (not `WorkflowSpec`)
5. **`CoverageReport`** - Generic (not workflow-specific)

---

## Alignment Strategy

### Option 1: Generic Base + Workflow Extensions (Recommended)

**Structure**:
```
chicago-tdd-tools/
├── src/
│   ├── fixture.rs          # Generic TestFixture
│   ├── builders.rs          # Generic builders
│   ├── assertions.rs        # Generic assertions
│   └── ...

knhk-workflow-engine/src/testing/
├── chicago_tdd.rs          # WorkflowTestFixture extends TestFixture
├── property.rs             # Uses generic PropertyTestGenerator
└── ...
```

**Benefits**:
- Generic tools reusable across projects
- Workflow-specific extensions in workflow engine
- Backward compatible

### Option 2: Extract Generic Parts Only

**Structure**:
```
chicago-tdd-tools/
├── src/
│   ├── fixture.rs          # Generic fixture (no workflow dependencies)
│   ├── builders.rs         # Generic builders
│   ├── assertions.rs       # Generic assertions
│   └── ...

knhk-workflow-engine/src/testing/
├── chicago_tdd.rs          # WorkflowTestFixture uses generic TestFixture
├── property.rs             # WorkflowPropertyTester uses generic PropertyTestGenerator
└── ...
```

**Benefits**:
- Clean separation
- Workflow engine uses generic tools
- No duplication

### Option 3: Full Extraction (Not Recommended)

Move everything to `chicago-tdd-tools` with workflow-specific features behind feature flags.

**Issues**:
- Tight coupling
- Breaks existing code
- Not truly generic

---

## Recommended Approach: Option 1

### Step 1: Make `chicago-tdd-tools` Generic

Keep generic parts:
- `TestFixture` (base fixture)
- `TestDataBuilder` (align API with workflow version)
- `assertions` (generic assertions)
- `property` (generic property testing)
- `mutation` (generic mutation testing)
- `coverage` (generic coverage)

### Step 2: Workflow Engine Uses Generic Tools

```rust
// In knhk-workflow-engine/src/testing/chicago_tdd.rs
use chicago_tdd_tools::prelude::*;

pub struct WorkflowTestFixture {
    base: TestFixture,  // Use generic fixture
    engine: WorkflowEngine,
    // ... workflow-specific fields
}

impl WorkflowTestFixture {
    pub fn new() -> WorkflowResult<Self> {
        Ok(Self {
            base: TestFixture::new()?,
            engine: WorkflowEngine::new(...)?,
            // ...
        })
    }
}
```

### Step 3: Align APIs

Make `TestDataBuilder` API match workflow version:
```rust
// Generic version
pub struct TestDataBuilder {
    data: HashMap<String, String>,  // Match workflow version
}

impl TestDataBuilder {
    pub fn build_json(self) -> serde_json::Value {
        serde_json::to_value(self.data).unwrap_or(serde_json::json!({}))
    }
    
    pub fn build(self) -> HashMap<String, String> {
        self.data
    }
}
```

---

## Next Steps

1. ✅ **Align `TestDataBuilder` API** - Match workflow engine version
2. ✅ **Make `TestFixture` extensible** - Allow workflow-specific extensions
3. ✅ **Update workflow engine** - Use generic tools where possible
4. ✅ **Document alignment** - Show how generic + specific work together

---

## Summary

**Current State**: ⚠️ Misaligned - Created generic package without referencing existing framework

**Recommended**: Align `chicago-tdd-tools` with existing framework patterns and make workflow engine use generic tools as base.

---

**Last Updated**: 2025-01-XX  
**Status**: ⚠️ **NEEDS ALIGNMENT**

