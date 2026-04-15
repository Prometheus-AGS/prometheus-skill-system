#!/usr/bin/env bash
# register-clusters.sh — Registers AKS and EKS clusters as ArgoCD remote destinations
set -euo pipefail

echo "→ Verifying ArgoCD CLI is authenticated..."
argocd account get-user-info --grpc-web 2>/dev/null || { echo "ERROR: not logged in. Run: argocd login {host}"; exit 1; }

register_aks() {
  local rg="$1" cluster="$2" name="${3:-aks-prod}"
  echo "→ Fetching AKS kubeconfig for $cluster..."
  az aks get-credentials --resource-group "$rg" --name "$cluster" --file /tmp/aks-kubeconfig --overwrite-existing
  argocd cluster add "$cluster" --kubeconfig /tmp/aks-kubeconfig --name "$name" \
    --system-namespace argocd --in-cluster=false --yes
  echo "✓ AKS cluster $cluster registered as $name"
}

register_eks() {
  local region="$1" cluster="$2" name="${3:-eks-prod}"
  echo "→ Fetching EKS kubeconfig for $cluster..."
  aws eks update-kubeconfig --name "$cluster" --region "$region" --kubeconfig /tmp/eks-kubeconfig
  local ctx; ctx=$(kubectl --kubeconfig /tmp/eks-kubeconfig config current-context)
  argocd cluster add "$ctx" --kubeconfig /tmp/eks-kubeconfig --name "$name" \
    --system-namespace argocd --in-cluster=false --yes
  echo "✓ EKS cluster $cluster registered as $name"
}

# Invocation examples — called by the skill with detected parameters:
# register_aks "prometheus-rg" "prometheus-aks-prod" "aks-prod"
# register_eks "us-east-1" "prometheus-eks-prod" "eks-prod"

echo "→ Current cluster list:"
argocd cluster list
