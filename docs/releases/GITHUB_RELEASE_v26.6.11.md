# Chicago TDD Tools v26.6.11: "Workspace Alignment & Tooling Update"

> **Version Bump & Workspace Cleanup** - Align dependencies, local paths, and remove obsolete test targets.

---

## 🎯 Highlights

**v26.6.11** is a maintenance release that consolidates crate versions and resolves build configuration issues across the repository:

- 📦 **Version Synchronization**: Bumped root, `proc-macros`, `spec-harness`, and `playground` package versions and inter-dependencies to `26.6.11`.
- 🔗 **Local Path Resolution**: Configured the playground dependency on `chicago-tdd-tools` to use `path = ".."` for direct local reference.
- ⚙️ **Git2 Alignment**: Aligned `git2` dependency in `spec-harness` to `^0.20` to match root config.
- 🧹 **Target Cleanup**: Removed the non-existent `spec_conformance` test target from `spec-harness`.
- 🔒 **Lockfiles Refreshed**: Refreshed all dependencies and Cargo lockfiles.

---

## 📦 Installation

```toml
[dev-dependencies]
chicago-tdd-tools = { version = "26.6.11" }
```

---

## ✨ What's Changed

- **Crate Versioning**: Consistently set all workspace crates to `26.6.11`.
- **Harness & Playground Compilation**: Fixed dependency version mismatches that could lead to unresolved dependency issues.
- **Git Integration**: Ensured uniform dependency on `git2 = "^0.20"`.
- **Test Discoverability**: Cleaned up stale `spec-harness` test metadata.
