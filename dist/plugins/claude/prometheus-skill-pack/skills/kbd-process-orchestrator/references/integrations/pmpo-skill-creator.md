# KBD Integration: pmpo-skill-creator

KBD invokes `pmpo-skill-creator` when the **KBD process itself needs to evolve**
— either to extend the orchestrator with new sub-skills, create project-specific
skill variants, or generate new skills discovered during a phase reflection.

**Global skill location**: `.agent/skills/pmpo-skill-creator/SKILL.md`
**Entry commands**: `/create-skill`, `/clone-skill`, `/extend-skill`, `/validate-skill`
**State backend**: `.creator/skills/<name>/state.json`

---

## When KBD Invokes pmpo-skill-creator

| KBD Phase                      | Creator Role                                                             | Entry Point                        |
| ------------------------------ | ------------------------------------------------------------------------ | ---------------------------------- |
| **Reflect** (meta-improvement) | Reflect surfaces need for new KBD sub-skill or domain adapter            | `/create-skill` or `/extend-skill` |
| **Reflect / session start** (candidates) | A human accepts a skill candidate the learning worker proposed | The `/pmpo-skill-creator` invocation `pk candidates accept --kind skill` prints |
| **Plan** (tooling gap)         | Phase plan requires tooling that doesn't exist as a skill                | `/create-skill`                    |
| **Post-init**                  | `/kbd-init` surfaces that a project needs a custom constraint skill      | `/clone-skill`                     |
| **Any phase**                  | Validate the kbd-process-orchestrator itself against agentskills.io spec | `/validate-skill`                  |

---

## Key Usage Patterns

### 1. Extending kbd-process-orchestrator with a new sub-skill

When a phase reflection (`reflection.md`) identifies a recurring orchestration
pattern that should be codified:

```
/extend-skill
  source_skill: .agent/skills/kbd-process-orchestrator
  intent: "Add /kbd-audit sub-skill that scans .kbd-orchestrator/ for state consistency"
  mode: extend
```

This adds a new directory to `skills/kbd-audit/SKILL.md` without touching existing files.

### 2. Creating a project-specific domain adapter for iterative-evolver

```
/clone-skill
  source_skill: .agent/skills/iterative-evolver
  intent: "Create a saas-product domain adapter for iterative-evolver"
  domain: saas-product
  mode: clone
```

### 3. Validating the kbd skill itself

Run periodically to ensure the skill remains spec-compliant as it evolves:

```
/validate-skill
  skill_path: .agent/skills/kbd-process-orchestrator
```

---

## Skill Candidates (learning worker)

The learning worker (pk >= 1.11.0) fingerprints finished sessions and
proposes two kinds of candidate. Neither creates or edits anything.

| Candidate type | Meaning                                                             | Accept prints                                   |
| -------------- | ------------------------------------------------------------------- | ----------------------------------------------- |
| `new-skill`    | A workflow seen in 3+ sessions or 2+ projects that no skill covers  | `/pmpo-skill-creator` with the evidence path    |
| `skill-update` | A skill users corrected 2+ times after it ran                       | `/pmpo-skill-creator --update <skill>`          |

- `kbd-open` lists pending candidates at session start (`pk candidates list
  --kind skill`) and prints nothing when pk is absent, older than 1.11.0, or has
  none pending.
- `kbd-reflect` presents each pending candidate to the human (step 10). It
  accepts or rejects **only on an explicit human instruction**:
  `pk candidates accept --kind skill <id>` (optionally `--update <skill>`) or
  `pk candidates reject --kind skill <id>`.
- Accepting moves the candidate to `accepted/` and prints the invocation. The
  human then runs it; `/pmpo-skill-creator --update` still shows a diff and
  applies it only on an explicit `y`.
- `shared/scripts/propose-skill-update.sh` is **not** part of this flow. No hook
  or script calls it: it is a manual entry point that files a placeholder note
  under `~/.prometheus/skill-updates/` when run by hand
  (`propose-skill-update.sh <skill-name>`). The worker writes the same directory
  for `skill-update` candidates so `/pmpo-skill-creator --update` finds them.

---

## What KBD Reads Back

After skill creation/extension, KBD:

- Verifies the new sub-skill `SKILL.md` exists and passes `/validate-skill`
- Updates `SKILL.md` Quick Start if a new slash command was added
- Commits the new skill files: `git add .agent/skills/ && git commit -m "kbd: add <skill-name> sub-skill"`

---

## Self-Improvement Protocol

`pmpo-skill-creator` is the mechanism by which KBD improves itself. The Reflect
phase should explicitly ask:

> "Should any recurring pattern in this phase be codified as a new KBD sub-skill?"

The answer is the reflection's `## Codify as Skill?` section (operator-only, never
written back to memory), read next to the worker's pending skill candidates. If
yes, and the human has accepted a candidate or asked for the skill → invoke `pmpo-skill-creator` in extend mode → new sub-skill → validate →
commit → document in phase reflection.

---

## When NOT to Use

- For minor updates to existing skill files: edit directly, no need for creator
- For simple `SKILL.md` documentation fixes: edit directly
- `pmpo-skill-creator` is for **structural additions** (new sub-skills, domain adapters,
  new hooks) — not for content edits
