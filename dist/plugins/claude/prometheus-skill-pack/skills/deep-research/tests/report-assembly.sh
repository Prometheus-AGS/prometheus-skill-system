#!/usr/bin/env bash
# report-assembly.sh — change-drt-005, task 5.
#
# The multi-pass report only works if the claim-set invariant is actually
# enforced, so every assertion here drives the real assembler against a fixture
# that violates one rule, and requires a non-zero exit naming the offender.
#
# The four rules, and why each exists:
#
#   claim-set drift   a section citing outside its assignment means the report's
#                     evidence base is whatever each writer reached for
#   missing reference a cited id resolving to no claim is a citation to nothing
#   label misuse      "confirms" over an `inferred` claim reads as authoritative
#                     and is not — the failure a long report hides best
#   editor may only   an editor that can ADD a citation can add an unsupported
#   remove            one at the point where prose reads most fluently
#
# One trap this suite deliberately avoids: testing "the editor added a claim" by
# adding an id ALREADY cited elsewhere. The cited set does not grow, so the
# check correctly passes and the test proves nothing. The add case below uses a
# claim no section cited in pass 1.
#
# Exit 0 only when every assertion holds; exit 2 when a prerequisite is missing
# (BLOCKED — never a pass). bash 3.2 compatible (C-05).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
SKILL="$(cd "$HERE/.." && pwd -P)"
AS="$SKILL/scripts/assemble-report.sh"
GOOD="$HERE/fixtures/report-passes/good"

PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); echo "  ok   - $1"; }
bad() { FAIL=$((FAIL+1)); echo "  FAIL - $1" >&2; }

command -v jq      >/dev/null 2>&1 || { echo "report-assembly: BLOCKED — jq required" >&2; exit 2; }
command -v python3 >/dev/null 2>&1 || { echo "report-assembly: BLOCKED — python3 required" >&2; exit 2; }
[ -x "$AS"   ] || { echo "report-assembly: BLOCKED — $AS missing" >&2; exit 2; }
[ -d "$GOOD" ] || { echo "report-assembly: BLOCKED — fixture missing at $GOOD" >&2; exit 2; }

WORK="$(mktemp -d)"; trap 'rm -rf "$WORK"' EXIT
fresh() { rm -rf "$WORK/$1"; cp -R "$GOOD" "$WORK/$1"; echo "$WORK/$1"; }

# Assert the assembler refuses, with exit 2 and a message naming the offender.
refuses() { # <label> <pkg> <needle> [--post-edit]
  local label="$1" pkg="$2" needle="$3"; shift 3
  local out rc
  set +e; out="$(bash "$AS" --package "$pkg" "$@" 2>&1 >/dev/null)"; rc=$?; set -e
  if [ "$rc" -ne 2 ]; then bad "$label: expected exit 2, got $rc"; return; fi
  if printf '%s' "$out" | grep -q "$needle"; then ok "$label (names $needle)"
  else bad "$label: refused but the message does not name $needle"; fi
}

echo "== 1. the happy path assembles =="
P="$(fresh happy)"
if bash "$AS" --package "$P" >/dev/null 2>&1; then ok "a conforming report assembles"
else bad "the good fixture failed to assemble"; fi

[ -f "$P/report/draft.md" ] && ok "draft.md is written" || bad "no draft.md"

# Markers must be resolved to the merge's global numbers, not left as ids.
if grep -qE '\[[0-9]+\]' "$P/report/draft.md" && ! grep -qE '\[claim-' "$P/report/draft.md"; then
  ok "every claim id resolved to a global citation number"
else
  bad "draft.md still contains unresolved claim markers"
fi

# The numbering authority is the merge, not this script: the numbers in the
# draft must be the ones citation-map.json assigned.
N1="$(jq -r '.citations[] | select(.entity_id=="src-aaa") | .citation_number' "$P/citation-map.json")"
grep -q "\[$N1\]" "$P/report/draft.md" \
  && ok "the number used is the one citation-map.json assigned ($N1)" \
  || bad "the draft's numbering does not match citation-map.json"

echo "== 2. a section may cite only what it was assigned =="
P="$(fresh drift)"
echo 'Also see [claim-aaaa1111].' >> "$P/report/sections/s02.md"
refuses "claim-set drift is caught" "$P" "s02"

echo "== 3. a cited id must resolve to a real claim =="
P="$(fresh missing)"
python3 - "$P" <<'PY'
import json,sys
p=sys.argv[1]+"/report/outline.json"; d=json.load(open(p))
d["sections"][0]["claim_ids"].append("claim-zzzz9999")
json.dump(d,open(p,"w"))
PY
echo 'And [claim-zzzz9999].' >> "$P/report/sections/s01.md"
refuses "missing reference is caught" "$P" "claim-zzzz9999"

echo "== 4. labels bound how strongly a claim may be written =="
P="$(fresh label)"
cat > "$P/report/sections/s01.md" <<'X'
## Collection ceilings

The standard tier caps collections at ten million vectors [claim-aaaa1111].
Research confirms the ceiling is a soft quota rather than a hard limit
[claim-bbbb2222].
X
refuses "an inferred claim written as confirmed is caught" "$P" "claim-bbbb2222"

# The mirror case: the same wording over a VERIFIED claim must be allowed, or
# the check would just ban a vocabulary rather than enforce the label rule.
P="$(fresh labelok)"
cat > "$P/report/sections/s01.md" <<'X'
## Collection ceilings

Research confirms the standard tier caps collections at ten million vectors
[claim-aaaa1111]. The quota may be raised on request [claim-bbbb2222].
X
if bash "$AS" --package "$P" >/dev/null 2>&1; then
  ok "the same wording over a verified claim is allowed"
else
  bad "the label check bans a vocabulary instead of enforcing the label"
fi

echo "== 5. the editor may remove, never introduce =="
# ADD: use a claim NO section cited in pass 1, or the cited set does not grow
# and the assertion passes without testing anything.
P="$(fresh editadd)"
python3 - "$P" <<'PY'
import json,sys
p=sys.argv[1]+"/graph.json"; d=json.load(open(p))
d["claims"].append({"id":"claim-dddd4444","label":"verified",
                    "text":"A claim no section cited.","source_id":"src-aaa"})
json.dump(d,open(p,"w"))
PY
bash "$AS" --package "$P" >/dev/null 2>&1
BEFORE="$(jq -r '.cited | length' "$P/report/cited-claims.json")"
[ "$BEFORE" = "3" ] \
  && ok "pass 1 recorded 3 cited claims (the new one is genuinely uncited)" \
  || bad "pass 1 recorded $BEFORE cited claims, expected 3"
python3 - "$P" <<'PY'
import json,sys
p=sys.argv[1]+"/report/outline.json"; d=json.load(open(p))
d["sections"][1]["claim_ids"].append("claim-dddd4444")
json.dump(d,open(p,"w"))
PY
echo 'The editor introduced [claim-dddd4444].' >> "$P/report/sections/s02.md"
refuses "the editor ADDING a claim is caught" "$P" "claim-dddd4444" --post-edit

# REMOVE without logging.
P="$(fresh editrm)"
bash "$AS" --package "$P" >/dev/null 2>&1
sed -i.bak 's/\[claim-cccc3333\]//' "$P/report/sections/s02.md" && rm -f "$P/report/sections/s02.md.bak"
refuses "an unlogged removal is caught" "$P" "claim-cccc3333" --post-edit

# REMOVE with a log entry.
printf '\n## Decision log\n\n### Coherence edit\nRemoved claim-cccc3333 from s02: undated vendor figure.\n' >> "$P/plan.md"
if bash "$AS" --package "$P" --post-edit >/dev/null 2>&1; then
  ok "a logged removal is allowed"
else
  bad "a logged removal was refused"
fi

echo "== 6. no section may exceed the 3000-word cap =="
# The schema enforces it; assert the fixture and the schema agree, so a future
# outline that raises a budget past the cap fails validation rather than
# silently asking one call for the whole report again.
CAP="$(jq -r '.properties.sections.items.properties.word_budget.maximum' \
      "$SKILL/references/schemas/report-outline.schema.json")"
[ "$CAP" = "3000" ] && ok "the schema caps word_budget at 3000" || bad "schema cap is $CAP, expected 3000"
OVER="$(jq -r '[.sections[] | select(.word_budget > 3000)] | length' "$GOOD/report/outline.json")"
[ "$OVER" = "0" ] && ok "no fixture section exceeds the cap" || bad "$OVER fixture sections exceed the cap"

echo
echo "report-assembly: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
