# chicago-tdd-tools migration plan v26.11.16

Standing: UNVERIFIED

## Identity
- Path: /Users/sac/chicago-tdd-tools
- Remote: https://github.com/seanchatmangpt/chicago-tdd-tools.git
- Branch: main@5070224 (tracks origin/main, ahead 0 / behind 0)
- Last commit: 2026-10-13 ("docs(commands): repair 24 links into the flattened docs tree")
- Commits: 595; size 115 MB
- Languages/build: Rust (Cargo.toml, crates/, proc_macros/, build.rs), Justfile, ggen.toml,
  nextest.toml, Dockerfile.weaver
- Nested repos:
  - registry/ -- VENDOR clone of open-telemetry/semantic-conventions, detached HEAD@ae3a986,
    1 commit (shallow), 10 MB, untracked otel/ dir

## Observed state
- Dirty: 3 modified -- Cargo.toml, cookbook/archive/EXAMPLE-AAA-REFACTORED.md,
  src/conformance/validate.rs
- Untracked: 2 -- src/bin/run_court.rs, tests/run_court_cli_tests.rs (new feature, uncommitted)
- Ahead/behind origin/main: 0/0
- Stash count: 136 (census)
- Local branches: 6; 4 without upstream: exp/weaver-registry-version-config,
  security/rotate-signing-keys-v26.9.24, subagent-Application-Guide-Writer-self-*,
  subagent-Cookbook-Writer-self-*
- Build dirs: no target/; mutants.out and mutants.out.old present (cargo-mutants output),
  coverage.lcov present
- Submodules: none (registry/ is a plain nested clone)

## What it is
A Rust testing framework enforcing Chicago-style (classicist) TDD through compile-time
guarantees, plus feature-gated assurance surfaces: a POWL conformance kernel, a tamper-evident
court-receipt ledger, and a 20-court NATO assurance matrix.

## Classification
ACTIVE (census agrees; last commit ~5 weeks ago with in-flight uncommitted feature work).

## Migration risk
- Uncommitted run_court binary + its tests and 3 modified files.
- 136 stashes; 4 unpushed local branches (one is a signing-key rotation branch --
  security-relevant, verify it was superseded by what landed on main).
- registry/ vendor clone has an untracked otel/ dir of unknown origin.
- Rate: MEDIUM

## Recommended action
COMMIT-PUSH

1. `git add Cargo.toml src/conformance/validate.rs src/bin/run_court.rs
   tests/run_court_cli_tests.rs cookbook/archive/EXAMPLE-AAA-REFACTORED.md` and commit
   `"feat(court): run_court CLI (pre-migration checkpoint)"`, then `git push origin main`.
2. DECISION: push or drop the 4 upstream-less branches; the two subagent-* branches look like
   scratch. `git push -u origin exp/weaver-registry-version-config
   security/rotate-signing-keys-v26.9.24` at minimum.
3. DECISION: triage 136 stashes (`git stash list`); branch any keepers, then accept the rest.
4. Bundle: `git bundle create /Volumes/ARCHIVE/chicago-tdd-tools.bundle --all`.
5. Cleanup: `rm -rf mutants.out mutants.out.old registry/` (registry is re-clonable from
   open-telemetry/semantic-conventions; check justfile for how it is fetched).
6. Clone fresh on the new machine.

## First commands on the new machine
- `git clone https://github.com/seanchatmangpt/chicago-tdd-tools.git && cd chicago-tdd-tools`
- `cargo check --workspace --all-features` then `just --list` (stated, not run)

## See Also
- ../v26.10.12/ (previous jira dir; census listed older ones, tree has v26.10.11, v26.10.12)
- /Users/sac/docs/jira/v26.11.16/README.md (master index)
