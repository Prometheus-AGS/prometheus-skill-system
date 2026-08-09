---
title: Use Cases
description: Six starting situations, what the bootstrap does in each, and what it deliberately leaves for a human.
---

# Use Cases

Six situations, distinguished by what already exists in the repository. Find
yours; the rest do not apply.

| # | Situation | Entry point | Refuses? |
|---|---|---|---|
| 1 | Brand new project, no agent files | `bootstrap.sh` | no |
| 2 | Existing project, no agent files | `bootstrap.sh` | no |
| 3 | Hand-written `CLAUDE.md`, no v3 | `bootstrap.sh` | no |
| 4 | Base Rules v3, whole file | `migrate.sh` | bootstrap exits 2 |
| 5 | v3 embedded in a larger file | `migrate.sh` | bootstrap exits 2 |
| 6 | Already bootstrapped | `bootstrap.sh` re-run | no, idempotent |

---

## 1. Brand new project

```bash
bash "$SK/scripts/bootstrap.sh" --path . --dry-run
bash "$SK/scripts/bootstrap.sh" --path .
```

Everything is CREATE. `AGENTS.md` is written whole, `CLAUDE.md` becomes a symlink
to it, and stack rules follow whatever manifest files exist.

If no stack is detected the run still succeeds and reports that no stack rules
were installed. Pass `--stacks` when the manifests are not at the root.

Expected result: `PASS 10 / FAIL 0`.

---

## 2. Existing project, no agent files

Identical to case 1 in mechanism, different in what you do afterward. The
repository has conventions that live in people's heads, in the README, or in CI
config. The bootstrap cannot see them.

After it runs, do this once:

1. Put the real build, test, and lint commands into `.claude/rules/<stack>.md`.
   The templates ship with generic commands and a comment telling you to replace
   them. A rules file describing commands nobody runs is worse than no file.
2. Put the pins that matter into `versions.toml`.
3. Write the first `.prometheus/decisions.md` entry describing why the project
   is structured the way it is.

---

## 3. Hand-written `CLAUDE.md`, no v3 content

The common case for a repo that adopted Claude Code early.

```bash
bash "$SK/scripts/bootstrap.sh" --path . --dry-run   # read the diff first
```

`AGENTS.md` absent, `CLAUDE.md` a real file with content:

- `AGENTS.md` is CREATEd with the managed region.
- `CLAUDE.md` is **not** replaced. `@AGENTS.md` is prepended and every existing
  byte is preserved. Reported as `import prepended; prose kept, NOT shrunk`.

If `AGENTS.md` already exists and is not v3, the managed region is appended below
your prose rather than replacing it.

**Your large file is not shrunk, and that is deliberate.** Deleting rules someone
wrote against an observed failure is not a decision a script should make.

To act on it, reduce by hand and measure:

1. `/context` first. Record resident tokens.
2. Move tier ladders, taxonomies, and schemas into `.claude/rules/` and skills.
3. For each remaining line ask whether removing it would cause a mistake.
4. `/context` again. Re-run a fixed task set. Compare pass rate, not feel.

A reduction that lowers token cost and lowers pass rate is a regression, and only
the second measurement tells you which one you got.

**Note the double-load.** `@AGENTS.md` resolves at launch, so `AGENTS.md` is
loaded twice while `CLAUDE.md` remains a real file. `verify.sh` reports the
combined figure. Once the prose is migrated or removed, replace `CLAUDE.md` with a
symlink to end it.

---

## 4. Base Rules v3, whole file

```bash
$ bash "$SK/scripts/bootstrap.sh" --path .
REFUSED: AGENTS.md looks like a Prometheus Base Rules v3 file (45 rule IDs found).
bootstrap exit=2
```

Detection uses two independent signals — a title match and at least five
`**X-N ·**` rule IDs — so a file that merely *mentions* a rule ID is not mistaken
for the constitution.

```bash
bash "$SK/scripts/migrate.sh" --path .            # report
bash "$SK/scripts/migrate.sh" --path . --apply    # archive + migrate
```

**Report** maps every rule ID in `references/migration-map.tsv` to its
destination, lists tool-owned regions, and lists project-added headings.

**Apply** archives the original to
`.prometheus/knowledge/AGENTS.pre-migration-<date>.md`, writes
`.prometheus/MIGRATION-REPORT.md`, removes the file, and runs the bootstrap.
Nothing is deleted.

### Both agent files are handled

If `CLAUDE.md` is also a real file carrying v3 — common, since the two are often
kept as copies — it is archived separately and removed, so bootstrap can replace
it with a symlink.

This was a defect once. Detection ran on `AGENTS.md` only, so a v3 `CLAUDE.md`
received an `@import` above it and the session loaded new rules and the old
constitution together — the exact state the `AGENTS.md` branch exits 2 to prevent,
reached through the other entry point.

### Some rules become enforcement

They are absent from the new file rather than condensed:

| v3 | Now |
|---|---|
| A-9 tier discipline | `tier-guard.sh` |
| A-10 single-writer | `single-writer.sh` |
| E-1, E-5 sycophancy gate | `sycophancy-gate.sh` |
| E-2 critic isolation | `artifact-critic` subagent |
| §0, F-4 bootstrap and re-anchor | `reanchor.sh` |

The report records each, so absence reads as intentional rather than as a gap.

---

## 5. v3 embedded in a larger file

The realistic case, and the one that produced the most fixes.

A mature repository's `AGENTS.md` is often: project rules, then v3 pasted in the
middle, then several fenced regions written by other tools. One measured example
ran 4,720 words with v3 starting at line 102.

Migration handles the three parts differently.

**Project prose above and below v3** — archived, and every non-canonical `##`
heading is listed in the report with its line number in the archive. It is
**neither carried over nor discarded**, because a script cannot tell a
load-bearing client constraint from an expired note. Placing it is your step.

**Tool-owned fenced regions** — `<!-- agent-rules:start v1 -->`,
`<!-- uiux-routing:start v1 -->`, `<!-- zed-workspace:begin -->` and any other
well-formed `<!-- name:start|begin -->…<!-- name:end -->` pair — are
self-delimited and owned elsewhere, so they are **carried over verbatim** below
the managed region with markers intact. Their owning tools can still re-inject.
They are excluded from the human-placement list and from the managed word budget.

**The v3 skeleton** — mapped and replaced.

```
Tool-owned regions carried over verbatim: 3
  agent-rules
  uiux-routing
  zed-workspace
Project-added headings needing a human: 6
Second agent file also carries v3: CLAUDE.md (45 rule IDs) — will be archived and removed
```

### Content buried inside the v3 block

Known gap, stated plainly. The residue detector scans `##` headings. Project
content added *inside* a v3 section — a paragraph appended under Appendix C, for
instance — is archived but **not surfaced in the report**.

If your v3 block was edited rather than pasted verbatim, diff the archive against
the canonical v3 before you commit.

---

## 6. Already bootstrapped

Re-running is safe and is how you pick up skill updates.

```bash
$ bash "$SK/scripts/bootstrap.sh" --path .
SKIP     AGENTS.md          managed region already current
SKIP     CLAUDE.md          already a symlink
SKIP     .claude/settings.json   already wired
```

`AGENTS.md` stays byte-identical when nothing changed. Content outside the markers
is never touched, so a `## Project rules` section you added survives every re-run.

Re-run to:

- **Switch profile.** `--profile lean` re-splices the region without the execution
  scaffold. One marker pair, so switching is a re-splice, not a second region.
- **Restore a deleted hook.** `--force` re-copies hooks, rules, and settings.
- **Pick up a skill fix.** Repos bootstrapped before a fix self-heal — the
  settings-backup gitignore rule is added on the next run.

---

## What is never automatic

| Left to you | Why |
|---|---|
| Project prose from a migrated file | A script cannot tell a live constraint from an expired note |
| Shrinking an existing large `CLAUDE.md` | Deleting someone's hard-won rule is not a script's call |
| Real build commands in stack rules | The templates are generic by construction |
| `versions.toml` contents | `Edit(versions.toml)` is denied by design; it is the deliberate record |
| The model fleet table | Only you know which models read the repo |
| Switching to `lean` | Requires a measured entry; `verify.sh` fails a lean repo without one |
