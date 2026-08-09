---
name: prometheus-context-bootstrap
description: >
  Scaffold the Prometheus agent structure into a new or existing project — a
  portable AGENTS.md carrying compaction-surviving invariants, CLAUDE.md
  pointing at it, path-scoped rules per detected stack, deterministic hooks for
  tier discipline and single-writer builds, .prometheus append-only learning,
  and a skill-budget-safe settings.json. Profile-aware for mixed model fleets:
  includes an execution scaffold by default for non-frontier models, omits it
  only when measured. Creates what is absent, splices a marked region into what
  exists, and never overwrites operator prose.
license: MIT
version: '1.0.0'
compatibility: bash >=4, git; jq optional
metadata:
  author: prometheus
  version: '1.0.0'
  category: process
  tags: [bootstrap, context-engineering, agents-md, claude-md, hooks, scaffold]
---

# Prometheus Context Bootstrap

Standing policy that is not in the root context file does not survive
compaction. Rules that exist only as prose are advisory, and adherence falls as
instruction count rises. This skill installs the structure that addresses both:
a resident file for invariants, on-demand layers for everything else, and hooks
for the constraints prose cannot enforce.

It does not assume which model is reading. See **Profiles** below — that choice
matters more than any other setting here.

## When to use

- Starting a repository that agents will work in.
- Adopting an existing repository that has a large `CLAUDE.md` and no
  deterministic enforcement.
- Adding a second harness (Codex, Cursor, Gemini CLI, Cline) to a project whose
  context files were written for Claude Code alone.

Do not use it to author project-specific rules. It installs the frame. The
stack-specific and domain-specific content is yours to write afterward.

## What it composes with

This skill does not reimplement work that already exists in the pack.

| Concern | Owner | Relationship |
|---|---|---|
| `.kbd-orchestrator/project.json`, `constraints.md` | `kbd-init` | Bootstrap writes a waypoint stub only. Run `/kbd-init` after. |
| Karpathy + Claude Code rule packs in agent files | `kbd-inject-agent-rules` | Same `<!-- pack:start v1 -->` marker contract, different pack name. Both regions coexist in one file. |
| Sycophancy detection | `sycophancy-correction` | The installed Stop hook calls it and degrades to exit 0 when the binary is absent. |

It carries its own marker splice rather than calling
`kbd-inject-agent-rules --pack prometheus-base` for one reason: that script
renders a static cached template, while this region is generated per project
from detected stacks. A static pack cannot carry a Rust tier ladder into a Rust
repo and a Flutter one into a Flutter repo.

## Run it

```bash
bash scripts/migrate.sh   --path /path/to/project            # report only
bash scripts/migrate.sh   --path /path/to/project --apply    # archive + migrate
bash scripts/bootstrap.sh --path /path/to/project --dry-run
bash scripts/bootstrap.sh --path /path/to/project
bash scripts/verify.sh    --path /path/to/project
bash scripts/skill-budget.sh --path /path/to/project
```

If the project already carries a Prometheus Base Rules v3 file, start with
`migrate.sh`. `bootstrap.sh` refuses to run against one — see Migration.

| Flag | Meaning |
|---|---|
| `--path <dir>` | Project root. Default `.`. |
| `--dry-run` | Print the plan and a `diff -u` for agent files. Write nothing. |
| `--force` | Re-copy hooks, rules, and settings that already exist. Never touches operator prose. |
| `--stacks a,b` | Override detection: `rust`, `typescript`, `flutter`, `go`, `python`. |
| `--profile p` | `mixed` (default), `strict`, or `lean`. See Profiles. |
| `--no-hooks` | Skip `.claude/hooks` and the settings hook block. |

| Exit | Meaning |
|---|---|
| 0 | Plan applied, or dry run completed |
| 1 | Usage error, or path is not a directory |
| 2 | Refused: corrupt marker pair in an agent file |

## Profiles

`AGENTS.md` is per repository, not per model. When Opus 5, Kimi, MiniMax, and
GPT all read the same file, it cannot be tuned to any one of them — the weakest
reader in the fleet governs its content.

The costs are asymmetric, which decides the default:

| | Frontier model | Smaller model |
|---|---|---|
| Scaffold present | wasted tokens, some over-verification | none |
| Scaffold absent | none | fabricated APIs, elided code, skipped checks |

Wasted tokens are recoverable. A fabricated identifier that reaches a commit is
not. So `mixed` ships the execution scaffold by default and `lean` is opt-in.

`verify.sh` fails a repo that declares `lean` without a measured entry in
`.prometheus/model-fleet.md`. Going lean is a measurement result, not a
preference.

Profile is one marker region, not two, so switching is a re-run:

```bash
bash scripts/bootstrap.sh --path . --profile lean
```

Full procedure and per-model starting assumptions:
[references/MODEL-PROFILES.md](references/MODEL-PROFILES.md).

## Migration from Base Rules v3

A v3 file is not operator prose. Appending the managed region to one leaves two
constitutions in resident context, overlapping in substance and differing in
wording — measured at 3,133 to 4,521 words on the canonical file. That degrades
adherence to both, so `bootstrap.sh` **exits 2** rather than doing it.

`migrate.sh` handles it in two passes.

**Report** (default, writes nothing). Detects v3 by two independent signals — a
title match and at least five `**X-N ·**` rule IDs — so a file that merely
mentions a rule ID is not mistaken for the constitution. Then it maps every
rule ID in `references/migration-map.tsv` to its destination and reports
presence, plus every project-added heading.

**Apply** (`--apply`). Archives the original to
`.prometheus/knowledge/AGENTS.pre-migration-<date>.md`, writes
`.prometheus/MIGRATION-REPORT.md`, then runs `bootstrap.sh`. Nothing is deleted.

### What a script cannot do

G-2 permits projects to add stricter local rules. Those additions cannot be
classified mechanically — a script cannot tell a load-bearing client constraint
from an expired note. So they are **neither carried over nor discarded**. Every
non-canonical H2 heading is listed in the report with its line number in the
archive, for a human to place.

Silently keeping them would rebuild the bloat. Silently dropping them would
lose a rule someone wrote against a real failure. Listing them is the only
honest option.

### Tool-owned regions travel automatically

Regions written by another tool — `<!-- agent-rules:start v1 -->`,
`<!-- uiux-routing:start v1 -->`, `<!-- zed-workspace:begin -->` — are
self-delimited and owned elsewhere, so they are carried into the new
`AGENTS.md` verbatim, below the managed region, with markers intact. Their
owning tools can still re-inject over them.

They are reported by region name and excluded from the human-placement list,
because a heading inside a fenced region is not hand-written prose.

Carried regions are excluded from the managed-region word budget. `verify.sh`
reports them separately (`1396 managed words + 466 carried`) rather than
failing a repo for content it does not own.

### Both agent files are handled

If `CLAUDE.md` is a real file that also carries v3 — common, since the two are
often kept as copies — it is archived alongside `AGENTS.md` and removed, so
bootstrap can replace it with a symlink. Leaving it in place and prepending an
import would load the new rules and the old constitution together, and would
also double-load `AGENTS.md`, since imports resolve at launch.

A symlink avoids both. `verify.sh` checks that no v3 rule IDs survive in
`CLAUDE.md` and reports the combined resident total across both files.

### Existing settings.json is merged, not skipped

When `.claude/settings.json` exists, the hook wiring and skill budget are
merged into it with `jq`. Only absent keys are added; existing keys, including
`mcpServers` and `permissions`, are never overwritten. A timestamped `.bak` is
kept.

Skipping instead would leave hooks installed and unreferenced — the prose gone
and nothing enforcing it. Without `jq` the merge cannot run, so it is reported
as a SKIP with a warning rather than assumed.

### What stops being prose

Several v3 rules become enforcement, which is why they are absent from the new
file rather than merely condensed:

| v3 | Now enforced by |
|---|---|
| A-9 tier discipline | `tier-guard.sh` — exits 2 on a Tier 3 command outside a release gate |
| A-10 single-writer | `single-writer.sh` |
| E-1, E-5 sycophancy gate | `sycophancy-gate.sh` |
| E-2 critic isolation | `artifact-critic.md` subagent |
| §0, F-4 bootstrap and re-anchor | `reanchor.sh` |

A hook installed but not wired into `settings.json` enforces nothing.
`verify.sh` checks exactly that.

### Opening the Tier 3 gate

`tier-guard.sh` reads `.status` from the waypoint, not `.phase`. Measured across
the estate, `.phase` holds a phase identity (`uar-uiux-full-migration-2026-08`)
and `.status` holds the lifecycle (`running`, `execute_ready`, `completed`).
An earlier version matched `.phase` against `milestone|release|certify`, which
no waypoint in the estate could satisfy — it blocked Tier 3 unconditionally
with no reachable unblock path.

Tier 3 is allowed when `.status` begins with `completed`, `release`, `certify`,
`milestone`, or `delivery`, or through either explicit opt-in:

```bash
PROMETHEUS_TIER3=1 cargo build --release   # one command
touch .kbd-orchestrator/tier3.allow        # this session; delete when done
```

Both are deliberate and reversible. Editing the waypoint to get past the gate
is not — it is the position record, not a permission switch.

## What it writes

```
AGENTS.md                    # single source of truth; ~750 words lean, ~1200 mixed
CLAUDE.md                    # symlink to AGENTS.md, or an @AGENTS.md import line
.claude/settings.json        # permissions, skill budget, hook wiring
.claude/hooks/*.sh           # tier-guard, single-writer, sycophancy-gate, reanchor
.claude/rules/<stack>.md     # path-scoped; costs nothing until a matching file is read
.claude/agents/artifact-critic.md
.prometheus/                 # session-log, decisions, gotchas, postmortems/, knowledge/
.prometheus/model-fleet.md   # which models read this repo, and the profile decision
.kbd-orchestrator/current-waypoint.json
versions.toml                # stub, if absent
```

## The three write modes

Every path falls into exactly one, and the report names which applied.

**CREATE** — the file is absent. Written whole.

**SPLICE** — `AGENTS.md` exists. Only the region between
`<!-- prometheus-base:start v1 -->` and `<!-- prometheus-base:end -->` is
replaced. Bytes outside it are preserved. Written to a temp file and moved, so
an interrupted run never leaves a half-rewritten agent file.

**SKIP** — the file exists and is not marker-managed. Reported, not touched.
`--force` promotes SKIP to CREATE for hooks, rules, and settings only. It never
promotes SKIP for `AGENTS.md`, `CLAUDE.md`, or anything under `.prometheus/`,
because those hold operator prose and append-only history.

## Refusals

- A start marker with no matching end marker → exit 2. Repair by hand.
- More than one start marker → exit 2. Deduplicate by hand.
- An end marker with no start → exit 2.
- `CLAUDE.md` exists as a real file with content → it is not replaced with a
  symlink. An `@AGENTS.md` line is prepended instead, if not already present.

## Existing projects: what does not happen automatically

A large `CLAUDE.md` is not shrunk. The bootstrap adds the import line and
leaves the prose in place, because deleting rules an operator wrote against
observed failures is not a decision a script should make.

To act on it, run the reduction by hand and measure:

1. `/context` first. Record resident tokens.
2. Move tier ladders, taxonomies, and schemas into `.claude/rules/` and skills.
3. For each remaining line, ask whether removing it would cause a mistake.
4. `/context` again. Re-run a fixed task set. Compare pass rate, not feel.

A reduction that lowers token cost and lowers pass rate is a regression, and
only the second measurement will tell you which one you got.

## Skill budget

`settings.json` is written with `skillListingBudgetFraction: 0.02`. At the 1%
default, a large profile silently drops skill descriptions: the name stays, the
description vanishes, and auto-triggering stops for whatever was dropped.
Eviction favours recently used skills, so a newly installed skill scores zero
and goes dark first — which is what "the skill exists, tested fine, didn't fire"
actually is.

**Measure it; do not assume it.** A repo-local count is the wrong denominator.
The budget spans every scope the harness loads, and user plus plugin scopes
dominate. Measured on one estate 2026-08-09:

| Scope | Skills | Description chars |
|---|---|---|
| repo `.claude/skills` | 56 | 13,078 |
| user `~/.claude/skills` | 916 | 251,125 |
| plugins `~/.claude/plugins` | 1,295 | 388,816 |
| **total** | **2,267** | **653,019 (~163,000 tokens)** |

Against a ~4,000-token budget that is **~41x over**. Counting only the 56
repo-local skills reports ten-fold headroom that does not exist.

```bash
bash scripts/skill-budget.sh --path .            # human-readable
bash scripts/skill-budget.sh --path . --json     # machine-readable
```

Exit 0 within budget, 1 over, 2 could not measure. `verify.sh` runs it and
reports SKIP — never PASS — when `python3` is absent.

**Raising the fraction does not fix a multiple this large.** At 41x, moving 0.02
to 0.03 changes nothing that matters. Gate the long tail behind plugins, enable
one profile at a time, and route by name through a skill router for the rest.

### Parse frontmatter properly

`description: >` is a YAML folded block scalar; the text continues on the lines
below. `grep -m1 '^description:'` reports a one-character description and is
simply wrong — a measurement pass built on it will invent dead skills that are
fine and miss real ones. `skill-budget.sh` uses PyYAML, with a folding fallback
when it is unavailable.

## Verification

`scripts/verify.sh` asserts, reporting each as PASS, FAIL, WARN, or SKIP:

- `AGENTS.md` present, and its marker pair well-formed
- `CLAUDE.md` resolves to `AGENTS.md`, or contains the import line
- every file in `.claude/hooks/` is executable
- `settings.json` parses (SKIP when `jq` is absent, never PASS)
- the skill-listing budget, measured across repo, user, and plugin scopes
- `.prometheus/` exists with the five expected entries
- resident word count against the ceiling for the declared profile
- the declared profile matches what is actually in the file
- `lean` is backed by a measured fleet entry

SKIP is never counted as PASS. A check that could not run is unverified, and
reporting it as passing is how a gate becomes decorative.

WARN is separate from FAIL and does not change the exit code. It marks a real
finding the repo cannot fix on its own — the machine-wide skill budget is the
case that matters. Reporting it as FAIL would leave every repo permanently red,
and a gate that always fails stops being read: the same decorative failure by a
different route.

## After bootstrap

```bash
/kbd-init                 # project.json + constraints.md
/doctor                   # confirm no skill descriptions dropped
```

Then write the project-specific content: pins in `versions.toml`, the real
build and test commands in `.claude/rules/<stack>.md`, and the first entry in
`.prometheus/decisions.md`.

## The uncomfortable thing

The `lean` profile bets that the model supplies the judgment the omitted prose
used to supply. Anthropic's guidance for its newest models supports that bet for
those models specifically. It does not generalize, and applying it to a mixed
fleet is how a rules file ends up optimized for whichever model happened to
write it.

The default here is set against that failure, and it has its own cost: every
frontier-model session in a `mixed` repo pays for scaffolding it did not need,
and the over-verification that scaffolding induces is a documented, measurable
quality loss on those models. If the fleet consolidates, `mixed` becomes the
wrong default and nothing in this skill will notice — `verify.sh` checks that
the profile is *consistent*, never that it is *correct*.

The hooks carry a different risk. A hook with a bad path fails closed and
blocks legitimate work, or fails open and enforces nothing — and unlike prose,
nobody re-reads it each turn to notice. Run `verify.sh` after any edit under
`.claude/hooks/`, and keep `disableAllHooks` in mind as the kill switch.

## References

- [references/AGENTS.base.md](references/AGENTS.base.md) — the resident invariants
- [references/AGENTS.scaffold.md](references/AGENTS.scaffold.md) — execution scaffold for non-frontier models
- [references/MODEL-PROFILES.md](references/MODEL-PROFILES.md) — profile choice and measurement procedure
- [references/migration-map.tsv](references/migration-map.tsv) — v3 rule ID to destination
- [references/settings.template.json](references/settings.template.json)
- [references/rules-rust.md](references/rules-rust.md)
- [references/rules-typescript.md](references/rules-typescript.md)
- [references/rules-flutter.md](references/rules-flutter.md)
