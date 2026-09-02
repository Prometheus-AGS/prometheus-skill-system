---
id: integration-contract
title: Integration Contract
sidebar_label: Integration Contract
---

# Integration Contract

The skill pack is an open-source **skill collection**, and it is complete on its
own: every skill works with nothing else installed. Extensions add capability
the pack does not provide — cross-machine sync, a supervisor that guarantees
services are running, a remote control plane — and they live in their own
repositories.

This page is the operator-facing summary. The normative document is
[`docs/integration-contract.md`](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/docs/integration-contract.md)
in the repository, and it is authoritative where the two differ.

**Contract version: 1.0.0**

## The two rules

1. **The pack never depends on an extension.** No build-time dependency, no
   install-time requirement, no runtime assumption. Extensions depend on the
   pack, never the reverse.
2. **Capability is discovered, never assumed.** When an extension is present the
   pack uses it. When it is absent the pack behaves exactly as it does today and
   emits no warning, no degraded-mode notice, and no error.

## The four seams

| Seam | What an extension gets | Where it lives |
|---|---|---|
| 1. Control-endpoint discovery | A fixed resolution order the pack already uses for every `prometheus kbd` command | `PROMETHEUS_CONTROL_ENDPOINT`, then `SOVEREIGN_SYNC_SOCKET`, then the default socket under the platform data directory, then TCP on non-Unix |
| 2. Hook bundle extension points | Ship verified bundles resolved through `run-hook --bundle`, namespaced by package | `hooks/hooks.json` entries are pack-owned and generated; extensions register their own bundles |
| 3. Service manifest | A generated, drift-checked projection of the pack's service templates, so a supervisor can adopt services where they run | `shared/services.manifest.json`, from `shared/launchagents/*.plist` and `shared/systemd/*` |
| 4. Connected skill package | Declare skills, hook bundles, MCP servers, and a minimum contract version | `skill-package.json`, validated against `shared/schemas/skill-package.schema.json` |

## Reading the contract

```bash
prometheus contract show --json
```

```json
{
  "contract_version": "1.0.0",
  "endpoint": null,
  "endpoint_source": "absent",
  "service_manifest": "shared/services.manifest.json"
}
```

`endpoint: null` with source `absent` is the normal case on a machine with no
extension installed. The command exits 0 and writes nothing to stderr.

## Validating an extension

```bash
prometheus contract validate path/to/skill-package.json
```

A declaration requiring a contract version newer than the pack implements is
refused, and the failure names both versions.

## Regenerating the service manifest

The plists and units are generator inputs. Any change that edits one of them
must regenerate the manifest in the same change (constraint C-01):

```bash
npm run generate:services-manifest
npm run check:services-manifest
```

## Versioning

`MAJOR.MINOR.PATCH`, versioned independently of the pack's package version.
Minor releases are additive and backward compatible; a major release is a
breaking change to a seam. Three identifiers are preserved by name in v1 for CLI
compatibility: `sovereign-sync.sock`, `SOVEREIGN_SYNC_SOCKET`, and the device
key path under `sovereign-sync/`. They are contract surface, not a statement
about which software is listening.
