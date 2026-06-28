#!/usr/bin/env bash
# carry-forward-aggregate.sh — Walk all KBD reflection files, extract carry-forwards, deduplicate
# Usage: carry-forward-aggregate.sh [<kbd-orchestrator-dir>]
# Output: JSON array of deduplicated carry-forward items
# [MODEL_ROUTING] phase=evolver-carry-forward class=small (none — pure bash+python3)
set -euo pipefail

KBD_DIR="${1:-.kbd-orchestrator}"

echo "[carry-forward] Scanning: ${KBD_DIR}/phases/*/reflection.md" >&2

if [ ! -d "${KBD_DIR}/phases" ]; then
  echo "[carry-forward] No phases directory found at ${KBD_DIR}/phases" >&2
  printf '{"total_phases_scanned": 0, "total_carry_forwards": 0, "carry_forwards": []}\n'
  exit 0
fi

# Collect all reflection.md files
REFLECTION_FILES=$(find "${KBD_DIR}/phases" -name "reflection.md" 2>/dev/null | sort)

if [ -z "${REFLECTION_FILES}" ]; then
  echo "[carry-forward] No reflection.md files found" >&2
  printf '{"total_phases_scanned": 0, "total_carry_forwards": 0, "carry_forwards": []}\n'
  exit 0
fi

FILE_COUNT=$(echo "${REFLECTION_FILES}" | wc -l | tr -d ' ')
echo "[carry-forward] Found ${FILE_COUNT} reflection file(s)" >&2

python3 -c "
import json, re, sys, os

reflection_files = '''${REFLECTION_FILES}'''.strip().split('\n')
reflection_files = [f for f in reflection_files if f.strip()]

all_carry_forwards = []
seen_texts = set()
phases_scanned = 0

for path in reflection_files:
    if not os.path.exists(path):
        continue
    phases_scanned += 1
    phase_name = os.path.basename(os.path.dirname(path))

    with open(path) as f:
        content = f.read()

    # Find carry-forwards section — match ## Carry-Forward(s) through next ## heading or EOF
    section_match = re.search(
        r'##\s+Carry[- ]?Forwards?\s*\n(.*?)(?=\n##|\Z)',
        content,
        re.DOTALL | re.IGNORECASE
    )
    if not section_match:
        continue

    section = section_match.group(1).strip()

    # Parse bullet points
    bullets = re.findall(r'^[-*]\s+(.+)', section, re.MULTILINE)
    # Also catch numbered items
    bullets += re.findall(r'^\d+\.\s+(.+)', section, re.MULTILINE)

    for bullet in bullets:
        text = bullet.strip()
        if not text:
            continue

        # Dedup by normalized text (lowercase, collapse whitespace)
        key = re.sub(r'\s+', ' ', text.lower()).strip()
        if key in seen_texts:
            continue
        seen_texts.add(key)

        all_carry_forwards.append({
            'text': text,
            'source_phase': phase_name,
            'source_file': path
        })

result = {
    'total_phases_scanned': phases_scanned,
    'total_carry_forwards': len(all_carry_forwards),
    'carry_forwards': all_carry_forwards
}
print(json.dumps(result, indent=2))
"

echo "[carry-forward] Done." >&2
