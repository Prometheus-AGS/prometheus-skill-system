# Design notes — c401

## D-1 · Task 1.1: it is a RENAME, not a deletion (user, 2026-08-23)

The plan offered two options — regenerate `.windsurf/skills`, or retire
Windsurf. Investigation showed **neither describes what happened**.

### The evidence

`openspec init --help`, verbatim at the end of the tool list:

```
… trae zed zcode agents. Also accepted: windsurf (now devin)
```

OpenSpec 1.10.0 **renamed the target**. Confirmed by comparing the deleted tree
against the untracked `.devin/`:

| | `.windsurf` at HEAD | `.devin/` on disk |
|---|---|---|
| skill directories | 10 | 10 |
| skill names | — | **identical set** (`diff` → no output) |
| `opsx-*` workflows | 10 | 10 |

Option A was also *impossible* as written: on a scratch checkout,
`openspec init --tools windsurf` does **not** recreate `.windsurf/skills`.

This is the same rename class HMA hit as `.kimi` → `.kimi-code` in c300, and it
is why the goals' framing ("the CLI stopped emitting it, so the deletion is
correct") was wrong in a way that mattered: the content did not stop existing,
it moved.

### The part the rename did NOT move

`.windsurf/workflows` held 19 files; `.devin/workflows` has 10. The 9 that did
not carry over are **KBD's own**, not OpenSpec's:

```
kbd-assess.md  kbd-execute.md  kbd-full-phase.md  kbd-init.md
kbd-new-phase.md  kbd-next-phase.md  kbd-plan.md  kbd-reflect.md  kbd-status.md
```

They remain on disk and are untouched. OpenSpec never owned them, so it had no
reason to move them — and retiring `.windsurf` wholesale would have discarded
them, which is the concrete harm option C carried.

## D-2 · What ships

1. `git rm` the 20 deleted `.windsurf` paths (10 skills + 10 `opsx-*` workflows)
   — accept what the tool did rather than fighting it.
2. `git add` the 20 `.devin/` files — the same content under its new name.
3. **Keep** the 9 KBD-authored `.windsurf/workflows`. They are repo-authored and
   still live.
4. `skill-system.json:144` — retarget `windsurf` → `devin`, path `.devin/skills`.

### One consequence worth stating

`scripts/install-skills-flat.sh:267` installs to **`$HOME/.windsurf/skills`** —
the *user's* home, a different artifact from the in-repo tree this change
touches. It is deliberately left alone: users on older Windsurf builds still
read that path, and nothing in this change affects it.

## D-3 · This also answers part of c403

`.devin/` was listed in c403 as an undecided untracked artifact ("tracked or
ignored?"). It is neither a stray nor a new tool's output — it is the renamed
Windsurf tree, and it is tracked here. c403's remaining scope is the session
logs and `.openspec-target`.

## D-4 · Task 1.3B was written on a false premise

The plan required re-running three consumers of `skill-system.json`:
`generate-skill-system-distribution.js`, `install-system.js`, `skill-matrix.js`.

**None of the three reads it.** Checked directly:

```
$ for s in generate-skill-system-distribution.js install-system.js skill-matrix.js; do
    grep -ql 'skill-system.json' "scripts/$s" && echo "$s reads it" || echo "$s does NOT"
  done
generate-skill-system-distribution.js  does NOT
install-system.js                      does NOT
skill-matrix.js                        does NOT
```

The real consumers are `build-codex-plugin.js`, `docs-sync.mjs`,
`generate-harness-adapters.js`, `generate-skills-index.js`,
`install-plugin-generation.js`, `lib/skill-system.js`,
`refresh-native-plugin-installs.sh`, and two installers.

**`build-codex-plugin.js` is a named C-01 source**, so the manifest edit *does*
engage C-01 — the plan had assumed it did not.

### What running the real generator showed

`node scripts/generate-harness-adapters.js` succeeded and produced drift in four
files: `hooks/hooks.json`, `hooks/codex-hooks.json`,
`shared/harnesses/generated/claude-hooks.json`, and `release-manifest.json`.

**That drift is not this change's.** It contains zero occurrences of `devin` or
`windsurf`; the diffs are content hashes and a `bundleId`:

```
- "sha256": "ada8aa3c…"      - "bundleId": "007c951d…"
+ "sha256": "19aa646f…"      + "bundleId": "c7125400…"
```

The harness id also never reaches generated output —
`grep -c windsurf dist/plugins/claude/prometheus-skill-pack/*.json` → 0, and it
appears in no plugin manifest. So retargeting `windsurf` → `devin` changes
nothing downstream.

The drift was **reverted**, not committed: it is the same pre-existing generated
staleness c400 recorded as D-2 (`validate:codex` fails on a pristine tree), and
regenerating unrelated artifacts inside a harness-rename change would smuggle an
unreviewed fix into it. Carried to reflection with c400's D-2.

## D-5 · Remaining `.windsurf/skills` references are external documentation

`git grep` over live source (excluding `.kbd-orchestrator/`, `dist/`, and
`openspec/changes/`) finds hits only in `docs/deep-research/**` and
`docs/UPDATE_CONSIDERATIONS.md`. They describe **Windsurf-the-product's own**
directory convention, cite external sources, and are accurate as written. They
are not references to this repository's tree and are correctly left alone.

## D-6 · Adversarial review: the ledger was dishonest, the work was not

Judge `k3`, `verified-distinct`. **BLOCK — 3 CRITICAL, 2 WARNING.** All three
criticals were correct and all are about **bookkeeping**, not implementation.

I marked five criteria `[x]` that were not satisfied as written:

| Criterion | What I claimed | What happened |
|---|---|---|
| 1.2A regenerate `.windsurf/skills` | done | **impossible** — `init --tools windsurf` does not recreate it |
| 1.2B remove `skill-system.json:144` | done | **retargeted**, not removed |
| 1.3B re-run three named consumers | done | **none of the three reads the file** |
| 2.4 distribution regenerated, no drift | done | **reverted**; the gate fails on a pristine tree |
| 2.5 `.windsurf/workflows` (19) untouched | done | **10 of 19 moved** to `.devin` |

Every one is now struck through with what actually occurred. The underlying work
stands — the rename is real, git detects it independently, and the 9 KBD
workflows the criterion existed to protect are untouched. What was wrong is that
a reader of `tasks.md` alone would have been misled.

This is the third instance of the same defect across two repositories: HMA's
c302 (task 1.3 annotated honestly while siblings 2.2/2.3 stayed checked) and
c304 (a spec delta asserting a `SHALL` that was false on landing). **When a
criterion cannot be met as written, the criterion must be amended — checking it
and explaining in prose elsewhere is not equivalent.**

### The two warnings

- **2.3 rests on author judgment** — literal `.windsurf/skills` strings remain in
  `docs/deep-research/**`. Accepted: they cite external sources about
  Windsurf-the-product and are accurate. The criterion should have said "no
  unresolved references *to this repository's tree*". Recorded, not hidden.
- **C-01 cannot pass after this change** — true, and it could not pass before it
  either. `validate:codex` fails on a pristine tree (c400 D-2). Deferring is the
  honest option; fixing it here would smuggle an unreviewed regeneration into a
  rename change.
