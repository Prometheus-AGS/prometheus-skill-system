# Prometheus deployment catalog

This directory is the reproducible build and workload-packaging authority for the
KnowMe reference stack. It follows TJ-CICD-001:

- Terraform/OpenTofu owns clusters, registries, networking, and cloud IAM.
- ArgoCD owns live resources inside clusters.
- GitHub Actions builds images and promotes immutable digests to a separate GitOps
  repository. It never deploys directly.

## Catalog files

- `sources.lock.yaml` pins every first-party source repository to a full commit.
- `third-party.lock.yaml` pins externally supplied images and source archives.
- `images.lock.yaml` is the release-approved digest interface used by Compose and
  Kustomize.
- `docker-bake.hcl` is the single Buildx entry point.
- `compose.yaml` provides `core`, `realtime`, `authenticated`, and `full-agentic`
  profiles using published images. `compose.dev.yaml` is the explicit local-source
  override.
- `gitops/` contains reusable, non-live Kustomize templates. Environment-specific
  values belong in the configured external GitOps repository.

## Build

```bash
node deploy/scripts/validate-catalog.mjs
docker buildx bake -f deploy/docker-bake.hcl --print
docker buildx bake -f deploy/docker-bake.hcl validate
```

Private Git contexts use BuildKit pre-flight authentication:

```bash
export BUILDX_BAKE_GIT_AUTH_TOKEN="$(gh auth token)"
docker buildx bake -f deploy/docker-bake.hcl publish
```

Never pass repository credentials through `ARG`, `ENV`, or a layer-producing
`RUN` instruction.

## Edge routing and certificates

`gitops/components/edge/` provides mutually exclusive examples for:

- F5 NGINX Ingress Controller (`Ingress`)
- NGINX Gateway Fabric (`Gateway` / `HTTPRoute`)
- Traefik Kubernetes Ingress
- Traefik Gateway API
- Envoy Gateway

The retired community `kubernetes/ingress-nginx` controller is documented only as
a migration example and must not be selected for a new production deployment. Its
upstream maintenance ended in March 2026.

Certificate components include:

- cert-manager HTTP-01 for individual hostnames
- cert-manager DNS-01 for wildcard certificates
- an external Certbot DNS-01 workflow for operators who intentionally manage the
  TLS secret outside cert-manager

Wildcard issuance always uses DNS-01. HTTP-01 cannot issue wildcard certificates.

## Validation

```bash
node deploy/scripts/validate-catalog.mjs
node deploy/scripts/validate-gitops.mjs
docker compose -f deploy/compose.yaml config
```

The validators reject floating source revisions, `latest` images, inline secret
values, direct deployment commands in workflows, and Kustomize layer violations.
