#!/usr/bin/env bash
# build-review-packet.sh — deterministic assembly of the review packet the
# judge receives. The packet is the ONLY context the judge ever sees:
# it contains the work product and its success criteria — never the
# producing session's chat history. Isolation is structural.
#
# Usage:
#   build-review-packet.sh --mode diff     --phase <phase> --target <change-id>       [--out <path>]
#   build-review-packet.sh --mode artifact --phase <phase> --target assess|analyze|plan [--out <path>]
#   build-review-packet.sh --mode skill    --target <skill-dir>     [--intent <file>] [--out <path>]
#   build-review-packet.sh --mode agent    --target <workspace-dir> [--intent <file>] [--out <path>]
#   build-review-packet.sh --mode decision --target <decision.md>    [--intent <file>] [--out <path>]
#
# Emits packet JSON to --out (default: stdout).
# Exit codes: 0 ok · 1 usage · 2 missing inputs
#
# MODES
#   diff|artifact — review work inside a KBD phase; --phase is required and
#                   --target names a change id or a stage.
#   skill|agent   — review a GENERATED artifact (change-arc-003). --target is a
#                   filesystem path, and --phase is optional: a creator can run
#                   outside any KBD phase, so requiring one would make the gate
#                   unreachable exactly where generation happens.
#   decision      — review an IDEA before committing to it (change-idt-001).
#                   --target is a FILE. The judge is not asked to score novelty:
#                   pre-execution novelty ratings FLIP after execution
#                   (Si/Hashimoto/Yang 2025), so the packet carries what is
#                   claimed, what it rests on, what would falsify it, and what
#                   was already decided on this topic.
#
# Both creation modes are MANIFEST-LEVEL. They record what each file is and does,
# never its full body. A generated Cargo workspace does not fit in a judge's
# context, and a packet that silently drops half its content would let the judge
# return PASS on material it never saw. See --mode agent below and the packet cap.
#
# bash 3.2 compatible (no mapfile, no declare -A). No LLM calls (class=small).
set -uo pipefail

MODE="" PHASE="" TARGET="" OUT="" INTENT=""
while [ $# -gt 0 ]; do
  case "$1" in
    --mode)   MODE="${2:-}"; shift 2 ;;
    --phase)  PHASE="${2:-}"; shift 2 ;;
    --target) TARGET="${2:-}"; shift 2 ;;
    --intent) INTENT="${2:-}"; shift 2 ;;
    --out)    OUT="${2:-}"; shift 2 ;;
    *) echo "usage: $0 --mode diff|artifact|skill|agent|decision [--phase <phase>] --target <id|stage|path> [--intent <file>] [--out <path>]" >&2; exit 1 ;;
  esac
done
case "$MODE" in
  diff|artifact|skill|agent|decision) ;;
  *) echo "[packet] ERROR: --mode must be diff, artifact, skill, agent, or decision" >&2; exit 1 ;;
esac
[ -n "$TARGET" ] || { echo "[packet] ERROR: --target is required" >&2; exit 1; }
case "$MODE" in
  diff|artifact)
    [ -n "$PHASE" ] || { echo "[packet] ERROR: --phase is required for --mode $MODE" >&2; exit 1; } ;;
esac

find_kbd_root() {
  local d="$PWD"
  while [ "$d" != "/" ]; do
    [ -d "$d/.kbd-orchestrator" ] && { printf '%s' "$d/.kbd-orchestrator"; return 0; }
    d="$(dirname "$d")"
  done
  return 1
}
KBD_ROOT="$(find_kbd_root 2>/dev/null || true)"
case "$MODE" in
  diff|artifact)
    # Phase-scoped modes cannot proceed without the phase they review.
    [ -n "$KBD_ROOT" ] || { echo "[packet] ERROR: .kbd-orchestrator not found above $PWD" >&2; exit 2; }
    PHASE_DIR="$KBD_ROOT/phases/$PHASE"
    [ -d "$PHASE_DIR" ] || { echo "[packet] ERROR: phase dir not found: $PHASE_DIR" >&2; exit 2; }
    ;;
  skill|agent)
    # Creation modes review a path, not a phase. A KBD root is used when present
    # (for constraints.md and the producer record) and simply absent otherwise —
    # generators frequently run outside any phase.
    PHASE_DIR=""
    if [ -n "$KBD_ROOT" ] && [ -n "$PHASE" ] && [ -d "$KBD_ROOT/phases/$PHASE" ]; then
      PHASE_DIR="$KBD_ROOT/phases/$PHASE"
    fi
    [ -d "$TARGET" ] || { echo "[packet] ERROR: --target must be an existing directory for --mode $MODE: $TARGET" >&2; exit 2; }
    ;;
  decision)
    # Decision mode reviews a single decision document, not a directory or a
    # phase — an idea is authored as one file. A KBD root is used when present
    # (constraints, producer record) but never required: ideation legitimately
    # happens outside any phase, which is most of the point.
    PHASE_DIR=""
    if [ -n "$KBD_ROOT" ] && [ -n "$PHASE" ] && [ -d "$KBD_ROOT/phases/$PHASE" ]; then
      PHASE_DIR="$KBD_ROOT/phases/$PHASE"
    fi
    [ -f "$TARGET" ] || { echo "[packet] ERROR: --target must be an existing FILE for --mode decision: $TARGET" >&2; exit 2; }
    ;;
esac

echo "[MODEL_ROUTING] phase=adv-review-packet class=small" >&2

command -v python3 >/dev/null 2>&1 || { echo "[packet] ERROR: python3 required" >&2; exit 2; }

WORK="$(mktemp -d "${TMPDIR:-/tmp}/adv-review-packet.XXXXXX")"
# shellcheck disable=SC2064
trap "rm -rf '$WORK'" EXIT

# --- producer model -----------------------------------------------------------
# Best-effort record of which model produced the work under review, for the
# judge!=producer collision check. Sources, in order: progress.json,
# KBD_PRODUCER_MODEL, ANTHROPIC_MODEL, "unknown".
PRODUCER=""
if [ -n "$PHASE_DIR" ]; then
  PRODUCER="$(python3 - "$PHASE_DIR/progress.json" <<'PY' 2>/dev/null || true
import json, sys
try:
    d = json.load(open(sys.argv[1]))
    print(d.get("producer_model") or "")
except Exception:
    pass
PY
)"
fi
[ -n "$PRODUCER" ] || PRODUCER="${KBD_PRODUCER_MODEL:-${ANTHROPIC_MODEL:-}}"
# Harness-provided identifiers, before giving up. CLAUDE_MODEL / CLAUDECODE_MODEL
# are set by some Claude Code builds; CLAUDE_CODE_MODEL by others.
[ -n "$PRODUCER" ] || PRODUCER="${CLAUDE_MODEL:-${CLAUDE_CODE_MODEL:-${CLAUDECODE_MODEL:-}}}"

# "unknown" is not a harmless default: the judge's collision check compares
# candidate != producer, so an unknown producer makes it pass TRIVIALLY. Every one
# of the 8 historical reviews carried producer_model="unknown", which is why
# judge!=producer was never actually enforced despite the check being present.
# Warn loudly so a trivially-passing check is visible at packet-build time.
#
# WHY THIS STILL RECORDS "unknown" RATHER THAN REFUSING
# Recording the truth is the correct behaviour for a general caller: a KBD stage
# reviewing an artifact it did not generate may legitimately not know the producer,
# and the honest record of that is cross_model_check: unverified-producer-unknown.
#
# The CREATORS are the stricter case — they always know their own producer, so an
# absent value there means misconfiguration, not genuine uncertainty. They call
# kbd_require_producer_model (shared/scripts/lib/kbd-model-resolve.sh) and abort
# with exit 2 BEFORE reaching this script, so no packet and no findings file are
# written. See change-arc-002; the boundary is deliberate, not an inconsistency.
if [ -z "$PRODUCER" ]; then
  PRODUCER="unknown"
  echo "[packet] WARN: PRODUCER_UNKNOWN — cannot determine which model produced this" >&2
  echo "[packet]       work, so the judge!=producer guarantee cannot be enforced." >&2
  echo "[packet]       Set KBD_PRODUCER_MODEL (e.g. export KBD_PRODUCER_MODEL=claude-opus-5)" >&2
  echo "[packet]       to restore the cross-model check." >&2
fi

# --- shared context -----------------------------------------------------------
if [ -n "$KBD_ROOT" ]; then
  CONSTRAINTS_FILE="$KBD_ROOT/constraints.md"
  [ -f "$CONSTRAINTS_FILE" ] && cp "$CONSTRAINTS_FILE" "$WORK/constraints.md"
  REPO_ROOT="$(dirname "$KBD_ROOT")"
else
  REPO_ROOT="$PWD"
fi

# File tree: for creation modes the tree that matters is the generated artifact's
# own, not the host repo's — the judge is reviewing what was produced.
case "$MODE" in
  skill|agent) TREE_ROOT="$TARGET" ;;
  # decision's target is a FILE, so the tree that gives context is its directory.
  decision)    TREE_ROOT="$(cd "$(dirname "$TARGET")" && pwd)" ;;
  *)           TREE_ROOT="$REPO_ROOT" ;;
esac
# Top 2 levels, pruning bulk dirs. Deterministic (sorted).
( cd "$TREE_ROOT" && find . -maxdepth 2 \
    -not -path '*/node_modules*' -not -path '*/.git*' -not -path '*/target*' \
    -not -path '*/dist*' -not -path '*/.refiner*' 2>/dev/null | sort ) > "$WORK/file_tree.txt" || true

# --- mode-specific content ----------------------------------------------------
if [ "$MODE" = "diff" ]; then
  CHANGE_DIR=""
  for cand in "$KBD_ROOT/changes/$TARGET" "$REPO_ROOT/openspec/changes/$TARGET"; do
    [ -d "$cand" ] && { CHANGE_DIR="$cand"; break; }
  done

  # Acceptance criteria: tasks.md / spec.md / verification.md from the change dir.
  if [ -n "$CHANGE_DIR" ]; then
    for f in tasks.md spec.md proposal.md verification.md; do
      [ -f "$CHANGE_DIR/$f" ] && cat "$CHANGE_DIR/$f" >> "$WORK/acceptance_criteria.md"
    done
  fi
  [ -s "$WORK/acceptance_criteria.md" ] || \
    echo "[packet] WARN: no acceptance criteria found for change $TARGET" >&2

  # Diff: scope to the change's file list when recorded, else uncommitted work.
  FILES_LIST="$WORK/changed_files.txt"
  if [ -n "$CHANGE_DIR" ] && [ -f "$CHANGE_DIR/files.txt" ]; then
    cp "$CHANGE_DIR/files.txt" "$FILES_LIST"
  fi
  if [ -s "$FILES_LIST" ]; then
    ( cd "$REPO_ROOT" && git diff HEAD -- $(cat "$FILES_LIST") 2>/dev/null ) > "$WORK/diff.patch" || true
  else
    ( cd "$REPO_ROOT" && git diff HEAD 2>/dev/null ) > "$WORK/diff.patch" || true
  fi
  [ -s "$WORK/diff.patch" ] || \
    ( cd "$REPO_ROOT" && git show --patch HEAD 2>/dev/null ) > "$WORK/diff.patch" || true
  [ -s "$WORK/diff.patch" ] || { echo "[packet] ERROR: no diff content resolvable for $TARGET" >&2; exit 2; }
elif [ "$MODE" = "skill" ]; then
  # ---- skill mode: manifest-level review of a generated SKILL.md tree --------
  SKILL_MD="$TARGET/SKILL.md"
  [ -f "$SKILL_MD" ] || { echo "[packet] ERROR: no SKILL.md in $TARGET" >&2; exit 2; }
  cp "$SKILL_MD" "$WORK/skill_md.md"

  # Frontmatter, parsed rather than pasted: the judge should see the declared
  # contract (name/description/version/license/tags) as data it can check against
  # the body, not as an opaque YAML blob it has to re-derive.
  python3 - "$SKILL_MD" > "$WORK/frontmatter.json" 2>/dev/null <<'PY' || true
import json, sys, re
text = open(sys.argv[1], encoding="utf-8", errors="replace").read()
fm = {}
m = re.match(r"^---\n(.*?)\n---\n", text, re.S)
if m:
    key = None
    for line in m.group(1).splitlines():
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        km = re.match(r"^(\s*)([A-Za-z0-9_.-]+):\s*(.*)$", line)
        if km:
            indent, key, val = len(km.group(1)), km.group(2), km.group(3).strip()
            fm[key] = val if val else {}
        elif line.lstrip().startswith("-") and key:
            fm.setdefault(key, [])
            if isinstance(fm[key], list):
                fm[key].append(line.lstrip()[1:].strip())
print(json.dumps(fm, indent=2, sort_keys=True))
PY

  # Script inventory: path, size, executable bit, interpreter, and the leading
  # comment line as a stated purpose. NEVER the script body — see header.
  if [ -d "$TARGET/scripts" ]; then
    for s in "$TARGET"/scripts/*; do
      [ -f "$s" ] || continue
      _exec="no"; [ -x "$s" ] && _exec="yes"
      _shebang="$(head -1 "$s" 2>/dev/null | grep '^#!' || echo '(none)')"
      _purpose="$(sed -n '2,6p' "$s" 2>/dev/null | grep '^#' | head -1 | sed 's/^# *//')"
      printf '%s\t%s bytes\texecutable=%s\t%s\t%s\n' \
        "scripts/$(basename "$s")" "$(wc -c <"$s" | tr -d ' ')" "$_exec" \
        "$_shebang" "${_purpose:-(no stated purpose)}" >> "$WORK/script_inventory.txt"
    done
  fi
  [ -f "$WORK/script_inventory.txt" ] || echo "(no scripts/ directory)" > "$WORK/script_inventory.txt"

  # Cross-reference map: every markdown link in SKILL.md, marked resolved or
  # BROKEN. A skill that advertises references it does not ship is a defect the
  # judge should see stated, not have to infer from the file tree.
  python3 - "$TARGET" > "$WORK/crossref.txt" 2>/dev/null <<'PY' || true
import os, re, sys
root = sys.argv[1]
skill_md = os.path.join(root, "SKILL.md")
text = open(skill_md, encoding="utf-8", errors="replace").read()
seen, out = set(), []
for label, href in re.findall(r"\[([^\]]*)\]\(([^)]+)\)", text):
    if href.startswith(("http://", "https://", "#", "mailto:")):
        continue
    target = href.split("#", 1)[0]
    if not target or target in seen:
        continue
    seen.add(target)
    ok = os.path.exists(os.path.join(root, target))
    out.append("%-8s %s  (%s)" % ("OK" if ok else "BROKEN", target, label))
print("\n".join(sorted(out)) if out else "(no relative links in SKILL.md)")
PY

  # Validator output: the objective, already-computed verdict. Including it stops
  # the judge from re-deriving mechanical checks and lets it spend its attention
  # on what a validator cannot see.
  VALIDATOR=""
  for cand in \
    "$REPO_ROOT/skills/process/pmpo-skill-creator/scripts/validate-skill.sh" \
    "${CLAUDE_PLUGIN_ROOT:-}/skills/process/pmpo-skill-creator/scripts/validate-skill.sh"; do
    [ -f "$cand" ] && { VALIDATOR="$cand"; break; }
  done
  if [ -n "$VALIDATOR" ]; then
    bash "$VALIDATOR" "$TARGET" > "$WORK/validate_output.txt" 2>&1 || true
  else
    echo "(validate-skill.sh not found — validator output unavailable)" > "$WORK/validate_output.txt"
  fi

  # Original intent: what the skill was ASKED to be. Without it the judge can
  # only assess internal consistency, never whether the artifact answers the ask.
  if [ -n "$INTENT" ] && [ -f "$INTENT" ]; then
    cp "$INTENT" "$WORK/intent.md"
  elif [ -f "$TARGET/.intent.md" ]; then
    cp "$TARGET/.intent.md" "$WORK/intent.md"
  else
    echo "[packet] WARN: no --intent supplied; the judge cannot check the artifact" >&2
    echo "[packet]       against what was requested, only against itself." >&2
  fi

elif [ "$MODE" = "agent" ]; then
  # ---- agent mode: manifest-level review of a generated Cargo workspace ------
  # A generated workspace is 6+ crates of Rust. It does not fit in a judge's
  # context and would drown the signal if it did, so this records the CONFIGURED
  # SURFACE — what the agent is wired to be — never crate source.
  AGENT_TOML="$TARGET/agent.toml"
  [ -f "$AGENT_TOML" ] || { echo "[packet] ERROR: no agent.toml in $TARGET" >&2; exit 2; }
  cp "$AGENT_TOML" "$WORK/agent_toml.txt"

  # system_prompt.md IS the agent's behaviour. Reviewing an agent without it
  # would judge the plumbing and ignore what the thing actually does.
  if [ -f "$TARGET/system_prompt.md" ]; then
    cp "$TARGET/system_prompt.md" "$WORK/system_prompt.md"
  else
    echo "(no system_prompt.md — the agent has no declared behaviour)" > "$WORK/system_prompt.md"
    echo "[packet] WARN: no system_prompt.md in $TARGET" >&2
  fi

  # Workspace members with a stated purpose per crate. Purpose comes from the
  # crate's own description/doc-comment; "(no stated purpose)" is itself a
  # finding the judge should be able to see.
  python3 - "$TARGET" > "$WORK/workspace_members.txt" 2>/dev/null <<'PY' || true
import os, re, sys
root = sys.argv[1]
ws = os.path.join(root, "Cargo.toml")
members, out = [], []
if os.path.exists(ws):
    text = open(ws, encoding="utf-8", errors="replace").read()
    m = re.search(r"members\s*=\s*\[(.*?)\]", text, re.S)
    if m:
        members = re.findall(r'"([^"]+)"', m.group(1))
for pat in members or []:
    # Expand a simple "crates/*" glob; anything else is treated literally.
    if pat.endswith("/*"):
        base = os.path.join(root, pat[:-2])
        entries = sorted(os.listdir(base)) if os.path.isdir(base) else []
        paths = [os.path.join(pat[:-2], e) for e in entries]
    else:
        paths = [pat]
    for p in paths:
        ct = os.path.join(root, p, "Cargo.toml")
        purpose = ""
        if os.path.exists(ct):
            t = open(ct, encoding="utf-8", errors="replace").read()
            d = re.search(r'^\s*description\s*=\s*"([^"]*)"', t, re.M)
            if d:
                purpose = d.group(1)
        if not purpose:
            lib = os.path.join(root, p, "src", "lib.rs")
            main = os.path.join(root, p, "src", "main.rs")
            for f in (lib, main):
                if os.path.exists(f):
                    for line in open(f, encoding="utf-8", errors="replace"):
                        if line.startswith("//!"):
                            purpose = line[3:].strip()
                            break
                    if purpose:
                        break
        out.append("%-28s %s" % (p, purpose or "(no stated purpose)"))
print("\n".join(out) if out else "(no workspace members declared)")
PY

  # MCP servers: the agent's external tool surface, and the part most likely to
  # be misconfigured (wrong transport, disabled, unreachable port).
  python3 - "$AGENT_TOML" > "$WORK/mcp_servers.txt" 2>/dev/null <<'PY' || true
import re, sys
text = open(sys.argv[1], encoding="utf-8", errors="replace").read()
blocks = re.findall(r"\[\[mcp_servers\]\](.*?)(?=\n\[|\Z)", text, re.S)
out = []
for b in blocks:
    f = {}
    for k in ("name", "url", "transport", "enabled"):
        m = re.search(r'^\s*%s\s*=\s*"?([^"\n]+)"?' % k, b, re.M)
        if m:
            f[k] = m.group(1).strip()
    if f:
        out.append("%-22s %-34s transport=%-6s enabled=%s" % (
            f.get("name", "(unnamed)"), f.get("url", "(no url)"),
            f.get("transport", "?"), f.get("enabled", "?")))
print("\n".join(out) if out else "(no MCP servers configured)")
PY

  # cargo check: the objective build verdict. Reuse a recorded result when the
  # creator already ran it; only shell out when asked, since a cold workspace
  # build is far too slow to sit inside packet assembly.
  if [ -f "$TARGET/.cargo-check.txt" ]; then
    cp "$TARGET/.cargo-check.txt" "$WORK/cargo_check.txt"
  elif [ "${PACKET_RUN_CARGO_CHECK:-0}" = "1" ] && command -v cargo >/dev/null 2>&1; then
    ( cd "$TARGET" && cargo check --workspace --message-format short 2>&1 | tail -40 ) \
      > "$WORK/cargo_check.txt" || true
  else
    echo "(cargo check not run — no .cargo-check.txt recorded by the creator," > "$WORK/cargo_check.txt"
    echo " and PACKET_RUN_CARGO_CHECK=1 was not set)" >> "$WORK/cargo_check.txt"
  fi

  # Original intent: what the agent was ASKED to be.
  if [ -n "$INTENT" ] && [ -f "$INTENT" ]; then
    cp "$INTENT" "$WORK/intent.md"
  elif [ -f "$TARGET/.intent.md" ]; then
    cp "$TARGET/.intent.md" "$WORK/intent.md"
  else
    echo "[packet] WARN: no --intent supplied; the judge cannot check the artifact" >&2
    echo "[packet]       against what was requested, only against itself." >&2
  fi

elif [ "$MODE" = "decision" ]; then
  # ---- decision mode: review an IDEA before it is committed to ---------------
  # The judge's job here is not to score novelty. Si, Hashimoto & Yang (2025)
  # showed LLM idea rankings FLIP after execution — novelty measured before
  # execution is the wrong signal. What the packet must carry is the material a
  # reviewer needs to attack the reasoning: what is being claimed, what it rests
  # on, and what would prove it wrong.
  cp "$TARGET" "$WORK/decision.md"

  # Structured fields, parsed from the document rather than demanded as separate
  # flags. A decision that states no assumptions and no falsifier is itself a
  # finding, so absence is recorded rather than treated as an error.
  python3 - "$TARGET" > "$WORK/decision_fields.json" 2>/dev/null <<'PY' || true
import json, re, sys
text = open(sys.argv[1], encoding="utf-8", errors="replace").read()

def section(*names):
    """Pull a '## <name>' section body, case-insensitive, first match wins."""
    for n in names:
        m = re.search(r"^#{1,6}\s*%s\s*$\n(.*?)(?=^#{1,6}\s|\Z)" % n,
                      text, re.M | re.I | re.S)
        if m and m.group(1).strip():
            return m.group(1).strip()
    return None

def items(body):
    if not body:
        return []
    return [re.sub(r"^[-*+]\s*", "", l).strip()
            for l in body.splitlines() if re.match(r"^\s*[-*+]\s+", l)]

assumptions = section("assumptions?", "what this rests on")
falsifier   = section("falsifier", "what would falsify (?:this|it)",
                      "what would prove (?:this|me) wrong", "disconfirming evidence")
decision    = section("decision", "the decision", "what i am deciding")

print(json.dumps({
    "decision": decision,
    "assumptions": items(assumptions) or (assumptions and [assumptions]) or [],
    "falsifier": falsifier,
    # Recorded, not enforced here: a decision with no falsifier is exactly the
    # kind of unfalsifiable claim the judge should flag, so the packet states
    # the absence plainly instead of refusing to build.
    "missing_fields": [k for k, v in
                       (("decision", decision), ("assumptions", assumptions),
                        ("falsifier", falsifier)) if not v],
}, indent=2))
PY

  # Prior decisions on the same topic — the Karpathy loop's whole point. Without
  # this the judge re-litigates settled ground and cannot see that the operator
  # already tried this and it failed.
  if command -v pk >/dev/null 2>&1; then
    # NB: `tr '-_' '  '` fails on BSD/macOS — a leading '-' is parsed as an
    # option flag ("illegal option -- _"). Use sed, which has no such ambiguity.
    _q="$(basename "$TARGET" | sed -e 's/\.[^.]*$//' -e 's/[-_]/ /g')"
    pk search "$_q" 2>/dev/null | head -40 > "$WORK/prior_decisions.txt" || true
  fi
  [ -s "$WORK/prior_decisions.txt" ] || \
    echo "(no prior decisions found; pk unavailable or nothing matched)" > "$WORK/prior_decisions.txt"

  # Original intent: what the operator was actually trying to achieve.
  if [ -n "$INTENT" ] && [ -f "$INTENT" ]; then
    cp "$INTENT" "$WORK/intent.md"
  fi

else
  # artifact mode: TARGET selects the stage artifact set.
  case "$TARGET" in
    assess)  ARTS="assessment.md" ;;
    analyze) ARTS="analysis.md library-candidates.json" ;;
    plan)    ARTS="plan.md" ;;
    *) echo "[packet] ERROR: artifact --target must be assess|analyze|plan" >&2; exit 1 ;;
  esac
  FOUND=0
  for a in $ARTS; do
    if [ -f "$PHASE_DIR/$a" ]; then
      { echo "===== $a ====="; cat "$PHASE_DIR/$a"; echo; } >> "$WORK/artifact.md"
      FOUND=1
    fi
  done
  [ "$FOUND" -eq 1 ] || { echo "[packet] ERROR: no artifacts found for stage $TARGET in $PHASE_DIR" >&2; exit 2; }

  [ -f "$PHASE_DIR/goals.md" ] && cp "$PHASE_DIR/goals.md" "$WORK/goals.md"

  # Prior-stage handoff summaries (JSON files under phases/<phase>/handoffs/).
  if [ -d "$PHASE_DIR/handoffs" ]; then
    for h in "$PHASE_DIR"/handoffs/*.json; do
      [ -f "$h" ] || continue
      { echo "===== $(basename "$h") ====="; cat "$h"; echo; } >> "$WORK/handoffs.md"
    done
  fi
fi

# --- assemble packet ----------------------------------------------------------
# Capture the assembler status separately: inside "$( ... )" a non-zero exit from
# python3 is discarded, so the manifest-level guard below would print its refusal
# and the script would still exit 0 with an empty packet. Assign, then check.
PACKET=""
ASSEMBLE_RC=0
PACKET="$(MODE="$MODE" PHASE="$PHASE" TARGET="$TARGET" PRODUCER="$PRODUCER" WORK="$WORK" \
python3 <<'PY'
import json, os, re, sys

work = os.environ["WORK"]

def slurp(name):
    p = os.path.join(work, name)
    if os.path.exists(p):
        return open(p, encoding="utf-8", errors="replace").read()
    return None

packet = {
    "packet_version": 1,
    "mode": os.environ["MODE"],
    "phase": os.environ["PHASE"],
    "target": os.environ["TARGET"],
    "producer_model": os.environ["PRODUCER"],
    "constraints": slurp("constraints.md"),
    "file_tree": slurp("file_tree.txt"),
}
mode = packet["mode"]
if mode == "diff":
    packet["diff"] = slurp("diff.patch")
    packet["acceptance_criteria"] = slurp("acceptance_criteria.md")
elif mode == "skill":
    # Manifest-level: skill_md is the contract, the rest describes what ships
    # alongside it. No script bodies — see the header note on why.
    packet["skill_md"] = slurp("skill_md.md")
    fm = slurp("frontmatter.json")
    try:
        packet["frontmatter"] = json.loads(fm) if fm else None
    except ValueError:
        packet["frontmatter"] = None
    packet["script_inventory"] = slurp("script_inventory.txt")
    packet["cross_reference_map"] = slurp("crossref.txt")
    packet["validator_output"] = slurp("validate_output.txt")
    packet["original_intent"] = slurp("intent.md")
elif mode == "agent":
    packet["agent_toml"] = slurp("agent_toml.txt")
    packet["system_prompt"] = slurp("system_prompt.md")
    packet["workspace_members"] = slurp("workspace_members.txt")
    packet["mcp_servers"] = slurp("mcp_servers.txt")
    packet["cargo_check"] = slurp("cargo_check.txt")
    packet["original_intent"] = slurp("intent.md")
elif mode == "decision":
    packet["decision_document"] = slurp("decision.md")
    fields = slurp("decision_fields.json")
    try:
        packet["decision_fields"] = json.loads(fields) if fields else None
    except ValueError:
        packet["decision_fields"] = None
    packet["prior_decisions"] = slurp("prior_decisions.txt")
    packet["original_intent"] = slurp("intent.md")
else:
    packet["artifact"] = slurp("artifact.md")
    packet["goals"] = slurp("goals.md")
    packet["prior_handoffs"] = slurp("handoffs.md")

# --- per-field cap, recorded in the packet (creation modes) -------------------
# A judge sizes its attention to what it receives. If a packet were silently
# truncated, the judge would return a verdict on material it never saw and the
# artifact would record that verdict as if it covered everything — a PASS that
# means nothing. So: cap, and make the cap part of the packet the judge reads.
#
# The cap is per FIELD, not per packet. One oversized field (a 4000-line SKILL.md)
# must not crowd out the small fields that carry the most signal per byte
# (frontmatter, the MCP server list, the validator verdict).
if mode in ("skill", "agent", "decision"):
    try:
        cap = int(os.environ.get("PACKET_FIELD_CAP_BYTES", "") or 40000)
    except ValueError:
        cap = 40000
    cap = max(cap, 1000)   # a cap below this cannot hold even a manifest

    truncated = []
    for key in sorted(packet):
        value = packet[key]
        if not isinstance(value, str) or len(value) <= cap:
            continue
        original = len(value)
        # Cut on a line boundary so the judge never sees a half-line it might
        # misread as real content from the artifact.
        clipped = value[:cap]
        nl = clipped.rfind("\n")
        if nl > cap // 2:
            clipped = clipped[:nl]
        packet[key] = clipped + (
            "\n\n[TRUNCATED by build-review-packet.sh: %d of %d bytes shown "
            "(cap %d). The omitted remainder was NOT reviewed.]"
            % (len(clipped), original, cap))
        truncated.append({
            "field": key,
            "original_bytes": original,
            "included_bytes": len(clipped),
            "omitted_bytes": original - len(clipped),
        })

    # Always present, even when nothing was cut: a reader must be able to tell
    # "nothing was dropped" from "this packet predates truncation recording".
    packet["truncation"] = {
        "cap_bytes_per_field": cap,
        "any_truncated": bool(truncated),
        "fields": truncated,
    }
    if truncated:
        sys.stderr.write(
            "[packet] WARN: %d field(s) exceeded the %d-byte cap and were truncated:\n"
            % (len(truncated), cap))
        for t in truncated:
            sys.stderr.write("[packet]   - %s: %d of %d bytes included\n"
                             % (t["field"], t["included_bytes"], t["original_bytes"]))
        sys.stderr.write("[packet]   Recorded in packet.truncation so the judge can see it.\n")

# --- manifest-level enforcement (creation modes) ------------------------------
# "Manifest-level, never full source" is a contract the judge relies on: it sizes
# its attention to a summary. If a future edit slurped a script or a .rs file into
# a descriptive field, the packet would silently become a source dump — bloating
# cost, burying the signal, and risking truncation of the very fields that matter.
#
# Enforce it structurally rather than trusting the code above to stay correct.
# Descriptive fields are line-oriented summaries, so a body leak shows up as
# unmistakable syntax (a shell function definition, a Rust fn/use/impl) inside
# them. Fields that legitimately carry prose or config verbatim are exempt by
# name, not by guesswork:
#   skill_md / system_prompt — the declared contract itself IS what gets reviewed
#   agent_toml               — configuration, and the point of the review
#   validator_output         — tool output, already summarised
#   original_intent          — the human ask
if mode in ("skill", "agent"):
    VERBATIM_OK = {
        "skill_md", "system_prompt", "agent_toml",
        "validator_output", "original_intent", "file_tree", "constraints",
        "decision_document", "prior_decisions",
    }
    # Deliberately UNANCHORED. Descriptive fields are tab-delimited records like
    #   scripts/x.sh<TAB>120 bytes<TAB>executable=yes<TAB>#!/usr/bin/env bash<TAB><purpose>
    # so leaked source lands mid-line, after the metadata columns. An `^`-anchored
    # pattern silently matches nothing on exactly the fields most at risk — which
    # is how the first version of this guard passed a fixture that really did leak.
    SOURCE_SIGNATURES = (
        ("shell function definition", re.compile(r"[A-Za-z_][A-Za-z0-9_]*\s*\(\)\s*\{")),
        ("rust fn",                   re.compile(r"\b(pub\s+)?(async\s+)?fn\s+\w+\s*\(")),
        ("rust use",                  re.compile(r"\buse\s+[\w:]+\s*;")),
        ("rust impl",                 re.compile(r"\bimpl\s+\w+\s*(<|\{|for\b)")),
    )
    leaks = []
    for key, value in packet.items():
        if key in VERBATIM_OK or not isinstance(value, str):
            continue
        for label, rx in SOURCE_SIGNATURES:
            if rx.search(value):
                leaks.append("%s contains %s" % (key, label))
                break
    if leaks:
        sys.stderr.write(
            "[packet] ERROR: manifest-level contract violated — full source leaked into:\n")
        for l in leaks:
            sys.stderr.write("[packet]   - %s\n" % l)
        sys.stderr.write(
            "[packet] Creation packets record what each file IS, never its body.\n")
        raise SystemExit(2)

print(json.dumps(packet, indent=2))
PY
)" || ASSEMBLE_RC=$?

if [ "$ASSEMBLE_RC" -ne 0 ]; then
  # The assembler already explained itself on stderr (manifest-level violation,
  # or a genuine assembly failure). Never fall through and write a partial packet:
  # a truncated or empty packet reaching the judge is the failure mode this whole
  # change exists to prevent.
  echo "[packet] ERROR: packet assembly failed (exit $ASSEMBLE_RC) — no packet written" >&2
  exit "$ASSEMBLE_RC"
fi

if [ -n "$OUT" ]; then
  mkdir -p "$(dirname "$OUT")"
  printf '%s\n' "$PACKET" > "$OUT"
  echo "[packet] wrote $OUT" >&2
else
  printf '%s\n' "$PACKET"
fi
