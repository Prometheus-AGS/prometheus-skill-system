#!/usr/bin/env bash
# assert-ci-coverage.sh — pin which invariants CI actually verifies.
#
# Exit: 0 coverage matches expectation · 1 usage · 2 coverage REGRESSED
#
# WHY PIN A PARTIAL RESULT
# CI can verify only two of four invariants, because `know-me-system` is private
# and in a different org (Know-Me-Tools/know-me-system). Reaching it would mean
# putting a cross-org PAT in a public workflow to compare two version strings —
# a poor trade.
#
# SKIP is honest. But a SKIP that *appears* without anyone noticing is how
# coverage rots: an invariant verified today silently becomes unverified
# tomorrow and nothing complains, because SKIP never fails a build. So the
# expected split is asserted, and any drift in EITHER direction fails.
#
# Expected on a CI runner:
#   PASS  loro-minor-aligned        (flint-realtime-fabric is public)
#   PASS  iroh-floor-1.0.2          (in-repo)
#   SKIP  wasmtime-major-aligned    (needs know-me-system)
#   SKIP  wit-world-version-pinned  (needs know-me-system)
#
# bash 3.2 compatible.
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
command -v python3 >/dev/null 2>&1 || { echo "[coverage] python3 required" >&2; exit 1; }

OUT="$(bash "$HERE/check-invariants.sh" --json 2>/dev/null)"
[ -n "$OUT" ] || { echo "[coverage] check-invariants.sh produced no JSON" >&2; exit 2; }
printf '%s\n' "$OUT"

printf '%s' "$OUT" | python3 -c '
import json, sys
d = json.load(sys.stdin)
st = {r["invariant"]: r["status"] for r in d["results"]}
want_pass = {"loro-minor-aligned", "iroh-floor-1.0.2"}
want_skip = {"wasmtime-major-aligned", "wit-world-version-pinned"}
bad  = ["%s: want PASS, got %s" % (k, st.get(k, "ABSENT")) for k in sorted(want_pass) if st.get(k) != "PASS"]
bad += ["%s: want SKIP, got %s" % (k, st.get(k, "ABSENT")) for k in sorted(want_skip) if st.get(k) != "SKIP"]
if bad:
    print("[coverage] REGRESSED:", file=sys.stderr)
    for b in bad:
        print("  " + b, file=sys.stderr)
    print("  (a SKIP becoming PASS is good news — update this expectation.)", file=sys.stderr)
    raise SystemExit(2)
print("[coverage] as expected: 2 verified, 2 skipped (know-me-system is private)")
'
