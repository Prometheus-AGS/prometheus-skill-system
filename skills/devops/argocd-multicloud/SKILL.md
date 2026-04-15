---
name: argocd-multicloud
description: >
  Installs, configures, and manages ArgoCD as a multi-cloud GitOps control plane on GKE,
  with AKS and EKS registered as remote cluster destinations. Covers: fresh ArgoCD
  installation with --insecure mode behind Envoy Gateway, wildcard TLS certificate setup,
  cluster registration via argocd CLI, App-of-Apps root application creation, ArgoCD
  ApplicationSet authoring for cross-cluster fan-out delivery, ArgoCD project isolation
  per client/environment, RBAC configuration, and Dex OIDC SSO wiring. Use when setting
  up ArgoCD for the first time or adding remote clusters to an existing control plane.
license: MIT
compatibility: >
  Requires: kubectl (pointed at GKE cluster), argocd CLI v2.10+, helm 3+, bash 5+, jq.
  Target: GKE as ArgoCD host. AKS and EKS as remote destinations.
  Network: ArgoCD control plane must have API server reachability to AKS and EKS clusters.
  Supported agents: Claude Code, Antigravity, Codex, OpenCode, Gemini CLI, Roo Code.
metadata:
  standard: TJ-CICD-001 v1.1
  owner: Prometheus AGS
  contact: tjames@prometheusags.ai
  version: "1.0.0"
allowed-tools: Bash(kubectl:) Bash(argocd:) Bash(helm:) Bash(jq:) Bash(curl:) Read Write
---

# ArgoCD Multi-Cloud Skill

Establishes ArgoCD on GKE as the single control plane for all cluster environments.
AKS and EKS are registered as remote destinations — ArgoCD does NOT run on those clusters.

## Invocation

Use this skill when the user says any of:
- "install ArgoCD"
- "set up ArgoCD"
- "register AKS/EKS cluster with ArgoCD"
- "create App-of-Apps"
- "configure ArgoCD ApplicationSet"
- "add cluster to ArgoCD"
- "set up ArgoCD RBAC"
- "wire SSO to ArgoCD"

## Step 1 — Installation

Detect the latest stable ArgoCD version and install using server-side apply (handles large CRDs):

```bash
ARGOCD_VERSION=$(curl -s https://api.github.com/repos/argoproj/argo-cd/releases/latest \
  | jq -r .tag_name)
kubectl create namespace argocd --dry-run=client -o yaml | kubectl apply -f -
kubectl apply -n argocd \
  -f https://raw.githubusercontent.com/argoproj/argo-cd/${ARGOCD_VERSION}/manifests/install.yaml
# Server-side apply handles applicationsets CRD annotation size limit:
curl -sL https://raw.githubusercontent.com/argoproj/argo-cd/${ARGOCD_VERSION}/manifests/install.yaml \
  | kubectl apply --server-side --force-conflicts -n argocd -f -
```

Patch argocd-server to --insecure mode (TLS terminates at Envoy Gateway):
```bash
kubectl patch deployment argocd-server -n argocd --type='json' \
  -p='[{"op":"add","path":"/spec/template/spec/containers/0/args/-","value":"--insecure"}]'
```

Patch argocd-cm with the public URL:
```bash
kubectl patch configmap argocd-cm -n argocd --type merge \
  -p '{"data":{"url":"https://argocd.{domain}"}}'
```

## Step 2 — TLS and Gateway

If a wildcard TLS secret exists in another namespace, copy it to argocd:
```bash
kubectl get secret {wildcard-secret} -n {source-ns} -o json \
  | jq 'del(.metadata.namespace,.metadata.resourceVersion,.metadata.uid,
            .metadata.creationTimestamp,
            .metadata.annotations."kubectl.kubernetes.io/last-applied-configuration")
        | .metadata.name = "wildcard-tls"
        | .metadata.namespace = "argocd"' \
  | kubectl apply -f -
```

Generate Envoy Gateway Gateway + HTTPRoute + GRPCRoute from `assets/templates/argocd-gateway.yaml`.
Check Gateway API version to use correct GRPCRoute API group (v1 vs v1alpha2).

## Step 3 — Remote Cluster Registration

For each non-GKE cluster, fetch kubeconfig and register:

```bash
# AKS
az aks get-credentials --resource-group {rg} --name {cluster} --file /tmp/aks-kubeconfig
argocd cluster add {context-name} \
  --kubeconfig /tmp/aks-kubeconfig \
  --name aks-prod \
  --system-namespace argocd \
  --in-cluster=false

# EKS
aws eks update-kubeconfig --name {cluster} --region {region} --kubeconfig /tmp/eks-kubeconfig
argocd cluster add {context-name} \
  --kubeconfig /tmp/eks-kubeconfig \
  --name eks-prod \
  --system-namespace argocd \
  --in-cluster=false

argocd cluster list  # verify all three show as Successful
```

## Step 4 — App-of-Apps Bootstrap

Create the root Application CR pointing at the gitops/apps/ directory.
Generate from `assets/templates/root-app.yaml`, substituting the GitOps repo URL.
Apply to the cluster — this creates all child Application CRs automatically.

## Step 5 — ApplicationSet (Optional)

If the user wants fan-out delivery across all clusters for a service, generate an
ApplicationSet using the list generator pattern from `references/APPLICATIONSET.md`.

The ApplicationSet template path `services/{service}/overlays/{{.cloud}}-{{.env}}` must
match the Kustomize overlay structure created by the gitops-bootstrap skill.

## Step 6 — RBAC

Generate `argocd-rbac-cm` policy for the standard role set:
- `role:admin` — full access (Prometheus AGS operators only)
- `role:developer` — sync, get application; no delete, no cluster management
- `role:readonly` — get only; for client stakeholders

## Step 7 — Dex SSO (Recommended)

Generate `dex.config` for the argocd-cm ConfigMap. Supported connectors:
- Google Workspace (OAuth2)
- GitHub (OAuth2)
- Microsoft Entra ID (OIDC)

After SSO is configured, disable local admin:
```bash
kubectl patch configmap argocd-cm -n argocd --type merge \
  -p '{"data":{"admin.enabled":"false"}}'
```

## Step 8 — Initial Login and Credential Rotation

Output initial admin password retrieval command and remind user to:
1. Log in immediately
2. Change the password
3. Delete the `argocd-initial-admin-secret`

```bash
kubectl -n argocd get secret argocd-initial-admin-secret \
  -o jsonpath="{.data.password}" | base64 -d
argocd login {domain} --username admin
argocd account update-password
kubectl -n argocd delete secret argocd-initial-admin-secret
```

## Key Rules

- ArgoCD runs on GKE only — never install on AKS or EKS as separate instances
- All Application CRs have `selfHeal: true` and `prune: true`
- Remote cluster credentials are managed by Terraform, not manually
- SSO must be enabled before admin login is disabled
