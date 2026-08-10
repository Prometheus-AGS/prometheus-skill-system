#!/usr/bin/env bash
# /start-business-build — the headline pipeline orchestrator.
#
# Chains every layer of the Prometheus pipeline:
#   Layer 1: ZeeSpec constraint interrogation
#   Layer 2: iterative-evolver strategic plan + kbd-process-orchestrator tactical
#   Layer 3: OpenSpec change set
#   Layer 4: forge enrich + AI implementation + forge reflect (Karpathy loop)
#   Then optionally: forge package-librefang + /upload-to-bossfang
#
# This script is the v1 shell-driven implementation. The Rust-based
# `prometheus orchestrate` subcommand is queued as a separate change for
# better state-machine semantics, JSON event emission, and resume support.
# In v1 this provides a working pipeline-glue; in v2 it gets replaced.

set -euo pipefail
set +x  # protect any provider tokens in env from xtrace

# ── arg parsing ──────────────────────────────────────────────────────────────
CONCEPT=""
DRY_RUN=false
SKIP_DEPLOY=false
BOSSFANG_URL=""
TARGET="librefang-wasm"
while [[ $# -gt 0 ]]; do
    case "$1" in
        --dry-run)     DRY_RUN=true; shift ;;
        --skip-deploy) SKIP_DEPLOY=true; shift ;;
        --bossfang)    BOSSFANG_URL="$2"; shift 2 ;;
        --target)      TARGET="$2"; shift 2 ;;
        --help|-h)
            sed -n '2,15p' "$0" | sed 's/^# //;s/^#//'
            exit 0
            ;;
        -*) echo "Unknown flag: $1" >&2; exit 2 ;;
        *)  if [ -z "$CONCEPT" ]; then CONCEPT="$1"; shift; else echo "Extra positional arg: $1" >&2; exit 2; fi ;;
    esac
done

if [ -z "$CONCEPT" ]; then
    echo "Usage: orchestrate.sh \"<concept>\" [--dry-run] [--skip-deploy] [--bossfang <url>] [--target docker|librefang-wasm|both]" >&2
    exit 2
fi

# ── slug + state dir ─────────────────────────────────────────────────────────
SLUG="$(printf '%s' "$CONCEPT" | tr '[:upper:]' '[:lower:]' \
    | sed -E 's/[^a-z0-9]+/-/g; s/^-+|-+$//g' | head -c 60)"
STATE_DIR=".prometheus/business-builds/$SLUG"
mkdir -p "$STATE_DIR"
STATE_FILE="$STATE_DIR/state.json"
[ -f "$STATE_FILE" ] || echo '{"stages_completed":[],"started":"'"$(date -Iseconds)"'"}' >"$STATE_FILE"

emit() {
    # JSON-line event for any UI watching. Don't emit raw concept (could be PII).
    printf '{"stage":%s,"status":"%s","slug":"%s","msg":"%s"}\n' \
        "${1:-0}" "${2:-info}" "$SLUG" "${3:-}"
}

stage_done() {
    # Record stage in state.json. jq is preferred but optional; fall back to a
    # flat append. The actual orchestrator runtime should use jq.
    local stage="$1"
    if command -v jq >/dev/null 2>&1; then
        jq --arg s "$stage" '.stages_completed += [$s] | .stages_completed |= unique' \
            "$STATE_FILE" >"$STATE_FILE.tmp" && mv "$STATE_FILE.tmp" "$STATE_FILE"
    else
        echo "$stage" >>"$STATE_DIR/stages.log"
    fi
}

stage_done_already() {
    local stage="$1"
    if command -v jq >/dev/null 2>&1; then
        jq -e --arg s "$stage" '.stages_completed | index($s)' "$STATE_FILE" >/dev/null 2>&1
    else
        grep -qx "$stage" "$STATE_DIR/stages.log" 2>/dev/null
    fi
}

# ── pre-flight ───────────────────────────────────────────────────────────────
emit 0 "running" "preflight"
PREFLIGHT_FAIL=0
require_bin() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "❌ $1 not on PATH ($2)" >&2
        PREFLIGHT_FAIL=$((PREFLIGHT_FAIL + 1))
    fi
}
require_bin forge "Layer 4 enrichment — install via 'npm run doctor'"
require_bin pk    "Karpathy learning loop — install via 'npm run doctor'"

if [ "$TARGET" = "librefang-wasm" ] || [ "$TARGET" = "both" ]; then
    if command -v rustup >/dev/null 2>&1; then
        if ! rustup target list --installed 2>/dev/null | grep -q "^wasm32-unknown-unknown$"; then
            echo "❌ wasm32-unknown-unknown target not installed" >&2
            echo "   Run: rustup target add wasm32-unknown-unknown" >&2
            PREFLIGHT_FAIL=$((PREFLIGHT_FAIL + 1))
        fi
    fi
fi

if [ "$PREFLIGHT_FAIL" -gt 0 ]; then
    echo "❌ $PREFLIGHT_FAIL preflight failure(s); resolve and retry" >&2
    exit 3
fi
emit 0 "ok" "preflight passed"

# ── dry-run summary ──────────────────────────────────────────────────────────
if $DRY_RUN; then
    cat <<EOF
[Dry run] Pipeline plan for slug '$SLUG':
  Stage 1: Ideation mindmap (stub in v1)
  Stage 2: ZeeSpec interrogation
  Stage 3: Iterative-Evolver assess+plan
  Stage 4: OpenSpec / native-KBD change generation
  Stage 5: forge enrich + AI implement + forge reflect per change
  Stage 6: forge package-librefang + offer /upload-to-bossfang
           (skipped: $([ "$SKIP_DEPLOY" = true ] && echo yes || echo no))
  Target: $TARGET
  Bossfang URL: ${BOSSFANG_URL:-(prompt at end)}
  State: $STATE_FILE
EOF
    exit 0
fi

# ── stage execution ──────────────────────────────────────────────────────────
# Each stage is a thin wrapper over an existing skill or tool. The actual
# stage logic lives in those skills; this orchestrator just sequences them
# and records progress.

run_stage() {
    local n="$1" name="$2"; shift 2
    if stage_done_already "stage-$n-$name"; then
        emit "$n" "skip" "already done (resumed)"
        return 0
    fi
    emit "$n" "running" "$name"
    if "$@"; then
        stage_done "stage-$n-$name"
        emit "$n" "ok" "$name complete"
    else
        emit "$n" "fail" "$name failed"
        echo "❌ Stage $n ($name) failed. Resume with the same command after fixing." >&2
        exit 5
    fi
}

stage_1_ideation() {
    # v1 stub: write the concept verbatim to the mindmap file. Phase
    # phase-ideation-onramp will replace this with surreal-memory's
    # generate_ideation_mindmap tool call.
    cat >"$STATE_DIR/mindmap.md" <<EOF
# Ideation Mindmap (v1 stub)

Concept: $CONCEPT

(Full mindmap generation lands in phase-ideation-onramp.)
EOF
}

stage_2_zeespec() {
    # Hand the mindmap to zeespec. The actual zeespec entry-point is
    # /zeespec-interrogate <input-file>; in v1 we delegate the prompt
    # construction to the calling AI tool by leaving a sentinel file.
    cat >"$STATE_DIR/zeespec-input.md" <<EOF
# ZeeSpec Interrogation Input

$CONCEPT

(Stage-2 runner: invoke /zeespec-interrogate $STATE_DIR/zeespec-input.md
 and write the constraint manifest to $STATE_DIR/zeespec.md)
EOF
    # Sentinel so the next stage knows constraints are expected to be filled.
    : > "$STATE_DIR/.zeespec.expected"
}

stage_3_evolver() {
    # Hand zeespec output to iterative-evolver. Same delegation pattern.
    cat >"$STATE_DIR/evolver-input.md" <<EOF
# Iterative Evolver Input

Constraints in: $STATE_DIR/zeespec.md
Working dir:    $STATE_DIR

(Stage-3 runner: invoke /evolve-assess and /evolve-plan; write the change list
 to $STATE_DIR/evolver-plan.md)
EOF
}

stage_4_changes() {
    # If openspec/ exists at repo root, emit OpenSpec proposals; else native KBD.
    if [ -d "$(git rev-parse --show-toplevel 2>/dev/null)/openspec" ]; then
        echo "OpenSpec backend selected" > "$STATE_DIR/changes.backend"
    else
        echo "native-kbd backend selected" > "$STATE_DIR/changes.backend"
    fi
}

stage_5_forge() {
    # forge enrich + AI dispatch + forge reflect per change. The dispatch
    # itself is what the orchestrator AI tool drives; this stage prepares
    # the per-change directories and seeds the forge cache.
    if ! forge --version >/dev/null 2>&1; then
        echo "❌ forge not on PATH" >&2; return 1
    fi
    : >"$STATE_DIR/.forge.ready"
}

stage_6_package() {
    if $SKIP_DEPLOY; then
        emit 6 "skip" "--skip-deploy set"
        return 0
    fi
    if [ "$TARGET" = "docker" ]; then
        emit 6 "skip" "target=docker; nothing to package for librefang"
        return 0
    fi

    # Look for forge package-librefang. If not present, print the manual fallback.
    if forge --help 2>&1 | grep -q "package-librefang"; then
        forge package-librefang . || return 1
    else
        cat <<EOF >&2
ℹ️  forge package-librefang not yet installed (queued as phase-librefang-wasm-onramp).
   Manual fallback:
     cargo build --target wasm32-unknown-unknown --release -p <agent-name>-skill
     mkdir -p dist
     cp target/wasm32-unknown-unknown/release/<agent>_skill.wasm dist/
     cp skill.toml README.md dist/
     (cd dist && zip ../<agent>.lf-skill.zip *.wasm skill.toml README.md)
EOF
    fi

    if [ -n "$BOSSFANG_URL" ]; then
        # Find the produced zip and offer to upload.
        zip_file="$(find . -maxdepth 2 -name "*.lf-skill.zip" -print -quit)"
        if [ -n "$zip_file" ]; then
            echo "📦 Uploading $zip_file → $BOSSFANG_URL"
            bash "$(dirname "$0")/../../upload-to-bossfang/scripts/upload.sh" \
                "$BOSSFANG_URL" --zip "$zip_file"
        fi
    fi
}

run_stage 1 ideation     stage_1_ideation
run_stage 2 zeespec      stage_2_zeespec
run_stage 3 evolver      stage_3_evolver
run_stage 4 changes      stage_4_changes
run_stage 5 forge        stage_5_forge
run_stage 6 package      stage_6_package

emit 99 "done" "all stages complete; state at $STATE_FILE"
echo ""
echo "✨ /start-business-build complete for '$SLUG'"
echo "   State:      $STATE_FILE"
echo "   Next stage runners' inputs are seeded under $STATE_DIR/"
echo "   To resume after editing inputs: re-run the same command"
