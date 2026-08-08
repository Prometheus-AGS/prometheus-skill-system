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
bash scripts/bootstrap.sh --path /path/to/project --dry-run
bash scripts/bootstrap.sh --path /path/to/project
bash scripts/verify.sh    --path /path/to/project
```

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
Raising the fraction buys headroom; it is not a fix. Confirm with `/doctor`
that zero descriptions are dropped, and gate the long tail behind plugins.

## Verification

`scripts/verify.sh` asserts, reporting each as PASS, FAIL, or SKIP:

- `AGENTS.md` present, and its marker pair well-formed
- `CLAUDE.md` resolves to `AGENTS.md`, or contains the import line
- every file in `.claude/hooks/` is executable
- `settings.json` parses (SKIP when `jq` is absent, never PASS)
- `.prometheus/` exists with the five expected entries
- resident word count against the ceiling for the declared profile
- the declared profile matches what is actually in the file
- `lean` is backed by a measured fleet entry

SKIP is never counted as PASS. A check that could not run is unverified, and
reporting it as passing is how a gate becomes decorative.

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
- [references/settings.template.json](references/settings.template.json)
- [references/rules-rust.md](references/rules-rust.md)
- [references/rules-typescript.md](references/rules-typescript.md)
- [references/rules-flutter.md](references/rules-flutter.md)
