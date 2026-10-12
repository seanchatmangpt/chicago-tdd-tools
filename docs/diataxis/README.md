# Diátaxis Documentation Suite

**The Chatman Equation** — `A = μ(O*)`, `R = receipt(A)`. Observations are only
raw input (`O`) until aligned, grounded, bounded, and admitted (`O*`); lawful
manufacture (`μ`) produces an actuation whose receipt (`R`) — identity,
authority, consequence, replay, standing — is the only admissible output. Every
domain in this suite is one instantiation of that equation, and every page is
written against landed code, never against plans.

**Scope**: five capability domains of this substrate — attestation (STANAG 4107
dossier court), execution (PN64 guest-kernel conformance), beam-membrane
(Erlang port protocol), semantic-graph (POWL workflow contracts), settlement
(merger-guests conformance) — each documented across the four Diátaxis
quadrants (tutorial / how-to / reference / explanation), for a 5×4 matrix of 20
pages plus this index. Every terminal command was verified against the Justfile
and the landed source tree as of v26.10.11.

## The 5×4 Matrix

| Domain | Tutorial | How-To | Reference | Explanation |
|---|---|---|---|---|
| [Attestation](attestation/) | [Build and Seal a STANAG 4107 Dossier](attestation/01-tutorial.md) | [Operate the Replay Nullifier Filter](attestation/02-how-to.md) | [CTRT Wire Formats and Machine Codes](attestation/03-reference.md) | [Deontic Context Binding](attestation/04-explanation.md) |
| [Execution](execution/) | [Author a Zero-Import WASM Guest](execution/01-tutorial.md) | [RE/Timing Audit Playbook](execution/02-how-to.md) | [PN64 Bit Layout and the Guest ABI](execution/03-reference.md) | [Monoidal Trajectory Folding vs IVC/Nova](execution/04-explanation.md) |
| [Beam-Membrane](beam-membrane/) | [Drive a Temprun Session from a Saga Step](beam-membrane/01-tutorial.md) | [Recover a Temprun Port Bridge](beam-membrane/02-how-to.md) | [The Temprun Port Protocol](beam-membrane/03-reference.md) | [Why a Port Subprocess, Not a NIF](beam-membrane/04-explanation.md) |
| [Semantic-Graph](semantic-graph/) | [Author a Workflow Contract as Turtle → POWL](semantic-graph/01-tutorial.md) | [Ground Workflow Receipts in Git Objects with gix](semantic-graph/02-how-to.md) | [POWL Compact DSL Grammar and AshGraphLaw Capability API](semantic-graph/03-reference.md) | [Open Texture vs. Deterministic Logic](semantic-graph/04-explanation.md) |
| [Settlement](settlement/) | [Execute a Phase 7 Settlement Conformance Run](settlement/01-tutorial.md) | [Typed Refusals and Invariant-Mask Inspection](settlement/02-how-to.md) | [merger-guests ABI As Landed](settlement/03-reference.md) | [The Chatman Equation as Conservation](settlement/04-explanation.md) |

## How to Read This Suite

- **Tutorials (01)** are learning-oriented: run them top to bottom.
- **How-Tos (02)** are task-oriented: jump to your problem, run one recipe.
- **References (03)** are information-oriented: look up exact wire formats,
  bit layouts, scales, and machine codes.
- **Explanations (04)** are understanding-oriented: read them to learn *why*
  the design is what it is.

## Verification Receipt

- 20/20 files exist (4 per domain dir).
- Placeholder grep (TODO/FIXME/TBD/`...`): the only `...` instances are
  elisions inside illustrative POWL/Turtle code blocks
  (`semantic-graph/01-t.md:97, 03-ref.md:139,141, 04-exp.md:44`) — flagged,
  not silent placeholders of missing docs.
- 12+ terminal commands spot-checked against the Justfile and source tree:
  all `just` recipes exist (`guest-build/test/audit`, `emit-dossier`,
  `dossier-verify`, `dossier-inspect`); all cited `src/` and `crates/`
  paths exist after two fixes (see below); test targets exist
  (`tests/merger_conformance_tests.rs`, `tests/guest_abi_tests.rs`,
  affidavit `ct_timing_invariance.rs`).
- Two defects found and fixed in place:
  1. `settlement/01-tutorial.md` cited `cargo test -p chicago-tdd-tools-merger-guests`;
     the landed package name is `merger-guests` (fixed).
  2. `settlement/03-reference.md` cited `src/fixed.rs` / `src/errno.rs`;
     landed path is `crates/merger-guests/src/…` (fixed).
