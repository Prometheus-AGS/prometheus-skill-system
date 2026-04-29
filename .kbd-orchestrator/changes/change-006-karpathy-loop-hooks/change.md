---
id: change-006-karpathy-loop-hooks
title: UserPromptSubmit pk-focus + Stop forge-reflect + per-skill license
phase: phase-compliance-and-power-multiplier
gaps: [C1, C2, A4]
priority: P1
effort: S
agent: harness-optimizer
evolver_item_id: null
status: proposed
---

# change-006 — Close the Karpathy Loop

## Context

The `prometheus-knowledge` (`pk`) wiki is the strongest differentiator of this
pack but currently only fires when an engineer manually runs `forge enrich` or
`forge reflect`. Two hooks would close the loop automatically:

- **C1**: `UserPromptSubmit` → `pk focus "<keywords>"` injects relevant prior
  knowledge before the LLM sees the user's query.
- **C2**: `Stop` → `forge reflect` so every completed task feeds the learning
  loop without manual prompting.

This change also adds per-skill `license` frontmatter fields (Gap A4) since they
are XS-effort and align thematically with compliance hygiene.

## Scope

In:

- New script `shared/scripts/pk-focus-on-prompt.sh`:
  - Reads the user prompt from stdin (per UserPromptSubmit hook contract).
  - Extracts keywords (simple TF-IDF or just top-N nouns).
  - If `pk` is on `$PATH` and `prometheus-knowledge` is reachable, calls
    `pk focus "<keywords>" --max-articles 3` and prints the result on stdout
    so it is appended to the system prompt.
  - Silently exits 0 on any failure (the hook must never block input).
- Update `hooks/hooks.json` to add a `UserPromptSubmit` matcher that runs the
  above script with a 3000ms timeout.
- Update the `Stop` hook to add a `forge reflect` step (after the existing
  `state-finalize.sh`) that processes the just-completed iteration if a
  `.forge/iterations/` directory exists.
- Add a `SubagentStop` fallback matcher (matcher: `*` or no matcher) that
  emits a generic checkpoint for unrecognized sub-agent names.
- Sweep all SKILL.md files and add `license: MIT` to frontmatter where missing.
- Update `scripts/validate-skills.js` schema to mark `license` as optional but
  emit a warning when absent (forward-compat with future strict validation).

Out:

- Anything that requires `pk` to be running — the hook degrades gracefully if
  it's not. Bringing `pk` up by default lives in change-004.

## Deliverables

1. `shared/scripts/pk-focus-on-prompt.sh` with full graceful-degradation logic.
2. Updated `hooks/hooks.json` with `UserPromptSubmit` matcher and Stop hook
   addition.
3. License field added to ~29 SKILL.md files.
4. Validator emits a warning for missing `license`.

## Acceptance Criteria

- With `pk` not running: `UserPromptSubmit` hook completes in under 200ms and
  outputs nothing (no errors).
- With `pk` running: hook injects pk-focus output into the prompt context within
  3 seconds.
- After a Claude Code session that does work in a `.forge/iterations/` project:
  the Stop hook runs `forge reflect`, ingests via `pk ingest`, and the resulting
  article appears in `~/.prometheus-knowledge/articles/`.
- All SKILL.md files have a `license` field in frontmatter; validator passes.

## Files to Touch

- `hooks/hooks.json`
- `shared/scripts/pk-focus-on-prompt.sh` (new)
- `scripts/validate-skills.js` (warning for missing license)
- All 29 `SKILL.md` files (add license)

## Test Plan

- Unit: invoke the hook script with a sample prompt, confirm output behavior
  with and without `pk` available.
- Hook lifecycle: verified by spinning up a fresh Claude Code session and
  observing that the hook runs (visible in `~/.claude/logs/hooks.log`).
- Failure mode: kill `pk` mid-session, confirm hook continues to exit 0.
