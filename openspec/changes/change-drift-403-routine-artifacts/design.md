
## D-7 — C-02 scan, re-run 2026-08-23 (task 1.1)

**Re-run, not inherited.** The change body warned that the HMA repo's standing
"always commit .prometheus" authorization does not transfer here; C-02 governs.

Candidate set derived from `git status --porcelain -- .prometheus
.agents/skills/.openspec-target` at scan time (6 paths), not from a hardcoded list:

- `.prometheus/knowledge/wiki/index.md` (M)
- `.prometheus/knowledge/wiki/log.md` (M)
- `.prometheus/knowledge/wiki/kimi-desktop-extensibility-executor-completion-unknown-change.md` (M)
- `.prometheus/knowledge/wiki/karpathy-session-3841ee7d13011f2c.md` (??)
- `.prometheus/knowledge/wiki/karpathy-session-3c67e1717b254152.md` (??)
- `.agents/skills/.openspec-target` (??)

Patterns: `sk-[A-Za-z0-9_-]{16,}`, `api[_-]?key`, `bearer <token>`, `token=`,
`secret[_-]?key`, `ghp_…`, `AKIA…`, `-----BEGIN … PRIVATE KEY`.

**Result: 0 files with hits.**

**POSITIVE CONTROL (required — c301's lesson).** A scan that only ever passes
proves nothing. The same pattern was run against a synthetic file carrying one
planted secret per class; it matched **5/5**. The zero-hit result is therefore
meaningful rather than vacuous. Had the control failed, the scan would have been
treated as not-run.

**Manual read.** Both untracked session records were read in full (58 lines each).
Content is a PR-merge narrative — SHAs, file paths, gate names. No credentials.
The three tracked diffs are wiki bookkeeping: two index links, three log lines,
a timestamp bump, and one sentence reworded ("This entry is" → "This record is").

**Incidental finding (not C-02, not fixed here).** The two session records are
byte-identical apart from `id`, the frontmatter timestamps, and a **4-second**
difference in `Captured:` (12:03:50.72 vs 12:03:54.93) — both cite session
`9db42325`. One session was ingested twice, producing two wiki pages and two index
entries for one event. Recorded as phase debt; deduplicating the ingest is not in
this change's scope and deleting one now would desync `index.md`/`log.md`.

## D-8 — `.devin/`: TRACKED (task 1.3)

**Already settled by c401, not re-decided here.** c401 (`086e92b`, "accept the
windsurf -> devin rename from OpenSpec 1.10.0") tracked all 20 files under
`.devin/skills/` and retargeted `skill-system.json:144` to
`{ "id": "devin", "path": ".devin/skills", "mode": "symlink",
"detect": [".devin", ".windsurf", "command:devin", "command:windsurf"] }`.

**Decision: TRACKED.** Two reasons, both structural rather than stylistic:

1. `skill-system.json` declares `.devin/skills` a distribution target. Gitignoring
   a path the manifest says the pack ships would put the manifest and the tree in
   permanent disagreement — the exact drift class this phase exists to remove.
2. Every sibling harness mirror (`.agents/`, `.claude/`, `.codex/`, `.kimi-code/`,
   `.opencode/`) is tracked. Ignoring only `.devin` would be arbitrary.

`git status --porcelain -- .devin` is already empty, so criterion 2.3 needs no
work for this path. Recording the decision is the deliverable; the tree is correct.

## D-9 — `.agents/skills/.openspec-target`: TRACKED, and NOT a C-01 artifact (task 1.4)

The plan flagged this as possibly generated under C-01. **It is generated, but
C-01 does not apply**, and those are separate questions the plan conflated.

**What it is.** Read the implementation rather than guessing:
`@fission-ai/openspec/dist/core/shared-skill-target.js`. `.agents/skills/` is a
**shared skills root** — verified by probing the CLI's own `AI_TOOLS` table:

    SHARED: .agents -> codex + zed + agents

Three tools render into one directory, and a shared root can hold only one
rendered variant of each skill. `writeSharedSkillTarget()` records the winner, and
`reconcileSharedSkillTargets()` reads it first on every subsequent `openspec
update`. Our file contains `agents`.

**Why it must be tracked.** Without the marker the CLI falls back to
`inferSharedSkillTarget()`, which guesses ownership from generated invocation
syntax: `$openspec-` ⇒ codex, `/openspec-` ⇒ agents. Our tree scores 0 codex-syntax
and 8 generic-syntax files, so today inference happens to return `agents` too.
That is a coincidence, not a guarantee — the value is derived from skill *bodies*,
which change. Ignoring the marker would mean every clone re-derives ownership from
whatever the bodies look like at that moment, and a future body edit could silently
flip the root to `codex` and rewrite all 10 skills in the wrong variant. Ten bytes
pin it deterministically.

**Why C-01 does not apply.** C-01 names its sources exactly: `.claude-plugin/*`,
`.mcp.json`, `hooks/hooks.json`, `scripts/build-codex-plugin.js`, gated by
`npm run validate:codex`. This marker is written by a **third-party CLI**, is a
consequence of a human tool-selection choice, and no generator in this repo
produces it — a repo-wide grep for `openspec-target` returns only this phase's own
docs. Adding it to C-01 would require a generator that can reproduce it, which
would mean reimplementing OpenSpec's tool-selection state. Not worth it for a
10-byte file.

**Decision: TRACKED, outside C-01.** Also noted: `.agents` is the only harness
directory carrying a marker, which is correct — the other five roots
(`.claude`, `.codex`, `.kimi-code`, `.opencode`, `.devin`) are single-tool, and
`writeSharedSkillTarget()` returns early when `sharingRoot.length < 2`.
