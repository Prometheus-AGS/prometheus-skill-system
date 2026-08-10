#!/usr/bin/env bash
set -euo pipefail

# Detect contradictory claims across sources.
# Usage: bash detect-contradictions.sh sources/registry.json
# Output: JSON array of {claim_a, source_a, claim_b, source_b, topic, severity}

SOURCES_JSON="${1:-${SOURCES_JSON:-}}"

if [[ -z "$SOURCES_JSON" ]]; then
  echo '{"error": "SOURCES_JSON path required. Usage: bash detect-contradictions.sh sources/registry.json"}' >&2
  exit 1
fi

if [[ ! -f "$SOURCES_JSON" ]]; then
  echo "{\"error\": \"File not found: $SOURCES_JSON\"}" >&2
  exit 1
fi

if ! command -v python3 &>/dev/null; then
  echo '{"error": "python3 required for contradiction detection"}' >&2
  exit 1
fi

python3 - "$SOURCES_JSON" <<'PYEOF'
import json
import sys
import re

sources_path = sys.argv[1]
with open(sources_path) as f:
    sources = json.load(f)

# Simple heuristic: detect numeric claim contradictions on the same topic keyword
# Production implementation would use LLM-assisted claim comparison

CONTRADICTION_PATTERNS = [
    (r'(\d+(?:\.\d+)?)\s*%', 'percentage'),
    (r'(\d+(?:\.\d+)?)\s*(?:ms|milliseconds?)', 'latency_ms'),
    (r'(\d+(?:[,\d]+)?)\s*(?:QPS|queries?\s*per\s*second)', 'throughput_qps'),
    (r'(\d+(?:\.\d+)?)\s*(?:GB|MB|TB)', 'storage'),
]

contradictions = []

# Group claims by topic keyword (naive: shared noun phrases)
claim_map = {}
for source in sources:
    url = source.get("url", "")
    for claim in source.get("claims", []):
        for pattern, topic in CONTRADICTION_PATTERNS:
            matches = re.findall(pattern, claim, re.IGNORECASE)
            if matches:
                key = topic
                if key not in claim_map:
                    claim_map[key] = []
                claim_map[key].append({
                    "claim": claim,
                    "url": url,
                    "value": matches[0]
                })

# Find conflicts: same topic, significantly different values
for topic, entries in claim_map.items():
    if len(entries) < 2:
        continue
    for i in range(len(entries)):
        for j in range(i + 1, len(entries)):
            a, b = entries[i], entries[j]
            if a["url"] == b["url"]:
                continue
            try:
                va = float(a["value"].replace(",", ""))
                vb = float(b["value"].replace(",", ""))
                ratio = max(va, vb) / max(min(va, vb), 0.001)
                if ratio > 2.0:
                    severity = "high" if ratio > 5.0 else "medium"
                    contradictions.append({
                        "topic": topic,
                        "claim_a": a["claim"],
                        "source_a": a["url"],
                        "value_a": a["value"],
                        "claim_b": b["claim"],
                        "source_b": b["url"],
                        "value_b": b["value"],
                        "ratio": round(ratio, 2),
                        "severity": severity
                    })
            except (ValueError, TypeError):
                pass

print(json.dumps(contradictions, indent=2))
PYEOF
