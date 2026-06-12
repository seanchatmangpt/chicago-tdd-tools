# Release Notes: v26.6.11

## Summary

v26.6.11 is a release preparation update focusing on **dependency alignment**, **version consolidation**, and **compilation/toolchain fixes** across the root crate, proc_macros, spec-harness, and playground. This release aligns versions to `26.6.11` to match the project release pipeline, fixes local path resolutions, and cleans up legacy/non-existent test targets.

## Changes

### 1. Version Consolidation to v26.6.11
All crates in the project repository have had their package versions consolidated or their dependencies updated to version `26.6.11`:
- **Root `chicago-tdd-tools`**: Bumped version to `26.6.11` and updated its internal dependency on `chicago-tdd-tools-proc-macros` to `26.6.11`.
- **`chicago-tdd-tools-proc-macros`**: Bumped version to `26.6.11`.
- **`chatman-spec-harness`**: Bumped version to `26.6.11` and updated its dependencies on `chicago-tdd-tools` and `chicago-tdd-tools-proc-macros` to `26.6.11`.
- **`chicago-tdd-tools-playground`**: Bumped version to `26.6.11` and updated its dependency on `chicago-tdd-tools` to `26.6.11`.

### 2. Dependency Alignments & Local Path Resolution
- **Playground Local Path**: Updated the `chicago-tdd-tools` dependency declaration in `playground/Cargo.toml` to use `path = ".."` in addition to specifying version `26.6.11`. This ensures it builds against the local version of the codebase under development rather than attempting to resolve from the registry.
- **Git2 Dependency Alignment**: Aligned the `git2` dependency in `spec-harness/Cargo.toml` from version `^0.13` to `^0.20` to match the root crate's configuration, avoiding duplicate crate resolutions and compilation errors.

### 3. Cleanup of Non-Existent Targets
- **Removed `spec_conformance` test**: In `spec-harness/Cargo.toml`, the non-existent `spec_conformance` test target referencing `tests/spec_conformance.rs` was removed to prevent cargo target discovery errors.

### 4. Lockfile Refresh
- Updated Cargo.lock files across the workspace by executing `cargo update` in the root, `proc_macros`, and `playground` directories.
