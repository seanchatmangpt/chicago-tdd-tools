# chicago-tdd-tools: land or delete remote branch `agent/v26.7.30-ggen-standards`

- Standing: OPEN
- Created: 2026-09-19 (v26.9.19 gh survey wave)
- Source: remote branch `agent/v26.7.30-ggen-standards` — not merged into `main`, no open PR
- Evidence: `git branch -r --no-merged origin/main` lists it; absent from `gh pr list` heads

## Work to complete
- Decide: open a PR (`gh pr create -R seanchatmangpt/chicago-tdd-tools --head agent/v26.7.30-ggen-standards`) or delete (`git push origin --delete agent/v26.7.30-ggen-standards`).
- If superseded, delete; otherwise land through review.

## Acceptance
- After `git fetch --prune`, `git branch -r --no-merged origin/main` no longer lists `agent/v26.7.30-ggen-standards`.

## History
- 2026-09-19 | OPEN | survey found PR-less unmerged branch | agent/v26.7.30-ggen-standards | decision pending
