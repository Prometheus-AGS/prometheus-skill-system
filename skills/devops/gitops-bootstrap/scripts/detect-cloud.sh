#!/usr/bin/env bash
# detect-cloud.sh — Identifies which cloud providers are in scope for GitOps bootstrap
# Output: JSON object with detected clouds, registries, and confidence scores
set -euo pipefail

result="{\"gke\":false,\"aks\":false,\"eks\":false,\"registries\":[],\"hints\":[]}"

# Helper: append to JSON array field
append_hint() { result=$(echo "$result" | jq ".hints += [\"$1\"]"); }
set_cloud() { result=$(echo "$result" | jq ".$1 = true"); }
add_registry() { result=$(echo "$result" | jq ".registries += [\"$1\"]"); }

# --- Terraform files ---
if find . -name "*.tf" -maxdepth 5 | xargs grep -l "google_container_cluster" 2>/dev/null | grep -q .; then
  set_cloud gke; append_hint "terraform: google_container_cluster found"
fi
if find . -name "*.tf" -maxdepth 5 | xargs grep -l "azurerm_kubernetes_cluster" 2>/dev/null | grep -q .; then
  set_cloud aks; append_hint "terraform: azurerm_kubernetes_cluster found"
fi
if find . -name "*.tf" -maxdepth 5 | xargs grep -l "aws_eks_cluster" 2>/dev/null | grep -q .; then
  set_cloud eks; append_hint "terraform: aws_eks_cluster found"
fi

# --- kubeconfig contexts ---
if command -v kubectl &>/dev/null && kubectl config get-contexts 2>/dev/null | grep -q "gke_"; then
  set_cloud gke; append_hint "kubeconfig: gke_ context detected"
fi
if command -v kubectl &>/dev/null && kubectl config get-contexts 2>/dev/null | grep -qE "aks|azmk8s"; then
  set_cloud aks; append_hint "kubeconfig: AKS context detected"
fi
if command -v kubectl &>/dev/null && kubectl config get-contexts 2>/dev/null | grep -q "eks"; then
  set_cloud eks; append_hint "kubeconfig: EKS context detected"
fi

# --- existing GitHub Actions ---
for wf in .github/workflows/*.yml .github/workflows/*.yaml; do
  [ -f "$wf" ] || continue
  if grep -q "pkg.dev" "$wf"; then add_registry "artifact-registry"; append_hint "workflow: Artifact Registry push detected in $wf"; fi
  if grep -q "azurecr.io" "$wf"; then add_registry "acr"; append_hint "workflow: ACR push detected in $wf"; fi
  if grep -q "dkr.ecr" "$wf"; then add_registry "ecr"; append_hint "workflow: ECR push detected in $wf"; fi
  if grep -q "google-github-actions" "$wf" || grep -q "gcloud" "$wf"; then set_cloud gke; fi
  if grep -q "azure/login\|az aks\|azurerm" "$wf"; then set_cloud aks; fi
  if grep -q "aws-actions\|aws eks\|eksctl" "$wf"; then set_cloud eks; fi
done

# --- environment variables ---
[ -n "${GOOGLE_PROJECT:-}" ] && { set_cloud gke; append_hint "env: GOOGLE_PROJECT set"; }
[ -n "${AZURE_SUBSCRIPTION_ID:-}" ] && { set_cloud aks; append_hint "env: AZURE_SUBSCRIPTION_ID set"; }
[ -n "${AWS_ACCOUNT_ID:-}${AWS_DEFAULT_REGION:-}" ] && { set_cloud eks; append_hint "env: AWS env vars set"; }

echo "$result" | jq .
