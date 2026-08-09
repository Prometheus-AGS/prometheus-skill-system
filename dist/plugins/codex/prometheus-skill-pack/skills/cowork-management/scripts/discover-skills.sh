#!/usr/bin/env bash
# discover-skills.sh — search for EXTERNAL skills that might already solve a
# capability gap, and emit them as a build-vs-adopt input.
#
# Complementary to skills/process/kbd-goal/references/skill-discovery.md, which
# maps keywords onto skills ALREADY IN THIS PACK — a closed local table. This
# script asks the opposite question: does something outside the pack already do
# this, so we do not build it again?
#
# Usage:
#   discover-skills.sh --capability "<what you need>" [--limit N] [--out <json>]
#   discover-skills.sh --capability "postgres backup verification" --limit 5
#
# Exit: 0 ok (including "nothing found") · 1 usage · 2 cowork unavailable
#
# SECURITY POSTURE — read before extending this script
# It searches and reports. It NEVER installs. A discovery step that installed
# what it found would execute unreviewed third-party code as a side effect of
# asking a question. Adoption is a separate, explicit operator decision routed
# through `cowork audit` / `cowork verify` — see the "Adopting" section that this
# script prints, and docs/guide/16a-cowork.md.
#
# bash 3.2 compatible. Makes no LLM calls.
set -uo pipefail

CAPABILITY="" LIMIT=5 OUT=""
while [ $# -gt 0 ]; do
  case "$1" in
    --capability) CAPABILITY="${2:-}"; shift 2 ;;
    --limit)      LIMIT="${2:-5}";     shift 2 ;;
    --out)        OUT="${2:-}";        shift 2 ;;
    *) echo "usage: $0 --capability \"<need>\" [--limit N] [--out <json>]" >&2; exit 1 ;;
  esac
done
[ -n "$CAPABILITY" ] || { echo "[discover] ERROR: --capability is required" >&2; exit 1; }
case "$LIMIT" in ''|*[!0-9]*) echo "[discover] ERROR: --limit must be a number" >&2; exit 1 ;; esac

if ! command -v cowork >/dev/null 2>&1; then
  echo "[discover] cowork not on PATH — skipping discovery, proceeding with build." >&2
  echo "[discover]   Install it with: bash scripts/install-binaries.sh" >&2
  exit 2
fi

echo "[discover] searching for external skills matching: $CAPABILITY" >&2

# `cowork search` emits human-readable text, not JSON (verified against the CLI:
# there is no --format flag). Parse the numbered result lines rather than
# inventing a machine format the tool does not offer.
RAW="$(cowork search "$CAPABILITY" --limit "$LIMIT" 2>&1)" || {
  echo "[discover] cowork search failed — proceeding with build." >&2
  printf '%s\n' "$RAW" | tail -3 >&2
  exit 0
}

# The parser reads the search output from a FILE, not stdin: `python3 - <<'PY'`
# already consumes stdin for the script body, so a second redirect silently wins
# and the program reads its own source as input.
RAW_FILE="$(mktemp "${TMPDIR:-/tmp}/cowork-search.XXXXXX")"
printf '%s\n' "$RAW" > "$RAW_FILE"
trap 'rm -f "$RAW_FILE"' EXIT

CANDIDATES="$(CAPABILITY="$CAPABILITY" python3 - "$RAW_FILE" <<'PY' 2>/dev/null || true
import json, os, re, sys

text = open(sys.argv[1], encoding="utf-8", errors="replace").read()
out = []
# Result lines look like:  "1. owner/repo ⭐ 7814 [source]" followed by an
# indented description line.
lines = text.splitlines()
for i, line in enumerate(lines):
    m = re.match(r"\s*(\d+)\.\s+(\S+/\S+)\s*(?:⭐\s*([\d,]+))?", line)
    if not m:
        continue
    desc = ""
    if i + 1 < len(lines) and lines[i + 1].startswith("   "):
        desc = lines[i + 1].strip()
    stars = m.group(3)
    out.append({
        "id": "cw-%03d" % int(m.group(1)),
        "name": m.group(2),
        "kind": "external-skill",
        "repo_url": "https://github.com/%s" % m.group(2),
        "registry": "github",
        "stars": int(stars.replace(",", "")) if stars else None,
        "fit_for_gap": desc,
        # Deliberately NOT a verdict from the library-candidates enum. Search
        # relevance is not evaluation, and emitting "adopt" here would launder a
        # keyword match into a decision nobody made.
        "verdict": "unevaluated",
        "evidence": "cowork search result; not inspected, not audited",
        "risks": "third-party code; unreviewed; license and maintenance unverified",
    })
print(json.dumps({
    "schema": "skill-candidates/v1",
    "capability": os.environ["CAPABILITY"],
    "source": "cowork search",
    # Why this is NOT library-candidates.json, despite the deliberate field
    # overlap: that schema sets additionalProperties:false and constrains
    # kind/registry/verdict to enums describing PACKAGES ("library", "npm",
    # "adopt"). A GitHub skill repo is none of those, and none of these entries
    # has been evaluated, so every one would have to claim a verdict it has not
    # earned. Forcing the fit would make an unreviewed search hit look like a
    # vetted adoption decision — the opposite of what this file is for.
    "not_library_candidates": (
        "Shares field names with library-candidates.json for readability, but is "
        "a separate document: these entries are unevaluated and carry no verdict."
    ),
    "candidates": out,
}, indent=2))
PY
)"

if [ -z "$CANDIDATES" ]; then
  echo "[discover] could not parse search output — proceeding with build." >&2
  exit 0
fi

COUNT="$(printf '%s' "$CANDIDATES" | python3 -c 'import json,sys; print(len(json.load(sys.stdin)["candidates"]))' 2>/dev/null || echo 0)"

if [ "$COUNT" = "0" ]; then
  echo "[discover] no external candidates found — build is the remaining option." >&2
else
  echo "[discover] found $COUNT candidate(s). None are evaluated, and none were installed." >&2
  echo "[discover]" >&2
  echo "[discover] To adopt one, follow references/adopting-external-skills.md:" >&2
  echo "[discover]   1. read the repository source and licence first" >&2
  echo "[discover]   2. cowork install <owner/repo> --agent claude-code   (PROJECT scope)" >&2
  echo "[discover]   3. cowork audit --project --format json" >&2
  echo "[discover]   4. cowork verify" >&2
  echo "[discover]" >&2
  echo "[discover] Note: 'cowork audit' scans INSTALLED skills and takes no repo" >&2
  echo "[discover] argument, so it cannot vet a candidate before installation." >&2
  echo "[discover] Project scope keeps an unvetted skill removable." >&2
fi

if [ -n "$OUT" ]; then
  mkdir -p "$(dirname "$OUT")"
  printf '%s\n' "$CANDIDATES" > "$OUT"
  echo "[discover] wrote $OUT" >&2
else
  printf '%s\n' "$CANDIDATES"
fi
exit 0
