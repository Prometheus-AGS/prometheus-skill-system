#!/usr/bin/env bash
# run-research.sh — the deep-research stage-contract driver.
#
# Enforces references/stage-contracts.md: every stage's required artifacts are
# validated before the stage counts, hooks fire only after validation, the
# checkpoint is written at every boundary, --resume skips only stages whose
# artifacts still validate, stage 06 cannot start until stage 05 validates, and a
# provenance sidecar is written on every exit path.
#
# Usage:
#   run-research.sh --query "<text>" [--depth shallow|deep|exhaustive] [--scale auto|direct|full]
#                   [--citation-style APA] [--kb <id>]... [--job-id <id>] [--output-root DIR]
#   run-research.sh --resume <package_dir|package_id> [--scale ...]
#   run-research.sh --check-tools
#
# Execution modes (references/stage-contracts.md):
#   RESEARCH_STAGE_RUNNER=<command>   runner mode: <command> <stage> <package_dir> runs each stage
#   (unset)                           checkpoint mode: validate, stop at the first incomplete stage,
#                                     print the stage skill to run plus one JSON line
#                                     {"next_stage":..,"skill":..,"package_dir":..}, exit 3
#
# Exit codes: 0 complete, 1 blocked or invalid usage, 3 awaiting a stage (checkpoint mode).
# Env passthrough: RESEARCH_OUTPUT_DIR (root), RESEARCH_HOOK_LOG (a file the driver appends
# one line per hook firing to; used by the contract test), RESEARCH_AUTO_ESCALATE, etc.
#
# bash 3.2 compatible (constraint C-05): no mapfile, no declare -A, no ${var,,}.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SKILL_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
# RESEARCH_HOOK_DIR lets a test substitute stub hooks; production uses the skill's hooks/.
HOOK_DIR="${RESEARCH_HOOK_DIR:-$SKILL_DIR/hooks}"
REPO_ROOT="$(cd "$SKILL_DIR/../../.." && pwd)"
SLUG_LIB="$REPO_ROOT/shared/scripts/lib/slug.sh"
[ -f "$SLUG_LIB" ] && . "$SLUG_LIB"

log()  { echo "[deep-research] $*" >&2; }
now()  { date -u +"%Y-%m-%dT%H:%M:%SZ"; }

# ---------------------------------------------------------------- arguments
QUERY=""; DEPTH="deep"; SCALE="auto"; CITATION_STYLE="APA"; KB_IDS=""; JOB_ID=""
RESUME=""; CHECK_TOOLS=0
OUTPUT_ROOT="${RESEARCH_OUTPUT_DIR:-$HOME/.prometheus/research}"
while [ $# -gt 0 ]; do
  case "$1" in
    --query) shift; QUERY="${1:-}" ;;
    --depth) shift; DEPTH="${1:-deep}" ;;
    --scale) shift; SCALE="${1:-auto}" ;;
    --citation-style) shift; CITATION_STYLE="${1:-APA}" ;;
    --kb) shift; KB_IDS="${KB_IDS:+$KB_IDS,}${1:-}" ;;
    --job-id) shift; JOB_ID="${1:-}" ;;
    --resume) shift; RESUME="${1:-}" ;;
    --output-root) shift; OUTPUT_ROOT="${1:-}" ;;
    --check-tools) CHECK_TOOLS=1 ;;
    -h|--help) sed -n '2,25p' "$0"; exit 0 ;;
    *) if [ -z "$QUERY" ] && [ -z "$RESUME" ]; then QUERY="$1"; else log "unknown argument: $1"; exit 1; fi ;;
  esac
  shift
done
case "$DEPTH" in shallow|deep|exhaustive) ;; *) log "depth must be shallow, deep, or exhaustive (got $DEPTH)"; exit 1 ;; esac
case "$SCALE" in auto|direct|full) ;; *) log "scale must be auto, direct, or full (got $SCALE)"; exit 1 ;; esac

# ---------------------------------------------------------------- --check-tools
if [ "$CHECK_TOOLS" -eq 1 ]; then
  rc=0
  report() { printf '%-18s %s\n' "$1" "$2"; }
  [ -n "${TAVILY_API_KEY:-}" ] && report search:tavily present || report search:tavily absent
  [ -n "${FIRECRAWL_API_KEY:-}" ] && report search:firecrawl present || report search:firecrawl absent
  if [ -z "${TAVILY_API_KEY:-}${FIRECRAWL_API_KEY:-}" ]; then report search "NONE (stage 02 will be blocked)"; fi
  command -v python3 >/dev/null 2>&1 && report python3 present || { report python3 absent; }
  command -v jq >/dev/null 2>&1 && report jq present || { report jq "absent (required)"; rc=1; }
  GW=""; if [ -f "$REPO_ROOT/shared/scripts/lib/kbd-model-resolve.sh" ]; then . "$REPO_ROOT/shared/scripts/lib/kbd-model-resolve.sh" 2>/dev/null || true; GW="$(kbd_resolve_gateway 2>/dev/null || true)"; fi
  [ -n "$GW" ] && report gateway "$GW" || report gateway "unreachable (judge and semantic stages degrade to blocked)"
  SM="${SURREAL_MEMORY_URL:-http://127.0.0.1:8090}"; if curl -s --max-time 3 --noproxy '*' "$SM/health" >/dev/null 2>&1; then report surreal-memory "$SM"; else report surreal-memory "absent (stages 04 and 07 degrade to on-disk only)"; fi
  [ -n "${RESEARCH_STAGE_RUNNER:-}" ] && report runner "$RESEARCH_STAGE_RUNNER" || report runner "none (checkpoint mode)"
  report output-root "$OUTPUT_ROOT"
  exit "$rc"
fi

command -v jq >/dev/null 2>&1 || { log "jq is required"; exit 1; }

# ---------------------------------------------------------------- package resolution
if [ -n "$RESUME" ]; then
  if [ -d "$RESUME" ]; then PKG="$(cd "$RESUME" && pwd)"; elif [ -d "$OUTPUT_ROOT/$RESUME" ]; then PKG="$(cd "$OUTPUT_ROOT/$RESUME" && pwd)"; else log "package to resume not found: $RESUME"; exit 1; fi
  [ -f "$PKG/checkpoint.json" ] || { log "no checkpoint.json in $PKG; cannot resume"; exit 1; }
  PACKAGE_ID="$(basename "$PKG")"
  QUERY="$(jq -r '.query // ""' "$PKG/checkpoint.json")"
  DEPTH="$(jq -r '.depth // "deep"' "$PKG/checkpoint.json")"
  JOB_ID="$(jq -r '.job_id // ""' "$PKG/checkpoint.json")"
  CITATION_STYLE="$(jq -r '.citation_style // "APA"' "$PKG/checkpoint.json")"
  [ "$SCALE" = "auto" ] && SCALE="$(jq -r '.scale // "auto"' "$PKG/checkpoint.json")"
  log "Resuming $PACKAGE_ID (depth $DEPTH, scale $SCALE)"
else
  [ -n "$QUERY" ] || { log "--query is required (or --resume)"; exit 1; }
  [ "${#QUERY}" -ge 5 ] || { log "query is too short (min 5 chars)"; exit 1; }
  if command -v slug_from_text >/dev/null 2>&1; then SLUG="$(slug_from_text "$QUERY")"; PACKAGE_ID="$(package_id_new "$SLUG")"; else
    SLUG="$(printf '%s' "$QUERY" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]\{1,\}/-/g; s/^-//; s/-$//' | cut -d- -f1-5)"; PACKAGE_ID="$SLUG-$(date -u +%Y%m%d)-$(od -An -N2 -tx1 /dev/urandom | tr -d ' \n')"; fi
  [ -n "$JOB_ID" ] || JOB_ID="job-$(date +%s)-$(od -An -N4 -tx1 /dev/urandom | tr -d ' \n')"
  PKG="$OUTPUT_ROOT/$PACKAGE_ID"
  mkdir -p "$PKG/sources"
  log "Package $PACKAGE_ID at $PKG (job $JOB_ID, depth $DEPTH, scale $SCALE)"
fi
export RESEARCH_PACKAGE_ID="$PACKAGE_ID" RESEARCH_JOB_ID="$JOB_ID" RESEARCH_QUERY="$QUERY" RESEARCH_DEPTH="$DEPTH" RESEARCH_OUTPUT_DIR="$OUTPUT_ROOT" RESEARCH_PACKAGE_PATH="$PKG"
CP="$PKG/checkpoint.json"

# ---------------------------------------------------------------- checkpoint helpers
cp_init() {
  [ -f "$CP" ] && return 0
  jq -n --arg pid "$PACKAGE_ID" --arg jid "$JOB_ID" --arg q "$QUERY" --arg d "$DEPTH" --arg sc "$SCALE" --arg cs "$CITATION_STYLE" --arg kb "$KB_IDS" --arg ts "$(now)" \
    --arg slug "${PACKAGE_ID%-[0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9]-[0-9a-f][0-9a-f][0-9a-f][0-9a-f]}" \
    --arg pm "${KBD_PRODUCER_MODEL:-${RESEARCH_PRODUCER_MODEL:-unknown}}" \
    '{package_id:$pid, slug:$slug, producer_model:$pm, job_id:$jid, query:$q, depth:$d, scale:$sc, citation_style:$cs,
      kb_ids: ($kb | if . == "" then [] else split(",") end), model_routing:{},
      stages_planned:[], stages_completed:[], current_stage:null, status:"running", blocked:null,
      created_at:$ts, last_updated_at:$ts, completed_at:null,
      integrations:{surreal_memory_used:false, sycophancy_correction_used:false, feynman_gate_used:false, adversarial_review_used:false},
      hook_log:[], notes:[]}' > "$CP"
}
cp_set() { # cp_set '<jq expression>' [--arg k v ...]
  local expr="$1"; shift
  local tmp; tmp="$(mktemp)"
  jq "$@" --arg ts "$(now)" "$expr | .last_updated_at = \$ts" "$CP" > "$tmp" && mv "$tmp" "$CP"
}
cp_get() { jq -r "$1" "$CP"; }
note() { log "NOTE: $*"; cp_set '.notes += [$n]' --arg n "$*"; }

# ---------------------------------------------------------------- plan.md ledger
ledger_ensure() {
  local plan="$PKG/plan.md"; [ -f "$plan" ] || return 0
  for sec in "## Task ledger" "## Verification log" "## Decision log"; do
    grep -q "^$sec" "$plan" || printf '\n%s\n\n' "$sec" >> "$plan"
  done
  # Fold in any ledger lines written before stage 01 produced plan.md.
  if [ -s "$PKG/.ledger-pending.md" ]; then
    while IFS= read -r pending; do ledger_append_now "## Verification log" "$pending"; done < "$PKG/.ledger-pending.md"
    rm -f "$PKG/.ledger-pending.md"
  fi
}
ledger_append() { # ledger_append "<section>" "<line>"
  local plan="$PKG/plan.md"
  if [ ! -f "$plan" ]; then
    # plan.md is stage 01's artifact; until it exists, keep the line so it is never lost.
    printf '%s\n' "$2" >> "$PKG/.ledger-pending.md"
    return 0
  fi
  ledger_ensure
  ledger_append_now "$1" "$2"
}
ledger_append_now() { # ledger_append_now "<section>" "<line>"  (plan.md must exist)
  local plan="$PKG/plan.md" sec="$1" line="$2"
  # Insert the line at the end of the section: find the section, then the next "## " or EOF.
  awk -v sec="$sec" -v line="- $(now) $line" '
    BEGIN{done=0}
    { if (insec && /^## / && !done) { print line; print ""; done=1; insec=0 } }
    { print }
    $0 == sec { insec=1 }
    END{ if (insec && !done) { print line } }' "$plan" > "$plan.tmp" && mv "$plan.tmp" "$plan"
}

# ---------------------------------------------------------------- hooks
fire_hook() { # fire_hook <name> [extra env assignments via caller's export]
  local name="$1"
  local rc=0
  local script="$HOOK_DIR/$name.sh"
  [ -f "$script" ] || { log "hook $name not found; skipped"; return 0; }
  set +e; bash "$script" >&2; rc=$?; set -e
  cp_set '.hook_log += [{hook:$h, exit:($e|tonumber), at:$ts}]' --arg h "$name" --arg e "$rc"
  # Marker: stage-level hooks carry the stage number; run-level hooks (pre-research,
  # post-export) do not.
  local marker="$name"
  case "$name" in post-stage|on-contradiction) marker="$name${RESEARCH_CURRENT_STAGE:+ $RESEARCH_CURRENT_STAGE}" ;; esac
  [ -n "${RESEARCH_HOOK_LOG:-}" ] && printf '%s %s exit=%s\n' "$(now)" "$marker" "$rc" >> "$RESEARCH_HOOK_LOG"
  ledger_append "## Verification log" "hook $name exit $rc"
  return "$rc"
}

# ---------------------------------------------------------------- stage contracts
# Run threaded stage 02: dispatch bounded workers, then merge deterministically.
#
# Refuses to run half of the pair. The merge is also the enforcement point for
# the director's no-search rule, so skipping it would drop that check as well as
# leaving stage 02 unsatisfiable.
run_threaded_stage_02() {
    local sched merge
    sched="$(command -v prometheus-research 2>/dev/null || true)"
    merge="$SCRIPT_DIR/merge-threads.sh"

    if [ ! -d "$PKG/threads" ]; then
        log "threaded stage 02 needs $PKG/threads/<tid>/brief.json written by the director"
        return 1
    fi
    if [ -z "$sched" ]; then
        log "threaded stage 02 needs the prometheus-research binary on PATH"
        return 1
    fi
    if [ ! -x "$merge" ]; then
        log "threaded stage 02 needs $merge"
        return 1
    fi

    "$sched" threads run --package "$PKG" \
        --max-parallel "${RESEARCH_MAX_PARALLEL:-3}" \
        --thread-seconds "${RESEARCH_THREAD_SECONDS:-1800}" \
        --job-seconds "${RESEARCH_JOB_SECONDS:-1800}" >&2 || {
        # A thread that failed or timed out still has a ledger row, and the merge
        # can work from the threads that did return. Only a scheduler that could
        # not run at all is fatal, and that exits before writing the ledger.
        [ -f "$PKG/threads/index.json" ] || { log "scheduler produced no ledger"; return 1; }
        log "some threads did not complete; merging what returned"
    }

    "$merge" --package "$PKG" >&2 || { log "merge-threads.sh failed"; return 1; }
    return 0
}

# Stage 09 as four passes: outline, parallel section writers, deterministic
# assembly, coherence edit, then a re-assembly that enforces the claim-set
# invariant. `direct` scale keeps the single-call report-synthesizer path — a
# short report has no ceiling to remove, and splitting it would add passes
# without buying accuracy.
#
# The assembler runs TWICE by design. The first pass records which claims the
# report cites; the second proves the editor only removed from that set. An
# editor that could add a citation could add an unsupported one at exactly the
# point where the prose reads most fluently.
run_multipass_stage_09() {
    local assemble="$SCRIPT_DIR/assemble-report.sh"
    [ -x "$assemble" ] || { log "stage 09 multi-pass needs $assemble"; return 1; }

    if [ ! -f "$PKG/report/outline.json" ]; then
        log "stage 09 multi-pass needs report/outline.json from the outline architect"
        return 1
    fi

    # Pass 3: assemble what the section writers produced.
    "$assemble" --package "$PKG" >&2 || { log "assembly failed"; return 1; }

    # Pass 4 is the coherence edit, performed by the harness against
    # report/draft.md. When it has run, re-assemble to enforce the invariant.
    if [ "${RESEARCH_SKIP_COHERENCE_EDIT:-0}" != "1" ]; then
        "$assemble" --package "$PKG" --post-edit >&2 || {
            log "post-edit assembly failed: the coherence edit changed the cited-claim set"
            return 1
        }
    fi

    cp "$PKG/report/draft.md" "$PKG/report.md" 2>/dev/null || {
        log "no report/draft.md to promote"; return 1; }
    return 0
}

stage_name() { case "$1" in 01) echo planner;; 02) echo search;; 03) echo retrieve;; 04) echo collect;; 05) echo verify;; 06) echo resolve;; 07) echo graph;; 08) echo cite;; 09) echo report;; 10) echo export;; esac; }
json_ok() { jq -e "$2" "$1" >/dev/null 2>&1; }
# validate_stage <NN> → 0 when the stage's required artifacts exist and validate; prints the reason on failure
validate_stage() {
  local s="$1"
  case "$s" in
    01) [ -f "$PKG/plan.md" ] || { echo "plan.md missing"; return 1; }
        [ "$(subq_count)" -ge 1 ] || { echo "plan.md has no '## Sub-questions' bullets"; return 1; } ;;
    02) json_ok "$PKG/sources/url-list.json" '.source_urls | type == "array"' || { echo "sources/url-list.json missing or lacks source_urls array"; return 1; } ;;
    03) local n; n=$(ls "$PKG/sources"/chunk-*.json 2>/dev/null | wc -l | tr -d ' '); [ "$n" -ge 1 ] || { echo "no sources/chunk-<n>.json"; return 1; }
        for f in "$PKG/sources"/chunk-*.json; do json_ok "$f" '(.url|type=="string") and (.chunk_id|type=="string") and (.text|type=="string")' || { echo "$(basename "$f") lacks url/chunk_id/text"; return 1; }; done ;;
    04) json_ok "$PKG/sources/registry.json" '(type=="array") or (.sources|type=="array")' || { echo "sources/registry.json missing or lacks sources array"; return 1; } ;;
    05) json_ok "$PKG/sources/credibility.json" '(.credibility_scores|type=="object") or (.verified_sources|type=="array")' || { echo "sources/credibility.json missing or lacks credibility_scores/verified_sources"; return 1; } ;;
    06) json_ok "$PKG/contradictions.json" '.contradictions | type == "array"' || { echo "contradictions.json missing or lacks contradictions array"; return 1; } ;;
    07) json_ok "$PKG/graph.json" '((.claims|type=="array") and (.relations|type=="array")) or ((.nodes|type=="array") and (.edges|type=="array"))' || { echo "graph.json missing or lacks claims/relations (or legacy nodes/edges)"; return 1; } ;;
    08) json_ok "$PKG/citations.json" '(type=="array") or (.citations|type=="array")' || { echo "citations.json missing or lacks citations array"; return 1; } ;;
    09) [ -f "$PKG/report.md" ] || { echo "report.md missing"; return 1; }
        head -1 "$PKG/report.md" | grep -q '^---$' || { echo "report.md has no front matter"; return 1; }
        # Only the frontmatter block counts: a body that quotes these keys must not pass.
        local fm; fm="$(awk 'NR==1 && /^---$/ {f=1; next} f==1 && /^---$/ {exit} f==1 {print}' "$PKG/report.md")"
        [ -n "$fm" ] || { echo "report.md front matter block is empty or unterminated"; return 1; }
        printf '%s\n' "$fm" | grep -q '^type: research-report' || { echo "report.md front matter lacks type: research-report"; return 1; }
        printf '%s\n' "$fm" | grep -Eq '^verification_status: (verified|partial|unverified)$' || { echo "report.md verification_status missing or not in enum"; return 1; } ;;
    10) [ -f "$PKG/manifest.json" ] && [ -f "$PKG/index.md" ] || { echo "manifest.json or index.md missing"; return 1; }
        bash "$SCRIPT_DIR/check-research-package.sh" --package "$PKG" >/dev/null 2>"$PKG/.check.log" || { echo "package check failed: $(grep FAIL "$PKG/.check.log" | head -1)"; return 1; } ;;
  esac
  return 0
}
# requires_for <NN>: upstream stages whose artifacts must validate before NN may start.
# 02..06 require the nearest PLANNED predecessor (a direct run has no 04, so 05 requires 03);
# 07, 08, 09 always require 05 (verified sources); 10 requires 09.
prev_planned() { local prev="" s; for s in $STAGES; do [ "$s" = "$1" ] && { echo "$prev"; return; }; prev="$s"; done; echo ""; }
requires_for() { case "$1" in 01) echo "";; 02|03|04|05|06) prev_planned "$1";; 07|08|09) echo "05";; 10) echo "09";; esac; }
subq_count() {
  [ -f "$PKG/plan.md" ] || { echo 0; return 0; }
  awk '/^## Sub-questions/{f=1;next} /^## /{f=0} f' "$PKG/plan.md" | grep -c '^- ' || true
}

planned_stages() {
  # Every depth ends with report and export so a run always produces a package.
  # shallow skips resolve, graph, and cite; direct additionally skips collect.
  case "$SCALE" in
    direct) echo "01 02 03 05 09 10" ;;
    *) case "$DEPTH" in shallow) echo "01 02 03 04 05 09 10";; *) echo "01 02 03 04 05 06 07 08 09 10";; esac ;;
  esac
}

# ---------------------------------------------------------------- report review (change-rah-006)
# Verifier before reviewer: the report is judged by adversarial-review after
# stage 09 and before stage 10, and only when the stage 05 verification artifact
# still validates. Verification and review never run in one dispatch. Every
# outcome is recorded in checkpoint.json (review / blocked_review) and the
# sidecar; nothing here stops export, so a judged-bad run is still auditable.
#
#   RESEARCH_JUDGE_CMD=<command>  test seam: <command> <packet.json> <findings.json>
#                                 replaces dispatch-judge.sh (the contract test uses
#                                 tests/fixtures/judge.sh); production leaves it unset.
#   RESEARCH_ADV_DIR=<dir>        test seam: an adversarial-review skill directory to use
#                                 instead of the installed one (the contract test points it
#                                 at a copy whose dispatch-judge.sh is a CLI-faithful stub).
ADV_DIR=""
for _cand in "${RESEARCH_ADV_DIR:-}" "$REPO_ROOT/skills/process/adversarial-review" "${CLAUDE_PLUGIN_ROOT:-}/skills/process/adversarial-review"; do
  [ -n "$_cand" ] && [ -f "$_cand/scripts/build-review-packet.sh" ] && { ADV_DIR="$_cand"; break; }
done
# The producer identity travels with the run so the judge!=producer check is
# real: from the environment when set, else from the checkpoint of a resumed run.
PRODUCER_MODEL="${KBD_PRODUCER_MODEL:-${RESEARCH_PRODUCER_MODEL:-}}"
[ -n "$PRODUCER_MODEL" ] || PRODUCER_MODEL="$(jq -r '.producer_model // ""' "$CP" 2>/dev/null || true)"
[ -n "$PRODUCER_MODEL" ] || PRODUCER_MODEL="unknown"

sync_report_status() {
  # Stage 09 writes a provisional verification_status: it cannot know the review
  # outcome. After the review, rewrite report.md's frontmatter to the value the
  # documented rule derives (check-research-package.sh --derive), so the report,
  # the manifest stage 10 copies from it, and the drift check all agree. This
  # raises as well as lowers: a run whose gates and review pass ends `verified`.
  local rp="$PKG/report.md" derived; [ -f "$rp" ] || return 0
  derived="$(bash "$SCRIPT_DIR/check-research-package.sh" --derive "$PKG" 2>/dev/null)" || derived=""
  case "$derived" in verified|partial|unverified) ;; *) log "could not derive verification_status; report.md left as written"; return 0 ;; esac
  awk -v v="$derived" 'BEGIN{fm=0} NR==1 && /^---$/ {fm=1; print; next} fm==1 && /^---$/ {fm=2} fm==1 && /^verification_status: / {print "verification_status: " v; next} {print}' "$rp" > "$rp.tmp" && mv "$rp.tmp" "$rp"
  ledger_append "## Verification log" "report verification_status set to $derived (derived after review)"
}
review_blocked() { # review_blocked "<reason>" — records the blocked review; never exits
  cp_set '.blocked_review = $r | .integrations.adversarial_review_used = false' --arg r "blocked: $1"
  sync_report_status
  ledger_append "## Verification log" "adversarial review blocked: $1"
  note "adversarial review blocked: $1"
}
review_report() {
  # A recorded verdict is final: a resumed run is not re-judged. A recorded
  # BLOCKED review (refused, or judge unavailable) is retried, because both are
  # conditions a resume can repair: stage 05 is re-run when its artifact stopped
  # validating, and a gateway can come back. The retry refuses again if stage 05
  # still does not validate, so the guarantee holds either way.
  if jq -e '.review != null' "$CP" >/dev/null 2>&1; then
    log "report review already recorded; not re-judging"; return 0
  fi
  if jq -e '.blocked_review != null' "$CP" >/dev/null 2>&1; then
    log "retrying the report review ($(cp_get '.blocked_review'))"
    cp_set '.blocked_review = null'
  fi
  local reason prc jrc verdict crit warn
  if ! reason="$(validate_stage 05)"; then
    review_blocked "review refused, stage 05 verification missing or invalid"; return 0
  fi
  [ -n "$ADV_DIR" ] || { review_blocked "judge unavailable (adversarial-review skill not installed)"; return 0; }
  command -v python3 >/dev/null 2>&1 || { review_blocked "judge unavailable (python3 is required to build the review packet)"; return 0; }
  mkdir -p "$PKG/review"
  # What the judge sees must be honest at packet time: the frontmatter is set to
  # the PRE-review derivation (a full run derives at most `partial` here, because
  # the review has not run) and the sidecar is the interim one, which says the
  # review is pending. The post-review sync below raises the value if it passes.
  sync_report_status
  bash "$SCRIPT_DIR/write-provenance.sh" "$PKG" --interim >/dev/null 2>"$PKG/review/packet.log" || { review_blocked "judge unavailable (interim sidecar could not be written)"; return 0; }
  set +e
  KBD_PRODUCER_MODEL="$PRODUCER_MODEL" bash "$ADV_DIR/scripts/build-review-packet.sh" --mode artifact --target research --package "$PKG" --out "$PKG/review/packet.json" >/dev/null 2>"$PKG/review/packet.log"; prc=$?
  set -e
  [ "$prc" -eq 0 ] || { review_blocked "judge unavailable (packet build exited $prc: $(tail -1 "$PKG/review/packet.log"))"; return 0; }
  ledger_append "## Verification log" "review packet built (report.md, sidecar, plan.md; producer $PRODUCER_MODEL)"
  set +e
  if [ -n "${RESEARCH_JUDGE_CMD:-}" ]; then
    $RESEARCH_JUDGE_CMD "$PKG/review/packet.json" "$PKG/review/findings.json" >/dev/null 2>"$PKG/review/judge.log"; jrc=$?
  else
    KBD_PRODUCER_MODEL="$PRODUCER_MODEL" bash "$ADV_DIR/scripts/dispatch-judge.sh" --mode artifact --packet "$PKG/review/packet.json" --out "$PKG/review/findings.json" >/dev/null 2>"$PKG/review/judge.log"; jrc=$?
  fi
  set -e
  # dispatch-judge.sh exit codes: 3 no gateway / no usable completion, 2 unusable
  # output, 4 usage or environment. Each is a different blocked reason so the
  # sidecar distinguishes "no judge" from "the judge refused".
  if [ "$jrc" -ne 0 ]; then
    case "$jrc" in
      3) review_blocked "judge unavailable (dispatch exited 3: $(tail -1 "$PKG/review/judge.log" 2>/dev/null))" ;;
      2) review_blocked "judge refused (dispatch exited 2, unusable judge output: $(tail -1 "$PKG/review/judge.log" 2>/dev/null))" ;;
      *) review_blocked "judge failed (dispatch exited $jrc: $(tail -1 "$PKG/review/judge.log" 2>/dev/null))" ;;
    esac
    return 0
  fi
  if ! jq -e '.verdict and (.findings | type == "array")' "$PKG/review/findings.json" >/dev/null 2>&1; then
    review_blocked "judge refused (findings.json is not shape-valid)"; return 0
  fi
  verdict="$(jq -r '.verdict' "$PKG/review/findings.json")"
  crit="$(jq '[.findings[] | select(.severity == "CRITICAL")] | length' "$PKG/review/findings.json")"
  warn="$(jq '[.findings[] | select(.severity == "WARNING")] | length' "$PKG/review/findings.json")"
  cp_set '.review = {verdict:$v, critical:($c|tonumber), warning:($w|tonumber), findings:"review/findings.json", warnings:$ws} | .integrations.adversarial_review_used = true' \
    --arg v "$verdict" --arg c "$crit" --arg w "$warn" \
    --argjson ws "$(jq -c '[.findings[] | select(.severity == "WARNING") | .claim]' "$PKG/review/findings.json")"
  ledger_append "## Verification log" "adversarial review $verdict ($crit CRITICAL, $warn WARNING)"
  sync_report_status
  if [ "$verdict" = "BLOCK" ]; then
    ledger_append "## Task ledger" "report review BLOCK: $crit CRITICAL finding(s); package exported as partial"
    log "report review BLOCK: $crit CRITICAL finding(s); exporting as partial"
  else
    log "report review $verdict ($crit CRITICAL, $warn WARNING)"
  fi
  return 0
}

# ---------------------------------------------------------------- exit handling
finish() {
  rc=$?   # captured first, as a global, before anything can change it
  trap - EXIT
  [ -f "$CP" ] || exit "$rc"
  # A run that did not reach "complete" or "awaiting_stage" can never exit 0.
  if [ "$rc" -eq 0 ]; then
    st="$(cp_get '.status')"
    case "$st" in complete) ;; awaiting_stage) rc=3 ;; *) rc=1 ;; esac
  fi
  if [ "$rc" -eq 3 ]; then
    # Awaiting a stage: not a failure, no BLOCKED sidecar. The interim sidecar is
    # still mandatory; if it cannot be written the environment is broken, so the
    # run is blocked (visible) rather than left waiting with no sidecar.
    if ! bash "$SCRIPT_DIR/write-provenance.sh" "$PKG" --interim >/dev/null 2>"$PKG/.provenance.log"; then
      log "BLOCKED: interim provenance sidecar could not be written: $(tail -1 "$PKG/.provenance.log")"
      cp_set '.status = "blocked" | .blocked = {stage: "provenance", reason: "interim sidecar could not be written"} | .notes += ["interim sidecar write failed while awaiting stage " + (.current_stage // "?")]'
      exit 1
    fi
    exit 3
  fi
  if [ "$rc" -ne 0 ] && [ "$(cp_get '.status')" != "blocked" ]; then
    cp_set '.status = "blocked" | .blocked = {stage: (.current_stage // "unknown"), reason: ("driver exited with status " + $r)}' --arg r "$rc"
  fi
  # The sidecar is a gate: a run without one is not a delivered run.
  if ! bash "$SCRIPT_DIR/write-provenance.sh" "$PKG" >/dev/null 2>"$PKG/.provenance.log"; then
    log "BLOCKED: provenance sidecar could not be written: $(tail -1 "$PKG/.provenance.log")"
    cp_set '.status = "blocked" | .blocked = {stage: "provenance", reason: "sidecar could not be written"}'
    [ "$rc" -eq 0 ] && rc=1
  fi
  exit "$rc"
}
trap finish EXIT
block() { # block <stage> <reason>
  cp_set '.status = "blocked" | .blocked = {stage:$s, reason:$r}' --arg s "$1" --arg r "$2"
  ledger_append "## Task ledger" "stage $1 BLOCKED: $2"
  log "BLOCKED at stage $1: $2"
  exit 1
}

# ---------------------------------------------------------------- run
cp_init
if [ -z "$RESUME" ]; then
  export RESEARCH_CURRENT_STAGE=""
  fire_hook pre-research || block 01 "pre-research hook failed"
fi

STAGES="$(planned_stages)"
cp_set '.stages_planned = ($p | split(" ")) | .scale = $sc' --arg p "$STAGES" --arg sc "$SCALE"

for STAGE in $STAGES; do
  NAME="$(stage_name "$STAGE")"
  # Resume: skip a completed stage only if its artifacts still validate.
  if jq -e --arg s "$STAGE" '.stages_completed | index($s)' "$CP" >/dev/null 2>&1; then
    if [ "$STAGE" = "10" ] && jq -e '(.blocked_review != null) and (.review == null)' "$CP" >/dev/null 2>&1; then
      # A blocked report review is retried on resume: export is re-run so the
      # manifest and sidecar copy the retried outcome (see review_report).
      log "stage 10 recorded complete with a blocked report review; retrying the review and re-exporting"
      cp_set '.stages_completed -= ["10"] | .completed_at = null'
    elif reason="$(validate_stage "$STAGE")"; then log "stage $STAGE ($NAME) already complete; skipping"; continue
    else note "stage $STAGE was recorded complete but no longer validates ($reason); re-running"; cp_set '.stages_completed -= [$s]' --arg s "$STAGE"; fi
  fi
  # Upstream requirement (this is where stage 06 refuses to start without a valid stage 05).
  for req in $(requires_for "$STAGE"); do
    if ! reason="$(validate_stage "$req")"; then block "$STAGE" "cannot start: stage $req artifacts invalid ($reason)"; fi
  done
  cp_set '.current_stage = $s' --arg s "$STAGE"
  export RESEARCH_CURRENT_STAGE="$STAGE" RESEARCH_STAGE_START_TS="$(now)"

  if [ "$STAGE" = "10" ]; then
    # The report is judged after stage 09 and before export (verifier before
    # reviewer, change-rah-006). Its outcome is in the checkpoint before the
    # sidecar and manifest are written, so both copy it.
    review_report
    # Export is deterministic: the driver runs it. The manifest copies the run's
    # final state, so stage 10 is recorded complete and the provenance sidecar is
    # written BEFORE export; the sidecar is rewritten identically at exit.
    cp_set '.stages_completed += ["10"] | .status = "complete" | .current_stage = null | .completed_at = $ts'
    bash "$SCRIPT_DIR/write-provenance.sh" "$PKG" >/dev/null 2>&1 || block 10 "provenance sidecar could not be written"
    # The driver fires post-export itself (below, with its real exit code), so the
    # exporter is told not to fire it.
    set +e; RESEARCH_POST_EXPORT_BY_CALLER=1 bash "$SCRIPT_DIR/export-package.sh" "$PKG" >/dev/null 2>"$PKG/.export.log"; erc=$?; set -e
    if [ "$erc" -ne 0 ]; then cp_set '.stages_completed -= ["10"] | .completed_at = null'; block 10 "export-package.sh exited $erc: $(tail -1 "$PKG/.export.log")"; fi
    if ! reason="$(validate_stage 10)"; then cp_set '.stages_completed -= ["10"] | .completed_at = null'; block 10 "artifact contract not met: $reason"; fi
    ledger_append "## Task ledger" "stage 10 (export) complete"
    ledger_append "## Verification log" "stage 10 package validated"
    # A hook failure here must not wedge the run: roll stage 10 back so a later
    # --resume re-runs export and the hooks instead of skipping a "complete" stage.
    if ! fire_hook post-stage; then cp_set '.stages_completed -= ["10"] | .completed_at = null'; block 10 "post-stage hook failed"; fi
    if ! fire_hook post-export; then cp_set '.stages_completed -= ["10"] | .completed_at = null'; block 10 "post-export hook failed"; fi
    cp_set '.blocked = null'
    continue
  elif [ "$STAGE" = "09" ] && [ "$SCALE" = "full" ] && [ -f "$PKG/report/outline.json" ] && ! validate_stage 09 >/dev/null 2>&1; then
    log "stage 09 (report): multi-pass assembly"
    if ! run_multipass_stage_09; then
      block 09 "multi-pass stage 09 failed (see the log above)"
    fi
  elif [ "$STAGE" = "02" ] && [ "${RESEARCH_THREADED:-0}" = "1" ] && ! validate_stage 02 >/dev/null 2>&1; then
    # Threaded stage 02: the scheduler dispatches workers, the merge folds their
    # output into the stage 02/03/04 artifacts. These two run TOGETHER or not at
    # all — the scheduler alone leaves threads/ populated and sources/url-list.json
    # absent, so stage 02 could never validate and the run would block with an
    # artifact error that says nothing about the real cause.
    log "stage 02 (search): threaded dispatch, cap ${RESEARCH_MAX_PARALLEL:-3}"
    if ! run_threaded_stage_02; then
      block 02 "threaded stage 02 failed (see the log above)"
    fi
  elif ! validate_stage "$STAGE" >/dev/null 2>&1; then
    if [ -n "${RESEARCH_STAGE_RUNNER:-}" ]; then
      log "stage $STAGE ($NAME): running $RESEARCH_STAGE_RUNNER"
      set +e; $RESEARCH_STAGE_RUNNER "$STAGE" "$PKG" >&2; rrc=$?; set -e
      [ "$rrc" -eq 0 ] || block "$STAGE" "stage runner exited $rrc"
    else
      cp_set '.status = "awaiting_stage"'
      log "stage $STAGE ($NAME) is next. Run the stage skill against the package, then re-invoke with --resume:"
      log "  /stage-$STAGE-$NAME  (package: $PKG)"
      log "  bash $SCRIPT_DIR/run-research.sh --resume \"$PKG\""
      jq -nc --arg s "$STAGE" --arg n "$NAME" --arg p "$PKG" '{next_stage:$s, skill:("stage-" + $s + "-" + $n), package_dir:$p}'
      exit 3
    fi
  fi

  if ! reason="$(validate_stage "$STAGE")"; then block "$STAGE" "artifact contract not met: $reason"; fi
  # Completing a stage clears any earlier block (a resumed run has recovered).
  cp_set '.stages_completed += [$s] | .status = "running" | .blocked = null' --arg s "$STAGE"
  ledger_append "## Task ledger" "stage $STAGE ($NAME) complete"
  ledger_append "## Verification log" "stage $STAGE artifacts validated"

  # A failing hook blocks the stage: it is rolled back out of stages_completed so a
  # later --resume re-runs the stage and its hooks instead of skipping it.
  if ! fire_hook post-stage; then cp_set '.stages_completed -= [$s]' --arg s "$STAGE"; block "$STAGE" "post-stage hook failed"; fi
  if [ "$STAGE" = "06" ] && jq -e '[.contradictions[] | select(.resolved == false)] | length > 0' "$PKG/contradictions.json" >/dev/null 2>&1; then
    export RESEARCH_CONTRADICTION_TOPIC="$(jq -r '[.contradictions[] | select(.resolved == false)][0].topic // "unknown"' "$PKG/contradictions.json")"
    if ! fire_hook on-contradiction; then cp_set '.stages_completed -= ["06"]'; block 06 "on-contradiction hook failed"; fi
  fi

  if [ "$STAGE" = "01" ]; then
    ledger_ensure
    if [ "$SCALE" = "auto" ]; then
      n="$(subq_count)"
      if [ "$n" -lt 3 ]; then SCALE="direct"; else SCALE="full"; fi
      ledger_append "## Decision log" "scale $SCALE: planner emitted $n sub-question(s)"
      STAGES="$(planned_stages)"; cp_set '.stages_planned = ($p | split(" ")) | .scale = $sc' --arg p "$STAGES" --arg sc "$SCALE"
      log "scale decision: $SCALE ($n sub-questions)"
      # Restart the loop over the recomputed stage set; 01 is complete and is skipped on resume.
      trap - EXIT
      exec "$0" --resume "$PKG" --scale "$SCALE"
    fi
  fi
done

# A resumed run that re-ran a stale stage (its artifact had stopped validating)
# set status back to "running"; when export was skipped as still valid, nothing
# restores "complete". Every planned stage is now recorded, so record it here.
if jq -e '(.stages_planned - .stages_completed) | length == 0' "$CP" >/dev/null 2>&1 && [ "$(cp_get '.status')" = "running" ]; then
  cp_set '.status = "complete" | .current_stage = null | .completed_at = (.completed_at // $ts) | .blocked = null'
  bash "$SCRIPT_DIR/write-provenance.sh" "$PKG" >/dev/null 2>&1 || block 10 "provenance sidecar could not be rewritten after resume"
fi
ledger_append "## Task ledger" "run complete ($SCALE scale, $(echo "$STAGES" | wc -w | tr -d ' ') stages)"
log "Run complete: $PKG"
exit 0
