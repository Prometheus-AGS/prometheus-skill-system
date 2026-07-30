# 12a · The Skill Creator (`pmpo-skill-creator`)

The sibling of the [agent creator](12-native-agent-generator.md). Where that one produces a
*service*, this one produces a **skill** — a portable, spec-compliant capability that any
agentskills.io-compatible harness can load.

## Why not just write a SKILL.md by hand

You can. A minimal skill is one markdown file with frontmatter, and for a purely
instructional skill that is the right answer — the creator's own **simple** tier emits 3–5
files and stops.

The creator earns its keep when the skill needs to be more than prose:

- **A state machine.** Standard and full tiers emit `state-init.sh`, `state-checkpoint.sh`,
  `state-finalize.sh` and a `state.json` with resumable iterations.
- **Multi-platform output.** One source, five targets: `agentskills-io`, `claude-code`,
  `opencode`, `cursor`, `gemini-cli` — including OpenCode TypeScript tool definitions.
- **Structural consistency with the pack.** Generated skills follow the same PMPO phase
  layout as `iterative-evolver` and `artifact-refiner`, which the creator reads as
  exemplars.
- **A convergence loop rather than one-shot generation.** Generate → validate → fix →
  re-validate, up to 3 iterations, with a weighted quality score gating the exit.

That last point is the real difference. A one-shot generator hands you a plausible skill.
This runs a loop that keeps failing itself until the artifact passes.

## The four entry points

```
/create-skill                          # from scratch
/clone-skill                           # adapt an existing skill to a new domain
/extend-skill                          # add capabilities to an existing skill
/validate-skill                        # check against the agentskills.io spec
/pmpo-skill-creator --update <name>    # refine from learning patterns (human-gated)
```

`create-skill` is a 49-line front door that asks five questions and delegates.
`pmpo-skill-creator` is the engine behind it: a state machine, five phase controllers,
three agents, five references, two JSON schemas, six scripts, and hooks.

## Complexity tiers

Chosen during Specify, and the single biggest lever on what you get:

| Tier | Characteristics | Files |
|---|---|---|
| **Simple** | instructional only, no scripts, no state | 3–5 |
| **Standard** | PMPO loop, schemas, some scripts | 15–25 |
| **Full** | complete evolver/refiner-class skill | 30–50+ |

## The loop

Five phases, not four:

```
Specify → Plan → Execute → Reflect → Persist → (loop or terminate)
```

**Reflect is the validation phase**, and **Persist** is a distinct fifth phase that writes
validated state. Each phase checkpoints to
`.creator/skills/<name>/checkpoints/<phase>_<timestamp>.json`.

State carries `max_iterations: 3` and a `convergence_status` of `running`, `converged`, or
`failed`. Re-running a converged skill seeds a new cycle with a `prior_creation_id` rather
than clobbering the old one.

### The exit gate

`reflect.md` computes a weighted score:

| Category | Weight |
|---|---|
| Spec compliance | 30% |
| Schema validity | 20% |
| Cross-references | 15% |
| Script quality | 15% |
| Completeness | 20% |

**Pass threshold: ≥95% with zero FAILs.** Failures route back to different phases
depending on where they are:

| Condition | Action |
|---|---|
| Zero FAILs, score ≥ 0.95 | terminate → finalize |
| FAILs in generated files only | loop back to **Execute** with a fix list |
| FAILs in architecture/structure | loop back to **Plan** with constraints |
| 3+ iterations, no progress | terminate with warnings |

## What actually validates

`scripts/validate-skill.sh` is a real executable gate returning exit 1 on failure. Seven
check groups:

1. `SKILL.md` exists (hard fail)
2. Frontmatter delimiters; `name` ≤64 chars; `description` present; line count ≤500 *(warn)*
3. Every `*.json` parses
4. Every `scripts/*.sh` is executable, has a shebang, and passes `bash -n`
5. Cross-references — every `references/...` path mentioned in prompts must exist
6. `.claude-plugin/plugin.json` has `name` + `description`
7. Every sub-skill `SKILL.md` starts with `---`

```
=== RESULT ===
  Passes:   42
  Failures: 0
  Warnings: 2

  ✅ SKILL VALID
```

Repo-level validation is stricter and is what gates a skill entering this pack:

```bash
npm run validate           # all native skills, 0 errors required
npm run validate:strict    # adds license, version, metadata.tags as HARD errors
```

## Honest scope

Three things you might reasonably expect that the creator does **not** do:

- **It does not run adversarial review.** There are zero references to the
  [adversarial-review](09a-adversarial-review.md) skill anywhere in it. If you want a
  second model to critique a generated skill, run that gate yourself.
- **It does not generate evals.** The repo's eval corpus is a fixed 36-case, 6-skill set
  and contains no entry for generated skills.
- **Sycophancy correction is instruction, not enforcement.** `execute.md` and `reflect.md`
  both mandate a sycophancy pass and `reflect.md` will FAIL a skill with "no edge cases or
  failure modes" — but `validate-skill.sh` implements none of it. It is prompt-level
  guidance the model is asked to honour, not a gate that runs.

Similarly, `validate-skill.sh` implements roughly steps 1–4 of `reflect.md`'s eleven, and
outputs human-readable text rather than the JSON report that document claims. The
quality score is computed by the model, not by a script.

> **Known bug, now fixed.** `workflow-dispatch.sh` launched its Python through a quoted
> heredoc without exporting `EVENT`/`PAYLOAD`/`TRIGGER_FILE`, so the event was always empty
> and **no trigger ever matched** — every one of the twelve hook invocations was a silent
> no-op. Corrected on 2026-07-30; triggers now fire.

## A worked run

```
/create-skill
> Name: code-reviewer
> Intent: Review code changes for quality, security, and best practices
> Complexity: standard
> Platforms: agentskills-io, claude-code
```

Output lands in `dist/<skill-name>/`:

```bash
mkdir -p dist/<skill_name>/{prompts,agents,references/schemas,scripts,hooks,skills,assets/templates}
```

Clone and extend take a source:

```
/clone-skill
> Source: .agent/skills/iterative-evolver
> Name: compliance-auditor
> Domain: compliance and regulatory review
```

Clone enforces source fidelity — *file maps must account for 100% of source files*, and
extend is constrained to *never delete, never rename existing files*.

## Self-test

The creator ships a runnable self-check worth stealing for your own skills:

```bash
# Every schema parses
for f in references/schemas/*.json; do
  python3 -c "import json; json.load(open('$f'))" && echo "✅ $f" || echo "❌ $f"
done

# Every script is executable
for f in scripts/*.sh; do [ -x "$f" ] && echo "✅ $f" || echo "❌ $f"; done

# Every referenced file exists
grep -roh 'references/[a-zA-Z0-9/_.-]*' prompts/ | sort -u | while read f; do
  [ -e "$f" ] && echo "✅ $f" || echo "❌ $f"
done
```

## Skill or agent?

| | Skill | Native agent |
|---|---|---|
| Artifact | markdown + optional scripts | Cargo workspace + frontend + Docker |
| Runs | inside the harness | own process, own port |
| Portable to | any agentskills.io harness | anywhere you can run a binary or container |
| Reach | the agent that loaded it | any HTTP client, other agents via A2A |
| Cost to make | seconds to minutes | minutes, plus a Rust build |

**Skill** when you are teaching an existing agent *how* to do something. **Agent** when you
need a new thing that runs, listens, and can be called. That decision — and the case for
both — is [22a · Self-Extending Agents](22a-self-extending-agents.md).

## See also

- [12 · Agent Creator](12-native-agent-generator.md) — the sibling generator
- [08 · Skills Overview](08-skills-overview.md) — the agentskills.io standard
- [21 · Contributing](21-contributing.md) — the validation gates for merging a skill
