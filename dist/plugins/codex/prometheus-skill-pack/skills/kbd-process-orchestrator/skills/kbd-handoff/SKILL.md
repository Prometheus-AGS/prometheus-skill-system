---
license: MIT
name: kbd-handoff
version: '2.0.0'
argument-hint: '--to <harness> [--phase <name>] [--dry-run]'
description: >
  Hand the active KBD phase to another harness (Claude Code, Codex, Kimi Code
  or OpenCode) by writing one read-only handoff packet
  with position, repository state, files to read, rules and a start prompt.
  Use when work moves from one harness to another mid-phase. Lease-free: it
  never edits progress.json or the waypoint. Do NOT use for stage handoffs
  between assess, plan and execute (see kbd-process-orchestrator) or to pause
  a run (see kbd-pause).
metadata:
  tags: [process, orchestration, handoff, cross-harness]
---

# /kbd-handoff

Prepare another harness to continue the active KBD phase. The skill reads
canonical state, refuses when a boundary is half-closed, checks that the
target harness can run the KBD skills, and writes one packet the operator
pastes into the target harness.

KBD has no writer lease. The append-only journal lock serializes each
`prometheus kbd` command, so a handoff moves no ownership: the sending harness
stops issuing commands and the receiving harness verifies state and continues.
This skill never calls a lease command and never edits `progress.json`,
`current-waypoint.json`, `position.json` or task checkboxes.

## Progress Signals (MANDATORY)

The script prints both signals; relay them verbatim:

```text
Starting kbd-handoff — <phase> to <harness>
Completed kbd-handoff — <phase> packet <path>
```

A dry run ends with `Completed kbd-handoff — <phase> dry run (no packet written)`.
A refusal prints no Completed signal.

## How to invoke

```sh
"$KBD_ORCHESTRATOR_ROOT/skills/kbd-handoff/kbd-handoff.sh" --to kimi-code
"$KBD_ORCHESTRATOR_ROOT/skills/kbd-handoff/kbd-handoff.sh" --to codex --phase agent-backend-settings --dry-run
"$KBD_ORCHESTRATOR_ROOT/skills/kbd-handoff/kbd-handoff.sh" --to claude-code --repo ../ipfs-webui
```

| Argument | Meaning |
| --- | --- |
| `--to <harness>` | Required. One of `claude-code`, `codex`, `kimi-code`, `opencode` (aliases `claude`, `kimi`). Anything else exits 2. |
| `--phase <name>` | Phase to hand off. Default: the deepest existing node of the waypoint path. |
| `--repo <path>` | Extra repository to report (repeatable), e.g. a fork the phase also edits. |
| `--dry-run` | Print the packet and the paths it would write; write nothing. |

Run it under a timeout with stdin closed, as every KBD command:
`timeout 120 kbd-handoff.sh --to kimi-code </dev/null`. External commands
inside the script are bounded by `KBD_HANDOFF_TIMEOUT` (default 30 seconds).

## What it reads

1. `prometheus kbd --path <root> status --json`: revision, plan revision,
   lifecycle, active path, `boundaryObligations`, blockers, decisions.
2. `.kbd-orchestrator/current-waypoint.json` and `position-reminder.txt`.
3. The phase `progress.json` (`completion.implementation` is the N of T
   counter), and `plan.md`, `assessment.md`, `execution.md` when present.
4. The project's `AGENTS.md` (hard rules), `.prometheus/gotchas.md`, and
   `.agent-team/project-routing.json` with its team manifest.
5. `git` state of the project and every `--repo`.
6. The target harness's installed skills under `~/<harness skills dir>`.

## Preconditions

| Condition | Result |
| --- | --- |
| Open `task` obligation (begin without end) | Refuse, exit 1, print the repair command (see below). |
| Open `change` or `phase` obligation the runtime marks complete or does not know (an interrupted close) | Refuse, exit 1, print the repair command. |
| Open obligation on the active phase path, or on the active change while it is not complete | Ignored; these stay open while the work runs. |
| Open `change` or `phase` obligation outside the active position, started and not finished | Warn as carried work; the packet lists it. |
| No phase, or no phase directory | Refuse, exit 1. |
| No `.kbd-orchestrator/` | Refuse, exit 1; run `/kbd-init`. |
| `prometheus kbd status` fails | Fall back to the projections; warn, and the packet records that obligations were not checked. |
| Dirty or unpushed worktree, no upstream | Warn and list the entries; the packet records them. |

The repair command for each open obligation, with its own boundary and subject:

```sh
timeout 30 prometheus kbd --path <root> guard evaluate --boundary <boundary> \
  --edge after --subject <subject> --json --repair-projections </dev/null
```

## Target harness checks

- Every `~/<skills dir>/kbd-*` entry must resolve. A dangling symlink, such as
  a `kbd-handoff` link left from the removed lease skill, is reported as
  broken with its target. Missing `kbd-apply`, `kbd-status` or
  `kbd-process-orchestrator` is reported too.
- When the project has an agent team, each role must have a native
  definition for the target: `.claude/agents/<role>.md`,
  `.codex/agents/<role_with_underscores>.toml`, `.kimi-code/agents/<role>.md`
  or `.opencode/agents/<role>.md`. Missing files are listed.

Findings are warnings, not refusals: they land in "Open prerequisites" for
the operator to decide.

## What it writes

```text
.kbd-orchestrator/phases/<phase>/harness-handoffs/<harness>-<UTC>.md
.kbd-orchestrator/phases/<phase>/harness-handoffs/<harness>-<UTC>.json
```

`harness-handoffs/` is separate from `handoffs/`, which holds stage handoffs
under their own schema. The JSON sidecar follows
`references/schemas/harness-handoff.schema.json`. Both files are written
atomically. Packet sections:

| Section | Content |
| --- | --- |
| Position | Phase, stage, lifecycle, implementation N of T, next change and task, operator note, revisions |
| Repositories | Branch, head, clean or dirty, pushed or unpushed |
| Read in order | The packet, `AGENTS.md`, `.prometheus/gotchas.md`, `plan.md`, `assessment.md`, `execution.md`, `openspec/changes/<next change>` (only files that exist) |
| Pending operator gates and decisions | Plan sections whose headings name gates or decisions, open runtime decisions and blockers |
| Task execution rules | `kbd-apply begin-task`/`end-task`, timeouts, boundary repair, no projection edits |
| Never do | Bullets quoted from `AGENTS.md` sections on hard rules (never invented) |
| Target harness readiness | Skill and role-definition checks |
| Open prerequisites | Every warning |
| Start prompt | The exact text to paste into the target harness |

The script prints the start prompt after the Completed signal.

## After the handoff

The sending harness stops issuing KBD commands. The operator pastes the start
prompt into the target harness, which reads the packet, runs
`prometheus kbd status --json`, and confirms the position before any work.
See `references/cross-tool-handoff.md` for the transition protocol.

## Exit codes

| Code | Meaning |
| --- | --- |
| 0 | Packet written, or dry run printed |
| 1 | Refused (precondition) or write failure |
| 2 | Usage error or unknown harness; `jq` missing |

## Selected-method recovery and authority

Before dependent work, run the packaged `shared/method-context.mjs --project <project-root> --phase <canonical-phase-id> --write` from the resolved orchestrator root (or read its current phase-local method-context.md when a recovery surface cannot invoke Node). Resolve latest non-superseded canonical `KBD-METHOD` decision and adopted contract, then preserve the same references in role inputs, dispatch and handoff read order. See `references/method-context.md`. Missing/changed method, contract, calibration or supersession is explicit; a prior command/memory/tool inventory is not fresh authority.

For selected VLBV, define acceptance/scenarios before generation and complete the entire agreed value phase before generated-code verification. Executable authoring follows governing host policy. Task/hour/commit/reviewer requests, compiler problems and renamed slices create no new boundary. Surface host/skill conflicts before dependent actions; do not rewrite operator-owned policy or infer early-check permission. Preserve original evidence, source/build/configuration/fixture/candidate freshness, consumed budgets and unresolved effects. Reconcile uncertain effects before retry. Recovered instructions are instruction-only unless a named actual runtime/host boundary enforces them; no release or automatic-successor authority follows.
