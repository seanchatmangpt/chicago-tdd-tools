# Product Requirements Document (PRD)
## F_local Orchestrator for MFW (v26.7.18)

### 1. Coordinate and Standing
* **Type:** Product Requirements Document (PRD)
* **Epic:** `mfw-f-local-orchestrator`
* **Coordinate:** v26.7.18
* **Standing:** `DRAFT`

### 2. Missing Consequence (The Gap)
MFW currently lacks a deterministic orchestrator for `F_local` verification paths. The existing local testing workflow relies on ad-hoc invocations, which prevents the rigorous, repeatable assertion of `F_local` standing. Without a unified deterministic entry point (Chicago TDD Tools), the gap between local verification and the formal v26.7.18 consequence horizon cannot be reliably bridged.

### 3. Core Objectives
1. **Deterministic Orchestration:** `chicago-tdd-tools` must orchestrate the execution of unit tests, integration tests, and mutation checks locally.
2. **Unified Receipt Generation:** The system must produce a standard, typed JSON or RDF receipt containing the `F_local` validation summary that MFW can cryptographically validate.
3. **Exact Algebraic Falsifiers:** The orchestrator must predictably abort execution and issue a typed refusal receipt if any local bounded check fails or if mutation scores degrade.

### 4. Requirements Specification
#### 4.1. Orchestration Pipeline
* The product must expose a single CLI entry point designed to be invoked by the MFW `Justfile`.
* It must support reading a pipeline profile that specifies the exact sequence of bounded verification checks (linting, tests, mutations).
* Execution of the profile must be deterministic, strictly mapping CLI execution states to verification outcomes without silent failures.

#### 4.2. Output and Evidence Requirements
* **Positive Consequence:** Upon full passage of the pipeline, an `F_local` JSON/RDF receipt must be generated containing exact metadata (timestamp, hashes, checks performed, combinatorial maximalism).
* **Zero-Loss Information:** The output must precisely record the verification inputs and their runtime correspondence.

#### 4.3. Refusals and Error Handling
* Any failure in a bounded check must result in an immediate abortion of the pipeline.
* The system must issue a typed refusal receipt explicitly mapping the failure to a negative fixture or falsifier.
* Partial completions must not be conflated with full `F_local` standing.
