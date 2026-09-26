# Reusable GitOps workload templates

These manifests are inputs for a separate ArgoCD GitOps repository. They are not
live environment state and must not be applied from this repository's CI.

The `knowme-stack` service demonstrates the required three-dimensional layering:

- `base/`: cloud-neutral resources
- `cloud/<provider>/`: workload identity and secret-store integration
- `overlays/<provider>-<environment>/`: image digest, replicas, and resources

Choose exactly one component from `components/edge/`. NGINX Ingress, NGINX
Gateway Fabric, Traefik Ingress, Traefik Gateway API, and Envoy Gateway examples
all route to the same `knowme-web` Service.

Certificate examples live under `components/certificates/`. Use HTTP-01 for
individual hostnames and DNS-01 for wildcards. See `docs/deployment/edge-routing-and-tls.md`
for controller installation and Certbot workflows.
