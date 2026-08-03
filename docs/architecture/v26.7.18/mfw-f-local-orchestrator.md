# Architecture Requirements Document (ARD)
## F_local Orchestrator for MFW (v26.7.18)

### 1. Coordinate and Standing
* **Type:** Architecture Requirements Document (ARD)
* **Epic:** `mfw-f-local-orchestrator`
* **Coordinate:** v26.7.18
* **Standing:** `DRAFT`

### 2. Architectural Vision
The `chicago-tdd-tools` CLI acts as the deterministic execution substrate (the broker) for the `F_local` admission and verification boundary within the MFW monorepo. It enforces topological invariants across local testing workflows, ensuring that the transition from admitted observation to receipted actuation is preserved with zero-loss information.

### 3. Component Architecture
#### 3.1. Justfile Integration Boundary
* **Trigger:** The MFW `Justfile` acts as the invocation surface, delegating all `F_local` targets directly to `chicago-tdd-tools`.
* **Profile Engine:** A deterministic profile configuration engine must parse the `F_local` verification sequence. Topological invariants guarantee that dependencies among checks (e.g., compile before mutate) are preserved.

#### 3.2. Execution Engine (Combinatorial Maximalism)
* The engine must execute unit tests, integration tests, and mutation checks without external state mutation outside of the `F_local` boundary.
* Each check operates as a bounded construction, enforcing rigorous type constraints over string-based generic logs.

#### 3.3. Receipt and State Machinery
* **Receipt Manifestation:** The final architecture must serialize verification states into JSON/RDF formats strictly mapping to MFW ontology definitions.
* **Algebraic Falsifiers:** Every failure mode must map to an exact algebraic falsifier (typed refusal). The state machine transitions to `BLOCKED` or `UNKNOWN` must be explicit and cryptographically tied to the failing node.

### 4. Interface and Data Contracts
#### 4.1. Inputs
* Subproject configuration paths and dependency trees.
* `F_local` profile (JSON/YAML) dictating the DAG of verification jobs.

#### 4.2. Outputs
* **Success Receipt:** Typed JSON/RDF containing the validation graph and execution hashes.
* **Refusal Receipt:** Typed negative consequence explaining the specific verification step that failed.

### 5. Standing Law Adherence
* **Canonical Boundary:** The `chicago-tdd-tools` CLI does not grant formal proof authority; it merely orchestrates runtime execution and structures the receipt. True standing is granted via the verifier consuming the receipt.
* **Separation of Authority:** `chicago-tdd-tools` acts strictly within the runtime/actuation boundary, producing evidence (receipts) to be assessed by the formal theory boundary.
