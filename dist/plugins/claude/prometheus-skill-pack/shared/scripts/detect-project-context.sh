#!/usr/bin/env bash
# detect-project-context.sh — SessionStart hook: detects cloud and GitOps context for skills
set -euo pipefail

# Observability: emit a start/end record to ~/.prometheus/hooks.log so this
# hook is visible to `doctor` and to latency analysis. The library no-ops when
# the log directory is not writable, and never changes this script's exit code.
HOOK_LOG_LIB="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/hook-log.sh"
# shellcheck source=/dev/null
[ -f "$HOOK_LOG_LIB" ] && . "$HOOK_LOG_LIB"
command -v hook_log_start >/dev/null 2>&1 && hook_log_start "SessionStart" "detect-project-context.sh"
command -v hook_log_end >/dev/null 2>&1 && trap 'hook_log_end $?' EXIT


# Verify (never refresh) OpenSpec at session start (VLBV CR-03). A new session —
# including a harness switch mid-phase — must not rewrite generated procedure text
# under a bound phase contract. The mutating refresh runs at phase entry only
# (kbd-new-phase / kbd-next-phase / kbd-new-child / kbd-next-child).
BUNDLE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
OPENSPEC_RUNNER="$BUNDLE_ROOT/skills/process/kbd-process-orchestrator/shared/openspec/cli.mjs"
if [[ ! -f "$OPENSPEC_RUNNER" ]]; then
  OPENSPEC_RUNNER="$BUNDLE_ROOT/skills/kbd-process-orchestrator/shared/openspec/cli.mjs"
fi
# The runner prints a JSON receipt (also saved under ~/.prometheus/openspec/receipts).
# Hook output is session context, and Codex parses output that opens with `{` as a
# structured hook response, so the receipt is captured, never echoed.
OPENSPEC_VERIFY="$(node "$OPENSPEC_RUNNER" verify --project "$PWD" 2>&1 >/dev/null || true)"
case "$OPENSPEC_VERIFY" in
  *'"status":"refresh-needed"'*)
    echo "OpenSpec integration is not at the pack-pinned version; run the KBD phase-entry refresh before phase work." ;;
  *'"status":"current"'*|*'"status":"skipped"'*|*'"status":"disabled"'*|"") ;;
  *) echo "OpenSpec session check did not complete; run: node $OPENSPEC_RUNNER verify --project \"\$PWD\"" ;;
esac

echo "═══ Prometheus GitOps Skills — Project Context ═══"

# GitOps structure presence
if find . -name "kustomization.yaml" -maxdepth 6 | grep -q overlays; then
  echo "GitOps: Kustomize overlay structure detected"
fi
if find . -name "*.yaml" -maxdepth 6 | xargs grep -l "argoproj.io/v1alpha1" 2>/dev/null | grep -q .; then
  echo "GitOps: ArgoCD Application CRs detected"
fi

# CI presence
WORKFLOWS=0
if [[ -d .github/workflows ]]; then
  WORKFLOWS=$(find .github/workflows -name "*.yml" -o -name "*.yaml" | wc -l | tr -d ' ')
fi
echo "CI: $WORKFLOWS GitHub Actions workflow(s) found"

# Cloud presence
find . -name "*.tf" -maxdepth 5 2>/dev/null | xargs grep -l "google_container_cluster\|azurerm_kubernetes_cluster\|aws_eks_cluster" 2>/dev/null \
  | sed 's/.*\(google_container\|azurerm_kubernetes\|aws_eks\).*/  Terraform: \1 resource found/' || true

echo "═══════════════════════════════════════════════════"
echo "Available skills: gitops-bootstrap · gitops-transform · argocd-multicloud · kustomize-overlay"
