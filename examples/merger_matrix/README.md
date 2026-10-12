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
