# kde-002 — emit `sessionStart` in the Kimi Desktop plugin manifest

**Phase:** kimi-desktop-extensibility
**Scope:** `scripts/install-kimi-desktop-plugin.sh` (generator only)
**Backend:** native-kbd

## Problem

Every other harness gets orientation discipline at session start (Claude Code
via `SessionStart` hooks, Codex via prompts). Kimi Desktop gets nothing, so a
session there begins with no awareness of the active KBD phase or waypoint.

## Approach

`sessionStart` is a manifest field whose shape is minimal — the `github` package
declares exactly `{"skill": "github"}`. Emit `{"skill": "kbd-status"}`.

## Evidence

- Shape confirmed from the vendor `github` package.
- `kbd-status` confirmed **present** among the 145 installed skills
  (`plugin-packages/prometheus-skill-pack/skills/kbd-status`).

## Known gap in that evidence

Adversarial review flagged this precisely: presence on disk was verified,
**suitability was not**. `kbd-status` is a KBD lifecycle skill that expects a
`.kbd-orchestrator/` to exist in the working directory. Kimi Desktop sessions may
start outside any KBD project, in which case the skill has nothing to report.

Task 1 therefore validates behaviour in a non-KBD directory before shipping.
A sessionStart skill that errors or emits noise on every unrelated session is
worse than none.

## Acceptance criteria

1. Generated manifest contains `"sessionStart": {"skill": "kbd-status"}`.
2. `kbd-status` degrades gracefully outside a KBD project — no error, no
   misleading output.
3. If it does not degrade gracefully, EITHER a different already-suitable skill
   is chosen, OR the change is dropped with the finding recorded. **Fixing
   `kbd-status` itself is OUT of scope** — this change edits the generator only,
   and a skill fix belongs in its own change so it is reviewed and verified on
   its own terms.
4. Existing package invariants hold: 145 skills, manifest valid, idempotent.

## Ordering

Apply AFTER `kde-001`. Both edit the same manifest dict in
`scripts/install-kimi-desktop-plugin.sh`; sequential application avoids a
silent merge that would drop a field while still producing a valid manifest.

## Out of scope

- Writing a new orientation skill (if `kbd-status` proves unsuitable, that is a
  separate change, not a silent substitution).
- Modifying `kbd-status` or any other skill. Declared scope is the generator
  script; a skill change would escape review under this change's criteria.
