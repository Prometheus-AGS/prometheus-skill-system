# Design notes — c400

## D-1 · Task 1.1: the 30 command/workflow files (measured, not inferred)

The plan flagged this as the one open question: the assessment had inspected a
single *skill* line-by-line and inferred the rest. Measured now:

### They carry the same upgrade

`git diff -- .cursor/commands/opsx-apply.md`, added lines only:

| Feature | Present |
|---|---|
| `--store` selection | yes |
| `schemas` / `view` in the store-scoped command list | yes |
| sticky-flag semantics | yes |
| `allowed-tools` frontmatter | **no** |

`30 files changed, 1736 insertions(+), 657 deletions(-)`.

### But they are a different artifact shape

```
.cursor/commands/opsx-apply.md            .cursor/skills/openspec-apply-change/SKILL.md
---                                       ---
name: "/opsx-apply"                       name: openspec-apply-change
id: "opsx-apply"                          description: …
category: "Workflow"                      allowed-tools: Bash(openspec:*)
description: "…"                          license: MIT
---                                       compatibility: Requires openspec CLI.
                                          metadata:
                                            generatedBy: "1.10.0"
```

Commands carry **no `generatedBy` and no `allowed-tools`**. So:

- the `1.3.1 → 1.10.0` version evidence exists **only in the 40 skills**;
- the commands are dated solely by their content.

### Decision: keep them in one change

They ship the same upstream behaviour from the same generator run, so splitting
them would fragment one upgrade across two commits for no reviewer benefit. The
plan's task 1.1 said "if they differ **in kind**, split them out" — they differ
in *frontmatter shape*, not in kind.

What this does change is the commit message: it must not claim a `generatedBy`
bump across all 70 files, because 30 of them have no such key. The message names
the bump as covering the 40 skills and the feature content as covering all 70.

## D-2 · Task 2.4: C-01 asserted, and a pre-existing red gate found

**The assertion c400 owns passes.** No C-01 source is in this change:

```
$ git diff --cached --name-only | grep -E '\.claude-plugin/|\.mcp\.json|hooks/hooks\.json|build-codex-plugin\.js'
(no matches — 70 staged files, all under the four harness trees)
```

**But `npm run validate:codex` fails, and it is not this change's doing.**
Measured by stashing everything and running against a pristine tree:

```
$ git stash push -u          # dirty: 1
$ npm run validate:codex
Error: generated output is stale: dist/plugins/claude/prometheus-skill-pack
```

It fails with **zero** uncommitted changes. Corroborating evidence that c400 is
not the cause:

- `dist/plugins/claude/prometheus-skill-pack/skills` contains **0** openspec
  skills — the harness mirrors this change touches do not feed `dist/`;
- `git status --porcelain dist/` is **empty** — `dist/` is not among the 98.

So the C-01 gate for the *Codex distribution* was already red before this phase
opened. That is a real finding and it is **out of c400's scope**: fixing it means
regenerating `dist/`, which is a different artifact set with its own review.

Task 2.4 is therefore satisfied as written — "assert no C-01 source was touched"
— and the broader gate failure is recorded here rather than absorbed silently or
mistaken for damage this change caused. It belongs in the phase reflection as
debt, and c401 option B (which regenerates distribution output) may collide with
it.
