---
name: gitops-bootstrap
description: >
  Scaffolds a complete multi-cloud GitOps CI/CD system from scratch following the TJ-CICD-001
  standard. Detects target cloud environments (GKE, AKS, EKS — any combination), creates the
  GitOps repository structure (base / cloud / environment Kustomize overlays), generates GitHub
  Actions workflows with keyless OIDC authentication per cloud, creates ArgoCD Application CRs
  or ApplicationSets, and registers remote clusters with an ArgoCD control plane on GKE. Use
  when a project has no existing GitOps pipeline and needs one built from the ground up.
license: MIT
compatibility: >
  Requires: git, kubectl, kustomize, argocd CLI, bash 5+, jq.
  Optional: terraform (for cluster registration automation), gh CLI (for repo creation).
  Supported agents: Claude Code, Antigravity, Codex, OpenCode, Gemini CLI, Roo Code, Windsurf,
  Cursor, Amp, Cline. Discovery paths: .claude/skills/ (Claude Code), .agents/skills/ (Codex,
  OpenCode, Antigravity, Amp), .gemini/ (Gemini CLI), .roo/skills/ (Roo Code).
metadata:
  standard: TJ-CICD-001 v1.1
  owner: Prometheus AGS
  contact: tjames@prometheusags.ai
  registry-url: https://github.com/prometheusags/prometheus-gitops-skills
  version: '1.0.0'
allowed-tools: Bash(git:) Bash(kubectl:) Bash(kustomize:) Bash(argocd:) Bash(jq:) Bash(find:) Bash(mkdir:) Bash(cp:) Read Write
---

# GitOps Bootstrap Skill

This skill builds a complete multi-cloud GitOps CI/CD system from scratch. It enforces the
**TJ-CICD-001 three-layer ownership model**:

- **Layer 1 — Infrastructure**: Terraform / OpenTofu owns clusters, registries, IAM
- **Layer 2 — Manifests**: ArgoCD owns everything inside every cluster
- **Layer 3 — Build & Promote**: GitHub Actions builds images and writes image tags only

The skill never uses `kubectl apply` or `helm upgrade` as deployment mechanisms. All workload
delivery flows through a Git commit to the GitOps repository.

## Invocation

Use this skill when the user says any of:

- "set up GitOps for this project"
- "create a CI/CD pipeline"
- "bootstrap ArgoCD deployment"
- "set up GitHub Actions for Kubernetes"
- "create gitops structure"
- "set up deployment pipeline from scratch"

## Execution Workflow

### Step 1 — Detect Target Clouds

Run the detection script to identify which clouds are in scope:

```
scripts/detect-cloud.sh
```

The script examines:

- Terraform files (`.tf`) for `google_container_cluster`, `azurerm_kubernetes_cluster`, `aws_eks_cluster`
- `~/.kube/config` contexts for `gke_`, `aks-`, `eks-` prefixes
- Environment variables: `GOOGLE_PROJECT`, `AZURE_SUBSCRIPTION_ID`, `AWS_ACCOUNT_ID`
- Existing GitHub Actions workflows for registry push patterns
- `.env` files and `docker-compose.yml` for cloud hints

Output a detection summary before proceeding. Ask the user to confirm the detected clouds or
specify which clouds to target if detection is ambiguous.

### Step 2 — GitOps Repository Structure

Create the following directory structure in the current project or a specified path:

```
gitops/
├── apps/                              # ArgoCD Application CRs
│   └── {service-name}/
│       ├── gke-prod.yaml              # one per detected cloud/env pair
│       ├── aks-prod.yaml
│       └── eks-prod.yaml
├── services/
│   └── {service-name}/
│       ├── base/                      # cloud-agnostic manifests
│       │   ├── deployment.yaml
│       │   ├── service.yaml
│       │   ├── serviceaccount.yaml
│       │   └── kustomization.yaml
│       ├── cloud/                     # platform-specific patches
│       │   ├── gke/kustomization.yaml
│       │   ├── aks/kustomization.yaml
│       │   └── eks/kustomization.yaml
│       └── overlays/                  # final env overlays
│           ├── gke-staging/kustomization.yaml
│           ├── gke-prod/kustomization.yaml
│           ├── aks-staging/kustomization.yaml
│           ├── aks-prod/kustomization.yaml
│           ├── eks-staging/kustomization.yaml
│           └── eks-prod/kustomization.yaml
├── platform/                          # third-party apps (ArgoCD Helm)
│   ├── cert-manager/argocd-app.yaml
│   ├── envoy-gateway/argocd-app.yaml
│   └── external-secrets/argocd-app.yaml
├── clusters/                          # ArgoCD cluster registration secrets
│   ├── gke-prod-secret.yaml
│   ├── aks-prod-secret.yaml
│   └── eks-prod-secret.yaml
└── config/
    ├── namespaces.yaml
    └── network-policies.yaml
```

Use the templates in `assets/templates/` for all generated files.

### Step 3 — Base Manifests

Generate cloud-agnostic base manifests for the service. Prompt the user for:

- Service name (kebab-case)
- Container port
- Minimum replicas
- Any environment variables needed (names only — values come from ExternalSecret CRs)

Generate `base/deployment.yaml` with:

- No hardcoded image tag (placeholder for Kustomize `newTag`)
- `serviceAccountName` referencing the service's SA
- No cloud-specific annotations
- `livenessProbe` and `readinessProbe` stubs

Generate `base/serviceaccount.yaml` with no annotations (cloud patches add those).

### Step 4 — Cloud Overlays

For each detected cloud, generate the cloud overlay. Use templates from `assets/templates/cloud/`.

**GKE overlay** (`cloud/gke/kustomization.yaml`):

- Patches ServiceAccount with `iam.gke.io/gcp-service-account` annotation
- References Artifact Registry image hostname

**AKS overlay** (`cloud/aks/kustomization.yaml`):

- Patches ServiceAccount with `azure.workload.identity/client-id` annotation
- Patches Deployment pod template with `azure.workload.identity/use: "true"` label
- References ACR image hostname

**EKS overlay** (`cloud/eks/kustomization.yaml`):

- Patches ServiceAccount with `eks.amazonaws.com/role-arn` annotation
- References ECR image hostname

### Step 5 — Environment Overlays

Generate staging and prod overlays for each cloud. Staging sets replicas=1. Prod sets
replicas=2 minimum with resource limits. Image tag is a placeholder — GitHub Actions writes
the real SHA tag on every deploy.

### Step 6 — GitHub Actions Workflows

Generate `.github/workflows/deploy.yml` in the **application repository** (not the GitOps repo).

For each detected cloud, generate the appropriate push job:

- **GCP**: `google-github-actions/auth@v2` with Workload Identity Federation
- **Azure**: `azure/login@v2` with federated credentials (OIDC, no client secret)
- **AWS**: `aws-actions/configure-aws-credentials@v4` with OIDC role assumption

The `promote` job depends on ALL push jobs completing successfully before writing to GitOps repo.

Commit message format: `deploy({service}): {sha} → staging [{clouds}]`

**NEVER** add `kubectl apply`, `helm upgrade`, or any direct cloud deploy command to workflows.

### Step 7 — ArgoCD Application CRs

For each cloud/environment pair, generate an Application CR in `apps/{service}/`.

If the service targets all detected clouds with identical structure, offer to generate an
**ApplicationSet** instead (see references/APPLICATIONSET.md).

The root App-of-Apps CR (`apps/root-app.yaml`) watches the entire `apps/` directory.

### Step 8 — Cluster Registration

If ArgoCD is accessible and remote clusters are detected, offer to register them:

```
scripts/register-clusters.sh
```

This script runs `argocd cluster add` for each non-GKE cluster using kubeconfig contexts.
If ArgoCD is not yet installed, output the installation command for the user to run first.

### Step 9 — Validation

Run validation checks:

1. `kustomize build` on every overlay — must produce valid manifests
2. Verify no Kubernetes Secret manifests contain data values (only ExternalSecret CRs)
3. Verify no workflow file contains `kubectl apply` or `helm upgrade` as deploy mechanism
4. Verify all image tags in overlays are placeholder strings, not `latest`

Report the validation summary. Flag any failures before completing.

## Output Summary

After completion, output:

1. List of all created files with paths
2. Environment variables the user must add to GitHub repository secrets
3. DNS records needed per cloud (one A record per Gateway LoadBalancer IP)
4. First-run commands to initialize ArgoCD App-of-Apps

## Key Rules — Never Violate

- Base manifests contain zero cloud-specific annotations
- Cloud overlays contain zero replica counts or resource limits
- Environment overlays contain zero cloud-specific annotations
- No Kubernetes Secret manifests with encoded values, anywhere
- GitHub Actions workflows write image tags only — no kubectl, no helm
- ArgoCD runs on GKE only — AKS and EKS are remote destinations

See `references/TJ-CICD-001.md` for the complete standard.

## Surreal-Memory Integration

When `surreal-memory` MCP server is available, the bootstrap process records:

- **Cluster entities** — one per detected cloud/env (type: `cluster`, observations: context name, cloud provider, region)
- **Service entities** — one per bootstrapped service (type: `service`, relation: `deployed_to` → cluster)
- **Pipeline entities** — GitHub Actions workflow metadata (type: `pipeline`, relation: `builds` → service)

This enables cross-session queries like "which services deploy to the EKS prod cluster?"
and feeds deployment context to iterative-evolver and KBD assessments.

Detection: check if `create_entity` tool is available. All surreal-memory
operations are optional — the skill works identically without it.
