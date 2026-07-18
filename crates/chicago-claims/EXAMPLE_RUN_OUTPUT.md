# Example run output

This is a real, captured run of `chicago-claims-verify` against the checked-in
reference claim, not a hand-written illustration. Provenance: captured during
integration of `chicago-claims` (no timestamp recorded; reproduce with the exact
command below against the current `main` tree of both `chicago-tdd-tools` and
`/Users/sac/bcinr`).

## Command

```
cd /Users/sac/chicago-tdd-tools
cargo run -p chicago-claims --bin chicago-claims-verify -- crates/chicago-claims/claims/cmca-fault-union.toml
```

Exit code: `0` (matches `Standing: Alive`; a `Blocked` standing exits nonzero).

## Captured stdout

```
Claim: cmca-numeric-fault-join-semilattice
Standing: Alive

Scan evidence: (syntax evidence only)
  - type `NumericFaultSet` found: true
  - field `0` observed private: Some(true)
  - method `union` found: true
  - method `is_empty` found: true
  - method `bits` found: true

Mutant evidence:
  - first-wins: KilledByIntendedOracle
  - last-wins: KilledByIntendedOracle
  - left-only: KilledByIntendedOracle
  - right-only: KilledByIntendedOracle
  - empty-set: KilledByIntendedOracle
  - overwrite: KilledByIntendedOracle

Delta:
  - intent -> implementation: none
  - implementation -> evidence: none

This report does NOT claim: object-code branchlessness of the scanned implementation (source-level AST evidence only, no disassembly performed); universal unforgeability or semantic correctness of any method body (syntax presence only, no proof of behavior); absence of runtime allocation or any other runtime property (no execution trace was collected). See FAQ #4, #13, #14 for the full evidence-scope caveats this report inherits.
```

## What this run demonstrates

- The `syn`-based scanner (`src/scan.rs`) parsed the real, unmodified
  `/Users/sac/bcinr/crates/bcinr-cmca/src/fixed.rs` and observed `NumericFaultSet`
  as a struct with a private tuple field and the three required methods
  (`union`, `is_empty`, `bits`).
- All six named mutant fixtures (`tests/fixtures/{first_wins,last_wins,left_only,
  right_only,empty_set,overwrite}.rs`) were activated in-process and their
  intended oracle test correctly detected each corruption
  (`MutantClassification::KilledByIntendedOracle`).
- `reconcile()` combined the clean structural scan and the all-killed mutant set
  into `Standing::Alive` with no deltas.
- `render_report()` printed the report shown above, including its own
  evidence-scope disclosure (this is a documented feature of the report format,
  not an ad hoc caveat added to this file).

See `README.md` for what this MLP proves and, more importantly, what it does
NOT prove.
