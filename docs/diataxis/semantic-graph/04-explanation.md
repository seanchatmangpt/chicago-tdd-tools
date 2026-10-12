# Explanation: Open Texture vs. Deterministic Logic in the Semantic Graph Stack

> Diataxis axis: **Explanation** (understanding-oriented). Grounded in the
> landed merger-matrix fixtures (/Users/sac/chicago-tdd-tools/examples/merger_matrix/)
> and the deontic layer as actually modeled in `deontic/*.ttl`. No legal
> advice; these are hand-authored fixtures.

## The tension

Legal standards are open-textured: Hart's point about "no vehicle in the
park" generalizes — a rule's pivotal terms (vehicle, material adverse effect,
ordinary course) have a core of settled application and a penumbra that no
rule text closes. Deterministic logic machines — SHACL validators, N3
forward chainers, POWL conformance replay — are closed by construction: a
triple validates or it does not; a rule fires or it does not; a trace
conforms or carries a deviation. The merger-matrix stack is an attempt to
make that boundary explicit instead of papering over it. Each layer states
which side of the line it lives on.

## Layer 1: SHACL shapes — determinism, deliberately

`shapes/merger-core.shacl.ttl` closes the entity topology:

- `sh:closed true` on every node shape; predicates outside the enumerated
  `sh:property` list fail.
- `ex:BetaHoldings` must have exactly one integer `ex:sharesOutstanding`
  >= 1; `ex:MergerSub` exactly one `ex:formedBy` pointing at the acquirer.

Nothing here interprets. The choice of what counts as an "entity" was made
at authorship time and frozen into the shape. That is the right trade for
topology: the deal graph has four individuals and the fixture says exactly
which. Determinism is cheap where the extension of the concept is fixed.

## Layer 2: materiality — an open-texture standard forced boolean

`shapes/materiality.shacl.ttl` is where the limitation lives, and the
fixture is honest about it. The MAE definition is a paradigm open-textured
standard: courts decide materiality qualitatively, weighing duration,
magnitude, and whether the effect is industry-wide. The fixture models it
as:

```turtle
sh:path ex:thresholdPctOfRevenue ; sh:datatype xsd:decimal ;
    sh:minInclusive "0.0"^^xsd:decimal ; sh:maxInclusive "100.0"^^xsd:decimal ...
ex:MaeDefinition a ex:MaterialAdverseEffect ;
    ex:thresholdPctOfRevenue "10.0"^^xsd:decimal ;
    ex:measuredOverMonths 12 ;
    ex:carveOut ex:IndustryWide , ex:AnnouncementEffect ;
    ex:occurred false .
```

The carve-outs (`ex:IndustryWide`, `ex:AnnouncementEffect`, `ex:ChangeOfLaw`,
`ex:Pandemic`, `ex:NaturalDisaster`, `ex:GeneralEconomic`) are an
`sh:in` enumeration — a closed list standing in for what is doctrinally an
unbounded evaluative category. This is the SK10-class limitation: an
indeterminate verbal standard forced through a boolean channel. The predicate
`ex:occurred` is a single `xsd:boolean`; the thing it compresses — whether
the effect satisfies the carve-out tests, measured over the agreed window —
is the entire penumbra. The fixture does not hide this; it encodes the
compression (`occurred false` for the fixture deal) and lets the record show
what was decided. The honest reading: SHACL validates the *form* of the
materiality finding (decimal in range, months >= 1, carve-outs from the
closed set), never the *merit*. The merit decision happens upstream of the
machine and is memorialized by it.

## Layer 3: N3 rules — determinism with declared scope

`rules/vote-threshold.n3` derives the vote outcome: `votesCast = for +
against`, `yesShare = 100*for/cast`, pass iff `yesShare >= thresholdPct`
(66.67), fail otherwise via `log:notImplies` negation-as-failure. The
fixture's own comment (F3 note) does the philosophical work a formal system
can't: it states that `notImplies` is cwm/EYE's *scoped* negation, not
classical material implication. Determinism is preserved by pinning the
semantics of the negation operator explicitly. Where the rule is genuinely
mechanical — arithmetic over recorded counts — the boolean output is not a
compression of anything; 700,000/1,000,000 >= 66.67% just is the answer.

Contrast with the MAE case: the vote threshold was *negotiated to a number*,
so the machine can own it. Materiality was never a number until the
parties made one. The architectural lesson is about which standards have
been crystallized into numbers by prior agreement, not about which logic
engine is stronger.

## Layer 4: LKIF deontic layer — defeasibility as structure

The `deontic/*.ttl` fixtures model Hart's open texture not by simulating
judgment but by giving it typed slots in an LKIF-core frame:

- `merger-obligations.ttl`: `lkif:Obligation` individuals
  (`ex:ObligationToConsummate`, `ex:ObligationToCallVote`) with
  `lkif:plays` / `lkif:heldBy` / `lkif:towards` roles, plus an
  `lkif:Permission` for ordinary-course operation between signing and
  closing.
- `breach.ttl`: `lkif:Violation` individuals each carrying
  `ex:violates`, `ex:excused` (boolean), and `ex:opensRemedy` — the
  remedy chain from violation to `RemedySpecificPerformance` or
  `RemedyTerminationFee` (USD 126,000,000).
- `closing-conditions.ttl`: the closing permission
  (`ex:Closing a lkif:Permission`) `ex:requires` four named conditions,
  each an `ex:ClosingCondition` with `ex:satisfied` true/false and
  `ex:traceableTo` back-references (`ex:MaeDefinition`,
  `ex:MasterNetting1`, `ex:MergerVote`).

Two modeling decisions carry the defeasibility:

1. **`ex:excused` as a first-class field on every Violation.** A breach does
   not entail liability; the excuse predicate mediates. In defeasible-logic
   terms the excuse is the defeater; in this TTL it is a slot that must be
   filled before `opensRemedy` is actionable. The fixture sets
   `ex:excused false` on both violations, but the field's existence is the
   architecture.
2. **`ex:waives` on a Permission** (`ex:WaiverOfNettingCondition` held by
   BetaHoldings over `ex:CondNettingExecuted`): the right-holder can retire
   their own condition failure without a breach arising. Waiver is the
   classic defeater of a condition precedent, and it is a *permission held
   by a specific party*, not a global fact — the frame encodes who may
   defeat what.

Note what is NOT here: no priority ordering among norms, no balancing test,
no "pro tanto obligation" calculus. LKIF-core's valuation vocabulary gives
the *roles*; the fixture does not implement a defeasible inference engine
over them. Divergence note: real LKIF deployments (and the ESTRELLA
methodology) pair the ontology with a defeasible rule language; this stack
pushes defeasibility up into workflow structure instead.

## Layer 5: POWL — open texture resolved by structure, at authorship time

The phase nets show the resolution mechanism. In `phase1_intake.powl`:

```text
xor { a03_covenant_conflicts silent }
```

The covenant-conflict branch is an exclusive choice between escalation and
silent clearance — A03 defeasibility lowered into the process as a branch,
not into a rule engine. The place where a lawyer would exercise judgment
("is this conflict material?") becomes a fork the trace either takes or
does not; the conformance replay then *audits the fork*, it does not decide
it. Similarly `loop { a05_cet1_buffers } redo { silent } max 3` bounds the
capital-adequacy retry loop; the bound is a legal-hold decision (how many
attempts before failure is declared) frozen into `max`.

This is the stack's actual answer to Hart: **open texture is not eliminated,
it is relocated to authorship time and made auditable at replay time.** The
penumbra gets exercised once, by a human or a drafting process, and the
choice is compiled into xor branches, `max` bounds, boolean `occurred`/
`satisfied`/`excused` fields, and closed `sh:in` lists. The machine then
gives the one thing penumbral reasoning cannot: a total order trace that
either conforms (`ConformanceReport::is_conforming`) or names its deviation.

## The residual limitation, stated plainly

- Every open standard in the fixture family ends in a boolean or a closed
  enumeration (SK10-class). The stack is sound *downstream* of the
  crystallization; it does not do the crystallizing, and nothing in it
  checks that the crystallization was faithful to the standard. A wrong
  `thresholdPctOfRevenue` validates perfectly.
- The deontic layer's defeaters (`excused`, `waives`) are asserted facts, not
  derived ones — no rule fires to establish them. Defeasibility here is a
  data model, not an inference regime.
- The negation in `vote-threshold.n3` is engine-scoped (cwm/EYE
  `log:notImplies`); porting the rule to another N3 engine with different
  scoped-negation semantics changes the answer. The fixture's F3 comment is
  load-bearing.

What the architecture buys, honestly stated: not automated judgment, but a
fixed, replayable record of where judgment was exercised, with every
judgment-shaped gap closed by an explicit authored decision — and a
fail-closed membrane (`sh:closed true`, `MAX_POWL_NODES`, positioned parse
errors, machine-coded git errors) around everything downstream.
