---
type: SessionRecord
id: karpathy-session-2ba917578c6f09bf
title: Karpathy session 2ba917578c6f
tags:
- karpathy
- session-learning
sources:
- resource: session:1db84095-da34-4e96-ab6a-dfa84d559c8f
generated:
  by: pk/1.11.0
  at: 2026-11-17T14:34:51.529074+00:00
created_at: 2026-11-17T14:34:51.529074+00:00
updated_at: 2026-11-17T14:34:51.529074+00:00
revision: 0
---

## Delta

`pmpo-skill-creator` is rewired. The changes are uncommitted on branch `feat/pmpo-skill-creator-kbd-certification`, stacked on PR #233, in a scratch worktree that will not survive a reboot.

## What changed

- **Creation runs as a KBD phase.** The controller finds `.kbd-orchestrator/` (running `/kbd-init` if absent) and opens `skill-<name>` as a phase or child phase. The five creator phases map onto KBD stages: Specify → assess and analyze, Plan → spec and plan, Execute → execute, Reflect → the phase boundary, Persist → reflect. KBD's `progress.json` replaces `.creator/` as the record.
- **Anthropic's skill-creator drafts when it is installed.** It writes the description, body and eval cases. The creator takes those drafts but does not follow skill-creator's run-and-iterate loop, because nothing runs before the boundary. Without it, the creator says so and drafts them itself.
- **`skill-certification` is the gate.** At the boundary the order is: static pre-check, adversarial review, then certification. Billed runs need the operator's yes. If the certifier is not installed, the skill is reported as uncertified.
- **A failed level goes to the operator.** They see what failed, the proposed fix, and the impact of fixing, accepting with a recorded exception, or abandoning. A fix re-runs only the affected level, with at most two rounds before asking again.
- **Skills are written to their real destination.** In the pack that is `skills/<domain>/<name>/`, not `dist/`, which is generated output here.

## Defects fixed along the way

- **Hooks format.** The creator's own `hooks/hooks.json` used a format no harness loads and depended on an unset variable, so I deleted it. The template for generated skills now writes the harness format, and the two doc examples were corrected.
- **Hooks check.** `validate-skill.sh` never checked hooks despite its docs saying so. It now does: it passes the new template's output and rejects the old format.
- **Tool names.** `allowed-tools` was `code_interpreter file_system`; it is now `Read Write Edit Bash Glob Grep`.
- **`/validate-skill` demoted.** It now describes itself as a static pre-check and points to `skill-certification`.

## Verification

Nine repository gates pass after regeneration: strict validation, domain plugin coverage, distribution check and test, Codex artifacts, skill matrix, skills index, docs sync and harness adapters. `create-skill` and `validate-skill` certify at C1.

## What you should know

- **The new workflow has not been run.** It is instructions and prompts; no model has followed it end to end, and there are no eval cases for it.
- **`pmpo-skill-creator` itself fails C0.** Its `model_routing` frontmatter field is not allowed by the specification, but `kbd-model-resolve.sh` and the repo validator both read it, so I left it. This is the same known issue as the other skills carrying that field.
- **The KBD command mapping is from the stage skills' descriptions.** I did not read each stage skill in full, so a mismatch in how `/kbd-new-child` or `/kbd-spec` expects to be driven is possible.
- **Not rewritten:** the Specify, Plan and Persist prompts keep their original steps. Persist gained a section making the KBD reflection the primary record; its provider steps remain as an optional secondary record.
- The process plugin goes from 1.9.0 to 1.10.0.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 1db84095-da34-4e96-ab6a-dfa84d559c8f
- Captured: 2026-11-17T14:33:49.548428Z
- Project: /Volumes/T7/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
