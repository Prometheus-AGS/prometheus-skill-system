#!/usr/bin/env bash
# Run a skill's certification cases (<skill>/evals/) with `claude plugin eval`.
#
# Two modes measure different things:
#   isolated  the skill directory is the only thing loaded. Measures whether the
#             skill changes the outcome (level C3) and whether its description
#             triggers when nothing competes with it.
#   pack      the whole generated Claude plugin is loaded. Measures triggering
#             under real competition (level C2 as users experience it).
#
# These are real, billed model runs. Run them at a phase boundary, not as a
# development loop.
#
# Usage:
#   run-evals.sh --mode isolated|pack --out <dir> [--runs 5] [--model <id>]
#                [--max-cost-usd 8] [--jobs 3] [--plugin <plugin-dir>] <skill-dir>...
#
# --plugin names the Claude plugin directory that pack mode loads (a directory
# with skills/ at its top level). In the skill pack's repository it defaults to
# the generated plugin, dist/plugins/claude/prometheus-skill-pack.
set -euo pipefail

MODE="" OUT="" RUNS=5 MODEL="" MAX_COST=8 JOBS=3 PLUGIN=""
SKILLS=()
while [ $# -gt 0 ]; do
  case "$1" in
    --mode) MODE="$2"; shift 2 ;;
    --out) OUT="$2"; shift 2 ;;
    --runs) RUNS="$2"; shift 2 ;;
    --model) MODEL="$2"; shift 2 ;;
    --max-cost-usd) MAX_COST="$2"; shift 2 ;;
    --jobs) JOBS="$2"; shift 2 ;;
    --plugin) PLUGIN="$2"; shift 2 ;;
    -h|--help) sed -n '2,22p' "$0"; exit 0 ;;
    *) SKILLS+=("$1"); shift ;;
  esac
done

[ "$MODE" = "isolated" ] || [ "$MODE" = "pack" ] || { echo "error: --mode must be isolated or pack" >&2; exit 64; }
[ -n "$OUT" ] || { echo "error: --out <dir> is required" >&2; exit 64; }
[ "${#SKILLS[@]}" -gt 0 ] || { echo "error: give at least one skill directory" >&2; exit 64; }
command -v claude >/dev/null 2>&1 || { echo "error: claude CLI not found" >&2; exit 69; }

mkdir -p "$OUT"
OUT="$(cd "$OUT" && pwd)"

# --no-publish keeps the HTML report on this machine. --scaffold runs each
# case's setup script, which is safe here because the cases are ours.
COMMON=(--runs "$RUNS" -j "$JOBS" --scaffold --max-cost-usd "$MAX_COST" --no-publish --trust-plugin)
[ -n "$MODEL" ] && COMMON+=(--model "$MODEL")

STATUS=0
if [ "$MODE" = "isolated" ]; then
  for skill in "${SKILLS[@]}"; do
    [ -d "$skill/evals" ] || { echo "skip $skill (no evals/)" >&2; continue; }
    name="$(basename "$skill")"
    mkdir -p "$OUT/$name"
    echo "== isolated: $name"
    (cd "$skill" && claude plugin eval . "${COMMON[@]}" \
      --output-dir "$OUT/$name" --json "$OUT/$name/result.json" >"$OUT/$name/stdout.txt" 2>&1) || STATUS=1
  done
else
  # `claude plugin eval` refuses an eval directory inside a loaded skills/ tree,
  # and the repository root is a marketplace rather than a plugin. So the pack
  # is evaluated from a scratch copy of the generated plugin with the cases
  # copied beside it.
  PLUGIN_SRC="$PLUGIN"
  if [ -z "$PLUGIN_SRC" ]; then
    TOP="$(git rev-parse --show-toplevel 2>/dev/null || true)"
    [ -n "$TOP" ] && PLUGIN_SRC="$TOP/dist/plugins/claude/prometheus-skill-pack"
  fi
  [ -n "$PLUGIN_SRC" ] && [ -d "$PLUGIN_SRC/skills" ] || {
    echo "error: no plugin to load; pass --plugin <plugin-dir> (or run npm run build in the skill pack)" >&2; exit 66; }
  WORK="$(mktemp -d "${TMPDIR:-/tmp}/skill-cert-pack.XXXXXX")"
  cp -R "$PLUGIN_SRC" "$WORK/pack"
  mkdir -p "$WORK/pack/certevals"
  for skill in "${SKILLS[@]}"; do
    [ -d "$skill/evals" ] || { echo "skip $skill (no evals/)" >&2; continue; }
    cp -R "$skill/evals" "$WORK/pack/certevals/$(basename "$skill")"
  done
  echo "== pack: ${#SKILLS[@]} skill(s), scratch copy at $WORK"
  (cd "$WORK/pack" && claude plugin eval . --eval-dir certevals "${COMMON[@]}" \
    --output-dir "$OUT" --json "$OUT/result.json" >"$OUT/stdout.txt" 2>&1) || STATUS=1
fi

# A non-zero status means at least one case scored below the threshold, which is
# a finding and not a tool failure. Read the summary, not the exit code.
echo "results in $OUT (summarise with summarize-evals.mjs, or pass them to certify.mjs --results)"
exit "$STATUS"
