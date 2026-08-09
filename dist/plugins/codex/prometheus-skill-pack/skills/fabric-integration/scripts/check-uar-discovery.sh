#!/usr/bin/env bash
# check-uar-discovery.sh — verify the premise that component work is in-repo.
#
# Usage: check-uar-discovery.sh
# Exit:  0 premise holds · 2 premise BROKEN · 3 unverifiable (UAR not present)
#
# WHY THIS EXISTS
# The mobile-skill-portability plan orders WIT and component work BEFORE the
# UAR host fix, on the strength of three cross-repo facts:
#
#   1. UAR declares this repo as a submodule at crates/prometheus-skill-system
#   2. UAR's wasm discovery reads crates/prometheus-skill-system/skills
#   3. this repo's origin IS that submodule URL
#
# Together they mean producing components is in-repo work and UAR picks them up
# by bumping its pointer. Adversarial review flagged that basing an ordering on
# files a single-repo reviewer cannot open is unsafe — so the check is
# mechanical and re-runnable rather than a claim in a document.
#
# If UAR's discovery path changes, the in-repo premise collapses and the
# ordering must be RE-PLANNED, not continued. Exit 2 says so loudly.
#
# Exit 3 (unverifiable) is deliberately NOT exit 0: an absent UAR checkout
# proves nothing, and reporting "holds" because a file could not be read is how
# a guard becomes decorative.
#
# bash 3.2 compatible. No network.
set -uo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
PACK="$(cd "$HERE/../../../.." && pwd)"
UAR="${UAR_ROOT:-$(dirname "$PACK")/universal-agent-runtime}"

if [ ! -d "$UAR" ]; then
  echo "[uar-discovery] UNVERIFIABLE: universal-agent-runtime not found at $UAR" >&2
  echo "[uar-discovery]   Set UAR_ROOT to check. This is NOT a pass." >&2
  exit 3
fi

FAIL=0
ok()  { echo "  PASS  $1"; }
bad() { echo "  FAIL  $1" >&2; FAIL=1; }

echo "UAR component-discovery premise"
echo "-------------------------------"

# 1. The submodule declaration.
SUB_URL="$(awk '/\[submodule "crates\/prometheus-skill-system"\]/{f=1;next} f&&/url/{print $3;exit}' \
           "$UAR/.gitmodules" 2>/dev/null)"
if [ -n "$SUB_URL" ]; then
  ok "UAR declares crates/prometheus-skill-system → $SUB_URL"
else
  bad "UAR does not declare a crates/prometheus-skill-system submodule"
fi

# 2. The discovery path. Read from source, not from a comment: a stale doc
#    comment would let this pass while the code looked elsewhere.
RT="$UAR/src/uar/runtime/skills/wasm_runtime.rs"
if [ -f "$RT" ]; then
  if grep -q 'PathBuf::from("crates/prometheus-skill-system/skills")' "$RT"; then
    ok "wasm discovery falls back to crates/prometheus-skill-system/skills"
  else
    bad "wasm discovery no longer reads crates/prometheus-skill-system/skills"
    echo "        → the in-repo premise has COLLAPSED; re-plan 005/006" >&2
  fi
else
  bad "wasm_runtime.rs not found at $RT"
fi

# 3. This repo is that submodule.
ORIGIN="$(git -C "$PACK" remote get-url origin 2>/dev/null)"
if [ -n "$SUB_URL" ] && [ "$ORIGIN" = "$SUB_URL" ]; then
  ok "this repo's origin matches the submodule URL"
elif [ -n "$ORIGIN" ]; then
  bad "origin mismatch: repo=$ORIGIN submodule=$SUB_URL"
else
  bad "cannot read this repo's origin"
fi

echo ""
if [ "$FAIL" -eq 0 ]; then
  echo "PREMISE HOLDS: producing components is in-repo work; UAR picks them up"
  echo "by bumping its submodule pointer."
  exit 0
fi
echo "PREMISE BROKEN — do not continue change-msp-005/006 as planned." >&2
exit 2
