# Task: Integrate Chicago CLI into MFW Justfile for F_local

## Metadata
* **Type:** Task
* **Subproject:** chicago-tdd-tools
* **Coordinate:** v26.7.18
* **Standing:** `DRAFT`

## Missing Consequence (The Gap)
The MFW `Justfile` and local scripts do not currently invoke `chicago-tdd-tools` for F_local verification. This manual approach circumvents formal tracking, preventing the automated generation of an F_local receipt required for the v26.7.18 horizon.

## Evidence Requirement
A successful run of the MFW `Justfile` local verification targets that deterministically executes through the `chicago-tdd-tools` CLI, producing the expected F_local JSON/RDF receipt.

## Bounded Construction
- Configure the MFW `Justfile` to use `chicago-tdd-tools` for local verification targets (e.g., test, lint, mutate).
- Define an F_local pipeline profile specifying the verification sequence.
- Wire the output format of Chicago CLI to emit the receipt format required by the MFW validation graph.

## Refusal/Negative Fixture
If a test or mutation script in the `Justfile` path fails, the `chicago-tdd-tools` CLI must correctly capture the failure and immediately exit with a typed refusal receipt, rather than continuing or emitting a success receipt.
