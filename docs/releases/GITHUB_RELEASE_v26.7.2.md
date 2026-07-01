# Chicago TDD Tools v26.7.2: "Multi-Agent Swarm Coordination & /teamwork-preview"

> **Coordinate multi-agent swarms with high integrity.** v26.7.2 introduces the `/teamwork-preview` command, structured prompt elicitation from `prompt_draft.md`, configurable integrity modes (Permissive, Strict, Cryptographic/Audit), progress heartbeats, and MCP tool bridging.

---

## 🎯 Highlights

- 👥 **`/teamwork-preview` Command** — Streamlines multi-agent workflows with a two-phase process: requirements elicitation via `prompt_draft.md` and subagent delegation.
- ⚙️ **Integrity Modes** — Choose from **Permissive** (warnings only), **Strict** (all tests/lint pass), or **Cryptographic/Audit** (tamper-evident BLAKE3 receipt chain signatures for all steps).
- 💓 **Swarm Coordination Heartbeats** — Workspace-based multi-agent coordination with isolation in `.agents/`, liveness tracking via `progress.md` heartbeats, and context handoff with `handoff.md`.
- 🔌 **MCP Agent Bridging** — Exposes external tools and resources to agents through the Model Context Protocol (MCP), configured using `mcp_config.json`.

---

## 📦 Installation

To use v26.7.2 in your project, add the following to your `Cargo.toml`:

```toml
[dev-dependencies]
chicago-tdd-tools = { version = "26.7.2", features = ["testing-extras", "ocel-generation", "receipt-validation"] }
```

---

## ✨ What's Changed

- **Multi-Agent Coordination**: Expose `/teamwork-preview` command for dry-running and validating swarm plans.
- **Prompt Elicitation**: Standardized parsing of requirements and acceptance criteria in `prompt_draft.md`.
- **Liveness Tracking**: Folder-isolated agent environments under `.agents/` with automated `progress.md` heartbeat checks.
- **Reliable Handoffs**: Standardized transition reports containing Observations, Logic Chain, Caveats, Conclusion, and Verification Method.
- **MCP Tool Integration**: Seamless integration of external tools via Model Context Protocol.

---

## 🔗 See Also

- [Release Notes](RELEASE_NOTES_v26.7.2.md)
- [Main Documentation](../README.md)
