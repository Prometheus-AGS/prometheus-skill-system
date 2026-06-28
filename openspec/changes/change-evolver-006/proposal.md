# change-evolver-006 — Staged idea validation sub-skill (validate-idea)

**Phase:** pmpo-evolver
**Priority:** HIGH — operator idea-validation perspective; three-gate Darwin pattern
**Gaps:** G-08
**Goals:** G1, G4
**Model class:** Gate 1 → small; Gate 2 → medium; Gate 3 → frontier
**Depends on:** change-evolver-001 (schema — Archive of Stepping Stones), change-evolver-003 (SKILL.md directory structure)

## Problem

No structured process exists for the idea-validation perspective (G-08). Operators have no way to take a nascent idea through plausibility → domain research → spec generation → human gate → KBD phase seeding. Without this, the "operator ideation" perspective is a manual, unstructured process with no traceability.

## Solution

Create `skills/process/pmpo-evolver/skills/validate-idea/SKILL.md` implementing the full Darwin Gödel Machine three-gate pipeline. Create `scripts/idea-gate-1.sh` for the fast plausibility check.

## New skill: skills/validate-idea/SKILL.md

**Frontmatter:**
```yaml
---
name: validate-idea
description: Staged idea validation for product evolution — three gates from plausibility to specification, with Archive of Stepping Stones persistence
version: '1.0.0'
license: MIT
metadata:
  author: prometheus-ags
  category: process
  tags: [idea-validation, evolution, darwin, staged-gating, liter-llm]
---
```

**Entry:**
```
/validate-idea "<idea text>" [--evolution-name <name>] [--auto-gate]
```
Or invoked by `/pmpo-evolver --perspective idea-validation --idea "<text>"`.

**Gate 1 — Plausibility (~30s):**
- Model: `[MODEL_ROUTING] phase=evolver-idea-gate1 class=small`
- Run `bash scripts/idea-gate-1.sh "<idea>" "<evolution-name>"` — exits 0=PASS, 1=REJECT
- Gate 1 checks:
  1. Is this already implemented? (scan `skills/`, `.kbd-orchestrator/phases/`, existing SKILL.md files)
  2. Is this in the backlog? (scan `.evolver/<name>/backlog.json`)
  3. Does this align with `design-philosophy.md` (if present)? (binary liter-llm check)
- On REJECT: write to archive with `revisit_weight: 0.1`, exit with REJECT message

**Gate 2 — Domain research (~5min):**
- Model: `[MODEL_ROUTING] phase=evolver-idea-gate2 class=medium`
- Web research: "prior art, existing implementations, similar solutions"
- Feasibility check: list required dependencies (packages, APIs, services)
- Competitive check: read `parity-matrix.json` (if present) — does a competitor already have this?
- Output: `{feasibility_score: 0-100, prior_art: [], missing_deps: [], competitive_status: "ahead|parity|behind", recommendation: "PROCEED|DEPRIORITIZE|PIVOT"}`
- Score < 30 → REJECT (write to archive with `revisit_weight: 0.3`); score 30-60 → human gate required; score > 60 → auto-proceed to Gate 3 if `--auto-gate`

**Gate 3 — Spec + human gate:**
- Model: `[MODEL_ROUTING] phase=evolver-idea-spec class=frontier`
- Generate `SPEC.md` draft:
  - Problem statement
  - Proposed solution
  - Acceptance criteria (each must be machine-checkable per Karpathy verifiability constraint)
  - Non-goals
  - Dependencies
  - Estimated effort
- Verifiability check: for each acceptance criterion, classify as "machine-checkable" or "human-judgeable"
  - If any are "human-judgeable": loop with pmpo-elicit to reformulate as verifiable criteria
- Human gate: pmpo-elicit presents spec + Gate 2 findings → APPROVE / REVISE / REJECT
  - REVISE: return to spec generation with operator feedback
  - APPROVE: proceed to phase seeding (change-evolver-010 protocol)
  - REJECT: archive with `revisit_weight: 0.0`

**Archive of Stepping Stones:**
Every idea — regardless of gate outcome — is written to `.evolver/<name>/archive/<idea-id>/manifest.json`:
```json
{
  "id": "idea-<timestamp>",
  "text": "string",
  "submitted_at": "ISO8601",
  "gate_reached": 1,
  "outcome": "PASS | REJECT",
  "reject_reason": "string (if rejected)",
  "lessons": ["string"],
  "revisit_weight": 0.0,
  "gate1_result": {},
  "gate2_result": {},
  "gate3_spec_path": "string (if Gate 3 reached)"
}
```
`revisit_weight`: 1.0 = approved and executed; 0.5 = approved but not yet executed; 0.3 = rejected at Gate 2; 0.1 = rejected at Gate 1; 0.0 = hard reject

**Platform compatibility:**
The skill file uses only bash + python3 (via `idea-gate-1.sh`). Works identically across all six harnesses.

## New script: scripts/idea-gate-1.sh

```bash
#!/usr/bin/env bash
set -euo pipefail
IDEA_TEXT="${1:?Usage: idea-gate-1.sh '<idea text>' <evolution-name>}"
EVOLUTION_NAME="${2:-default}"

echo "[gate-1] Running plausibility check for: ${IDEA_TEXT:0:80}..."

# Check 1: Already in skills directory?
KEYWORD=$(echo "${IDEA_TEXT}" | tr '[:upper:]' '[:lower:]' | tr ' ' '-' | cut -c1-40)
EXISTING=$(find skills/ -name "SKILL.md" -exec grep -li "${KEYWORD}" {} \; 2>/dev/null | head -3)
if [ -n "${EXISTING}" ]; then
  echo "[gate-1] REJECT: Similar skill may already exist: ${EXISTING}"
  exit 1
fi

# Check 2: Already in backlog?
BACKLOG=".evolver/${EVOLUTION_NAME}/backlog.json"
if [ -f "${BACKLOG}" ]; then
  BACKLOG_MATCH=$(python3 -c "
import json, sys
with open('${BACKLOG}') as f:
    backlog = json.load(f)
items = backlog.get('items', [])
keyword = '${KEYWORD}'.replace('-', ' ')
matches = [i for i in items if keyword in i.get('text','').lower()]
print(len(matches))
")
  if [ "${BACKLOG_MATCH}" -gt 0 ]; then
    echo "[gate-1] REJECT: Idea already in backlog"
    exit 1
  fi
fi

# Check 3: In completed KBD phases?
PHASE_MATCH=$(grep -rl "${KEYWORD}" .kbd-orchestrator/phases/*/goals.md 2>/dev/null | head -3 || true)
if [ -n "${PHASE_MATCH}" ]; then
  echo "[gate-1] INFO: Similar work found in phases: ${PHASE_MATCH}"
  # Not a hard reject — could be extension of prior work
fi

echo "[gate-1] PASS: Idea passed plausibility check"
exit 0
```

## Acceptance criteria

- [ ] `skills/process/pmpo-evolver/skills/validate-idea/SKILL.md` exists with valid frontmatter
- [ ] `npm run validate:strict skills/process/pmpo-evolver/skills/validate-idea` passes
- [ ] `scripts/idea-gate-1.sh` is executable
- [ ] Gate 1 script: exits 1 when a matching skill exists, exits 0 when it doesn't
- [ ] `bash scripts/idea-gate-1.sh "add rust skills" default` exits 1 (rust skills exist in this repo)
- [ ] `bash scripts/idea-gate-1.sh "add quantum entanglement router" default` exits 0
- [ ] Archive manifest format documented in validate-idea SKILL.md
- [ ] All three gates have `[MODEL_ROUTING]` directives with correct class assignments
