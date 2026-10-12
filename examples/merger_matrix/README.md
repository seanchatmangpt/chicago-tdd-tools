# merger_matrix — Legal-Reasoning Fixture Example

A self-contained semantic fixture set modeling a stock-merger scenario
(`ex:AcmeCorp` acquiring `ex:BetaHoldings` via `ex:MergerSub`) across three
layers: SHACL structural shapes, N3 deontic-lite rules, and LKIF deontic
roles. These fixtures exist to be **validated and reasoned over**, not to be
consumed as legal advice.

## Tree

```
merger_matrix/
├── README.md                          (this file)
├── shapes/                            closed-world SHACL structural shapes
│   ├── merger-core.shacl.ttl          deal entity topology + closed class
│   ├── deal-terms.shacl.ttl           exchange ratio, price, quorum terms
│   ├── party.shacl.ttl                party roles (Target/Acquirer/Sub)
│   └── materiality.shacl.ttl          material-adverse-effect thresholds
├── rules/                             N3 rules (cwm/eye style)
│   ├── vote-threshold.n3              66.67% supermajority vote rule
│   ├── hhi-bands.n3                   DOJ/FTC 2023 Merger Guidelines HHI bands
│   └── netting.n3                     mutual (two-way) netting rule
└── deontic/                           LKIF-core deontic roles
    ├── merger-obligations.ttl         Obligation / Permission assignments
    ├── closing-conditions.ttl         precedent conditions to Closing
    └── breach.ttl                     Violation / Remedy roles
```

Additional fixture layers on disk (not shown in the tree above):

- `models/` — 11 POWL normative-net models (`phase1..phase10` + `master.powl`),
  consumed by `ConformanceSession::from_powl_file`.
- `conformance/` — 19 OCEL 2.0 trace fixtures (per-phase goldens +
  `*_violation` negatives + `master_trace.ocel.json`), replayed through the
  pn64 bridge by `tests/merger_conformance_tests.rs`.
- Domain data: `analytics/`, `capital/`, `collateral/`, `contracts/`,
  `data/`, `loans/`, `settlement/`, `tax/` (TTL instance data, 2 files each).

## Layer responsibilities

| Layer   | Answers                                        | File format |
|---------|------------------------------------------------|-------------|
| shapes/ | Is the data structurally lawful?               | SHACL over Turtle |
| rules/  | What follows from the numbers?                 | N3 |
| deontic/| Who must, may, or violated what?               | LKIF-core vocabulary |

## Vocabulary citations (prefix-only, no `owl:imports`)

| Prefix | Vocabulary | Cited via |
|--------|-----------|-----------|
| `fibo-fbc` | FIBO Foundations-Business and Companies | `rdfs:seeAlso` |
| `fibo-fnd` | FIBO Foundations | `rdfs:seeAlso` |
| `lkif` | LKIF-core legal ontology | `rdfs:seeAlso` |
| `sh` | W3C SHACL | namespace declaration |
| `log`, `math`, `string` | N3 builtins (cwm/EYE) | namespace declaration |

All external vocabularies are **referenced by prefix and `rdfs:seeAlso`
only**. No `owl:imports` is used: each file is standalone-valid Turtle/N3 and
re-validation never requires network resolution.

## Regeneration commands

These fixtures are the **hand-authored source of truth**. Generators consume
them; they are not themselves generated.

| Command | Regenerates | Notes |
|---------|-------------|-------|
| `ggen sync run --manifest ggen.toml` | (future) typed accessors from shapes/ | not yet wired |
| `eye rules/*.n3 --nope --pass` | rule inference output | EYE reasoner |
| `pyshacl -s shapes/*.shacl.ttl -i rdfs data.ttl` | SHACL validation report | any conformant validator |

## GENERATED law

```
## GENERATED files carry headers; do not edit.
```

Any artifact in this tree bearing a `## GENERATED` header is a projection
(e.g. rendered accessors, sealed dossiers). **Do not edit GENERATED files by
hand.** Change the source (shapes/, rules/, deontic/) and regenerate. Files
without that header — everything currently in this tree — are hand-authored
sources of truth.

## Open-Texture Limitations

The merger deal is full of Hartian open-texture standards — "material adverse
effect", "ordinary course", "material breach" — whose whole point is that
reasonable readers can disagree at the margin. This fixture set flattens every
one of them into a boolean. That is deliberate for conformance testing, but it
means the fabric **resolves, it cannot abstain**: a contested condition is
still forced to a bit, and (see below) the POWL session latches that bit
permanently.

### Standards modeled as booleans

| Legal standard | Boolean encoding | Where |
|----------------|------------------|-------|
| Material adverse effect (MAE) | `ex:occurred` bit | `materiality.shacl.ttl` |
| Ordinary-course operation | unpredicated `lkif:Permission` | `merger-obligations.ttl` |
| Breach / excuse | `ex:excused false` hard-coded per violation | `breach.ttl` |
| 66.67% vote threshold | `votesCast = for + against`, abstentions vanish | `vote-threshold.n3` |

Each encoding side-steps exactly the hard question. Whether a downturn is
"material", whether interim operations were "ordinary", whether a breach is
excused, and whether an abstention counts as a vote cast are precisely the
disputes merger litigation is made of; here each is pre-answered.

### Constructed indeterminate fact patterns

The skeptic ensemble constructed three fact patterns that sit in the
open-texture zone and that this fabric cannot represent as indeterminate:

1. **Quantitative-vs-qualitative MAE** — a downturn that clears the
   quantitative carve-out math but plausibly falls in the qualitative
   exceptions. The `ex:occurred` bit must be set either way; the fabric has no
   third value.
2. **Vacuous ordinary-course** — BetaHoldings' business changed so completely
   during the interim period that "ordinary course" has no referent. The
   permission is unpredicated, so it is trivially satisfied.
3. **Contested votes-cast** — the certificate is ambiguous on whether
   abstentions count as votes cast. The N3 rule asserts
   `votesCast = for + against`, so the denominator is fixed by the rule, not
   by the disputed charter language.

### No abstention path; the latch makes it permanent

`ConformanceSession` (`src/conformance/session.rs`) is a latching state
machine: the first fault latches the session into `Phase::Faulted`, and a
completed trace latches `Phase::Complete` — in both cases the phase is
terminal and cannot be revised. Combine that with the boolean encodings above
and the behavior is: the fabric takes each contested legal question, forces it
to a bit during replay, and the session then holds that resolution permanently.
There is no representation for "this standard is indeterminate, withhold
judgment" — a court that needs an abstention path must model it upstream of
this fixture set.

## Closed world

Shapes use `sh:closed true` with explicit `sh:property` allow-lists over the
fixed subject set `ex:AcmeCorp`, `ex:BetaHoldings`, `ex:MergerSub`. Any
predicate outside the declared set is a validation failure by design — this
is a fixture for testing strict validation, not an open-ended model.

## See Also

- `src/nato/` — the courts that consume fixtures like these
- `docs/features/NATO_COURTS.md`
- FIBO: https://spec.edmcouncil.org/fibo/
- LKIF-core: https://www.estrellaproject.org/lkif-core/
- SHACL: https://www.w3.org/TR/shacl/
