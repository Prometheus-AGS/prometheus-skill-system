#!/usr/bin/env bash
# detect-stack.sh — Audits existing CI/CD for TJ-CICD-001 compliance
# Outputs a structured JSON report of violations, warnings, and compliant items
set -euo pipefail

VIOLATIONS=()
WARNINGS=()
COMPLIANT=()
CLOUDS=()
REGISTRIES=()

scan_workflow() {
  local wf="$1"
  # Direct deploy violations
  grep -n "kubectl apply" "$wf" 2>/dev/null | while IFS=: read -r lineno content; do
    VIOLATIONS+=("[V] kubectl apply in CI: $wf:$lineno — bypasses GitOps")
  done
  grep -n "helm upgrade" "$wf" 2>/dev/null | while IFS=: read -r lineno content; do
    VIOLATIONS+=("[V] helm upgrade in CI: $wf:$lineno — bypasses GitOps")
  done
  grep -n "argocd app sync\|argocd sync" "$wf" 2>/dev/null | while IFS=: read -r lineno content; do
    VIOLATIONS+=("[V] ArgoCD sync called from CI: $wf:$lineno — ArgoCD must be autonomous")
  done
  # Static credential violations
  grep -n "credentials_json\|GCP_SA_KEY\|GOOGLE_APPLICATION_CREDENTIALS" "$wf" 2>/dev/null | while IFS=: read -r lineno _; do
    VIOLATIONS+=("[V] Static GCP service account key: $wf:$lineno — migrate to WIF")
  done
  grep -n "aws-access-key-id\|AWS_ACCESS_KEY_ID\|AWS_SECRET_ACCESS_KEY" "$wf" 2>/dev/null | while IFS=: read -r lineno _; do
    VIOLATIONS+=("[V] Static AWS access keys: $wf:$lineno — migrate to OIDC role")
  done
  grep -n "AZURE_CREDENTIALS\|azure_credentials\|client-secret:" "$wf" 2>/dev/null | while IFS=: read -r lineno _; do
    VIOLATIONS+=("[V] Static Azure client secret: $wf:$lineno — migrate to federated credentials")
  done
  # Compliant patterns
  grep -q "workload_identity_provider" "$wf" && COMPLIANT+=("[C] GCP WIF auth: $wf")
  grep -q "role-to-assume" "$wf" && COMPLIANT+=("[C] AWS OIDC role: $wf")
  if grep -q "client-id" "$wf" && grep -q "tenant-id" "$wf" && ! grep -q "client-secret" "$wf"; then
    COMPLIANT+=("[C] Azure federated credentials: $wf")
  fi
  grep -q "kustomize edit set image" "$wf" && COMPLIANT+=("[C] GitOps image tag promotion: $wf")
  # Registry detection
  grep -q "pkg.dev" "$wf" && REGISTRIES+=("artifact-registry") && CLOUDS+=("gke")
  grep -q "azurecr.io" "$wf" && REGISTRIES+=("acr") && CLOUDS+=("aks")
  grep -q "dkr.ecr" "$wf" && REGISTRIES+=("ecr") && CLOUDS+=("eks")
  grep -q "docker.io\|hub.docker.com" "$wf" && WARNINGS+=("[W] Docker Hub registry: $wf — consider cloud-native registry")
}

scan_manifests() {
  find . -name "*.yaml" -o -name "*.yml" | grep -v ".github\|node_modules\|vendor" | while read -r f; do
    # Secret with values
    if grep -q "^kind: Secret" "$f" 2>/dev/null; then
      if grep -q "^  data:\|^  stringData:" "$f" 2>/dev/null; then
        VIOLATIONS+=("[V] Kubernetes Secret with values committed to git: $f — use ExternalSecret")
      fi
    fi
    # Latest tag
    grep -n "image:.*:latest" "$f" 2>/dev/null | while IFS=: read -r lineno _; do
      WARNINGS+=("[W] 'latest' image tag: $f:$lineno — use SHA tags")
    done
  done
}

# Scan all workflows
for wf in .github/workflows/*.yml .github/workflows/*.yaml; do
  [ -f "$wf" ] || continue
  scan_workflow "$wf"
done

scan_manifests

# Output structured report
echo "═══════════════════════════════════════════════════════"
echo "GITOPS TRANSFORM — DETECTION REPORT"
echo "═══════════════════════════════════════════════════════"
echo ""
echo "DETECTED CLOUDS"
for c in gke aks eks; do
  if printf '%s\n' "${CLOUDS[@]}" | grep -qi "$c"; then echo "  $c  ✓"; else echo "  $c  ✗"; fi
done
echo ""
echo "VIOLATIONS (must fix — ${#VIOLATIONS[@]} found)"
for v in "${VIOLATIONS[@]}"; do echo "  $v"; done
[ ${#VIOLATIONS[@]} -eq 0 ] && echo "  (none)"
echo ""
echo "WARNINGS (${#WARNINGS[@]} found)"
for w in "${WARNINGS[@]}"; do echo "  $w"; done
[ ${#WARNINGS[@]} -eq 0 ] && echo "  (none)"
echo ""
echo "COMPLIANT (${#COMPLIANT[@]} found)"
for c in "${COMPLIANT[@]}"; do echo "  $c"; done
[ ${#COMPLIANT[@]} -eq 0 ] && echo "  (none)"
echo "═══════════════════════════════════════════════════════"
