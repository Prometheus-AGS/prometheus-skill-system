---
license: MIT
name: pmpo-skill-creator
description: >
  Use this skill to create, clone, or extend Agent Skills. Runs the work as a KBD phase,
  drafts with Anthropic's skill-creator when it is installed, and certifies the result
  with skill-certification at the phase boundary, putting any failed level to the operator.
  Generates agentskills.io-conformant skills with Claude Code plugin support, phase
  controllers, hooks and eval cases. Three modes: create (from scratch), clone (adapt an
  existing skill to a new domain), and extend (add capabilities to an existing skill).
  Do not use to certify an existing skill without changing it; use skill-certification.
allowed-tools: Read Write Edit Bash Glob Grep
model_routing:
  policy_source: ".kbd-orchestrator/project.json → model_policy"
  phases:
    creator-specify: frontier
    creator-plan: frontier
    creator-execute: tiered
    creator-reflect: frontier
    creator-persist: small
  routing_reference: "references/model-routing.md"
metadata:
  version: '1.1.0'
  tags: "process, orchestration, automation"
---

# PMPO Skill Creator

A generator for Agent Skills that runs as a KBD phase and ends in a certificate. It creates complete skill packages — phase controllers, domain adapters, hooks, schemas, sub-skills, eval cases and multi-platform output — and does not call a skill finished until it has been certified or the operator has decided otherwise.

Three things frame every run:

- **KBD owns the process.** The five creator phases map onto KBD stages, and KBD's state is the record. See [references/kbd-integration.md](references/kbd-integration.md).
- **Anthropic's `skill-creator` drafts, when it is installed.** It writes the description, the body and the eval cases. Without it, this skill drafts them itself.
- **`skill-certification` is the gate.** It runs once at the phase boundary. A failed level is the operator's decision: fix, accept with a recorded exception, or abandon. See [references/certification-gate.md](references/certification-gate.md).

Model selection follows `references/model-routing.md`. When a project has no
model policy, use the documented frontier fallback rather than inventing a
provider-specific model mapping.

## Creation Modes

### `create` — New skill from intent

Generates a complete skill from a description of what it should do. Produces all files needed for PMPO orchestration, agentskills.io compliance, and multi-agent deployment.

### `clone` — Adapt existing skill

Copies the structure of an existing skill (like iterative-evolver or artifact-refiner) and adapts it for a new domain. Preserves architectural patterns while replacing domain-specific content.

### `extend` — Add capabilities

Adds new domain adapters, phases, sub-skills, or platform support to an existing skill without breaking current functionality.

### `update` — In-place skill refinement from learning patterns

Improves an existing installed skill using accumulated learning patterns from the continuous-learning pipeline. Requires explicit user approval before any file is modified.

**Invocation:**
```
/pmpo-skill-creator --update <skill-name>
```

**Flow:**
1. Read `~/.claude/skills/<skill-name>/SKILL.md` — current installed skill content
2. Search `~/.prometheus/learning-log/*.jsonl` for entries whose `scoped_paths` or `phase` field matches the skill's domain
3. Search surreal-memory for patterns related to `<skill-name>` (when `SURREAL_MEMORY_URL` is set and reachable)
4. Identify targeted additions: new examples, corrected instructions, updated references — no structural rewrites
5. Generate a unified diff (`unified_diff`) of proposed changes
6. Write proposed diff to `~/.prometheus/skill-updates/<skill-name>-<date>.diff`
7. Present the full diff to the user — do NOT apply automatically
8. Prompt: "Apply this update? (y/N)" — call `/pmpo-elicit` if available, otherwise prompt inline
9. Only on explicit `y`: apply the diff and re-validate with `npm run validate:strict` from the skill-pack root

**Rules:**
- Never modify the installed skill file without an explicit `y` from the user.
- On `n`, timeout, or any non-`y` response: skill file is unchanged; diff file is preserved for future reference.
- The diff must be minimal and surgical — it must not rewrite sections unrelated to the learning entries found.
- If zero learning entries match the skill, exit 0 with message: "No learning patterns found for `<skill-name>` — skill is up to date."
- `propose-skill-update.sh` (a manual entry point; no hook calls it) only logs candidates; it never invokes `--update` automatically. The `--update` invocation is always human-triggered.

## What Gets Generated

Every generated skill includes:

| Component  | Files                                | Purpose                                                           |
| ---------- | ------------------------------------ | ----------------------------------------------------------------- |
| Core       | `SKILL.md`, `CLAUDE.md`, `AGENTS.md` | Skill manifest, dev guide, contributor guide                      |
| Prompts    | `prompts/*.md`                       | PMPO phase controllers (specify, plan, execute, reflect, persist) |
| Agents     | `agents/*.md`                        | Specialized subagent definitions                                  |
| References | `references/*.md`                    | Domain adapters, theory docs, architecture refs                   |
| Schemas    | `references/schemas/*.json`          | JSON schema contracts for state and output                        |
| Scripts    | `scripts/*.sh`                       | State lifecycle management (init, checkpoint, finalize, dispatch) |
| Sub-skills | `skills/*/SKILL.md`                  | Slash command entry points                                        |
| Cases      | `evals/*/`                           | Eval cases for certification: two outcome, one near-miss at least |
| Hooks      | `hooks/hooks.json`                   | Lifecycle hooks with checkpoint + dispatch                        |
| Plugin     | `.claude-plugin/plugin.json`         | Claude Code marketplace manifest                                  |
| Templates  | `assets/templates/*`                 | Output scaffolding for generated artifacts                        |

## Inputs

```yaml
skill_name: string # Required — name for the new skill
intent: string # What the skill should do
mode: string # create | clone | extend
source_skill: string # Required for clone/extend — path to source skill
target_platforms:
  array # Which platforms to support (default: all)
  # Options: agentskills-io, claude-code, opencode, cursor, gemini-cli
tools_directory: string # Optional — custom tools output directory (e.g., .opencode/tools/)
domain: string # Optional — primary domain for the skill
```

## Outputs

```yaml
generated_skill:
  path: string # Directory containing the complete skill
  file_count: integer # Total files generated
  platforms: array # Platforms supported
  validation: object # Static validation results
certification:
  level: string # none | C0 | C1 | C2 | C3 — the highest level with evidence
  certificate: string # Path to certificate.json
  exceptions: array # Levels the operator accepted, with their reasons
```

## Artifact-Refiner Integration (Optional)

When the artifact-refiner skill is available, the creator can delegate markdown and prompt content generation to it using `content_type: meta:agent-prompt`. This produces iteratively refined prompt text. If artifact-refiner is not available, the creator generates content directly.

## Execution Model

### Startup Protocol

1. **Find or start KBD**: look for `.kbd-orchestrator/`; run `/kbd-init` if it is absent
2. **Open a phase**: `/kbd-new-phase skill-<name>`, or `/kbd-new-child` inside an active phase
3. **Resolve `<skill_dir>`**, and say which helper skills are available
4. **Mode Routing**: route to the create, clone or extend workflow

### Phases

| Creator phase | KBD stage | Controller | What happens |
|---|---|---|---|
| Specify | `/kbd-assess`, `/kbd-analyze` | `prompts/specify.md` | Clarify intent, classify mode, check for an existing skill |
| Plan | `/kbd-spec`, `/kbd-plan` | `prompts/plan.md` | Design the architecture as one change with tasks |
| Execute | `/kbd-execute` | `prompts/execute.md` | Author every file and the eval cases. Nothing is run |
| Reflect | the phase boundary | `prompts/reflect.md` | Static validator, adversarial review, then certification, once |
| Persist | `/kbd-reflect` | `prompts/persist.md` | Record the level held and the operator's decision |

### The boundary

Verification happens once, after authoring is complete, cheapest check first.
The static levels of certification are free. The trigger and outcome levels make
billed model runs, so tell the operator the mode, the run count and the cost cap
and wait for a yes.

### When a level fails

Do not loop on your own and do not ship quietly at a lower level. Show the
operator what failed, the proposed fix, and the impact of each choice:

1. **Fix**: apply it and re-run only the affected level.
2. **Accept**: record the exception in the certificate; the level is not raised.
3. **Abandon**: stop and say where the files are.

## Required Tools

- File read, write and edit — read source skills, write generated files
- A shell — run the validator, the adversarial review and the certifier
- KBD (`.kbd-orchestrator/`); started with `/kbd-init` when absent
- Optional: Anthropic's `skill-creator`, `skill-certification`, artifact-refiner

## Persistent State Files

- `.kbd-orchestrator/phases/skill-<name>/` — the phase that created the skill: assessment, plan, progress, reflection
- `<phase_dir>/certification/certificate.json` and `report.md` — the certificate
- `<skill_dir>/` — the skill itself, at its real destination (in the skill pack, `skills/<domain>/<name>/`; elsewhere `dist/<name>/` unless the operator names a directory)

## Quality Standards

A generated skill is finished when it holds a certificate, or the operator has accepted a recorded exception. Passing the static validator is not enough: in the pack's pilot, skills that validated cleanly never triggered.

Generated skills MUST:

1. Pass agentskills.io SKILL.md validation (required frontmatter, <500 lines)
2. Include all JSON schemas referenced in prompts
3. Have executable scripts with proper shebang lines
4. Include cross-reference integrity (no dangling file references)
5. Support at least agentskills.io + Claude Code plugin format
6. Include state management lifecycle (init, checkpoint, finalize)
7. Include hooks with per-phase checkpoint + dispatch, in the harness's hook format
8. Include eval cases, so the trigger and outcome levels can be measured
9. Have a description that says when to use the skill and when not to

## Exemplar Skills

The creator uses these as structural templates:

| Skill             | Location             | Files | Domains                               |
| ----------------- | -------------------- | ----- | ------------------------------------- |
| iterative-evolver | UAR `.agent/skills/` | 48    | 8 (software, business, product, etc.) |
| artifact-refiner  | standalone repo      | 79    | 7 (logo, ui, code, meta-prompt, etc.) |

Both demonstrate the complete feature set: PMPO loop, named state, providers, hooks, workflow triggers, sub-skills, domain adapters, JSON schemas.

## Quick Start

- `/create-skill` — Create a new skill from scratch
- `/clone-skill` — Clone an existing skill for a new domain
- `/extend-skill` — Add capabilities to an existing skill
- `/pmpo-skill-creator --update <skill-name>` — Refine an installed skill from learning patterns (human-gated)
- `/validate-skill` — Static pre-check of a skill (not certification)
- `/skill-certification` — Certify a skill: specification, static, trigger and outcome levels
