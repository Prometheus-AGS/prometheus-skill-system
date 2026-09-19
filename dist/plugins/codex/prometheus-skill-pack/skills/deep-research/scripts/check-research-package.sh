#!/usr/bin/env bash
# Drift check for the research package contract.
#
#   bash check-research-package.sh                 # contract mode: SKILL.md example, spec example,
#                                                  # a fresh export-package.sh output, A2UI table parity
#   bash check-research-package.sh --package DIR   # package mode: validate one package on disk
#   bash check-research-package.sh --derive DIR    # print the verification_status the rule derives
#
# Validation uses python3 `jsonschema` when importable. When it is not, python3
# compares the full key set and each declared type against the schema's
# `properties` and the result is PASS WITH NOTES. When python3 itself is absent
# every check that has a jq/awk form still runs (manifest structure, file
# presence, claim labels, the verification_status derivation) and the checks
# that do not (report/provenance agreement, a fresh export) are reported as
# NOTEs, so the result is PASS WITH NOTES, never a false PASS or a false FAIL.
# Either path exits non-zero on any mismatch. bash 3.2 compatible (C-05).
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
SKILL_DIR="$(cd "$HERE/.." && pwd)"
SCHEMA="$SKILL_DIR/references/schemas/research-manifest.schema.json"
SPEC="$SKILL_DIR/references/research-package-spec.md"
SKILL_MD="$SKILL_DIR/SKILL.md"
REGISTRY="$SKILL_DIR/../../../substrate/prometheus-research/src/a2ui/registry.rs"

MODE="contract"; PKG=""
while [ $# -gt 0 ]; do
  case "$1" in
    --package) shift; MODE="package"; PKG="${1:-}" ;;
    --derive)  shift; MODE="derive"; PKG="${1:-}" ;;   # print the derived verification_status and exit
    -h|--help) sed -n '2,12p' "$0"; exit 0 ;;
    *) echo "[check-research-package] unknown argument: $1" >&2; exit 2 ;;
  esac
  shift
done

FAIL=0; NOTES=0
ok()   { echo "[check-research-package] PASS  $*"; }
note() { echo "[check-research-package] NOTE  $*"; NOTES=1; }
bad()  { echo "[check-research-package] FAIL  $*" >&2; FAIL=1; }

command -v jq >/dev/null 2>&1 || { bad "jq is required"; exit 1; }
HAVE_PY=0; command -v python3 >/dev/null 2>&1 && HAVE_PY=1
# CHECK_RESEARCH_PACKAGE_NO_PYTHON=1 forces the jq fallback so it can be exercised on a machine that has python3.
[ "${CHECK_RESEARCH_PACKAGE_NO_PYTHON:-0}" = "1" ] && HAVE_PY=0

# validate_manifest_jq <file> <label>: fallback when python3 is absent. Compares
# the full key set, required keys, const and enum values, and the JSON type of
# every field against the schema's properties. Always PASS WITH NOTES because it
# cannot evaluate format, pattern, minimum, or nested object rules.
validate_manifest_jq() {
  local file="$1" label="$2" problems
  jq -e . "$file" >/dev/null 2>&1 || { bad "$label: not valid JSON"; return; }
  problems=$(jq -r --slurpfile s "$SCHEMA" '
    def jtype: if type == "number" then (if . == floor then "integer" else "number" end) else type end;
    ($s[0].properties) as $p | ($s[0].required) as $req | . as $doc |
    ( [ $req[] | . as $k | select(($doc | has($k)) | not) | "missing required \($k)" ]
    + [ $doc | keys[] | . as $k | select(($p | has($k)) | not) | "unexpected key \($k)" ]
    + [ $doc | to_entries[] | .key as $k | .value as $v | ($p[$k] // {}) as $d
        | select($d | has("const")) | select($v != $d.const)
        | "\($k): expected const \($d.const | tojson)" ]
    + [ $doc | to_entries[] | .key as $k | .value as $v | ($p[$k] // {}) as $d
        | select($d | has("enum")) | select(any($d.enum[]; . == $v) | not)
        | "\($k): \($v | tojson) not in enum" ]
    + [ $doc | to_entries[] | .key as $k | .value as $v | ($p[$k] // {}) as $d
        | select($d | has("type")) | ([$d.type] | flatten) as $ts | ($v | jtype) as $t
        | select( (any($ts[]; . == $t) | not) and (($t == "integer" and any($ts[]; . == "number")) | not) )
        | "\($k): type \($t) not in \($ts | tojson)" ]
    ) | .[]' "$file" 2>&1) || true
  if [ -n "$problems" ]; then
    bad "$label:"; echo "$problems" | sed 's/^/    /' >&2
  else
    note "$label: PASS WITH NOTES: python3 unavailable, jq structural check only"
  fi
}

# validate_manifest <file> <label>  → python3 (jsonschema when importable), else jq
validate_manifest() {
  local file="$1" label="$2"
  local out
  if [ "$HAVE_PY" -eq 0 ]; then validate_manifest_jq "$file" "$label"; return; fi
  if out=$(MANIFEST="$file" SCHEMA="$SCHEMA" python3 "$HERE/check-manifest-schema.py"); then
    case "$out" in
      jsonschema) ok "$label validates (jsonschema)" ;;
      structural) note "$label: PASS WITH NOTES: jsonschema unavailable, structural check only" ;;
    esac
  else
    bad "$label:"; echo "$out" | sed 's/^/    /' >&2
  fi
}

extract_json_block() {
  # extract_json_block <file> <marker-regex>: first ```json block after the marker.
  # awk (not python3) so contract mode runs on a machine without python3.
  awk -v mark="$2" '
    f == 0 && $0 ~ mark { f = 1; next }
    f == 1 && /^```json[[:space:]]*$/ { f = 2; next }
    f == 2 && /^```[[:space:]]*$/ { found = 1; exit }
    f == 2 { print }
    END { exit (found ? 0 : 1) }' "$1"
}

# check_labels_jq: the claim-label and derivation checks in jq, used when
# python3 is absent. Same rules as the python block below and the jq rule in
# references/okf-research-format.md (which it applies verbatim).
check_labels_jq() {
  local g="$PKG/graph.json" c="$PKG/checkpoint.json" s="$PKG/sources/credibility.json"
  local G S C fg ma out legacy
  G='{}'; S='{}'; C='{}'
  if [ -f "$g" ]; then
    if ! jq -e . "$g" >/dev/null 2>&1; then bad "graph.json is not valid JSON"; return; fi
    G=$(cat "$g")
  fi
  [ -f "$s" ] && jq -e . "$s" >/dev/null 2>&1 && S=$(cat "$s")
  [ -f "$c" ] && jq -e . "$c" >/dev/null 2>&1 && C=$(cat "$c")
  legacy=$(jq -n --argjson g "$G" '($g | has("claims") | not) and ($g | has("nodes"))')
  if [ ! -f "$g" ]; then
    note "graph.json absent; label checks skipped"
  elif [ "$legacy" = "true" ]; then
    note "graph.json is the legacy {nodes, edges} shape (no labels); accepted until change-rah-010 lands; package falls back to sources/credibility.json claims, else unverified"
  else
    out=$(jq -r '
      def labels: ["verified","unverified","blocked","inferred"];
      [ (.claims // []) | to_entries[] | .key as $i | .value as $c |
        ( ["id","text","label","critical","confidence","sources","contradicts"][] | . as $k
          | select(($c | has($k)) | not) | "graph.json claims[\($i)] (\($c.id)) missing \($k)" ),
        ( $c.label as $l | select(any(labels[]; . == $l) | not)
          | "graph.json claims[\($i)] (\($c.id)) label \($l | tojson) not in \(labels | tojson)" ),
        ( select(($c.label == "verified" or $c.label == "blocked") and (($c.evidence // "") == ""))
          | "graph.json claims[\($i)] (\($c.id)) is \($c.label) but carries no evidence" ),
        ( select($c.label != "inferred" and (($c.sources // []) | length) == 0)
          | "graph.json claims[\($i)] (\($c.id)) has no sources but is not inferred" )
      ] | .[]' "$g")
    if [ -n "$out" ]; then echo "$out" | while IFS= read -r line; do bad "$line"; done; FAIL=1
    else note "graph.json structural label check only (python3 unavailable): $(jq '(.claims // []) | length' "$g") claims"; fi
  fi
  local f key
  for f in citations contradictions; do
    [ -f "$PKG/$f.json" ] || continue
    jq -e . "$PKG/$f.json" >/dev/null 2>&1 || { bad "$f.json is not valid JSON"; continue; }
    key="$f"
    out=$(jq -r --arg key "$key" 'def labels: ["verified","unverified","blocked","inferred"];
      [ (.[$key] // [])[] | .label as $l | select(any(labels[]; . == $l) | not) | .id ] | tojson' "$PKG/$f.json")
    if [ "$out" != "[]" ]; then bad "$f.json entries without a valid label: $out"
    elif [ "$(jq --arg key "$key" '(.[$key] // []) | length' "$PKG/$f.json")" != "0" ]; then ok "$f.json labels valid"; fi
  done
  # Derivation: the jq rule from okf-research-format.md.
  if [ -f "$g" ] && [ -f "$c" ]; then
    local derived declared
    derived=$(derive_status_jq "$PKG")
    declared=$(jq -r '.verification_status // ""' "$PKG/manifest.json")
    if [ "$declared" != "$derived" ]; then bad "verification_status declared '$declared' but derived '$derived' from claim labels and checkpoint"
    else ok "verification_status $declared agrees with the derivation rule"; fi
  fi
}

# derive_status_jq <package-dir>: print the verification_status the documented
# rule derives from graph.json (or sources/credibility.json), checkpoint.json,
# and the stage 09 gate values (checkpoint first, report.md frontmatter second).
# This is the rule in references/okf-research-format.md, verbatim. The driver
# calls it through `--derive` after the report review so report.md and the
# manifest carry the derived value rather than the value stage 09 guessed.
derive_status_jq() {
  local pkg="$1" G S C fg ma
  G='{}'; S='{}'; C='{}'
  [ -f "$pkg/graph.json" ] && jq -e . "$pkg/graph.json" >/dev/null 2>&1 && G=$(cat "$pkg/graph.json")
  [ -f "$pkg/sources/credibility.json" ] && jq -e . "$pkg/sources/credibility.json" >/dev/null 2>&1 && S=$(cat "$pkg/sources/credibility.json")
  [ -f "$pkg/checkpoint.json" ] && jq -e . "$pkg/checkpoint.json" >/dev/null 2>&1 && C=$(cat "$pkg/checkpoint.json")
  fg=""; ma=""
  if [ -f "$pkg/report.md" ]; then
    fg=$(awk 'NR==1 && /^---/{n=1; next} n==1 && /^---/{exit} n==1 && /^feynman_grade:/{sub(/^feynman_grade:[[:space:]]*/,""); gsub(/"/,""); print}' "$pkg/report.md")
    ma=$(awk 'NR==1 && /^---/{n=1; next} n==1 && /^---/{exit} n==1 && /^misconceptions_absent:/{sub(/^misconceptions_absent:[[:space:]]*/,""); gsub(/"/,""); print}' "$pkg/report.md")
  fi
  C=$(jq --arg fg "$fg" --arg ma "$ma" '
    .feynman_grade //= (($fg | tonumber?) // null) | .misconceptions_absent //= (($ma | tonumber?) // null)' <<<"$C")
  jq -rn --argjson g "$G" --argjson s "$S" --argjson c "$C" '
    def claimset: if (($g.claims // []) | length) > 0 then $g.claims
                  else [$s.verified_sources[]?.claims[]? | . + {critical: true}] end;
    def labels: [claimset[] | .label | select(. != null)];
    def critical: [claimset[] | select(.critical == true)];
    if ((($c.stages_completed // []) | index("05")) == null) or ((labels | length) == 0) then "unverified"
    elif ([critical[] | select(.label != "verified")] | length) > 0
         or ([claimset[] | select(.label == "blocked" or .label == "unverified")] | length) > 0
         or (($c.review.verdict // "") == "BLOCK")
         or (($c.blocked_review // null) != null)
         or ($c.scale == "full" and ($c.integrations.adversarial_review_used | not))
         or ($c.integrations.feynman_gate_used | not)
         or (($c.feynman_grade // 0) < 0.7)
         or (($c.misconceptions_absent // 0) < 1.0)
    then "partial"
    else "verified" end'
}

if [ "$MODE" = "derive" ]; then
  [ -d "$PKG" ] || { echo "[check-research-package] package directory not found: $PKG" >&2; exit 1; }
  derive_status_jq "$PKG"
  exit 0
elif [ "$MODE" = "contract" ]; then
  [ -f "$SCHEMA" ] || { bad "schema missing: $SCHEMA"; exit 1; }
  TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

  # 1. SKILL.md literal example
  if extract_json_block "$SKILL_MD" 'manifest\.json.*literal, schema-valid example' > "$TMP/skill-example.json" 2>/dev/null; then
    validate_manifest "$TMP/skill-example.json" "SKILL.md manifest example"
  else
    bad "SKILL.md has no manifest.json example block"
  fi

  # 2. Spec example, and identity with the SKILL.md copy
  if extract_json_block "$SPEC" '## manifest\.json' > "$TMP/spec-example.json" 2>/dev/null; then
    validate_manifest "$TMP/spec-example.json" "research-package-spec.md manifest example"
    if cmp -s "$TMP/skill-example.json" "$TMP/spec-example.json"; then
      ok "SKILL.md example is byte-for-byte the spec example"
    else
      bad "SKILL.md example differs from the spec example byte-for-byte (hand-synced copies drifted):"
      diff "$TMP/spec-example.json" "$TMP/skill-example.json" | sed 's/^/    /' >&2 || true
    fi
  else
    bad "spec has no manifest.json example block"
  fi

  # 3. A fresh export from a fixture package
  FX="$TMP/drift-check-fixture-20260905-0000"; mkdir -p "$FX/sources"
  printf '{"job_id":"job-0-fixture","query":"drift check fixture query","depth":"shallow","scale":"direct","citation_style":"APA","kb_ids":[],"model_routing":{"01":"session-default"},"stages_completed":["01","02","03","05","09","10"],"created_at":"2026-09-05T00:00:00Z","completed_at":"2026-09-05T00:10:00Z","integrations":{}}\n' > "$FX/checkpoint.json"
  printf -- '---\ntype: research-report\ntitle: "Fixture"\nconfidence: 0.5\nverification_status: partial\nfeynman_grade: 0.7\nmisconceptions_absent: 1.0\n---\n' > "$FX/report.md"
  printf -- '- **Verification:** PASS WITH NOTES\n' > "$FX/drift-check-fixture.provenance.md"
  if [ "$HAVE_PY" -eq 0 ]; then
    note "python3 unavailable; export-package.sh needs it, fresh-export check skipped"
  elif bash "$HERE/export-package.sh" "$FX" >/dev/null 2>"$TMP/export.log"; then
    validate_manifest "$FX/manifest.json" "fresh export-package.sh output"
  else
    bad "export-package.sh failed on the fixture:"; sed 's/^/    /' "$TMP/export.log" >&2
  fi

  # 4. A2UI table in SKILL.md equals the registry
  if [ -f "$REGISTRY" ]; then
    awk '/^### A2UI component endpoints/{f=1;next} /^## /{f=0} f' "$SKILL_MD" | grep -o '^| `[a-z_]*`' | tr -d '|` ' | sort > "$TMP/table.txt"
    grep -o 'components.insert("[a-z_]*"' "$REGISTRY" | grep -o '"[a-z_]*"' | tr -d '"' | sort > "$TMP/registry.txt"
    if diff -q "$TMP/table.txt" "$TMP/registry.txt" >/dev/null; then
      ok "SKILL.md A2UI table matches registry.rs ($(wc -l < "$TMP/registry.txt" | tr -d ' ') components)"
    else
      bad "SKILL.md A2UI table differs from registry.rs:"; diff "$TMP/table.txt" "$TMP/registry.txt" | sed 's/^/    /' >&2
    fi
  else
    note "registry.rs not found; A2UI parity skipped"
  fi
else
  [ -d "$PKG" ] || { bad "package directory not found: $PKG"; exit 1; }
  [ -f "$PKG/manifest.json" ] || { bad "no manifest.json in $PKG"; exit 1; }
  validate_manifest "$PKG/manifest.json" "manifest.json"
  # files listed in manifest exist
  for key in report provenance plan graph citations contradictions index sources_dir; do
    f=$(jq -r --arg k "$key" '.files[$k] // ""' "$PKG/manifest.json")
    [ -n "$f" ] || continue
    if [ -e "$PKG/$f" ]; then ok "files.$key present ($f)"; else bad "files.$key missing on disk ($f)"; fi
  done
  if [ "$HAVE_PY" -eq 0 ]; then
    note "python3 unavailable; report.md/provenance agreement with the manifest not checked"
    check_labels_jq
  else
  # report frontmatter and provenance verdict agree with the manifest
  # Exit codes from the python blocks: 0 pass, 3 pass with notes, anything else fail.
  if MANIFEST="$PKG/manifest.json" PKG="$PKG" \
     python3 "$HERE/check-manifest-files.py"
  then :; else case $? in 3) NOTES=1 ;; *) FAIL=1 ;; esac; fi
  # Claim labels and the package-label derivation (change-rah-005). graph.json in the
  # spec shape validates against research-graph.schema.json; the legacy {nodes, edges}
  # shape is tolerated with a NOTE until change-rah-010. Labels are also checked on
  # citations.json and contradictions.json entries, and verification_status is
  # recomputed from graph.json + checkpoint.json with the rule in okf-research-format.md.
  GRAPH_SCHEMA="$SKILL_DIR/references/schemas/research-graph.schema.json"
    if PKG="$PKG" GRAPH_SCHEMA="$GRAPH_SCHEMA" MANIFEST="$PKG/manifest.json" \
       python3 "$HERE/check-graph-claims.py"
  then :; else case $? in 3) NOTES=1 ;; *) FAIL=1 ;; esac; fi
  fi
fi

if [ "$FAIL" -ne 0 ]; then echo "[check-research-package] RESULT: FAIL" >&2; exit 1; fi
if [ "$NOTES" -ne 0 ]; then echo "[check-research-package] RESULT: PASS WITH NOTES"; else echo "[check-research-package] RESULT: PASS"; fi
