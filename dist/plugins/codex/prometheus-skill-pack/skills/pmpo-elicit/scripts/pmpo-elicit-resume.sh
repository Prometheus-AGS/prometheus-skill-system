#!/usr/bin/env bash
# pmpo-elicit-resume.sh — read a completed elicitation result and output answer+provenance
#
# Usage:
#   pmpo-elicit-resume.sh <elicit-dir>
#
# Outputs JSON to stdout on success:
#   {"answer": "...", "provenance": "...", "id": "..."}
#
# Exit codes:
#   0 — success: answer+provenance on stdout
#   1 — not ready: result.json absent or malformed

set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "[pmpo-elicit-resume] ERROR: usage: $0 <elicit-dir>" >&2
  exit 1
fi

ELICIT_DIR="$1"
RESULT_FILE="${ELICIT_DIR}/result.json"
CHECKPOINT_FILE="${ELICIT_DIR}/checkpoint.json"

if [[ ! -f "$RESULT_FILE" ]]; then
  echo "[pmpo-elicit-resume] NOT READY: result.json not found in ${ELICIT_DIR}" >&2
  exit 1
fi

# Validate result.json has required fields using python3 (available on all platforms)
VALIDATION=$(python3 - "$RESULT_FILE" <<'PYEOF'
import json, sys

try:
    with open(sys.argv[1]) as f:
        d = json.load(f)
except Exception as e:
    print(f"ERROR: cannot parse result.json: {e}", file=sys.stderr)
    sys.exit(1)

if d.get("kind") != "result":
    print("ERROR: result.json kind must be 'result'", file=sys.stderr)
    sys.exit(1)

if d.get("answer") is None:
    print("ERROR: result.json answer is null or missing", file=sys.stderr)
    sys.exit(1)

valid_provenances = {"user", "source", "research", "implicit"}
prov = d.get("provenance", "")
if prov not in valid_provenances:
    print(f"ERROR: provenance must be one of {valid_provenances}, got '{prov}'", file=sys.stderr)
    sys.exit(1)

# Output the key fields as JSON
import json
out = {
    "answer": d["answer"],
    "provenance": d["provenance"],
    "id": d.get("id", ""),
}
if d.get("source_ref"):
    out["source_ref"] = d["source_ref"]
if d.get("confidence") is not None:
    out["confidence"] = d["confidence"]

print(json.dumps(out))
PYEOF
) || exit 1

# Mark checkpoint as resolved
if [[ -f "$CHECKPOINT_FILE" ]]; then
  if command -v python3 &>/dev/null; then
    python3 - "$CHECKPOINT_FILE" <<'PYEOF'
import json, sys
with open(sys.argv[1]) as f:
    d = json.load(f)
d["status"] = "resolved"
with open(sys.argv[1], "w") as f:
    json.dump(d, f, indent=2)
    f.write("\n")
PYEOF
  fi
fi

echo "$VALIDATION"
