# chicago-tdd-tools: land or delete remote branch `claude/andon-signals-workflow-011NSiynyua8uNTNA4XXUGtW`

- Standing: OPEN
- Created: 2026-09-19 (v26.9.19 gh survey wave)
- Source: remote branch `claude/andon-signals-workflow-011NSiynyua8uNTNA4XXUGtW` — not merged into `main`, no open PR
- Evidence: `git branch -r --no-merged origin/main` lists it; absent from `gh pr list` heads

## Work to complete
- Decide: open a PR (`gh pr create -R seanchatmangpt/chicago-tdd-tools --head claude/andon-signals-workflow-011NSiynyua8uNTNA4XXUGtW`) or delete (`git push origin --delete claude/andon-signals-workflow-011NSiynyua8uNTNA4XXUGtW`).
- If superseded, delete; otherwise land through review.

## Acceptance
- After `git fetch --prune`, `git branch -r --no-merged origin/main` no longer lists `claude/andon-signals-workflow-011NSiynyua8uNTNA4XXUGtW`.

## History
- 2026-09-19 | OPEN | survey found PR-less unmerged branch | claude/andon-signals-workflow-011NSiynyua8uNTNA4XXUGtW | decision pending
