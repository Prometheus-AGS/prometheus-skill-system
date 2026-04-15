#!/usr/bin/env bash
# guard-direct-deploy.sh — PreToolUse hook: blocks kubectl apply and helm upgrade as deploy mechanisms
# Reads the bash command from stdin (Claude Code hook JSON input) and exits 2 to block if violation detected
set -euo pipefail

INPUT=$(cat)
CMD=$(echo "$INPUT" | jq -r '.tool_input.command // ""' 2>/dev/null || echo "")

# Check for direct deploy patterns
if echo "$CMD" | grep -qE "kubectl apply.*(deploy|k8s|manifests|helm)" ; then
  echo "BLOCKED: kubectl apply as deployment mechanism violates TJ-CICD-001 §09." >&2
  echo "All production deployments must flow through a Git commit to the GitOps repository." >&2
  echo "ArgoCD detects the commit and deploys autonomously." >&2
  exit 2
fi

if echo "$CMD" | grep -qE "helm upgrade.*--install.*(prod|staging|production)" ; then
  echo "BLOCKED: helm upgrade as deployment mechanism violates TJ-CICD-001 §09." >&2
  echo "Helm charts in production are rendered by ArgoCD from values files in the GitOps repo." >&2
  exit 2
fi

if echo "$CMD" | grep -qE "argocd app sync|argocd sync" ; then
  echo "BLOCKED: Manually triggering ArgoCD sync violates TJ-CICD-001 §09." >&2
  echo "ArgoCD detects GitOps repo changes autonomously. Sync must not be triggered from CI or manually." >&2
  exit 2
fi

exit 0
