---
id: change-evolver-007
title: Carry-forward aggregation + domain taxonomy reference
phase: pmpo-evolver
gaps: [G-06, G-07]
priority: MEDIUM — unique-product and trend perspectives
goals: G1
agent: claude-code
status: done
scope:
  - references/domain-taxonomy.md
  - scripts/carry-forward-aggregate.sh
  - skills/process/pmpo-evolver/references/domain-taxonomy.md
  - pmpo-elicit/reflection.md
---

# change-evolver-007 — Carry-forward aggregation + domain taxonomy reference

## Problem

No mechanism aggregates carry-forwards from prior reflection phases (G-07). The trend perspective (G-06) has no mapping from product domain keywords to authoritative sources (standards bodies, community feeds, newsletters) — meaning trend research is ad-hoc and non-reproducible.

## Solution

Create `references/domain-taxonomy.md` — a curated mapping from domain keywords to sources with polling frequencies. Create `scripts/carry-forward-aggregate.sh` to walk all phase reflections and output normalized carry-forward data.

## New file: references/domain-taxonomy.md

**Contents:**

A curated table mapping domain keywords to research targets. Organized by domain cluster:

### AI Tooling / Agent Orchestration (matches this skill-pack's primary domain)

| Source type | URL / query | Frequency | Signal |
|-------------|-------------|-----------|--------|
| GitHub searches | `gh search repos "agent skills" --sort=updated` | weekly | New agent skill frameworks |
| GitHub searches | `gh search repos "mcp server skill" --sort=updated` | weekly | MCP integration patterns |
| Newsletter | Latent Space newsletter (latent.space) | weekly | LLM engineering developments |
| Standards | model context protocol spec (github.com/anthropics/mcp) | on-release | MCP spec changes |
| Standards | agentskills.io specification | on-release | Skills standard changes |
| Community | r/LocalLLaMA | weekly | Local model capability shifts |
| GitHub trending | filter: AI/ML agents repos | daily | Emerging projects |

### Rust Systems Programming

| Source type | URL / query | Frequency | Signal |
|-------------|-------------|-----------|--------|
| Standards | blog.rust-lang.org | on-release | Language features |
| Crates | `cargo search <keyword>` | monthly | New crate alternatives |
| Newsletter | This Week in Rust (this-week-in-rust.org) | weekly | Community trends |

### LLM Infrastructure / Model Routing

| Source type | URL / query | Frequency | Signal |
|-------------|-------------|-----------|--------|
| Standards | OpenAI API changelog | on-release | API surface changes |
| Standards | Anthropic changelog | on-release | Model + API changes |
| Community | r/MachineLearning | weekly | Research direction signals |
| GitHub | liter-llm releases | on-release | Router capability changes |

### Developer Tooling / CLI

| Source type | URL / query | Frequency | Signal |
|-------------|-------------|-----------|--------|
| Standards | OpenCode specification | on-release | Cross-harness skill format |
| Community | Hacker News "Ask HN" on AI coding tools | weekly | User sentiment |

### General Domain Detection

When the product domain is not in this taxonomy, use the following detection queries:
1. `gh search repos "<product-name>" --language=<dominant-lang> --sort=stars` — find adjacent repos
2. Web search: "<product-domain> newsletter site:substack.com OR site:beehiiv.com" — find community feeds
3. Web search: "<product-domain> working group OR specification OR RFC" — find standards bodies

### Polling Frequency Guidance

| Source type | Recommended TTL |
|-------------|----------------|
| Standards bodies (IETF, W3C, NIST, ISO) | monthly (1440 min × 30) |
| GitHub releases of direct competitors | daily (1440 min) |
| Community subreddits / HN | weekly (10080 min) |
| Newsletters | weekly (10080 min) |
| GitHub trending | daily (1440 min) |

## New script: scripts/carry-forward-aggregate.sh

```bash
#!/usr/bin/env bash
set -euo pipefail
PHASES_DIR="${1:-.kbd-orchestrator/phases}"
EVOLUTION_NAME="${2:-default}"
DEEP_MODE="${3:-}"

OUTPUT_DIR=".evolver/${EVOLUTION_NAME}"
mkdir -p "${OUTPUT_DIR}"
OUTPUT_FILE="${OUTPUT_DIR}/carry-forwards.json"

echo "[carry-forward] Scanning ${PHASES_DIR} for reflection.md files"

# [MODEL_ROUTING] phase=evolver-carry-forward class=small

PHASE_DIRS=$(find "${PHASES_DIR}" -name "reflection.md" -type f 2>/dev/null | sort)
TOTAL_PHASES=$(echo "${PHASE_DIRS}" | grep -c reflection || true)

if [ "${TOTAL_PHASES}" -eq 0 ]; then
  echo "[carry-forward] No reflection.md files found"
  echo '{"total_phases": 0, "carry_forwards": [], "deduplicated": []}' > "${OUTPUT_FILE}"
  cat "${OUTPUT_FILE}"
  exit 0
fi

echo "[carry-forward] Found ${TOTAL_PHASES} reflections to scan"

python3 -c "
import re, json, os, sys

phases_dir = '${PHASES_DIR}'
output_file = '${OUTPUT_FILE}'
deep_mode = '${DEEP_MODE}' == '--deep'

carry_forwards_by_phase = []
all_items = []

for root, dirs, files in os.walk(phases_dir):
    if 'reflection.md' in files:
        phase_name = os.path.basename(root)
        path = os.path.join(root, 'reflection.md')
        try:
            with open(path) as f:
                content = f.read()
        except Exception:
            continue

        # Extract Carry-Forwards section
        match = re.search(r'#{1,3}\s+Carry.Forwards?\s*\n(.*?)(?=\n#{1,3}|\Z)', content, re.DOTALL | re.IGNORECASE)
        if not match:
            continue

        section = match.group(1)
        # Extract bullet items
        items = re.findall(r'^[-*]\s+(.+?)$', section, re.MULTILINE)
        items = [i.strip() for i in items if i.strip()]

        if items:
            # Try to get date from reflection.md header
            date_match = re.search(r'20\d{2}-\d{2}-\d{2}', content)
            date_str = date_match.group(0) if date_match else 'unknown'

            carry_forwards_by_phase.append({
                'phase': phase_name,
                'date': date_str,
                'items': items
            })
            all_items.extend(items)

# Simple dedup: exact string match
seen = set()
deduplicated = []
for item in all_items:
    key = item.lower().strip()
    if key not in seen:
        seen.add(key)
        deduplicated.append(item)

result = {
    'total_phases': len(carry_forwards_by_phase),
    'total_items': len(all_items),
    'unique_items': len(deduplicated),
    'carry_forwards': carry_forwards_by_phase,
    'deduplicated': deduplicated
}

with open(output_file, 'w') as f:
    json.dump(result, f, indent=2)

print(json.dumps(result, indent=2))
"

echo "[carry-forward] Output: ${OUTPUT_FILE}"
```

## Acceptance criteria

- `skills/process/pmpo-evolver/references/domain-taxonomy.md` exists with at minimum: AI tooling, Rust, LLM infrastructure, and developer tooling domain clusters
- Domain taxonomy includes general domain detection queries for unknown domains
- `scripts/carry-forward-aggregate.sh` is executable
- `bash scripts/carry-forward-aggregate.sh .kbd-orchestrator/phases default` exits 0 and outputs valid JSON
- Script finds at least one carry-forward from `pmpo-elicit/reflection.md` (which has a Carry-Forwards section)
- Model routing comment present: `[MODEL_ROUTING] phase=evolver-carry-forward class=small`

## Tasks

- [x] 1. `skills/process/pmpo-evolver/references/domain-taxonomy.md` exists with at minimum: AI tooling, Rust, LLM infrastructure, and developer tooling domain clusters
- [x] 2. Domain taxonomy includes general domain detection queries for unknown domains
- [x] 3. `scripts/carry-forward-aggregate.sh` is executable
- [x] 4. `bash scripts/carry-forward-aggregate.sh .kbd-orchestrator/phases default` exits 0 and outputs valid JSON
- [x] 5. Script finds at least one carry-forward from `pmpo-elicit/reflection.md` (which has a Carry-Forwards section)
- [x] 6. Model routing comment present: `[MODEL_ROUTING] phase=evolver-carry-forward class=small`
