# PLAN: openspec-mirror-drift-cleanup

Repository: `prometheus-skill-system` (local dir `prometheus-skill-pack`)
Date: 2026-08-23
Assessment: `assessment.md` (cross-model vetted — judge `k3`, PASS, 4 findings applied)
Backend: **OpenSpec** · Changes: **6** (c400–c405)

## Ordering rationale

```
c400 ──┐
c401 ──┼── c404 ── c405
c402 ──┤
c403 ──┘
```

c400–c403 each resolve one slice of the 98 dirty files and are **mutually
independent** — different paths, no shared state. c404 (prevention) must land
after them because it asserts against the *settled* harness set, and asserting
against a tree still mid-decision would encode the wrong expectation. c405 is
the terminal barrier: it needs a fully clean tree, and it is the only change
that touches the outside world.

**A note on the convention this plan does not copy.** Recent changes in this
repo (`change-push-001`, `change-push-002`) ship without spec deltas, and both
**fail `openspec validate`**. Precedent is not permission: every change here
carries a real delta.

## Constraints in force (`.kbd-orchestrator/constraints.md`)

| | Applies to |
|---|---|
| **C-01** generated artifacts in sync | c403 (`.openspec-target`), c404 (if it adds a generator). **Verified: A does not touch any C-01 source**, so c400 does not trigger `npm run validate:codex` |
| **C-02** no committed secrets | **c403 must scan** — the assessment corrected a false claim of blanket session-log authorization |
| **C-03** docs updated with surface changes | c401 (if Windsurf support changes), c404 |
| **C-04** generators idempotent | c404 — run twice, assert clean |
| **C-05** bash 3.2 under launchd | c404 if it ships shell |

---

## c400 — Adopt the OpenSpec 1.10.0 harness upgrade

**Change:** `change-drift-400-openspec-110-upgrade` · Independent

### Scope, counted not inferred

70 modified files, **40 `SKILL.md` mirrors + 30 command/workflow files**:

```
10 .agent/skills/     10 .agent/workflows/
10 .agents/skills/    10 .cursor/commands/
10 .cursor/skills/    10 .opencode/commands/
10 .opencode/skills/
```

Not churn: **+4016/−1473**. In the one file inspected line-by-line, 2 of 54
changed lines are `generatedBy`; the rest is upstream feature content —
`--store` selection with sticky-flag semantics, an `allowed-tools` frontmatter
key, `schemas`/`view` added to the store-scoped command list, new scaffold
guidance.

### Tasks

1. **Verify the 30 command/workflow files carry the same upgrade.** The
   assessment inspected one *skill*; the command files are a different artifact
   class and are still unverified. If they differ in kind, split them out.
2. Confirm no file outside the four harness trees is in this change.
3. Commit with a message naming the shipped features, so the large diff is
   legible to the next reader.

### Acceptance

- [ ] `git status --porcelain` shows zero modified files under `.agent`, `.agents`, `.cursor`, `.opencode`
- [ ] The commit body names the `generatedBy` bump **1.3.1 → 1.10.0** (goal 2's
      explicit requirement) **and** the shipped features `--store`,
      `allowed-tools`, `schemas`/`view`
- [ ] Task 1 records what the command files actually contain — a finding either way
- [ ] No C-01 source touched (assert, do not assume)

---

## c401 — Decide `.windsurf/skills` · **DECISION**

**Change:** `change-drift-401-windsurf-harness` · Independent

### The evidence that reopened this

| Fact | Source |
|---|---|
| `windsurf` is a supported OpenSpec tool | `openspec init --tools` |
| This repo declares it a managed symlink harness | `skill-system.json:144` |
| `.windsurf/workflows` — 19 files, still tracked | `git ls-files` |
| `.agent/skills`, `.cursor/skills` still on disk | siblings kept theirs |
| At HEAD `.windsurf/skills` was a real tracked tree | `git ls-tree HEAD` → `040000 tree 94c299c4…` |

20 deletions, and **nothing yet explains why only this harness lost its skills**.

### Task 1 — the decision (blocks the rest)

| Option | Consequence |
|---|---|
| **A — regenerate** | The deletion is a bug; `.windsurf/skills` is restored and `skill-system.json:144` stays honest |
| **B — retire Windsurf** | The deletion is correct; `skill-system.json:144` must also go, or the manifest keeps promising a tree that will not exist. (An
earlier draft cited **C-03** here; C-03 governs *Codex plugin surface docs*, not
harness state. The consistency argument stands on its own — the citation was
wrong and is withdrawn.) |

Do not resolve by `git checkout` — that restores files without deciding anything.

### Acceptance

- [ ] The decision and its rationale are recorded in the change
- [ ] Under (A): `.windsurf/skills` present, count matches its sibling harnesses
- [ ] Under (B): `skill-system.json:144` removed **and** every other reference
      (`grep -rn '.windsurf/skills'` outside `.kbd-orchestrator/`) resolved
- [ ] Under (B): the **consumers** of `skill-system.json` are re-run and their
      output committed — `scripts/generate-skill-system-distribution.js`,
      `scripts/install-system.js`, `scripts/skill-matrix.js`. Removing a harness
      entry without regenerating distribution output would create exactly the
      drift class this phase exists to clean up (and c404 reads this same file)
- [ ] Either way `.windsurf/workflows` (19 files) is untouched — it was never in question

---

## c402 — Decide the submodule pointer · **DECISION, highest risk**

**Change:** `change-drift-402-entity-mgmt-submodule` · Independent

### Why this cannot simply be committed

```
branch:                main-takeover-kimi        (not main)
HEAD on disk:          55dc8dc → 4485696         (pinned at 1c40eaa; it MOVED
                                                 again during this phase — the
                                                 submodule is actively worked in)
git branch -r --contains HEAD:   (empty)         → on NO remote branch
git status --porcelain | wc -l:  52              → submodule itself dirty
```

Committing this pointer pins the parent to a commit that exists on no remote,
on a takeover branch, inside a dirty tree. **A fresh clone could not resolve it.**

### Task 1 — the decision (blocks the rest)

| Option | Consequence |
|---|---|
| **A — publish then pin** | Push `main-takeover-kimi` (or merge it), re-pin to a commit that resolves from a clone. Correct, but needs the submodule repo's owner |
| **B — restore the pin** | `git checkout -- skills/imported/prometheus-entity-management` in the parent only; the submodule's own checkout is left alone. Unblocks this phase, defers the upgrade |
| **C — defer entirely** | Leave the pointer dirty. **Rejected: it blocks c405**, which is the phase's reason to exist |

**B is the default** unless the owner confirms A is imminent. This phase exists
to unblock a reinstall, not to land a submodule upgrade.

### Acceptance

An earlier draft required "porcelain is empty" while also promising not to touch
the submodule's dirty files — a contradiction the cross-model review caught. It
then turned out **both** the draft and the review were working from stale facts.
Re-measured at plan time:

```
$ git diff --quiet -- skills/imported/…        → pointer DIFFERS
$ git -C skills/imported/… status --porcelain  → EMPTY (was 52 files)
```

**The submodule worktree is now clean** — its 52 files were committed while this
phase was being written, and its HEAD has moved twice (`55dc8dc` → `4485696`).
Only the parent's pointer is out of sync.

That collapses the contradiction: restoring the pointer *does* clear
`git status --porcelain`, so option B is achievable and `require_clean_source`
(plain porcelain, 5 calls) will pass. But it also means **the facts move under
this phase**, so:

- [ ] **Re-measure both conditions at execute time**, do not trust this table.
      `git diff --quiet -- <path>` for the pointer and
      `git -C <path> status --porcelain` for the content are two different
      questions with two different answers
- [ ] The decision, its owner, and its rationale are recorded
- [ ] `git status --porcelain -- skills/imported/prometheus-entity-management`
      is empty **after** the change — achievable only while the submodule
      worktree stays clean
- [ ] Under (A): `git branch -r --contains <new-pin>` is **non-empty** — the
      criterion the current pin fails
- [ ] If the submodule has gone dirty again by execute time, **stop and
      re-decide**: a residual ` m ` blocks `require_clean_source` and therefore
      c405, and clearing it is the submodule owner's call, not this phase's

---

## c403 — Session logs, `.devin/`, `.openspec-target`

**Change:** `change-drift-403-routine-artifacts` · Independent

### The correction this change carries

An earlier draft claimed a "standing authorization to always commit
`.prometheus/` session logs". **That authorization lives in the HMA repository's
`CLAUDE.md` and does not transfer here.** This repo has **C-02** instead.

### Tasks

1. **Scan before committing** (C-02): `git diff -- .prometheus/` and the
   untracked files for `sk-`, `api[_-]?key`, `bearer`, `token=`. A prior scan
   found 0 matches — re-run it; do not inherit the result.
2. Commit the 3 modified `.prometheus` wiki files + 2 untracked `karpathy-session` files.
3. **Decide `.devin/`** — tracked or `.gitignore`d? It is a new tool artifact.
4. **Decide `.agents/skills/.openspec-target`** — it holds `agents` and is how
   OpenSpec records its tool target. If tracked, it is arguably a **generated
   artifact under C-01**.

### Acceptance

- [ ] C-02 scan re-run, result recorded (not inherited)
- [ ] `.devin/` and `.openspec-target` each have a recorded decision, not a default
- [ ] `git status --porcelain -- .prometheus .devin .agents/skills/.openspec-target` is empty

---

## c404 — Prevention: normalizer + per-harness completeness

**Change:** `change-drift-404-harness-completeness` · Depends on c400–c403

### Both halves, per goal 5

An earlier draft of the assessment listed only the assertion. Restored:

- **Assertion** — detects a harness that has lost its tree. `.windsurf/skills`
  reached zero and *nothing noticed*; that is the gap.
- **Normalizer** — re-applies repo-local invariants an external generator
  strips. **Open question, to answer in this change:** unlike HMA, this repo
  enforces no `internal: true`, so there may be nothing to re-apply. If so,
  record that and ship the assertion alone — do not invent an invariant to
  justify the script.

### Tasks

1. Answer the normalizer question above. Evidence, not assumption.
2. Add the per-harness completeness assertion over the harness set
   `skill-system.json` declares, reading that file rather than hardcoding.
3. **Negative fixture** — delete one harness's skills in a scratch copy; the
   assertion must fail naming that harness. Write the fixture *before* trusting
   the passing run.
4. C-04: run twice, assert the second run is clean.
5. C-05 if shell: bash 3.2 compatible.
6. C-03: document the check.

### Acceptance

- [ ] Negative fixture fails naming the emptied harness
- [ ] Idempotent — second consecutive run leaves `git diff --exit-code` clean
- [ ] Harness list derived from `skill-system.json`, not hardcoded
- [ ] The normalizer question is answered in the change, either way

---

## c405 — Land, reinstall, verify · **terminal barrier**

**Change:** `change-drift-405-reinstall-verify` · Depends on all

### The sequence the assessment established

`update-skill-pack.sh` checks for a clean tree **five times** (`grep -c
require_clean_source` → 5) and runs `git pull --ff-only`. So:

1. Land c400–c404 to `main` (local `main` is `6de8181`; `origin/main` is
   `c0d2de1` — the merged PR #55).
2. `git checkout main && git pull --ff-only` — brings local up to `origin`.
3. **`git push origin main`** — a fast-forward *pull* moves nothing outward. An
   earlier draft omitted this, which would have left the final `ls-remote`
   criterion unsatisfiable.
4. `bash scripts/update-skill-pack.sh --force`.

A step that regenerates files mid-run will re-dirty the tree and abort a later
check — commit between stages.

### Acceptance — both halves of goal 6

- [ ] Working tree clean; `update-skill-pack.sh --force` exits 0
- [ ] `grep -c resolver_missing ~/.claude/skills/adversarial-review/scripts/preflight-models.sh` → **1**
- [ ] **With `CLAUDE_PLUGIN_ROOT` UNSET**, a run through `~/.claude/skills/…`
      reports `status: ok`. This is the criterion that reproduces the original
      failure; an earlier draft omitted it and the cross-model judge caught that
- [ ] `git ls-remote origin main` resolves to a commit containing **c400–c404**.
      c405 itself produces no source commit — it is a landing-and-verification
      change — so the criterion is stated over the five that do, plus this
      change's own phase-record commit

---

## Open decisions carried into execute

1. **c401** — regenerate `.windsurf/skills`, or retire Windsurf and remove `skill-system.json:144`?
2. **c402** — publish-and-pin (needs the submodule owner) or restore the pin?
3. **c403** — `.devin/` and `.openspec-target`: tracked or ignored?
4. **c404** — does this repo need a normalizer at all, having no `internal: true` invariant?

## Warnings carried from the assessment review

- The **30 command/workflow files in c400 are still unverified** — one *skill*
  was inspected; the rest is inference until task 1 runs.
- Two prior changes in this repo fail `openspec validate`. Do not copy them.
- Cross-model review is available (judge `k3`, `verified-distinct`) but requires
  `CLAUDE_PLUGIN_ROOT` exported **until c405 lands** — the fix that removes that
  requirement is the thing this phase is shipping.

## Adversarial review of this plan

Judge `k3`, producer `claude-opus-5`, `verified-distinct`, REST gateway.
**Verdict BLOCK** — 1 CRITICAL, 2 WARNING, 3 SUGGESTION. All six reproduced and
applied.

| # | Sev | Finding | Resolution |
|---|---|---|---|
| 1 | CRITICAL | c402's acceptance contradicted itself: "porcelain empty" while promising not to touch the submodule's dirty files | Re-measured — and **both the draft and the judge were working from stale facts**. The submodule worktree is now clean; only the pointer differs, so the criterion *is* achievable. Rewritten with the two conditions separated and a re-measure step, because the facts moved twice mid-phase |
| 2 | WARNING | c400 dropped goal 2's `generatedBy` bump from the commit-message requirement | Restored alongside the feature list |
| 3 | WARNING | c401 option B removed a manifest entry without regenerating its consumers | Task added for `generate-skill-system-distribution.js`, `install-system.js`, `skill-matrix.js` — otherwise it creates the drift class this phase exists to clean |
| 4 | SUGGESTION | c401 cited C-03, which governs Codex plugin docs, not harness state | Citation withdrawn; the consistency argument stands on its own |
| 5 | SUGGESTION | c405's sequence omitted `git push` — a fast-forward pull moves nothing outward | Added as step 3 |
| 6 | SUGGESTION | c405's "ls-remote confirms all six" was unsatisfiable since c405 lands no source commit | Restated over c400–c404 |

Finding 1 is the interesting one: the judge caught a real contradiction, and
verifying it disproved the shared premise underneath. Neither party had current
facts. That is an argument for measuring at execute time, which task 1.0 of c402
now requires.

## Change structures emitted

Six OpenSpec changes, **all passing `openspec validate`** — worth noting because
two existing changes in this repo (`change-push-001`, `change-push-002`) do not:

```
change-drift-400-openspec-110-upgrade      harness-mirror-currency
change-drift-401-windsurf-harness          harness-declaration-integrity
change-drift-402-entity-mgmt-submodule     submodule-pin-resolvability
change-drift-403-routine-artifacts         session-artifact-hygiene
change-drift-404-harness-completeness      harness-completeness-gate
change-drift-405-reinstall-verify          installed-surface-verification
```
