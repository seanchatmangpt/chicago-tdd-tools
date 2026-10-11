# chicago-tdd-tools: land or delete remote branch `claude/diataxis-playground-docs-01NoRVkHvwj8VoUhiRdPcHHk`

- Standing: OPEN
- Created: 2026-09-19 (v26.9.19 gh survey wave)
- Source: remote branch `claude/diataxis-playground-docs-01NoRVkHvwj8VoUhiRdPcHHk` — not merged into `main`, no open PR
- Evidence: `git branch -r --no-merged origin/main` lists it; absent from `gh pr list` heads

## Work to complete
- Decide: open a PR (`gh pr create -R seanchatmangpt/chicago-tdd-tools --head claude/diataxis-playground-docs-01NoRVkHvwj8VoUhiRdPcHHk`) or delete (`git push origin --delete claude/diataxis-playground-docs-01NoRVkHvwj8VoUhiRdPcHHk`).
- If superseded, delete; otherwise land through review.

## Acceptance
- After `git fetch --prune`, `git branch -r --no-merged origin/main` no longer lists `claude/diataxis-playground-docs-01NoRVkHvwj8VoUhiRdPcHHk`.

## History
- 2026-09-19 | OPEN | survey found PR-less unmerged branch | claude/diataxis-playground-docs-01NoRVkHvwj8VoUhiRdPcHHk | decision pending
