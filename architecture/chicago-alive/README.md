# CHICAGO_ALIVE

`CHICAGO_ALIVE` is the evidence standing between `PARTIAL_ALIVE` and `ALIVE`.

It exists so `ALIVE` does not have to be weakened merely because a system has accumulated much stronger evidence than ordinary partial execution.

## Standing ladder

```text
UNKNOWN
  ↓
PARTIAL_ALIVE
  ↓
CHICAGO_ALIVE
  ↓
ALIVE
```

- `PARTIAL_ALIVE`: meaningful bounded execution exists, but collaboration, world, falsifier, receipt, replay, or correspondence closure is incomplete.
- `CHICAGO_ALIVE`: the exact software subject executed through its real load-bearing collaboration topology against an admitted executable world; observable consequences, authority-path evidence, adversarial falsifiers, receipts, replay, and bounded target correspondence all close. The exact real-target crown is still absent.
- `ALIVE`: the `CHICAGO_ALIVE` obligations remain satisfied and the exact admitted real target has itself produced the claimed observed consequence.

`CHICAGO_ALIVE != ALIVE`.

## Why Chicago

The Chicago/Classical TDD fence is state-based verification with real collaborators. A load-bearing behavior cannot be replaced by a mock and then used as evidence that the real behavior works.

The hard rule is:

```text
load-bearing collaborator is TEST_DOUBLE
  => REFUSED
```

Test doubles remain lawful outside the claim boundary. The court only refuses when the double supplies behavior that is load-bearing for the standing being claimed.

## Court

The executable court is `scripts/verify_chicago_alive.py`.

The court derives standing from evidence. It does not accept a caller-supplied standing and it has no actuation primitive.

```text
exact subject
  ↓
admitted executable world
  ↓
real load-bearing collaborators
  ↓
observed execution
  ↓
observed consequence
  ↓
authority-path evidence
  ↓
adversarial falsifiers
  ↓
receipts
  ↓
replay MATCH
  ↓
bounded correspondence
  ↓
CHICAGO_ALIVE
  ↓ exact real target + observed real consequence
ALIVE
```

## DfCM interpretation

The court preserves the distinction between reversible evidence construction and irreversible real-world actuation.

```text
OBSERVE → ADMIT → CONSTRUCT EVIDENCE → VERIFY → RECEIPT → REPLAY
                                                │
                                                └─ no ambient DO
```

The verifier never performs production DO. `authority.direct_actuation=true` is a typed refusal. Production authority used in a synthetic, emulated, or test-tenant world is also refused.

## Ecosystem roles

### GymAct

GymAct owns executable world physics and the trial boundary. It should emit the world identity, actor/authority observations, consequences, receipts, and replay evidence used by this court.

### Gym ecosystem

The gym ecosystem composes bounded world physics. A cloud claim can therefore require cloud + network + identity + security + commerce worlds without inventing a monolithic simulator. DfCM applies `REUSE → COMPOSE → ADAPT → CREATE_PROVIDER`, with unknown physics remaining `BLOCKED_DISCOVERY`.

### AutoFDE-Lab

AutoFDE-Lab owns experiment manufacture: planner/policy variants, counterfactual worlds, adversarial scenario selection, objective verification, and falsifier expansion. Its output is trial evidence, not standing authority.

### AutoFDE

AutoFDE is the subject under trial. `CHICAGO_ALIVE` may be capability-scoped; one capability can be `CHICAGO_ALIVE` while another remains `UNKNOWN` or `PARTIAL_ALIVE`.

## Required evidence

A candidate can reach `CHICAGO_ALIVE` only when all of these are true:

1. `subject.ref` exists and `subject.commit_sha` is exact 40-hex.
2. `subject.exact=true`.
3. The world has a stable identity and `world.admitted=true`.
4. At least one load-bearing collaboration edge is evidenced.
5. Every load-bearing collaborator is `REAL`, never `TEST_DOUBLE`.
6. Exact-subject execution is observed in that world.
7. A world consequence is observed.
8. The authority path is observed and direct actuation is false.
9. At least one adversarial falsifier executes, every falsifier passes, and each has evidence.
10. At least one consequence receipt exists.
11. Runtime replay is `MATCH`.
12. Target correspondence is explicitly bounded and names both target and contract.
13. Exact-head receiver mode, when requested, matches `subject.commit_sha`.

The exact real target is intentionally *not* required for `CHICAGO_ALIVE`; requiring it would collapse the state into `ALIVE`.

## Typed outcomes

Structural/admission defects return `BLOCKED`. Contradictory or prohibited evidence returns `REFUSED`. Missing but non-contradictory evidence returns `PARTIAL_ALIVE`.

Examples:

| Condition | Result |
|---|---|
| world not admitted | `BLOCKED` |
| load-bearing mock | `REFUSED` |
| direct actuation | `REFUSED` |
| falsifier fails | `REFUSED` |
| replay not yet run | `PARTIAL_ALIVE` |
| no receipt yet | `PARTIAL_ALIVE` |
| all Chicago obligations close | `CHICAGO_ALIVE` |
| Chicago obligations + exact real target consequence | `ALIVE` |

## Receipts and replay

Receiver mode uses BLAKE3. The court receipt binds:

- exact candidate head,
- canonical trial input digest,
- derived decision digest,
- derived standing,
- zero-direct-actuation assertion.

The composite receipt can be replayed independently. A mismatch returns nonzero and cannot crown.

## Exact-head usage

```bash
python3 scripts/verify_chicago_alive.py --self-test

python3 scripts/verify_chicago_alive.py \
  architecture/chicago-alive/example-trial.json \
  --require-exact-head "$GITHUB_SHA" \
  --require-blake3 \
  --write-receipt /tmp/chicago-alive-receipt.json

python3 scripts/verify_chicago_alive.py \
  architecture/chicago-alive/example-trial.json \
  --require-exact-head "$GITHUB_SHA" \
  --require-blake3 \
  --replay-receipt /tmp/chicago-alive-receipt.json
```

For an exact-head run, replace the example trial's `subject.commit_sha` and `subject.ref` with the exact candidate SHA before invoking the court.

## Falsifier

The doctrine is falsified if any of the following can still produce `CHICAGO_ALIVE`:

- a load-bearing test double,
- an unadmitted world,
- no observed execution,
- no observed consequence,
- no authority-path evidence,
- direct actuation,
- a failed adversarial falsifier,
- missing receipt evidence,
- replay mismatch,
- unbounded correspondence,
- exact-head mismatch.

The built-in self-test mutates these boundaries and requires typed demotion/refusal.
