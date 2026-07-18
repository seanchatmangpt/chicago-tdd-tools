# chicago-claims

`chicago-claims` is a Chicago TDD claim-reconciliation MLP (minimum lovable
product): one complete loop for one reference law.

## What this proves

For exactly one hardcoded reference claim — the CMCA numeric fault
join-semilattice invariant `faults_out = faults_left UNION faults_right UNION
faults_local` (empty set = zero), as implemented by `NumericFaultSet` in
`bcinr-cmca/src/fixed.rs` — this crate demonstrates a real, running loop:

1. **TOML claim schema** (`src/claim.rs`): a claim names a scope (file +
   symbol), a required structure (type name, field privacy, required method
   names), a list of named mutant variants, and required evidence artifacts.
2. **`syn`-based AST scanner** (`src/scan.rs`): parses the real, unmodified
   target source file and reports syntax-level structural observations —
   whether the claimed type, field privacy, and methods are present. This is
   evidence, not a verdict: it does not resolve macros, does not check method
   bodies, and does not prove runtime behavior.
3. **Mutant activation + oracle attribution** (`src/mutate.rs`): a
   `MutationProvider` trait (`activate` + `run_oracle`, with a shared default
   `classify` that reuses `reconcile::MutantClassification`) is implemented by
   two concrete providers, dispatched per-mutant by `classify_mutant` based on
   which `MutantSpec` fields a claim's TOML populates:
   - **`FixtureProvider`**: the original mechanism — six named corrupted
     variants of the `union` operation (`first-wins`, `last-wins`, `left-only`,
     `right-only`, `empty-set`, `overwrite`) compiled in-process from
     self-contained fixture files (`tests/fixtures/*.rs`) and evaluated against
     their intended oracle predicate.
   - **`CargoFeatureProvider`**: drives a REAL `cargo test -p <crate> --features
     <feature> --test <test_binary> <test_name> -- --exact` subprocess against
     an external crate on disk and classifies from its real, parsed pass/fail —
     no fixture file or new match arm required per mutant. Proven working
     end-to-end against a real external mutant (`bcinr-cmca`'s `mutant_1` cfg
     feature) by `mutate::tests::
     cargo_feature_provider_kills_real_bcinr_cmca_mutant_1_via_real_subprocess`
     (an `#[ignore]`d real-subprocess test; run with `cargo test -p
     chicago-claims -- --ignored`). See "Generalizing mutant execution" below.
4. **Reconciliation into typed standing** (`src/reconcile.rs`): combines the
   scan result and mutant classifications into one of `Alive`,
   `PartialAlive(..)`, `Blocked(..)`, or `Unknown`.
5. **A human-readable report** (`src/report.rs`) and a **CLI**
   (`src/bin/chicago-claims-verify.rs`) that runs the whole loop against a
   claim TOML file and prints the report, exiting nonzero if the standing is
   `Blocked`.

A real, captured run against the actual `bcinr-cmca` source is recorded in
`EXAMPLE_RUN_OUTPUT.md` (not hand-written) — its captured result was
`Standing: Alive` with all six named mutants `KilledByIntendedOracle`.

## What this does NOT prove

This MLP is deliberately narrow. It does not prove, and does not claim to
prove:

- **Universal unforgeability.** Only this one hardcoded claim (one type, one
  file, six named mutants) is checked. There is no general claim language,
  claim discovery, or coverage guarantee over arbitrary code.
- **Object-code proof.** The scanner is source-level `syn` AST parsing only.
  No disassembly is performed; nothing here proves the compiled binary is
  branchless, or that source-level structure survives to the object file. See
  `bcinr`'s own `object-code-audit` process for that separate evidence class.
- **Allocation proof.** No execution trace, profiling, or memory
  instrumentation is collected. Nothing here proves absence of runtime
  allocation or any other runtime property.
- **General-purpose mutation testing.** Neither provider is a generic
  mutation-testing engine. `FixtureProvider` evaluates six specific,
  pre-written, self-contained fixture files against one hardcoded oracle
  predicate; adding a new fixture-shaped mutant still requires a new fixture
  file (though no longer a new match arm in `mutate.rs` — see below).
  `CargoFeatureProvider` can drive any REAL, already-existing `cfg(feature =
  "...")` mutant an external crate has hand-written and compiled in, from a
  TOML entry alone — but it does not GENERATE mutants. It cannot invent a
  corrupted variant of code that has no corresponding cfg-gated mutant already
  written and compiled into the target crate. See "Explicitly out of scope"
  below for the mutation-testing approaches this crate still does not
  implement.

This list mirrors the working-backwards FAQ's fenced disclosure list for this
MLP; the CLI's own printed report repeats the object-code/semantic/allocation
disclosures inline on every run.

## Generalizing mutant execution: the `MutationProvider` trait

The original MLP's `src/mutate.rs` had no data-driven path from a claim TOML's
`[[mutants]]` table to actual mutant execution for anything beyond the six
hardcoded pilot fixtures — `evaluate_fixture` was a hardcoded `match` over six
`include!`-ed fixture modules. Two claims added after the pilot
(`cmca-observatory-proposal-only.toml`, `cmca-rejection-invariance.toml`) could
therefore only ever produce structural-scan evidence; both printed `(no mutant
results recorded)` because there was no way to wire a real mutant to either
one without hand-writing new fixture files and match arms.

This is now closed on the mechanism side by a `MutationProvider` trait:

```rust
pub trait MutationProvider {
    fn activate(&self, spec: &MutantSpec) -> ActivationWitness;
    fn run_oracle(&self, spec: &MutantSpec) -> OracleResult;
    // classify(..) has a shared default impl: activate, then run_oracle, then
    // map the outcome into reconcile::MutantClassification (reused, not
    // redefined).
}
```

Two concrete providers implement it:

- **`FixtureProvider`** wraps the original `include!`-based mechanism, unchanged
  in behavior — `activate_and_test_mutant` is now a thin wrapper over
  `FixtureProvider.classify(..)`, proven byte-identical to its pre-refactor
  self by `mutate::tests::fixture_provider_refactor_matches_pre_refactor_behavior`
  and by a real before/after CLI diff against `cmca-fault-union.toml`.
- **`CargoFeatureProvider`** is the genuinely new capability: given a claim
  TOML's `crate_path` + `feature` + `test_name` (+ optional `test_binary`), it
  runs a real `cargo test -p <crate> --features <feature> --test <test_binary>
  <test_name> -- --exact` subprocess against an external crate and classifies
  from the real, parsed pass/fail — proven end-to-end against `bcinr-cmca`'s
  real `mutant_1` feature (see
  `mutate::tests::cargo_feature_provider_kills_real_bcinr_cmca_mutant_1_via_real_subprocess`,
  `#[ignore]`d by default; run with `cargo test -p chicago-claims --
  --ignored`).

`MutantSpec` (`src/claim.rs`) grew two optional shapes rather than a tagged
enum, matching the existing `RequiredStructure::forbidden_constructions`
`#[serde(default)]` precedent: `fixture_path` (`FixtureProvider`) or
`crate_path`/`feature`/`test_name`/`test_binary` (`CargoFeatureProvider`).
`classify_mutant` dispatches on which shape is populated and reports
`InfrastructureBlocked` for a spec matching neither or both shapes.

**Both two-of-two generalization claims still carry zero mutants**, and that
remains the honest finding for each claim's own law, not a mechanism gap: all
11 of `bcinr-cmca`'s real `mutant_1..mutant_11` cfg features were individually
read at their exact source locations and checked against each claim's specific
law (see the detailed per-mutant citations inside each claim TOML's own
comments). Neither "the Observatory never mints a `CertificateReceipt`"
(`cmca-observatory-proposal-only.toml`, a structural absence property) nor
`RefusalSet`'s union-preservation law (`cmca-rejection-invariance.toml`) has a
corresponding cfg-gated mutant in the current `bcinr-cmca` tree — the closest
candidates (`mutant_9`/`10`/`11` for the first; `mutant_1`-`5` for the second)
corrupt adjacent-but-distinct invariants in the same modules. Wiring one of
them in anyway would have been a mismatched claim, not a real generalization
proof. The mechanism itself is proven working against a real, genuinely
applicable mutant elsewhere (`mutant_1`, see above) — it is simply not (yet)
wired to either of these two specific claims, because no matching mutant
exists for either law today.

## Explicitly out of scope (this round)

The following mutation-testing approaches are NOT implemented by either
provider, named here so their absence is not silently assumed closed:

- **Patch-overlay mutants** — applying an arbitrary source-level patch/diff to
  a target file before compiling, rather than requiring the mutant to already
  exist as a hand-written `cfg(feature = "...")` block compiled into the
  target crate. `CargoFeatureProvider` can only drive mutants the target crate
  already declares and compiles in; it cannot construct a new corrupted
  variant of arbitrary source on the fly.
- **`cargo-mutants` integration** — no adapter to the `cargo-mutants` tool (or
  any other automatic mutant-generation engine) exists. Neither provider
  generates mutants; both only activate and classify mutants that already
  exist (a fixture file, or a compiled-in cfg feature).
- **MIR-level mutation** — no compiler-level (MIR/HIR/LLVM-IR) mutation
  mechanism exists. Both providers operate at the source/feature-flag level;
  neither instruments or rewrites the compiler's intermediate representation.

This mirrors the "product-vs-pilot" honesty already established for the
scanner and the original fixture mechanism: naming what is not built is part
of the evidence, not an afterthought.

## How to run it

```bash
cd /Users/sac/chicago-tdd-tools
cargo run -p chicago-claims --bin chicago-claims-verify -- \
    crates/chicago-claims/claims/cmca-fault-union.toml
```

Exit code is `0` for `Alive`/`PartialAlive`/`Unknown` standing, nonzero for
`Blocked`.

Run the test suite:

```bash
cargo test -p chicago-claims
```

Two grounding tests are `#[ignore]`d by default because they require the
`bcinr` checkout to exist on disk at a hardcoded absolute path — run both
explicitly with `cargo test -p chicago-claims -- --ignored`:

- `scan::tests::real_bcinr_fixed_rs_has_numeric_fault_set_correctly_structured`
- `mutate::tests::cargo_feature_provider_kills_real_bcinr_cmca_mutant_1_via_real_subprocess`
  (also spawns a real, several-second `cargo test` subprocess against the
  `bcinr-cmca` sibling checkout)

## Known limitation: cross-repository claim path

`claims/cmca-fault-union.toml`'s `scope.file` is an absolute,
cross-repository path (`/Users/sac/bcinr/crates/bcinr-cmca/src/fixed.rs`) into
a sibling checkout on this machine. This is expected and accepted for this
MLP slice: `chicago-claims` has no configurable claim root or vendoring
mechanism today, so the claim file hardcodes an absolute path rather than a
path relative to some claim-root config that does not yet exist. A real
product would resolve `scope.file` against a per-workspace configured root (or
require the referenced source to be vendored/pinned inside the claiming repo)
instead of hardcoding an absolute path to another repository's checkout
location.
