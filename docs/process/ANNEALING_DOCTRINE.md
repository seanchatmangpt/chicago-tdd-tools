# Annealing Doctrine — Design Rationale

Status: design rationale (derivation, not law). The compressed law lives at
`~/.claude/rules/entropic-annealing-doctrine.md` (loaded every session); repo
clauses live in each consumer repo's `CLAUDE.md`. This document carries the
full derivation the law file compresses.

## 1. The failure model

Agent swarms drift by entropy cascade, not single-point error. Three channels:

### Social hallucination reinforcement
An asserted claim becomes context. Downstream lanes cite the context, not the
referent — second-generation claims inherit first-generation confidence
without its evidence. A fleet's claim count grows at generation rate; its
verified-claim count grows at execution rate. N lanes agreeing is not N
observations; it is one observation echoed N times unless each lane executed
against the referent independently.

### Mock-laundering
A fake collaborator converts "the test ran" into "the system works". The mock
passes precisely because it was built to agree — London-style mocking launders
an unverified design into a green checkmark. The Chicago-school mock ban
(`docs/process/CODE_REVIEW_CHECKLIST.md`, testing discipline) is the mechanical
response; this section is its rationale.

### Tautological tests
A test that asserts what it just constructed carries zero bits about the
subject. Example witnessed this session: `property_all_data_valid` whose
generator could never produce an empty map — the `is_empty()` branch was dead
and the property could not fail. The fix pattern: give the predicate a
validator closure so a failing validator is expressible (non-vacuity), or
delete the test.

## 2. ERRC asymmetry

Reduce > Raise > Create. A deletion (dead feature flag, retired embedded-key
signature scheme, fabricated doc cite replaced) is a permanent entropy sink —
it cannot regress. A generation is a future audit liability until receipted.
This inverts the default dispatch order of most agent fleets, which treat
generation as the primary work supply. Capacity freed by wave completion goes
to kill/prune/tighten lanes FIRST, repairs second, generation last.

## 3. Mechanical annealing

Single-shot verification leaves lattice defects: latent races, flaky bounds,
stale claims, doc drift. A defect only surfaces if some lane observes the
faulting state; concurrent lanes over the same tree provide state diversity
(allocation ordering, thread interleaving, mid-edit snapshots) that no
single-shot run can. Witnessed this session: a flaky guest-suite failure that
survived two "fixes" (bound-capture ordering) and only yielded to the real
fix (mutex serialization of the shared static) after sustained re-runs by
independent lanes.

Saturation law: keep ~20 concurrent Audit/Mutate/Repair/Verify lanes running
until a full wave returns zero defects. Early termination preserves the
defects you have not yet observed.

## 4. TRIZ bindings

| Contradiction | Principle | Mechanism |
|---|---|---|
| N-lane speed ∥ one-truth | 17 Space division | Lane leases, coordinator-owned git |
| Thoroughness ∥ no false confidence | 3 Local quality | Skeptic-pairing (audit lanes never grade their own builders; process discipline, no hook) |
| Coverage ∥ no vacuous pass | 5 Merging | Concurrent Audit/Mutate/Repair/Verify on one tree |
| Entropy removal ∥ growth | 1 Segmentation | ERRC asymmetry dispatch order |

## 5. Enforcement boundary

The rules layer shapes dispatch and verification discipline. Mechanical
enforcement rides existing mechanisms: pre-commit unwrap/expect gate,
topology guard, NATO positive-control courts, receipt validation, vacuity
sweeps, trybuild compile-fail suite. Pair every law with its mechanism; an
unpaired law is prose, an unpaired mechanism is archaeology.
