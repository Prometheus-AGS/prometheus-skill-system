#!/usr/bin/env bash
# detect-project-context.sh — SessionStart hook: detects cloud and GitOps context for skills
set -euo pipefail

echo "═══ Prometheus GitOps Skills — Project Context ═══"

# GitOps structure presence
if find . -name "kustomization.yaml" -maxdepth 6 | grep -q overlays; then
  echo "GitOps: Kustomize overlay structure detected"
fi
if find . -name "*.yaml" -maxdepth 6 | xargs grep -l "argoproj.io/v1alpha1" 2>/dev/null | grep -q .; then
  echo "GitOps: ArgoCD Application CRs detected"
fi

# CI presence
WORKFLOWS=$(find .github/workflows -name "*.yml" -o -name "*.yaml" 2>/dev/null | wc -l | tr -d ' ')
echo "CI: $WORKFLOWS GitHub Actions workflow(s) found"

# Cloud presence
find . -name "*.tf" -maxdepth 5 2>/dev/null | xargs grep -l "google_container_cluster\|azurerm_kubernetes_cluster\|aws_eks_cluster" 2>/dev/null \
  | sed 's/.*\(google_container\|azurerm_kubernetes\|aws_eks\).*/  Terraform: \1 resource found/' || true

echo "═══════════════════════════════════════════════════"
echo "Available skills: gitops-bootstrap · gitops-transform · argocd-multicloud · kustomize-overlay"
