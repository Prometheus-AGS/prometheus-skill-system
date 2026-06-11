# Plan — position-and-handoff-guarantee

Backend: native-kbd (change.md format; manual task driving — the `nk_*` kbd-apply
adapter formalizes this in the next phase). Changes ordered; each carries a
`scope:` declaration (dogfooding Phase 3's scope guard ahead of its enforcement).

| # | Change | Addresses | Summary |
|---|--------|-----------|---------|
| 1 | change-001-position-render | F1, F2 | `shared/scripts/lib/waypoint-render.sh` — pure renderer: waypoint + active progress.json → sentinel-wrapped position block; dense/explain verbosity; camelCase-first key reads. + `tests/test-position-render.sh` (render fixtures). |
| 2 | change-002-position-prompt-hook | F1 | `shared/scripts/position-on-prompt.sh` (UserPromptSubmit) injecting the rendered block + response-format instruction; wire into `hooks/hooks.json`; always exit 0. |
| 3 | change-003-position-stop-gate | F1 | `shared/scripts/position-stop-gate.sh` (Stop, FIRST in array, no `\|\| true`): block once with rendered footer when final message lacks sentinel; respect `stop_hook_active`; soft cap via `~/.prometheus/position-stop-block.txt`. + tests. |
| 4 | change-004-stage-gates-handoffs | F3 | `KBD/shared/lib/stage-gate.sh` (`kbd_stage_gate <stage>`), `KBD/references/schemas/handoff.schema.json`, handoff writes + gate calls added to kbd-assess/kbd-plan/kbd-execute/kbd-reflect SKILL.md; legacy mode (no handoffs dir → warn, pass). |
| 5 | change-005-position-model | F2 | `KBD/shared/lib/position.sh` single-writer `kbd_position_sync` deriving `.kbd-orchestrator/position.json` (schema `KBD/references/schemas/position.schema.json`); `.evolver/`/`.zeespec/` read-only annotations; kbd-status renders tree; waypoint-render prefers position.json. |
| 6 | change-006-ci-signal-lint | F4 | `scripts/validate-progress-signals.js`, npm `validate:signals`, wire into `.github/workflows/validate.yml`. |

Completion criteria per change: tasks checked in change.md, tests green
(`bash shared/scripts/tests/test-*.sh`), `npm run validate:strict` and
`npm run build` clean at phase end. Commit per change.
