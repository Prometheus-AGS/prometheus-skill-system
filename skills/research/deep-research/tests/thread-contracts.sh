#!/usr/bin/env bash
# thread-contracts.sh — change-drt-001, task 5.
#
# Validates the thread artifact contracts from references/thread-contracts.md
# against fixture threads. These are the shapes drt-004's merge folds back into
# the existing stage 02/03/04 artifacts, so a break here is a break in the
# refactor's central premise: that threading changes how stages are produced,
# never the contract they satisfy.
#
# The load-bearing assertion is the quote traceability check. Stage 05's rule is
# "verify meaning, not topic overlap" and "read before you label" — prose that
# can only be applied by judgement. A verbatim quote span turns it mechanical:
# the fetched chunk either contains the quote or the claim is not verified.
#
# Exit 0 only when every assertion holds. bash 3.2 compatible (C-05).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
SKILL="$(cd "$HERE/.." && pwd -P)"
FIX="$HERE/fixtures/thread-artifacts"
SCHEMAS="$SKILL/references/schemas"

PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   - $1"; }
bad()  { FAIL=$((FAIL+1)); echo "  FAIL - $1" >&2; }
check(){ d="$1"; shift; if "$@" >/dev/null 2>&1; then ok "$d"; else bad "$d"; fi; }

command -v jq >/dev/null 2>&1 || { echo "thread-contracts: jq is required" >&2; exit 2; }
[ -d "$FIX" ] || { echo "thread-contracts: fixtures missing at $FIX" >&2; exit 2; }

echo "== 1. schema validation =="
# jsonschema is a hard requirement, not a nicety. An earlier draft fell back to
# checking required keys when the import failed — which reports a pass for a
# validation that never ran. A gate that cannot run is BLOCKED, never passed.
# jsonschema is a hard requirement. Which python3 is first on PATH varies by
# shell (Homebrew's often lacks it while /usr/bin/python3 has it), so probe the
# candidates and use whichever can actually import it. If none can, this is
# BLOCKED (exit 2) — a gate that cannot run is never reported as a pass.
PY=""
for cand in python3 /usr/bin/python3 /opt/homebrew/bin/python3 /usr/local/bin/python3; do
  command -v "$cand" >/dev/null 2>&1 || continue
  if "$cand" -c 'import jsonschema' 2>/dev/null; then PY="$cand"; break; fi
done
if [ -z "$PY" ]; then
  echo "thread-contracts: BLOCKED — no python3 on this machine can import jsonschema." >&2
  echo "                  Tried: python3, /usr/bin/python3, /opt/homebrew/bin/python3," >&2
  echo "                  /usr/local/bin/python3. Install it (pip install jsonschema)." >&2
  echo "                  Refusing to report a pass for validation that did not happen." >&2
  exit 2
fi

validate() { # validate <schema> <instance>
  "$PY" - "$1" "$2" <<'PYEOF'
import json, sys, jsonschema
jsonschema.validate(json.load(open(sys.argv[2])), json.load(open(sys.argv[1])))
PYEOF
}
check "brief.json validates against thread-brief.schema.json" validate "$SCHEMAS/thread-brief.schema.json" "$FIX/t01/brief.json"
check "index.json validates against thread-index.schema.json" validate "$SCHEMAS/thread-index.schema.json" "$FIX/index.json"
check "sources.json validates against thread-sources.schema.json" validate "$SCHEMAS/thread-sources.schema.json" "$FIX/t01/sources.json"
check "claims.json validates against thread-claims.schema.json" validate "$SCHEMAS/thread-claims.schema.json" "$FIX/t01/claims.json"

echo "== 2. every claim carries a usable verbatim quote =="
check "every claim has a non-empty quote" \
  jq -e 'map(.quote | type == "string" and (length > 0)) | all' "$FIX/t01/claims.json"
check "no quote exceeds 40 words" \
  jq -e 'map((.quote | split(" ") | length) <= 40) | all' "$FIX/t01/claims.json"
check "every claim id follows the content-addressed rule (claim- + 16 hex)" \
  jq -e 'map(.id | test("^claim-[0-9a-f]{16}$")) | all' "$FIX/t01/claims.json"

echo "== 3. the quote is traceable — the assertion stage 05 depends on =="
# Each claim's source_id must exist in the same thread's sources.json, and the
# quote must actually appear in the chunk it names. This is what makes the
# "read before you label" rule checkable instead of aspirational.
TRACE_FAIL=0
while IFS= read -r c; do
  sid=$(printf '%s' "$c" | jq -r '.source_id')
  cid=$(printf '%s' "$c" | jq -r '.chunk_id')
  q=$(printf '%s' "$c"  | jq -r '.quote')
  jq -e --arg s "$sid" 'map(.id == $s) | any' "$FIX/t01/sources.json" >/dev/null 2>&1 || {
    echo "    source_id $sid not in sources.json" >&2; TRACE_FAIL=1; }
  chunk="$FIX/t01/chunks/${cid}.json"
  [ -f "$chunk" ] || { echo "    chunk $cid missing" >&2; TRACE_FAIL=1; continue; }
  jq -e --arg q "$q" '.text | contains($q)' "$chunk" >/dev/null 2>&1 || {
    echo "    quote not found verbatim in $cid" >&2; TRACE_FAIL=1; }
done < <(jq -c '.[]' "$FIX/t01/claims.json")
check "every claim's source_id resolves and its quote appears verbatim in the named chunk" test "$TRACE_FAIL" -eq 0

echo "== 4. dossier citations stay inside the thread =="
# A citation naming a source this thread never fetched is how director-sourced
# content would leak in on a harness that ignores tools: frontmatter. drt-004's
# merge treats it as CRITICAL; here it is a contract violation.
CITE_FAIL=0
for ref in $(grep -oE '\[src:[0-9a-f]{8}\]' "$FIX/t01/dossier.md" | sed 's/\[src:\(.*\)\]/\1/' | sort -u); do
  jq -e --arg s "$ref" 'map(.id == $s) | any' "$FIX/t01/sources.json" >/dev/null 2>&1 || {
    echo "    dossier cites $ref, absent from this thread's sources.json" >&2; CITE_FAIL=1; }
done
check "every dossier citation resolves to a source in the same thread" test "$CITE_FAIL" -eq 0
check "the dossier actually carries inline citations" grep -qE '\[src:[0-9a-f]{8}\]' "$FIX/t01/dossier.md"

echo "== 5. chunks are already in the stage 03 shape =="
# The merge copies these to sources/chunk-<n>.json unchanged, so the driver's
# stage 03 validator must accept them as-is.
check "every chunk has url, chunk_id, and text (stage 03 validator's rule)" \
  bash -c 'for f in "$1"/t01/chunks/*.json; do jq -e "(.url|type==\"string\") and (.chunk_id|type==\"string\") and (.text|type==\"string\")" "$f" >/dev/null || exit 1; done' _ "$FIX"

echo "== 6. the ledger has no silent holes =="
check "every non-complete row carries a reason" \
  jq -e '.threads | map(select(.status != "complete") | (.reason | type == "string" and (length > 0))) | all' "$FIX/index.json"
check "every status is in the enum" \
  jq -e '.threads | map(.status | IN("complete","partial","failed","timeout")) | all' "$FIX/index.json"
check "the fixture exercises a non-complete row (the check is not vacuous)" \
  jq -e '[.threads[] | select(.status != "complete")] | length > 0' "$FIX/index.json"

echo
echo "thread-contracts: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
