# kde-002 finding — `sessionStart` dropped, no suitable payload exists

_2026-08-05. Outcome: **change dropped**, per its own AC3._

## What t1 asked

`sessionStart` names one skill that runs at the start of **every** Kimi Desktop
session — including sessions with nothing to do with this pack. t1 was blocking
precisely because presence on disk (verified at spec time) says nothing about
suitability.

## Three candidates, three distinct failures

### `kbd-status` — presumes a KBD project

Its own opening line: "Show current KBD process state for **the active
project**." It reads `progress.json` / waypoint state and renders phase, change
inventory, and goal completion.

Verified in a scratch directory: walking up from a non-KBD path finds no
`.kbd-orchestrator` at all. There is no state to render, so on every unrelated
session it would either error or print an empty status. That is worse than no
`sessionStart` — it is noise on every session, forever.

### `learn-harness` — needs a script the package cannot reach

Genuinely harness-agnostic (**zero** KBD-state references — it was the strongest
candidate on that axis). But it auto-detects via `detect-surface-tier.sh`, and
that script is **not inside the skill**: it lives in `shared/scripts/` and
`skills/learn/ui-surface/scripts/`, neither of which ships in
`plugin-packages/prometheus-skill-pack/skills/learn-harness/`.

Note this is NOT a packaging bug — 40 of the 145 installed skills do carry their
`scripts/`. `learn-harness` simply has none of its own. CLAUDE.md's mobile
portability rule applies directly: a skill that shells out is inert wherever the
binary is unreachable.

### `learn-about-system` — asks the operator a question

Manifest-only, and its single KBD mention is teaching material about waypoint
files rather than a state read. It fails on behaviour: without `--area` it
"enters interactive discovery mode and **asks what the operator wants to learn
about**."

As a `sessionStart` payload that interrogates the user at the start of every
session. `sessionStart` takes `{"skill": "<name>"}` with no argument field
(`readSessionStart` reads only `skill`), so the non-interactive `--area` form
cannot be selected.

## Decision

**No `sessionStart` is emitted.** AC3 permits exactly two outcomes when t1 fails:
choose an already-suitable skill, or drop the change with the finding recorded.
Fixing `kbd-status` is explicitly out of scope — that is a skill change, and it
belongs in its own change with its own review.

## What would make this shippable later

A small, manifest-only, argument-free orientation skill that degrades to a
one-paragraph "here is what this pack is" when no KBD project is present. None
of the 145 currently qualifies. That is a skill-authoring change, not a
manifest change.

## Cost of the alternative

Shipping `kbd-status` anyway would have put a broken status report at the head
of every Kimi Desktop session — the same inertness class this phase has now hit
four times (Codex `[hooks]`, `{{file:}}` commands, dangling symlinks,
`systemPrompt`), except louder because it is user-visible.
