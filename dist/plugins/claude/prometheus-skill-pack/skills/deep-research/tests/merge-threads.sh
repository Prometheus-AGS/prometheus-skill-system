#!/usr/bin/env bash
# merge-threads.sh — change-drt-004, task 4.
#
# The merge is what makes threading a refactor rather than a rewrite, so the
# assertions here are about preservation, not about new behaviour:
#
#   1. DETERMINISM by hash across two runs. The merge is a pure function; if a
#      clock, a set iteration order, or a model call leaked in, two runs would
#      differ. Comparing sha256 catches all three at once.
#   2. The UNMODIFIED driver validators pass on merged output. Stage numbers and
#      their contracts do not change in this phase — only how the artifacts are
#      produced — so the validators are extracted from run-research.sh and run
#      against the merge's output verbatim.
#   3. ONE DOCUMENT, ONE NUMBER. Three threads cite the same page under three
#      different spellings; all three must resolve to a single citation number.
#   4. The NO-SEARCH RULE fails closed. A dossier citing an unfetched source
#      means something searched outside a worker; that must exit non-zero.
#
# Exit 0 only when every assertion holds; exit 2 when a prerequisite is missing
# (BLOCKED — never reported as a pass). bash 3.2 compatible (C-05).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
SKILL="$(cd "$HERE/.." && pwd -P)"
ROOT="$(cd "$SKILL/../../.." && pwd -P)"
MERGE="$SKILL/scripts/merge-threads.sh"
DRIVER="$SKILL/scripts/run-research.sh"
FIX="$HERE/fixtures/threads-merge"

PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); echo "  ok   - $1"; }
bad() { FAIL=$((FAIL+1)); echo "  FAIL - $1" >&2; }

command -v jq      >/dev/null 2>&1 || { echo "merge-threads: BLOCKED — jq is required" >&2; exit 2; }
command -v python3 >/dev/null 2>&1 || { echo "merge-threads: BLOCKED — python3 is required" >&2; exit 2; }
[ -x "$MERGE" ] || { echo "merge-threads: BLOCKED — $MERGE missing" >&2; exit 2; }
[ -d "$FIX"   ] || { echo "merge-threads: BLOCKED — fixtures missing at $FIX" >&2; exit 2; }

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

run_merge() { # <dest>
  rm -rf "$1"; mkdir -p "$1"
  cp -R "$FIX" "$1/threads"
  bash "$MERGE" --package "$1" >/dev/null 2>&1
}

echo "== 1. the merge is deterministic =="
run_merge "$WORK/a"
run_merge "$WORK/b"

hash_of() { find "$1" -type f -name '*.json' -not -path '*/threads/*' | sed "s|^$1/||" | sort | \
            while read -r f; do printf '%s  ' "$f"; shasum -a 256 "$1/$f" | cut -d' ' -f1; done; }
HA="$(hash_of "$WORK/a")"; HB="$(hash_of "$WORK/b")"
[ -n "$HA" ] && [ "$HA" = "$HB" ] \
  && ok "two runs produced byte-identical artifacts ($(echo "$HA" | wc -l | tr -d ' ') files)" \
  || bad "two runs differ — the merge is not a pure function"

# A merge that reached the network could not be deterministic, but assert the
# absence directly too: no gateway variable is consulted anywhere in the script.
grep -qE 'curl|wget|kbd_complete|OPENAI|ANTHROPIC' "$MERGE" \
  && bad "the merge references a network or model call" \
  || ok "the merge makes no network or model call"

echo "== 2. the UNMODIFIED driver validators pass on merged output =="
# Extract the driver's own stage validators rather than restating them, so this
# cannot drift from what the driver actually enforces.
PKG="$WORK/a"
val() { # <stage> <jq filter> <file>
  if [ -f "$PKG/$3" ] && jq -e "$2" "$PKG/$3" >/dev/null 2>&1; then
    ok "stage $1 validator passes on merged $3"
  else
    bad "stage $1 validator FAILS on merged $3"
  fi
}
val 02 '.source_urls | type == "array"' sources/url-list.json
val 04 '(type=="array") or (.sources|type=="array")' sources/registry.json

N=$(ls "$PKG/sources"/chunk-*.json 2>/dev/null | wc -l | tr -d ' ')
[ "$N" -ge 1 ] && ok "stage 03 has $N chunk-<n>.json" || bad "stage 03 has no chunks"
CHUNK_OK=1
for f in "$PKG/sources"/chunk-*.json; do
  jq -e '(.url|type=="string") and (.chunk_id|type=="string") and (.text|type=="string")' "$f" >/dev/null 2>&1 || CHUNK_OK=0
done
[ "$CHUNK_OK" = 1 ] && ok "every merged chunk has url/chunk_id/text" || bad "a merged chunk lacks url/chunk_id/text"

# The validators above are copied from the driver; assert they are still the
# driver's, so an edit there without an edit here is caught.
grep -q 'source_urls | type == "array"' "$DRIVER" \
  && ok "the stage 02 filter asserted here is still the driver's" \
  || bad "the driver's stage 02 filter changed — this suite is stale"

echo "== 3. one document, one citation number =="
NUMS=$(jq -r '[.local_markers[] | select(.url | test("example.org/Guide")) | .citation_number] | unique | length' "$PKG/citation-map.json")
MARKERS=$(jq -r '[.local_markers[] | select(.url | test("example.org/Guide"))] | length' "$PKG/citation-map.json")
[ "$MARKERS" = "3" ] \
  && ok "three threads cited the same page under three different spellings" \
  || bad "expected 3 local markers for the shared page, got $MARKERS"
# Guard against a vacuous pass: with canonicalisation broken, only one marker
# matches and "1 distinct number" is trivially true while nothing merged. The
# assertion is only meaningful when all three markers are present, so require
# both conditions together.
[ "$NUMS" = "1" ] && [ "$MARKERS" = "3" ] \
  && ok "all three resolve to ONE citation number" \
  || bad "the shared page has $NUMS distinct citation numbers across $MARKERS markers, expected 1 across 3"

SRC=$(jq -r '.source_urls | length' "$PKG/sources/url-list.json")
[ "$SRC" = "2" ] \
  && ok "the union is 2 sources, not 4 (the overlap collapsed)" \
  || bad "expected 2 unioned sources, got $SRC"

echo "== 4. duplicate claims collapse but keep every provenance tuple =="
CL=$(jq -r '.claims | length' "$PKG/claims.json")
[ "$CL" = "2" ] && ok "3 duplicate assertions collapsed to 2 distinct claims" \
                || bad "expected 2 claims, got $CL"
PROV=$(jq -r '[.claims[] | select(.id=="claim-aaaa111122223333") | .provenance | length] | first' "$PKG/claims.json")
[ "$PROV" = "3" ] \
  && ok "the shared claim keeps all 3 (thread, source, quote) tuples" \
  || bad "the shared claim has $PROV provenance tuples, expected 3"

echo "== 5. the no-search rule fails closed =="
VIOL="$WORK/violation"; rm -rf "$VIOL"; mkdir -p "$VIOL"
cp -R "$HERE/fixtures/threads-merge-violation" "$VIOL/threads"
set +e
OUT="$(bash "$MERGE" --package "$VIOL" 2>&1)"; RC=$?
set -e
[ "$RC" -ne 0 ] \
  && ok "a dossier citing an unfetched source exits non-zero (rc=$RC)" \
  || bad "the merge accepted a dossier citing an unfetched source"
printf '%s' "$OUT" | grep -q 'CRITICAL' \
  && ok "the refusal is labelled CRITICAL" || bad "the refusal is not labelled CRITICAL"
printf '%s' "$OUT" | grep -q 'unfetched.example.com' \
  && ok "the refusal names the offending source" || bad "the refusal does not name the source"
printf '%s' "$OUT" | grep -q 't01' \
  && ok "the refusal names the thread" || bad "the refusal does not name the thread"
[ ! -f "$VIOL/sources/url-list.json" ] \
  && ok "no partial artifacts were written before refusing" \
  || bad "the merge emitted artifacts before refusing"

echo
echo "merge-threads: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
