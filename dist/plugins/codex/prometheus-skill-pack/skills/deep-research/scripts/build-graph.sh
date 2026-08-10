#!/usr/bin/env bash
set -euo pipefail

# Build a knowledge graph from collected sources JSON.
# Usage: bash build-graph.sh sources/registry.json
# Output: JSON {nodes[], edges[]} compatible with surreal-memory create_entity/create_relation

SOURCES_JSON="${1:-${SOURCES_JSON:-}}"

if [[ -z "$SOURCES_JSON" ]]; then
  echo '{"error": "SOURCES_JSON path required. Usage: bash build-graph.sh sources/registry.json"}' >&2
  exit 1
fi

if [[ ! -f "$SOURCES_JSON" ]]; then
  echo "{\"error\": \"File not found: $SOURCES_JSON\"}" >&2
  exit 1
fi

if ! command -v python3 &>/dev/null; then
  echo '{"error": "python3 required for graph building"}' >&2
  exit 1
fi

python3 - "$SOURCES_JSON" <<'PYEOF'
import json
import sys
import hashlib

sources_path = sys.argv[1]
with open(sources_path) as f:
    sources = json.load(f)

nodes = []
edges = []
seen_nodes = set()

def node_id(name):
    return "node-" + hashlib.md5(name.encode()).hexdigest()[:8]

for source in sources:
    url = source.get("url", "")
    domain = source.get("domain", url.split("/")[2] if "//" in url else url)
    score = source.get("credibility_score", 50)
    claims = source.get("claims", [])

    src_id = node_id(url)
    if src_id not in seen_nodes:
        nodes.append({
            "id": src_id,
            "type": "ResearchSource",
            "name": url,
            "properties": {
                "domain": domain,
                "credibility_score": score
            }
        })
        seen_nodes.add(src_id)

    for claim in claims:
        claim_id = node_id(claim)
        if claim_id not in seen_nodes:
            nodes.append({
                "id": claim_id,
                "type": "Claim",
                "name": claim,
                "properties": {"text": claim}
            })
            seen_nodes.add(claim_id)

        edges.append({
            "from": claim_id,
            "to": src_id,
            "relation": "cites"
        })

print(json.dumps({"nodes": nodes, "edges": edges}, indent=2))
PYEOF
