# Claude Code Development Guide

This file provides guidance for AI assistants working **on** this repository.
For the skill's functionality, see `SKILL.md`. For project overview, see `README.md`.

## Architecture

The skill follows PMPO (Prometheus Meta-Prompting Orchestration) adapted for
constraint interrogation:

- **Phase controllers** in `prompts/` drive each loop phase
- **State management** via pluggable providers (see `references/state-management.md` pattern from iterative-evolver)
- **Dimension references** in `references/dimensions/` contain the 10 canonical questions per dimension
- **Schemas** in `references/schemas/` define the manifest and state output contracts
- **Subskills** in `skills/` provide slash command entry points
- **Hooks** in `hooks/` trigger state persistence and workflow dispatch

## Key Files

| File | Role |
|---|---|
| `SKILL.md` | Canonical skill definition — source of truth for behavior |
| `prompts/meta-controller.md` | Orchestration entry point — provider resolution, loop, caller routing |
| `prompts/interrogate.md` | Phase 1 — 6-dimension questioning with gap classification |
| `prompts/score.md` | Phase 2 — coverage scoring with per-dimension thresholds |
| `prompts/manifest.md` | Phase 3 — constraint manifest generation, GO/CAUTION/NO-GO |
| `prompts/persist.md` | Phase 4 — provider-agnostic state persistence |
| `references/integration-contract.md` | Caller protocol — what KBD and evolver expect |
| `references/dimensions/*.md` | The 10 canonical questions per dimension |
| `references/coverage-scoring.md` | Threshold logic and per-dimension criticality rules |
| `references/schemas/constraint-manifest.schema.json` | Manifest output contract |
| `hooks/hooks.json` | Lifecycle hooks — phase checkpoints and workflow dispatch |
| `.mcp.json` | MCP server configuration |

## Development Guidelines

### Modifying Dimension Questions

Dimension questions live in `references/dimensions/<name>.md`. Each file contains
exactly 10 questions, numbered Q1–Q10, with:
- The question text
- Why it matters (rationale)
- Examples of good answers
- What an `implicit` answer means for the system

When modifying questions, preserve the Q1–Q10 numbering — the scoring script
references questions by ID: `<dimension>.<number>` (e.g., `why.3`).

### Modifying Coverage Thresholds

Thresholds are defined in two places:
1. `SKILL.md` — the table documenting default thresholds
2. `references/coverage-scoring.md` — the scoring logic reference
3. `scripts/score-coverage.sh` — the computation implementation

All three must stay in sync. The script is the authoritative executor.

### Adding a New Caller Integration

1. Add a row to the caller table in `references/integration-contract.md`
2. Add a conditional branch in `prompts/manifest.md` for the `caller_enrichment` format
3. Document the expected state file path the caller will read from
4. Do NOT add caller-specific logic to dimension question files

### Modifying the Manifest Schema

1. Edit `references/schemas/constraint-manifest.schema.json`
2. Update `prompts/manifest.md` to reflect the new output contract
3. Update `references/integration-contract.md` — all callers consume the manifest
4. Bump the schema `version` field

### Adding Workflow Triggers

Follow the same pattern as `iterative-evolver` — see that skill's
`references/workflow-integration.md` for the trigger semantics.
The dispatch script (`scripts/workflow-dispatch.sh`) is adapted from
iterative-evolver and follows the same interface.

## Key Design Principles

> **Dimension Purity**: Never put caller-specific logic or project-specific
> knowledge into dimension question files. Questions must be universally
> applicable across any system or change being interrogated.

> **Manifest as Contract**: The constraint manifest is the only output consumed
> by callers. All other state files are internal. The manifest schema is the
> public API of this skill — treat it with the same stability guarantees as
> a versioned API.

> **Non-Blocking GO/CAUTION**: A `CAUTION` recommendation must never block
> the caller outright. The caller decides whether to proceed. ZeeSpec reports;
> it does not govern.

> **Implicit is Valid**: A question answered as `implicit` is a valid, recorded
> answer — not an error. It means the AI or system will decide. Recording it
> is better than leaving it undocumented.

## Testing

```bash
# Validate YAML frontmatter
for f in SKILL.md skills/*/SKILL.md; do
  head -5 "$f" | grep -q "^---" && echo "✅ $f" || echo "❌ $f"
done

# Validate JSON files
for f in references/schemas/*.json hooks/hooks.json .mcp.json; do
  python3 -c "import json; json.load(open('$f')); print(f'✅ $f')" 2>/dev/null || echo "❌ $f"
done

# Check dimension files exist and have 10 questions
for dim in what where who when why how; do
  f="references/dimensions/${dim}.md"
  count=$(grep -c "^## Q" "$f" 2>/dev/null || echo 0)
  [ "$count" -eq 10 ] && echo "✅ ${dim}: 10 questions" || echo "❌ ${dim}: $count questions (expected 10)"
done

# Verify scripts are executable
for f in scripts/*.sh; do [ -x "$f" ] && echo "✅ $f" || echo "❌ $f (run: chmod +x $f)"; done

# Test provider resolution
bash scripts/state-resolve-provider.sh
```
