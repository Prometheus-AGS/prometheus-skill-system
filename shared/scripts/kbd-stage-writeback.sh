#!/usr/bin/env bash
# kbd-stage-writeback.sh — write a KBD stage's handoff summary for the lead.
#
# Builtin orchestrator hook for assess:after, analyze:after and plan:after
# (design §6, "KBD stage write-back"). The summary is the stage handoff's
# summaryForNext (<phase>/handoffs/<stage>.handoff.json); when the handoff is
# not written yet, the opening of the stage artifact stands in. It is stored
# through learning_write.py at visibility `lead` (agent_id <team>/@lead), kind
# progress, so the lead view (main-thread KBD stages, the team lead) recalls it.
#
# Usage: kbd-stage-writeback.sh [<stage>] [<phase>]
#   defaults: $KBD_HOOK_KIND and $KBD_HOOK_NAME (set by the hooks dispatcher).
# Always exits 0 and prints nothing to stdout. bash 3.2 compatible.
set -u

HERE="$(cd "$(dirname "$0")" && pwd)"
LW="$HERE/lib/learning_write.py"
stage="${1:-${KBD_HOOK_KIND:-}}"
phase="${2:-${KBD_HOOK_NAME:-}}"
case "$stage" in assess|analyze|plan) ;; *) exit 0 ;; esac
command -v python3 >/dev/null 2>&1 || exit 0
[ -f "$LW" ] || exit 0

root=""
dir="$PWD"
while [ -n "$dir" ] && [ "$dir" != "/" ]; do
  if [ -d "$dir/.kbd-orchestrator" ]; then root="$dir"; break; fi
  dir="$(dirname "$dir")"
done
[ -n "$root" ] || exit 0
if [ -z "$phase" ] && [ -f "$root/.kbd-orchestrator/current-waypoint.json" ]; then
  phase="$(python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); print(d.get("phase") or d.get("activePhaseId") or "")' \
    "$root/.kbd-orchestrator/current-waypoint.json" 2>/dev/null || true)"
fi
[ -n "$phase" ] || exit 0
phase_dir="$root/.kbd-orchestrator/phases/$phase"
[ -d "$phase_dir" ] || exit 0

python3 - "$LW" "$root" "$phase_dir" "$phase" "$stage" <<'PY' >/dev/null 2>&1 || true
import json, re, sys
from pathlib import Path

lw, root, phase_dir, phase, stage = Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3]), sys.argv[4], sys.argv[5]
sys.path.insert(0, str(lw.parent))
import learning_write  # noqa: E402

summary = ""
handoff = phase_dir / "handoffs" / f"{stage}.handoff.json"
try:
    data = json.loads(handoff.read_text(encoding="utf-8"))
    if not data.get("skipped"):
        summary = str(data.get("summaryForNext") or "").strip()
except (OSError, ValueError, AttributeError):
    pass
if not summary:
    artifact = {"assess": "assessment.md", "analyze": "analysis.md", "plan": "plan.md"}[stage]
    try:
        text = (phase_dir / artifact).read_text(encoding="utf-8", errors="replace")
    except OSError:
        text = ""
    body = re.sub(r"(?m)^#.*$", "", text)
    paragraphs = [p.strip() for p in re.split(r"\n\s*\n", body) if p.strip() and not p.strip().startswith("|")]
    summary = " ".join(paragraphs[:2])
summary = " ".join(summary.split())[:800]
if summary:
    learning_write.write_lesson(f"KBD {stage} summary for phase {phase}: {summary}", {}, cwd=root,
                                kind="progress", visibility="lead", stage=stage, importance=0.5)
PY
exit 0
