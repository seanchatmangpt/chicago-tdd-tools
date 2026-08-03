# Epic: Chicago TDD Tools as F_local Orchestrator for MFW

## Metadata
* **Type:** Epic
* **Subproject:** chicago-tdd-tools
* **Coordinate:** v26.7.18
* **Standing:** `DRAFT`

## Missing Consequence (The Gap)
Currently, MFW lacks a deterministic orchestrator for F_local verification paths. The local testing workflow is ad-hoc, preventing the rigorous, repeatable assertion of F_local standing. Without Chicago orchestrating these checks, the gap between local verification and the formal v26.7.18 consequence horizon cannot be reliably bridged.

## Evidence Requirement
A unified JSON or RDF receipt containing the F_local validation summary, demonstrating successful orchestration and execution of MFW's local bounded checks by Chicago TDD tools.

## Bounded Construction
Integrate `chicago-tdd-tools` into the `mfw` local development workflow. Standardize the CLI invocation paths for F_local verification. Ensure Chicago orchestrates the execution of unit tests, integration tests, and mutation checks locally, generating a standard F_local receipt that MFW can validate.

## Refusal/Negative Fixture
The orchestrator must predictably abort execution and issue a typed refusal if any local bounded check (e.g., test failure, degraded mutation score) fails.
