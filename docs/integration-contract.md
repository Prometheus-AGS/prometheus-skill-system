# Prometheus Skill Pack — Integration Contract

**Contract version: 1.0.0**
Status: stable
Audience: anyone extending this pack from another repository — the Prometheus
Companion, and third parties on equal terms.

## What this contract is for

`prometheus-skill-pack` is an open-source **skill collection**. It ships skills,
hooks, a CLI, and the services those skills need, and it is complete on its own:
every skill works with nothing else installed.

Extensions add capability the pack does not provide — cross-machine sync, a
supervisor that guarantees services are running, remote control planes, paid
products. An extension lives in **its own repository** and integrates through the
four seams below. Nothing in this contract requires the pack to know that any
particular extension exists.

Two rules make that work, and they are not negotiable:

1. **The pack never depends on an extension.** No build-time dependency, no
   install-time requirement, no runtime assumption. Extensions depend on the
   pack, never the reverse.
2. **Capability is discovered, never assumed.** When an extension is present the
   pack uses it. When it is absent the pack behaves exactly as it does today and
   emits **no warning, no degraded-mode notice, and no error**. Absence is the
   normal case, not a fault.

## Versioning

This contract is versioned independently of the pack's package version.

- The version is `MAJOR.MINOR.PATCH` and is reported by `prometheus contract show --json`.
- **PATCH**: editorial only. No behavioural change.
- **MINOR**: additive and backward compatible. New optional fields, new seams,
  new discovery sources. An extension written against an earlier minor version
  keeps working.
- **MAJOR**: a breaking change to any seam below. Extensions declaring a lower
  major version are refused by `prometheus contract validate` with both versions
  named.

An extension declares the minimum contract version it needs in its
`skill-package.json` (seam 4). The pack refuses a declaration whose requirement
exceeds the contract version it implements, and says so with both numbers.

---

## Seam 1 — Control-endpoint discovery

An extension may host a control plane. The pack finds it, or finds nothing, using
one fixed order. This is implemented in
`tools/prometheus-cli/crates/prometheus-cli/src/commands/control_transport.rs`
and is the same chain every `prometheus kbd` command already uses.

| Order | Source | Meaning |
|---|---|---|
| 1 | `PROMETHEUS_CONTROL_ENDPOINT` | An explicit HTTP base URL. Trailing slashes are trimmed. Wins over everything. |
| 2 | `SOVEREIGN_SYNC_SOCKET` | An explicit Unix socket path. An operator contract: it is honoured even while a supervised process is between unlink and bind during a restart, so a connect failure is reported as unreachable rather than silently falling back. |
| 3 | `<data_local_dir>/prometheus/run/sovereign-sync.sock` | The default Unix socket. On macOS `data_local_dir` is `~/Library/Application Support`; on Linux it follows XDG. |
| 4 | `http://127.0.0.1:7892` | Non-Unix platforms only, and the historical TCP default. |

Three identifiers in that table are **kept by name in contract v1** for CLI
compatibility even though the daemon that introduced them may not be what
answers: `sovereign-sync.sock`, `SOVEREIGN_SYNC_SOCKET`, and the device-key path
`<config_dir>/sovereign-sync/device-key.json`. They are contract surface, not a
statement about which software is listening. Renaming them is a MAJOR change.

**Silence when absent.** `prometheus contract show --json` reports
`"endpoint": null` with `"endpoint_source": "absent"`, exits 0, and writes
nothing to stderr. No pack command may warn, log at error level, or degrade its
output because no control endpoint was found.

An extension that hosts a control plane binds one of these targets. It does not
ask the pack to change.

---

## Seam 2 — Hook bundle extension points

The pack's hooks (`hooks/hooks.json`) do not run scripts directly. Every entry
invokes a **verified immutable bundle** through the runtime resolver:

```
$HOME/.prometheus/plugins/prometheus-skill-pack/runtime/v1/run-hook --bundle <name> [--resolve-only]
```

If the resolver is missing or cannot resolve the bundle, the entry falls back to
`shared/scripts/bootstrap-hook-runtime.sh` in the plugin root, which verifies the
bundle identity against the release manifest's `bundleId` and prints
`{"status":"NOT_ACTIVATED", ...}` rather than executing anything unverified.

**What an extension may do:** ship its own bundles and register them under its
own plugin payload, resolved by the same `run-hook --bundle` mechanism and
subject to the same identity verification.

**What an extension may not do:** edit `hooks/hooks.json` in this repository, or
substitute a resolver. The pack's hook entries and bundle identities are
generated release provenance; hand edits are overwritten and fail verification.

Bundle names are namespaced by their owning payload. A third party ships
`<their-package>/<bundle>`, never a bare name that could collide with a pack
bundle.

---

## Seam 3 — Service manifest

The pack installs and owns its services. Their definitions live as platform
templates: `shared/launchagents/*.plist` (macOS, with `__PLACEHOLDER__` tokens
substituted at install time) and `shared/systemd/*.service|.timer|.path` (Linux).

`shared/services.manifest.json` is the **generated, machine-readable projection**
of those templates, produced by `scripts/generate-service-manifest.mjs`. It is
the surface a supervisor reads to adopt the pack's services without parsing
plists or units itself.

Each entry carries: the service `label`, the platform sources it came from, the
program and arguments (placeholders intact), the port or socket it binds, a
health probe when one is known, the `RunAtLoad` / `KeepAlive` / `ThrottleInterval`
restart semantics from the template, and the timer schedule for periodic
services.

Two properties are guaranteed:

- **Idempotent.** Two runs on an unchanged tree produce byte-identical output.
- **Drift-checked.** `node scripts/generate-service-manifest.mjs --check` exits
  non-zero and names the stale entry when a template changed without
  regeneration.

**C-01 obligation, effective from this contract.** The plists and units are now
generator inputs. Any change that edits, adds, or deletes one of them must
regenerate `shared/services.manifest.json` and run `--check` **in the same
change**. This is recorded in `.kbd-orchestrator/constraints.md`.

**Adoption, not relocation.** A supervisor reads the manifest and manages the
services where they already run, through the platform supervisor
(`launchctl kickstart`, `systemctl --user restart`). The pack does not hand over
ownership of its services, and it keeps working when no supervisor is present.

---

## Seam 4 — Connected skill package declaration

A repository that extends the pack declares itself with a `skill-package.json`
at its root, validated against `shared/schemas/skill-package.schema.json`:

```json
{
  "name": "prometheus-companion",
  "version": "0.1.0",
  "minimumContractVersion": "1.0.0",
  "skills": "skills/",
  "hooks": { "bundles": ["prometheus-companion/sync-status"] },
  "mcpServers": {
    "sovereign-sync": { "command": "sovereign-sync", "args": ["--mode", "mcp"] }
  },
  "services": []
}
```

| Field | Required | Meaning |
|---|---|---|
| `name` | yes | Package identity; namespaces the extension's bundles and skills. |
| `version` | yes | The extension's own semver. |
| `minimumContractVersion` | yes | Refused if it exceeds the contract version the pack implements. |
| `skills` | no | Directory of skills the extension installs, in agentskills.io layout. |
| `hooks.bundles` | no | Bundle names the extension registers (seam 2, namespaced). |
| `mcpServers` | no | MCP servers the extension registers with harnesses. **The extension's own installer writes these entries and owns them**; the pack does not write an extension's MCP registration. |
| `services` | no | Services the extension installs and supervises itself. |

Validate with:

```bash
prometheus contract validate path/to/skill-package.json
```

An extension's installer is responsible for placing its skills and registering
its MCP servers and hooks with each harness. The pack provides validation and
the discovery surfaces; it does not install extensions.

---

## What the pack guarantees

- Every seam above is stable for the life of contract major version 1.
- No pack command requires an extension, warns about its absence, or changes
  behaviour when one is missing.
- The pack certifies fully with no extension installed
  (`scripts/certify-without-companion.sh`).

## What the pack does not guarantee

- That any particular extension exists, is installed, or is compatible.
- Anything about an extension's own surfaces, licensing, or support.
- Stability of internals not named in this document. Only the four seams and the
  three preserved identifiers are contract.

## Reading the contract at runtime

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

`endpoint_source` is one of `env:PROMETHEUS_CONTROL_ENDPOINT`,
`env:SOVEREIGN_SYNC_SOCKET`, `default:socket`, `default:tcp`, or `absent`.
