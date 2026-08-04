#!/usr/bin/env bash
set -euo pipefail

HOOK_ID=""
HARNESS=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --hook) HOOK_ID="${2:-}"; shift 2 ;;
    --harness) HARNESS="${2:-}"; shift 2 ;;
    *) printf 'hook-dispatch-v1: unknown argument: %s\n' "$1" >&2; exit 64 ;;
  esac
done

case "$HARNESS" in
  claude-code|codex) ;;
  *) printf 'hook-dispatch-v1: unsupported harness: %s\n' "$HARNESS" >&2; exit 64 ;;
esac

BUNDLE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)"
EVOLUTION="${EVOLUTION_NAME:-default}"

run_bundle_script() {
  local relative="$1"
  shift
  case "$relative" in
    /*|../*|*/../*) printf 'hook-dispatch-v1: unsafe target: %s\n' "$relative" >&2; return 65 ;;
  esac
  local target="$BUNDLE_ROOT/$relative"
  [[ -f "$target" ]] || { printf 'hook-dispatch-v1: missing target: %s\n' "$relative" >&2; return 66; }
  bash "$target" "$@"
}

case "$HOOK_ID" in
  'sessionstart-kbd-control')
    run_bundle_script 'shared/scripts/kbd-harness-adapter.sh' 'session_start' "$HARNESS"
    ;;
  'sessionstart-kbd-open')
    bash "$HOME/.local/bin/kbd-open" 2>&1 || true
    ;;
  'sessionstart-detect-project-context')
    run_bundle_script 'shared/scripts/detect-project-context.sh' 2>&1 || true
    ;;
  'sessionstart-memory-outbox-flush')
    run_bundle_script 'shared/scripts/memory-outbox-flush.sh' 2>&1 || true
    ;;
  'sessionstart-pk-health')
    run_bundle_script 'shared/scripts/pk-health.sh' 2>&1 || true
    ;;
  'prompt-karpathy-learning')
    run_bundle_script 'shared/scripts/karpathy-hook-dispatch.sh' 'prompt' "$HARNESS"
    ;;
  'pretool-protect-tests')
    run_bundle_script 'shared/scripts/protect-tests.sh'
    ;;
  'posttool-validate-evolution-state')
    run_bundle_script 'skills/process/iterative-evolver/scripts/validate-state.sh' 2>&1 || true
    ;;
  'posttool-validate-gitops-write')
    run_bundle_script 'shared/scripts/validate-gitops-write.sh' 2>&1 || true
    ;;
  'posttool-scope-record')
    run_bundle_script 'shared/scripts/scope-record.sh' 2>&1 || true
    ;;
  'posttool-write-position-reminder')
    run_bundle_script 'shared/scripts/write-position-reminder.sh' 2>&1 || true
    ;;
  'posttool-sycophancy-artifact')
    run_bundle_script 'shared/scripts/sycophancy-check-artifact.sh'
    ;;
  'posttool-memory-writeback')
    run_bundle_script 'shared/scripts/memory-writeback.sh' 2>&1 || true
    ;;
  'subagent-assessor-checkpoint')
    run_bundle_script 'skills/process/iterative-evolver/scripts/state-checkpoint.sh' "$EVOLUTION" 'assess' 'phase_complete' 2>&1 || true
    ;;
  'subagent-assessor-dispatch')
    run_bundle_script 'skills/process/iterative-evolver/scripts/workflow-dispatch.sh' "$EVOLUTION" 'phase_complete' 'assess' 2>&1 || true
    ;;
  'subagent-analyst-checkpoint')
    run_bundle_script 'skills/process/iterative-evolver/scripts/state-checkpoint.sh' "$EVOLUTION" 'analyze' 'phase_complete' 2>&1 || true
    ;;
  'subagent-analyst-dispatch')
    run_bundle_script 'skills/process/iterative-evolver/scripts/workflow-dispatch.sh' "$EVOLUTION" 'phase_complete' 'analyze' 2>&1 || true
    ;;
  'subagent-planner-checkpoint')
    run_bundle_script 'skills/process/iterative-evolver/scripts/state-checkpoint.sh' "$EVOLUTION" 'plan' 'phase_complete' 2>&1 || true
    ;;
  'subagent-planner-dispatch')
    run_bundle_script 'skills/process/iterative-evolver/scripts/workflow-dispatch.sh' "$EVOLUTION" 'phase_complete' 'plan' 2>&1 || true
    ;;
  'subagent-executor-karpathy-learning')
    run_bundle_script 'shared/scripts/karpathy-hook-dispatch.sh' 'executor_complete' "$HARNESS"
    ;;
  'subagent-executor-validate-state')
    run_bundle_script 'skills/process/iterative-evolver/scripts/validate-state.sh' 2>&1 || true
    ;;
  'subagent-executor-checkpoint')
    run_bundle_script 'skills/process/iterative-evolver/scripts/state-checkpoint.sh' "$EVOLUTION" 'execute' 'phase_complete' 2>&1 || true
    ;;
  'subagent-executor-evaluate-session')
    run_bundle_script 'shared/scripts/evaluate-session.sh' 2>&1 || true
    ;;
  'subagent-executor-dispatch')
    run_bundle_script 'skills/process/iterative-evolver/scripts/workflow-dispatch.sh' "$EVOLUTION" 'phase_complete' 'execute' 2>&1 || true
    ;;
  'subagent-reflector-sycophancy')
    run_bundle_script 'shared/scripts/sycophancy-check-reflection.sh' 2>&1
    ;;
  'subagent-reflector-log')
    run_bundle_script 'skills/process/iterative-evolver/scripts/log-reflection.sh' 2>&1 || true
    ;;
  'subagent-reflector-checkpoint')
    run_bundle_script 'skills/process/iterative-evolver/scripts/state-checkpoint.sh' "$EVOLUTION" 'reflect' 'phase_complete' 2>&1 || true
    ;;
  'subagent-reflector-dispatch')
    run_bundle_script 'skills/process/iterative-evolver/scripts/workflow-dispatch.sh' "$EVOLUTION" 'phase_complete' 'reflect' 2>&1 || true
    ;;
  'subagent-fallback-checkpoint')
    run_bundle_script 'shared/scripts/subagent-checkpoint-fallback.sh' 2>&1 || true
    ;;
  'stop-karpathy-learning')
    run_bundle_script 'shared/scripts/karpathy-hook-dispatch.sh' 'stop' "$HARNESS"
    ;;
  'precompact-kbd-control')
    run_bundle_script 'shared/scripts/kbd-harness-adapter.sh' 'pre_compact' "$HARNESS"
    ;;
  *) printf 'hook-dispatch-v1: unknown hook id: %s\n' "$HOOK_ID" >&2; exit 64 ;;
esac
