# ASSESSMENT: openspec-mirror-drift-cleanup

Repository: `prometheus-skill-system` (local dir `prometheus-skill-pack`)
Date: 2026-08-23
Branch: `kbd/openspec-mirror-drift-cleanup` @ `2be15c3`
Working tree: **98 dirty files**, untouched throughout this assessment (md5 `71445c7…`)

## 0 · Method

Adversarial review is genuinely cross-model for the first time: judge `k3`,
critic `MiniMax-M3`, producer `claude-opus-5`, gateway `http://localhost:4000/v1`,
`verified-distinct`. That capability exists because of PR #55 (`c0d2de1`), the
merged fix whose reinstall this phase is blocking on — the tool is reviewing the
work that will finish shipping it.

Every claim below is grounded in an executed command.

## 1 · Two of my own framings did not survive verification

`goals.md` was written before this assessment and got two things wrong. Both are
corrected here rather than quietly carried forward.

### 1.1 Concern A is **not** "mechanical version churn"

`goals.md` called A a `generatedBy` bump — 70 files of noise to sweep into one
commit. The diff says otherwise:

```
$ git diff --stat -- .agent .agents .cursor .opencode
70 files changed, 4016 insertions(+), 1473 deletions(-)

$ git diff -- .cursor/skills/openspec-explore/SKILL.md | grep -cE '^[-+]'
54            # of which exactly 2 are generatedBy
```

The other 52 lines are **real upstream features**: `--store` selection with
sticky-flag semantics, an `allowed-tools: Bash(openspec:*)` frontmatter key,
`schemas`/`view` added to the store-scoped command list, and new scaffold
guidance. A is an **upstream upgrade to adopt deliberately**, not noise. It
still belongs in its own commit, but the commit message must say what shipped.

### 1.2 Concern C is worse than "adopt or pin back"

`goals.md` framed the submodule as a choice between two safe options. It is not
safe in its current state:

```
$ git -C skills/imported/prometheus-entity-management branch --show-current
main-takeover-kimi                       # not main
$ git -C … rev-parse --short HEAD
55dc8dc                                  # pinned at HEAD: 1c40eaa
$ git -C … branch -r --contains HEAD
                                         # (empty — on NO remote branch)
$ git -C … status --porcelain | wc -l
52                                       # the submodule is itself dirty
```

Committing this pointer would pin the parent repository to a commit that exists
on no remote, on a branch named `main-takeover-kimi`, inside a working tree with
52 uncommitted files of its own. A fresh clone could not resolve it.

**This is the highest-risk item in the phase and the goals understated it.**

## 2 · Concern-by-concern

### A — OpenSpec mirror upgrade · 70 files · ADOPT, deliberately

**Composition, counted rather than inferred** (an earlier draft said "70 mirror
files"; only 40 are skills):

```
$ git status --porcelain | grep '^ M' | ... | sort | uniq -c
  10 .agent/skills/      10 .agent/workflows/
  10 .agents/skills/     10 .cursor/commands/
  10 .cursor/skills/     10 .opencode/commands/
  10 .opencode/skills/
```

**40 `SKILL.md` mirrors + 30 command/workflow files** across four harnesses. The
command files are a distinct artifact class and were not line-inspected; c400
must confirm they carry the same upstream upgrade and not something else.

Confirmed **not** the defect HMA's c300 fixed: that repo's
`check-skill-contracts.mjs` enforces `metadata.internal: true`, so the CLI
stripping it turned a gate red. Here, `grep -rn 'internal: true' scripts/*.mjs`
returns nothing — no gate exists, nothing is red. A is an upgrade, not a repair.

### B — `.windsurf/skills` · 20 deletions · GENUINELY OPEN

The goals leaned toward "the CLI stopped emitting it, so the deletion is
correct." Three findings say otherwise:

| Evidence | Implication |
|---|---|
| `openspec init --tools` lists `windsurf` | still a supported OpenSpec target |
| `skill-system.json:144` declares `{"id":"windsurf","path":".windsurf/skills","mode":"symlink"}` | this repo declares it a managed harness |
| `.windsurf/workflows` — 19 files, still tracked and on disk | only the *skills* half vanished |
| `.agent/skills`, `.cursor/skills` still present on disk | siblings did not lose theirs |

At HEAD `.windsurf/skills` was a real tracked tree (`040000 tree 94c299c4…`), and
it is now simply gone from disk while its declared siblings survive. Nothing
found so far explains *why*, which is precisely why it must not ride along inside
a bulk commit.

### C — submodule · 1 file · DO NOT ADOPT AS-IS

See §1.2. Options, none of which is "commit it":

1. Publish the submodule's branch, re-pin to a commit that exists on a remote.
2. Restore the pinned `1c40eaa` in the parent, leave the submodule checkout alone.
3. Land the submodule's own 52 dirty files first — its own decision, in its own repo.

### Routine · 7 files

3 modified `.prometheus` session logs, 4 untracked (`.devin/`,
`.agents/skills/.openspec-target`, 2 `karpathy-session` wiki files).

**Correction:** an earlier draft claimed a "standing authorization to always
commit `.prometheus/` session logs". That authorization is recorded in the *HMA*
repository's `CLAUDE.md` and does **not** transfer here. This repo's
`constraints.md` C-02 ("No committed secrets") applies instead, and session logs
are exactly the artifact class that can capture a secret without review. A scan
of the current diff finds nothing secret-shaped (0 matches for
`sk-|api[_-]?key|bearer|token=`), but c403 must **run that check**, not inherit
another repo's blanket permission.

`.devin/` and `.openspec-target` are new tool artifacts — decide whether they are
tracked or ignored rather than committing them by reflex.

## 3 · G1's premise needs widening

The phase exists to unblock `update-skill-pack.sh`. Two facts the goals missed:

- **It checks the tree is clean five times**, not once — `grep -c
  require_clean_source` → 5, at before-pull, after-native-refresh, and
  before-doctor-refresh stages. A tree that goes dirty *mid-run* also aborts, so
  any step that regenerates files must be committed before the next stage.
- **It runs `git pull --ff-only`**, so it must execute on `main`. Local `main` is
  `6de8181`; `origin/main` is `c0d2de1` (the merged PR #55). The phase branch is
  not `main`, so the sequence is: land these changes → switch to `main` → pull →
  then run the script.

## 4 · Recommended change set

| # | Change | Kind |
|---|---|---|
| c400 | Adopt the OpenSpec 1.10.0 mirror upgrade (A), message naming the shipped features | mechanical, largest diff |
| c401 | Decide `.windsurf/skills` (B) — regenerate or record the removal | **decision** |
| c402 | Decide the submodule (C) — publish, re-pin, or defer | **decision, highest risk** |
| c403 | Commit session logs; decide `.devin/` + `.openspec-target` tracked vs ignored | routine |
| c404 | Prevention: **normalizer + per-harness completeness assertion** (both halves of goal 5) | **the durable fix** |
| c405 | Land to `main`, `update-skill-pack.sh`, verify `resolver_missing` is live **and that `~/.claude/skills/…` reports `status: ok` with `CLAUDE_PLUGIN_ROOT` UNSET** | the unblock |

c404 is worth its own change precisely because of B: `.windsurf/skills` reached
zero and *nothing noticed*. HMA's c300 built exactly this assertion after the
same class of silent loss.

**Both halves of goal 5 are in scope.** An earlier draft of this table listed
only the assertion and silently dropped the normalizer, which is the wrong
trade: the assertion *detects* a vanished tree, the normalizer *re-applies* what
the external generator strips. Whether this repo needs the normalizer at all is
a real question — unlike HMA it enforces no `internal: true` invariant, so there
may be nothing to re-apply — but that must be **answered in c404**, not assumed
away by omitting it.

## 5 · Open questions for plan

- **B:** is Windsurf still a harness this pack supports? If yes, regenerating is
  the fix and the deletion is a bug. If no, `skill-system.json:144` should also
  be removed — otherwise the manifest keeps promising a tree that will not exist.
- **C:** who owns `prometheus-entity-management`, and is `main-takeover-kimi`
  intended to land? The parent cannot pin unpublished work either way.
- **c404 scope:** assert only that declared harnesses are non-empty, or full
  per-harness expected counts as HMA does?

## 6 · Sycophancy check

- **Two of my own goals are corrected, not defended.** A was mis-framed as churn
  when it is a 4016-line feature upgrade; C was framed as a safe binary choice
  when the pointer is unpublishable.
- **The highest-risk item is named as such** and given three options, none of
  which is the convenient one.
- **B is moved from "probably fine to delete" to "genuinely open"** on the
  strength of three pieces of evidence that contradict the original lean.
- **No claim rests on assertion** — every verdict cites the command that produced
  it, and the 98 files were never modified to make any of it easier.

## 7 · Adversarial review

Judge `k3`, producer `claude-opus-5`, `cross_model_check: verified-distinct`,
`isolation_mode: rest-gateway:http://localhost:4000/v1`. **Verdict PASS**, 3
WARNING + 1 SUGGESTION — all four reproduced and applied above.

The first genuinely cross-model review in this line of work. Its findings were
not cosmetic: it caught the assessment silently dropping half of goal 5, omitting
goal 6's harder acceptance criterion, importing another repository's permission
grant, and inferring the composition of 69 files from one inspected example. A
same-model judge had missed comparable defects eight times running in the HMA
phase.

| # | Severity | Finding | Resolution |
|---|---|---|---|
| 1 | WARNING | c404 dropped the normalizer, keeping only the assertion | Both halves restored; whether the normalizer is needed here is now an explicit c404 question |
| 2 | WARNING | c405 omitted goal 6's "no `CLAUDE_PLUGIN_ROOT`" criterion | Added — it is the criterion that reproduces the original failure |
| 3 | WARNING | Cited a session-log authorization that does not exist in this repo | Corrected; C-02 applies, and c403 must scan rather than assume |
| 4 | SUGGESTION | "70 mirror files" inferred from one example | Counted: 40 skills + 30 command/workflow files; the latter are unverified |
