# temprun v26.10.11 — Release Announcement (DRAFT)

> **Status: DRAFT** — PR1 audit (`~/.claude/plans/compressed-zooming-diffie.md`, section "Press-Release Claims Audit (PR1)") applied to body claims 2026-10-11; as-announced quotes untouched. Publication is an operator act.
> Date: 2026-10-11. Operator-issued release text, transcribed to the repo as the dated announcement artifact. Corrections land as follow-up edits after the audit; never by silent rewrite.

---

## Strata Releases `temprun`: The Ephemeral, Zero-Ambient-Authority Wasm Execution Leaf for Ash and the BEAM

**PASADENA, Calif. — October 11, 2026** — Strata today announced the general release of `temprun` (v26.10.11), an ephemeral, fail-closed computation leaf designed to eliminate ambient authority and unmetered execution across high-assurance distributed systems.

Positioned as the successor to the Docker- and gVisor-based prototype `clnrm`, `temprun` discards local container daemons, root-equivalent Unix sockets, and mutable container registries. Instead, workloads run as WebAssembly guests under the `temprun_step` host binary, reached from the BEAM node over a raw-stream Erlang Port with a hand-rolled 4-byte little-endian length prefix (the BEAM `:packet, 4` option is big-endian, so the prefix is framed in the worker) (a subprocess boundary, with no NIFs or linked-in drivers; a legacy `wasmtime`-CLI wrapper path also exists). Designed as the single-shot computation engine for Ash Reactor and the `ash_graphlaw` semantic membrane, `temprun` is built so that untrusted, transient workloads execute with strictly zero ambient capabilities, enforced reduction budgets, and inductive trajectory proofs.

## The Problem: The Ambient Authority Trap

When tasks execute inside standard container sandboxes, isolation is mediated by OS namespaces, cgroups, and seccomp filters. These environments leak host network stacks, shared memory segments, system clocks, and root-level Docker sockets (`/var/run/docker.sock`). Once inside, an attacker or runaway process can probe adjacent services, cause resource starvation, or establish persistence across tasks.

Furthermore, conventional execution engines evaluate task success using terminal string matches or exit codes (`exit 0`). They cannot prove whether intermediate security policies were skipped, whether execution paths diverged from specifications, or if unmetered loops held host threads hostage.

## The Solution: One-Shot Reduction Leaves

`temprun` completely decouples ecosystem orchestration from ephemeral execution. While **Ash Reactor** coordinates multi-step transaction DAGs, sagas, and semantic leases at the BEAM layer, `temprun` serves strictly as the one-shot, leaf-level execution unit:

- **Zero Ambient Authority (`imports == ∅`):** The `temprun` guest kernel imports no operating system syscalls. It possesses no host filesystem handles, no raw network sockets, and no access to `SystemTime`. External dependencies are not pulled over the network; on the roadmap, they will be synthesized directly into the guest via `ggen` from Turtle RDF ontologies (not yet landed).
- **Preemptive Reduction Accounting:** To prevent thread starvation and algorithmic complexity attacks, tasks execute within an explicit reduction budget (e.g., 2,000 steps). The execution loop yields control cooperatively to the BEAM process scheduler, preserving VM-wide responsiveness.
- **In-Memory Synthetic VFS & Actors (roadmap):** The design maps file operations to copy-on-write memory slices and ETS binary shards, and network requests to local BEAM process identifiers (`pid()`) via typed message passing. This synthetic-actor layer is on the roadmap; it is not part of the landed v26.10.11 surface.
- **Source Admission over Git Merkle Trees:** `temprun` rejects pre-compiled, opaque binary blobs. Landed today: Merkle-chained receipts and the N19 reproducible-build receipt, with clean-room compilation under `--locked` and `--offline` constraints (NATO STANAG 4107 Pillar V). Signed-Git source admission is on the roadmap, not yet an admission gate.
- **Inductive Trajectory Certification:** Powered by `chicago-tdd-tools` (CTT), every transition inside `temprun` is recorded as an IEEE OCEL 2.0 causal event graph and verified against normative Petri net models using branchless `pn64` bitpack lattices. Completed runs output an unforgeable, post-quantum signed audit receipt (ML-DSA-65 / FIPS 204).

## Executive & Engineering Quotes

> "With `clnrm`, we proved that developers wanted hermetic integration runs, but relying on Docker and gVisor was an architectural compromise that kept ambient host authority alive," said **Sean Chatman**, Principal System Architect at Strata. "With `temprun`, we have completely removed the daemon and the OS boundary. Workloads run inside an empty-import Wasm kernel that executes its assigned reduction slice and instantly ceases to exist. There is no filesystem to persist to, no network to scan, and no ambient authority to abuse."

> "By pairing `temprun` with `ash_graphlaw` and Ash Reactor, the entire execution lifecycle is algebraically closed," said an early pilot evaluator. "Adversaries expecting an interactive shell or a lingering container find only a mathematical graph. The moment an unexpected transition occurs, the entire state machine fails closed."

## Customer & Developer Experience

```elixir
# 1. Drive the landed surface: the Stratus.Temprun.Worker gen_statem
#    (init -> stepping -> yielding -> sealed) over the temprun_step port bridge
{:ok, worker} = Stratus.Temprun.Worker.start_link([])
:ok = Stratus.Temprun.Worker.step(worker)
:ok = Stratus.Temprun.Worker.seal(worker)

# 2. Results return with cryptographic trajectory evidence
# If the Wasm guest attempted an unauthorized branch, skipped validation,
# or exceeded its 2,000-step reduction quota, the state machine fails closed.
# (A `Temprun.run_one_shot/1` convenience facade over Worker is on the roadmap.)
```

## Availability

`temprun` v26.10.11 is available across the Strata ecosystem. It integrates natively with `chicago-tdd-tools` (v26.10.11) via the `conformance` and `nato` feature gates; Hex.pm publication of `temprun` and `ash_graphlaw` availability on Hex are planned, not yet confirmed.

- **Repository:** https://github.com/seanchatmangpt/temprun
- **Documentation:** https://hexdocs.pm/temprun (live after Hex publication — currently PLANNED per the claim ledger below)
- **Assurance Dossier:** `docs/nato/DOSSIER.md` + `DOSSIER.sha256` + `DOSSIER.sha256.sig` (per-court JSON receipts emitted under `docs/receipts/N01..N20.json` (N19 reproducible-build identity + N20 dossier-generator receipts landed post-train; N19 narrative record: `docs/nato/N19_BUILD_RECEIPT.md`); a consolidated `NATO_STANAG_4107_DOSSIER.json` projection is on the roadmap)

## Media Contact

Strata Systems Architecture Team — Pasadena, California
press@strata.systems | https://strata.systems

---

## Claim Status Ledger (do not publish)

PR1-audited (2026-10-11); body wording above corrected to landed truth. Per-claim status after correction:

| Claim | Status | Evidence |
| --- | --- | --- |
| `imports == ∅` guest kernel | BACKED | `src/conformance/guest.rs` (zero std ambient imports); `src/nato/guest_audit.rs` zero-import court |
| 2,000-step reduction budget, cooperative yield | BACKED | `src/conformance/guest_yield.rs` (`DEFAULT_REDUCTION_QUOTA = 2000`, yield protocol; BEAM owns termination) |
| OCEL 2.0 trajectory + branchless pn64 verification | BACKED | `src/observability/ocel/ocel64_bridge.rs`, `src/conformance/pn64.rs` + `session.rs` |
| ML-DSA-65 / FIPS 204 receipts | BACKED (host-side seal) | `src/ledger/signing/` (guest produces, host seals) |
| Execution story: `temprun_step` wasmi host bin via raw-stream BEAM Port (hand-rolled 4-byte LE length prefix) + wasmtime-CLI legacy path | BACKED (as reworded) | stratus worker.ex:13-15; temprun/src/bin/temprun_step.rs; default_arena.ex:158-171 |
| `Temprun.run_one_shot/1` facade | ROADMAP (marked inline) | Landed surface is `Stratus.Temprun.Worker` (gen_statem) + `temprun_step` port bridge |
| In-memory synthetic VFS & ggen actors | ROADMAP (marked inline) | No implementation in either repo |
| Signed-Git source admission | ROADMAP (marked inline); Merkle-chained receipts + N19 reproducible-build receipt BACKED | strata/src/cas.rs, docs/nato/N19_BUILD_RECEIPT.md; no signature verification landed |
| Assurance dossier pointer | CORRECTED | Points to `DOSSIER.md` + `.sig`; per-court JSON at `docs/receipts/N01..N20.json` (N19/N20 landed post-train); consolidated JSON projection on roadmap |
| Hex availability (temprun, ash_graphlaw) | PLANNED (marked inline) | No publish receipts found |
| temprun v26.10.11 released | PARTIAL | strata VERSION is 26.10.10; CHANGELOG wave-2 at 26.10.11; not published |
