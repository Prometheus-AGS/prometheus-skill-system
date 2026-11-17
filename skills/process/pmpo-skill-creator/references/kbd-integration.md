# Creating a skill as a KBD phase

A skill is created inside a KBD phase. KBD owns position, progress and the
boundary; this skill owns what a good Agent Skill looks like. The creator's own
five PMPO phases still describe the work, but they no longer track themselves:
each maps onto a KBD stage, and KBD's state is the record.

## Why

Before this, the creator kept its own state under `.creator/`, declared a skill
finished when its own validator scored it, and had no trigger or outcome
evidence at all. A skill could be "complete" and never start. Running inside
KBD gives one place to see where the work stands, one boundary at which
verification happens, and a certificate at the end.

## The mapping

| Creator phase | KBD stage | What happens | Artefact |
|---|---|---|---|
| Specify | `/kbd-assess`, then `/kbd-analyze` | Clarify intent; check for an existing skill that already does this; decide the mode | `skill_spec` in the phase's assessment and analysis |
| Plan | `/kbd-spec`, then `/kbd-plan` | One change for the skill; tasks for its files, its cases and its registration | The change and its task list |
| Execute | `/kbd-execute` (driving `/kbd-apply`) | Author every file of the skill and its cases. Nothing is run | The skill directory |
| Reflect | The phase boundary (B4) | Static validation, adversarial review, then certification, once | `certificate.json`, `report.md` |
| Persist | `/kbd-reflect` | Record the level held, the operator's decision and the lessons | The phase reflection |

## Starting

1. **Find KBD.** Look for `.kbd-orchestrator/` at or above the working directory.
2. **If it is absent**, say so and run `/kbd-init`. Creating a skill outside KBD
   is not supported; the operator may decline, in which case stop.
3. **Open a phase for the skill.**
   - No phase active: `/kbd-new-phase skill-<name>` with the intent as its goal.
   - A phase is active and the skill is part of its work: `/kbd-new-child skill-<name>`.
   - A phase is active and the skill is unrelated: ask the operator which they
     want. Do not replace the active phase on your own.
4. **Read position first on every turn**, as any KBD skill does:
   `.kbd-orchestrator/position-reminder.txt`, then the phase's `progress.json`.

## The boundary

VLBV applies, because this is a KBD phase:

- **Author everything first.** The skill's files, its eval cases and its
  registration are all written before anything is validated, reviewed or run.
- **Verify once.** At the boundary run, in this order: the static validator,
  the adversarial review, then certification. Each is cheaper than the next.
- **Repair, then re-run only what was affected.** After an operator-approved
  fix, re-run the checks the fix touches, not the whole packet.

Model runs for certification are billed. They belong to the boundary and need
the operator's yes each time; they are never a development loop.

## What KBD replaces

| Was | Now |
|---|---|
| `.creator/skills/<name>/state.json` as the record of progress | The KBD phase's `progress.json` |
| A per-phase checkpoint hook | KBD's task and phase boundaries |
| "Terminate when the quality score is at least 0.95" | The certificate's level and the operator's decision |
| A fixed cap of three iterations | The boundary: one run, then bounded repairs the operator approves |

The `scripts/state-*.sh` scripts remain for a generated skill that wants named
state of its own. They are no longer how this skill tracks a creation.

## Where the skill is written

`<skill_dir>` is the skill's real destination, not a staging copy:

- In the skill pack's repository (a `skill-system.json` at the root):
  `skills/<domain>/<name>/`. Never `dist/`, which is generated output there.
- Anywhere else: the directory the operator names, or `dist/<name>/` by default.

## Registering in the skill pack

When `<skill_dir>` is inside the skill pack, the change's tasks also include:

1. Add the skill to its domain's `.claude-plugin/plugin.json`, by its own path
   if it is nested inside another skill's directory, and bump that plugin's
   version there and in `skill-system.json`.
2. `npm run generate:skill-domains`, when the repository has that script.
3. `npm run generate:skills-index` and `npm run build`.

These are generators, so they run at the boundary with the checks.
