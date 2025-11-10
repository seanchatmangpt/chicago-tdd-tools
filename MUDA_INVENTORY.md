# Muda (Waste) Inventory

## Step 1: Waste Identification

### 🔴 Inventory (Dead Code) - HIGH PRIORITY

1. **Deleted files still tracked by git**:
   - `src/macros.rs` - Deleted but may still be referenced
   - `src/otel.rs` - Deleted but may still be referenced  
   - `src/otel_types.rs` - Deleted but may still be referenced
   - `src/testcontainers.rs` - Deleted but may still be referenced
   - `src/weaver.rs` - Deleted (causing compilation errors)
   - `src/weaver_types.rs` - Deleted but may still be referenced
   - **Impact**: HIGH - Causes compilation errors
   - **Effort**: LOW - Remove from git tracking

2. **Unused imports**:
   - `implementation::*` - Multiple files
   - `testcontainers::ImageExt` - Unused
   - `crate::chicago_test` - Unused
   - `crate::assert_err`, `crate::assert_ok` - Unused
   - **Impact**: LOW - Just noise
   - **Effort**: LOW - Remove unused imports

3. **Commented-out dependency**:
   - `criterion` in Cargo.toml (line 71)
   - **Impact**: LOW - Just noise
   - **Effort**: LOW - Remove comment

### 🟡 Transportation (Unnecessary Data Movement) - MEDIUM PRIORITY

4. **Excessive cloning**:
   - 251 instances of `.clone()`, `.to_owned()`, `.to_string()`
   - **Impact**: MEDIUM - Performance overhead
   - **Effort**: MEDIUM - Need to analyze each case
   - **Note**: Many may be necessary, need careful analysis

### 🟢 Over-processing - LOW PRIORITY

5. **Unused manifest key**:
   - `[build]` section in Cargo.toml (line 101-102)
   - **Impact**: LOW - Just warning
   - **Effort**: LOW - Remove or fix

6. **Dead code attributes**:
   - `#[allow(dead_code)]` in several files
   - **Impact**: LOW - May be intentional
   - **Effort**: LOW - Review and remove if truly dead

## Step 2: Waste Prioritization

### High Impact, Low Effort (Do First) ⭐
1. Remove deleted files from git tracking
2. Remove unused imports
3. Remove commented-out criterion dependency
4. Fix unused manifest key

### Medium Impact, Medium Effort (Plan)
5. Analyze and optimize unnecessary clones (sample first, prioritize hot paths)

### Low Impact, Low Effort (Cleanup)
6. Review and remove dead_code attributes if appropriate

## Step 3: Elimination Plan

1. **Immediate**: Fix compilation errors by removing deleted file references
2. **Quick wins**: Remove unused imports, commented code
3. **Analysis**: Sample clone usage to identify optimization opportunities

