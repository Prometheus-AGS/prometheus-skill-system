---
name: gitops-transform
description: >
  Detects and transforms existing GitHub Actions workflows and Kubernetes deployment
  configurations to the TJ-CICD-001 multi-cloud GitOps standard. Analyzes the current
  CI stack (registry provider, auth mechanism, deploy strategy, target cloud) and produces
  a diff-based transformation plan before making any changes. Handles: raw kubectl deploy
  workflows, Helm-based deploys, single-cloud workflows missing multi-cloud support,
  static credential auth (service account keys, access key IDs) needing OIDC migration,
  and workflows that deploy directly instead of committing to a GitOps repo. Use when
  a project has existing CI/CD that needs upgrading or modernizing.
license: MIT
compatibility: >
  Requires: git, bash 5+, jq, python3 (for workflow AST analysis), yq or python-yaml.
  Optional: kubectl, kustomize (for manifest validation).
  Supported agents: Claude Code, Antigravity, Codex, OpenCode, Gemini CLI, Roo Code,
  Windsurf, Cursor, Amp, Cline.
metadata:
  standard: TJ-CICD-001 v1.1
  owner: Prometheus AGS
  contact: tjames@prometheusags.ai
  version: '1.0.0'
allowed-tools: Bash(find:) Bash(grep:) Bash(git:) Bash(python3:) Bash(jq:) Bash(yq:) Read Write
---

# GitOps Transform Skill

This skill audits an existing CI/CD setup and transforms it to the TJ-CICD-001 standard.
It always produces a **detection report** and **transformation plan** before touching any file.
The user approves the plan before execution.

## Invocation

Use this skill when the user says any of:

- "transform our existing CI pipeline"
- "upgrade our GitHub Actions to GitOps"
- "migrate to ArgoCD from our current deploy setup"
- "fix our CI/CD to use proper GitOps"
- "convert our workflow to multi-cloud"
- "we use kubectl apply in our CI, fix it"
- "update our GitHub Actions for GKE/AKS/EKS"
- "modernize our deployment pipeline"

## Phase 1 — Detection

Run the full detection script before any other step:

```
scripts/detect-stack.sh
```

### What the detector examines

**GitHub Actions workflows** (`.github/workflows/*.yml`, `.github/workflows/*.yaml`):

Detect deploy mechanisms — flag each as COMPLIANT, VIOLATION, or UPGRADE-NEEDED:

- `kubectl apply` in workflow → **VIOLATION** (direct deploy bypasses GitOps)
- `helm upgrade` in workflow → **VIOLATION** (direct deploy bypasses GitOps)
- `docker push` with no subsequent GitOps commit → **VIOLATION** (image never reaches cluster)
- `kustomize edit set image` + `git commit` + `git push` to gitops repo → **COMPLIANT**
- ArgoCD API sync call from CI → **VIOLATION** (sync must be autonomous, not CI-triggered)

Detect authentication patterns — flag each:

- `gcloud auth activate-service-account --key-file` → **VIOLATION** (static key)
- `GOOGLE_APPLICATION_CREDENTIALS` env var set to JSON → **VIOLATION** (static key)
- `aws-actions/configure-aws-credentials` with `aws-access-key-id` → **VIOLATION** (static keys)
- `azure/login` with `creds: ${{ secrets.AZURE_CREDENTIALS }}` (JSON blob) → **VIOLATION** (static)
- `google-github-actions/auth` with `workload_identity_provider` → **COMPLIANT** (keyless)
- `aws-actions/configure-aws-credentials` with `role-to-assume` → **COMPLIANT** (OIDC)
- `azure/login` with `client-id` + `tenant-id` + `subscription-id` → **COMPLIANT** (federated)

Detect registry targets:

- `docker push` to Docker Hub → **NOTE** (should use cloud-native registry)
- `docker push` to `ghcr.io` → **NOTE** (GHCR acceptable for non-production)
- `docker push` to `*.pkg.dev` → **GCP** (Artifact Registry)
- `docker push` to `*.azurecr.io` → **AZURE** (ACR)
- `docker push` to `*.dkr.ecr.*.amazonaws.com` → **AWS** (ECR)

Detect target clouds from workflow content and repository structure:

- GKE: `gcloud container clusters`, `google-github-actions/`, `gke_` kubeconfig contexts
- AKS: `azure/aks-set-context`, `azurerm_kubernetes_cluster`, `az aks`
- EKS: `aws eks`, `eks.amazonaws.com`, `aws_eks_cluster`

**Kubernetes manifests** (any `.yaml` in the repo):

- Kubernetes Secret with `data:` or `stringData:` containing values → **VIOLATION**
- Deployment image tag of `latest` in a manifest committed to git → **WARNING**
- Missing `serviceAccount` in Deployment → **WARNING**

**GitOps repository indicators**:

- Presence of Kustomize `kustomization.yaml` files → detect overlay structure
- Presence of ArgoCD `Application` CRs → detect existing ArgoCD setup
- Presence of Helm `values.yaml` files → detect Helm-based delivery

### Detection report format

Output a structured detection report:

```
═══════════════════════════════════════════════════
GITOPS TRANSFORM — DETECTION REPORT
═══════════════════════════════════════════════════

DETECTED CLOUDS
  GKE  ✓  (found in: .github/workflows/deploy.yml:14)
  AKS  ✗  (not detected)
  EKS  ✗  (not detected)

VIOLATIONS (must fix)
  [V1] kubectl apply in workflow    .github/workflows/deploy.yml:47
  [V2] Static GCP service account key   secrets.GCP_SA_KEY
  [V3] Docker Hub push (use Artifact Registry)  .github/workflows/deploy.yml:32

WARNINGS (should fix)
  [W1] Image tag 'latest' in manifest   k8s/deployment.yaml:18
  [W2] No Kustomize overlay structure detected

COMPLIANT
  [C1] Tests run before deploy step
  [C2] Build and deploy are separate jobs

EXISTING GITOPS SETUP
  ArgoCD: not detected
  Kustomize: not detected
  GitOps repo: not detected
═══════════════════════════════════════════════════
```

## Phase 2 — Transformation Plan

After detection, produce a transformation plan listing every change to be made.
Present the plan to the user and wait for approval before executing.

Plan format:

```
TRANSFORMATION PLAN
═══════════════════════════════════════════════════
ACTION 1: Replace kubectl apply with GitOps promotion
  File: .github/workflows/deploy.yml
  Remove: lines 44-50 (kubectl apply -f k8s/)
  Replace with: Kustomize image tag update + git push to gitops repo

ACTION 2: Migrate GCP auth to Workload Identity Federation
  File: .github/workflows/deploy.yml
  Remove: google-github-actions/auth with key-file
  Replace with: google-github-actions/auth@v2 with workload_identity_provider
  Requires: Add WIF_PROVIDER and WIF_SA to GitHub secrets (instructions provided)

ACTION 3: Create GitOps repository structure
  Creates: gitops/services/{service}/base/ + cloud/gke/ + overlays/gke-{staging,prod}/
  Creates: gitops/apps/{service}/gke-prod.yaml (ArgoCD Application CR)

ACTION 4: Migrate registry from Docker Hub to Artifact Registry
  File: .github/workflows/deploy.yml
  Replace: docker push {user}/{image}:tag
  Replace with: docker push us-east5-docker.pkg.dev/{project}/{image}:{sha}
  Requires: Create Artifact Registry repository (Terraform resource provided)

ESTIMATED SCOPE: 4 files modified, 8 files created
APPROVAL REQUIRED before execution.
═══════════════════════════════════════════════════
```

## Phase 3 — Execution

After user approval, execute each action in the plan:

### Workflow transformation rules

**Replacing direct kubectl deploy:**

```yaml
# REMOVE this pattern:
- run: kubectl apply -f k8s/

# REPLACE with GitOps promotion:
- name: Checkout GitOps repo
  uses: actions/checkout@v4
  with:
    repository: ${{ env.GITOPS_REPO }}
    token: ${{ secrets.GITOPS_PAT }}
    path: gitops

- name: Update image tag
  run: |
    cd gitops/services/$SERVICE/overlays/gke-staging
    kustomize edit set image $SERVICE=$REGISTRY/$SERVICE:${{ github.sha }}
    git config user.name "github-actions[bot]"
    git config user.email "github-actions[bot]@users.noreply.github.com"
    git add .
    git commit -m "deploy($SERVICE): ${{ github.sha }} → staging [gke]"
    git push
```

**Migrating GCP static key to WIF:**

```yaml
# REMOVE:
- uses: google-github-actions/auth@v2
  with:
    credentials_json: ${{ secrets.GCP_SA_KEY }}

# REPLACE with:
- uses: google-github-actions/auth@v2
  with:
    workload_identity_provider: ${{ secrets.WIF_PROVIDER }}
    service_account: ${{ secrets.WIF_SA }}
```

**Migrating AWS static keys to OIDC:**

```yaml
# REMOVE:
- uses: aws-actions/configure-aws-credentials@v4
  with:
    aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
    aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}

# REPLACE with:
- uses: aws-actions/configure-aws-credentials@v4
  with:
    role-to-assume: ${{ secrets.AWS_ROLE_ARN }}
    aws-region: us-east-1
```

**Adding multi-cloud support to single-cloud workflow:**
If only GKE is detected but the user wants AKS and/or EKS, add parallel push jobs
following the pattern in `references/MULTICLOUD-WORKFLOW.md`.

### GitOps structure creation

If no GitOps structure exists, invoke `gitops-bootstrap` skill logic to create it,
pre-populated with the service name and cloud(s) detected in Phase 1.

### Manifest remediation

- Remove any committed `Secret` manifests containing encoded values
- Replace with `ExternalSecret` CR stubs pointing to the appropriate cloud secret store
- Change `latest` image tags to `$(PLACEHOLDER)` (Kustomize will replace at deploy time)

## Phase 4 — Verification

After transformation:

1. `git diff --stat` to show all changed files
2. `kustomize build` on each overlay — must succeed
3. YAML lint on all modified workflow files
4. Grep for remaining violations: `kubectl apply`, `helm upgrade`, static credential patterns
5. Report: "X violations resolved, Y warnings remain, Z items require manual action"

Output a list of GitHub secrets the user must create and the commands to create ArgoCD
Application CRs for any newly created GitOps structure.

## Secrets Migration Checklist

For each removed static credential, output the replacement setup:

**GCP (remove `GCP_SA_KEY`, add):**

- `WIF_PROVIDER` — Workload Identity Pool provider resource name
- `WIF_SA` — Service account email with Workload Identity User binding
- `GITOPS_PAT` — Fine-grained GitHub token with write access to GitOps repo only

**AWS (remove `AWS_ACCESS_KEY_ID` + `AWS_SECRET_ACCESS_KEY`, add):**

- `AWS_ROLE_ARN` — IAM role ARN configured with OIDC trust for GitHub Actions
- `GITOPS_PAT` — Fine-grained GitHub token

**Azure (remove `AZURE_CREDENTIALS` JSON blob, add):**

- `AZURE_CLIENT_ID` — App registration client ID
- `AZURE_TENANT_ID` — Azure AD tenant ID
- `AZURE_SUBSCRIPTION_ID` — Subscription ID
- `GITOPS_PAT` — Fine-grained GitHub token

See `references/SECRETS-MIGRATION.md` for detailed setup instructions per cloud.
