---
name: kustomize-overlay
description: >
  Generates a complete three-dimensional Kustomize overlay structure (base / cloud / environment)
  for a new or existing service following TJ-CICD-001. Creates cloud-specific annotation patches
  for GKE Workload Identity, Azure Workload Identity, and EKS IRSA. Validates existing overlays
  for structural compliance and repair broken overlay chains. Use when adding a new service to
  the GitOps repository, adding a new cloud target to an existing service, or auditing and
  repairing existing Kustomize structures.
license: MIT
compatibility: >
  Requires: kustomize 5+, bash 5+, kubectl (for manifest validation).
  Supported agents: Claude Code, Antigravity, Codex, OpenCode, Gemini CLI, Roo Code, Windsurf.
metadata:
  standard: TJ-CICD-001 v1.1
  owner: Prometheus AGS
  contact: tjames@prometheusags.ai
  version: '1.0.0'
allowed-tools: Bash(kustomize:) Bash(kubectl:) Bash(find:) Bash(mkdir:) Read Write
---

# Kustomize Overlay Skill

Generates, validates, and repairs Kustomize overlay structures for multi-cloud services.

## Invocation

Use when the user says any of:

- "add Kustomize overlays"
- "create the Kustomize structure"
- "scaffold overlays for this service"
- "add AKS/EKS overlay to existing service"
- "validate our Kustomize structure"
- "fix our overlay structure"

## Overlay Architecture

Three-dimensional model — never mix dimensions:

```
base/           → cloud-agnostic: Deployment, Service, SA, HPA, NetworkPolicy
cloud/{cloud}/  → cloud-specific: workload identity annotations ONLY
overlays/{cloud}-{env}/  → environment: replicas, resource limits, image tag ONLY
```

**The invariant:**

- `base/` has no cloud annotations and no replica counts
- `cloud/*/` has no replica counts and no resource limits
- `overlays/*/` has no cloud annotations

Violating this invariant will cause the manifest review to fail.

## Cloud Patch Templates

### GKE Workload Identity

```yaml
# cloud/gke/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - ../../base
patches:
  - target:
      kind: ServiceAccount
      name: { service }
    patch: |
      - op: add
        path: /metadata/annotations/iam.gke.io~1gcp-service-account
        value: "{service}@{project}.iam.gserviceaccount.com"
```

### Azure Workload Identity

```yaml
# cloud/aks/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - ../../base
patches:
  - target:
      kind: ServiceAccount
      name: { service }
    patch: |
      - op: add
        path: /metadata/annotations/azure.workload.identity~1client-id
        value: "{MANAGED_IDENTITY_CLIENT_ID}"
      - op: add
        path: /metadata/labels/azure.workload.identity~1use
        value: "true"
  - target:
      kind: Deployment
      name: { service }
    patch: |
      - op: add
        path: /spec/template/metadata/labels/azure.workload.identity~1use
        value: "true"
```

### EKS IRSA

```yaml
# cloud/eks/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - ../../base
patches:
  - target:
      kind: ServiceAccount
      name: { service }
    patch: |
      - op: add
        path: /metadata/annotations/eks.amazonaws.com~1role-arn
        value: "arn:aws:iam::{account}:role/{service}-irsa"
```

## Environment Overlay Template (prod)

```yaml
# overlays/gke-prod/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - ../../cloud/gke
images:
  - name: { service }
    newName: us-east5-docker.pkg.dev/{project}/{service}
    newTag: PLACEHOLDER # GitHub Actions writes the real SHA here
patches:
  - target:
      kind: Deployment
      name: { service }
    patch: |
      - op: replace
        path: /spec/replicas
        value: 3
      - op: add
        path: /spec/template/spec/containers/0/resources
        value:
          requests: {cpu: "100m", memory: "128Mi"}
          limits: {cpu: "500m", memory: "512Mi"}
commonLabels:
  environment: prod
  managed-by: argocd
```

## Validation

After generating or modifying any overlay, run:

```bash
kustomize build overlays/{cloud}-{env}
```

Every overlay must build without error. If it fails, diagnose and fix before reporting complete.

## Repair Mode

If invoked on an existing service with broken overlays, run:

1. `kustomize build` on each overlay — collect errors
2. Identify root cause: missing resource, wrong relative path, invalid patch
3. Propose fixes with diffs
4. Apply after user confirms
