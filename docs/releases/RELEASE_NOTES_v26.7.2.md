# Release Notes: v26.7.2

## Quick Navigation

- [GitHub Release Details](GITHUB_RELEASE_v26.7.2.md)
- [Main Documentation](../README.md)
- [Style Guide](../process/DOCUMENTATION_STYLE_GUIDE.md)

## Summary

v26.7.2 introduces the `/teamwork-preview` slash command, enabling multi-agent swarm coordination, requirements elicitation via `prompt_draft.md`, and robust security/integrity enforcement modes. This release brings advanced orchestrator and worker agent capabilities, complete with Model Context Protocol (MCP) tool bridging and directory-based coordination heartbeats.

## Key Features

### 1. The `/teamwork-preview` Command and Two-Phase Workflow

The `/teamwork-preview` slash command enables a structured, two-phase process for task execution:

- **Phase 1: Elicitation**: The user or orchestrator drafts a task definition in `prompt_draft.md`. The framework parses this file to perform static requirements elicitation, validating that acceptance criteria and working directory settings are clearly defined.
- **Phase 2: Execution & Swarm Delegation**: Once the prompt is refined and approved by the user, the orchestrator invokes specialized subagents in a swarm, distributing tasks cleanly across directories.

### 2. Multi-Agent Swarm Coordination

Swarm operations are coordinated entirely through the filesystem inside the `.agents/` directory:

- **Folder Isolation**: Each agent owns a dedicated workspace folder (e.g. `.agents/worker_doc_writer/`) and can only write to its own folder while reading from any folder.
- **Liveness Heartbeats**: Agents periodically write to `progress.md` inside their folder, updating a `Last visited: [timestamp]` header to serve as a liveness heartbeat.
- **Context Handoff**: When a subagent completes its task or transitions work, it writes a `handoff.md` containing the five mandatory sections: Observation, Logic Chain, Caveats, Conclusion, and Verification Method.

### 3. Integrity Modes

To govern agent behavior and compliance, the framework introduces three distinct Integrity Modes:

- **Permissive**: Relaxed validation rules. Warnings are logged for test or style failures, but they do not block progress. Suitable for development and exploration.
- **Strict**: Strong gatekeeping. All changes must compile, all unit tests must pass, and style guidelines must have zero violations before agent work is accepted.
- **Cryptographic / Audit**: Maximum security and accountability. Every agent action, verification result, and state change produces a cryptographically signed receipt. These receipts are chained using BLAKE3 digests, providing a tamper-evident audit trail that independent auditors can verify.

### 4. MCP Agent Bridging

MCP (Model Context Protocol) bridging allows agents to seamlessly discover and execute external tools. By reading configurations from `mcp_config.json`, the orchestrator exposes host-level tools (such as database access, repository management, or specialized compilation engines) directly to the subagent swarm.

## Installation

To upgrade your project to v26.7.2:

```toml
[dev-dependencies]
chicago-tdd-tools = { version = "26.7.2", features = ["testing-extras", "ocel-generation", "receipt-validation"] }
```

## See Also

- [GitHub Release Notes](GITHUB_RELEASE_v26.7.2.md)
- [Project Architecture and Milestones](../../PROJECT.md)
- [Swarm Development Plan](../../SWARM_PLAN.md)
