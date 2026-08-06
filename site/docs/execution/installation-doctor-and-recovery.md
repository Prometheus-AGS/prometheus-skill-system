---
title: Installation, doctor, and recovery
description: Strict local installation, LaunchAgent operation, non-mutating diagnosis, logs, and rollback.
---

# Installation, doctor, and recovery

The installer is strict by default. It builds or accepts a `prometheus-exec 1.7.0` release binary, checks its version, stages it on the destination filesystem, applies and verifies the platform signature, atomically replaces the destination, reads back the installed hash/version/signature, and writes a mode-`0600` installation receipt. Any failure restores the prior binary or removes the failed first install.

## Build and install locally

```bash
(cd crates/prometheus-exec && cargo build --release)
bash scripts/install-prometheus-exec.sh
prometheus-exec --version
```

Build from inside the crate directory. `crates/prometheus-exec/rust-toolchain.toml`
pins the stable toolchain, and `rust-toolchain.toml` is resolved from the current
directory — it is **not** honored via `cargo build --manifest-path`. Building
from the repo root uses whatever default toolchain the caller has, producing a
binary whose SHA256 cannot match `config/prometheus-exec-binary.json` and
failing the installer's hash gate on an otherwise correct build.

Expected output is exactly `prometheus-exec 1.7.0`.

Validate the checked documentation examples against the installed binary and active signed generation using disposable state:

```bash
npm run check:docs-exec-examples
npm run docs:examples
```

The first command checks source syntax, expected outputs, relationship semantics, and documentation drift. The second exercises the existing local certification driver and removes its temporary state on exit. Hosted workflows do not run these product examples.

Inspect service installation without changing the machine:

```bash
bash scripts/install-prometheus-exec-service.sh --dry-run
```

On macOS, the service installer creates a private identity if absent, renders the checked LaunchAgent, validates it with `plutil`, installs it atomically, and optionally loads `ai.prometheus.exec`. Use `--no-load` when only the definition should be installed.

## Non-mutating doctor

```bash
prometheus-exec doctor \
  --socket "$HOME/.prometheus/run/prometheus-exec.sock" \
  --state-dir "$HOME/.prometheus/exec" \
  --identity "$HOME/.prometheus/exec/identity.json" \
  --plugin-root "$HOME/.prometheus/plugins/prometheus-skill-pack" \
  --service-definition "$HOME/Library/LaunchAgents/ai.prometheus.exec.plist" \
  --mcp-schema ./docs/reference/api/prometheus-exec.mcp.json \
  --exclude service:sovereign-sync \
  --format json
```

Doctor reads binary identity, the service definition, the LaunchAgent label's loaded state, socket health/readiness and permissions, receipt identity, Tier P/W availability, signed component trust, ledger/receipt/CAS reconciliation, MCP schema parity, and an optional remote queue. A supplied service definition fails diagnosis if its LaunchAgent is not loaded. Exclusions are applied before check construction. Diagnosis does not install, start, stop, compile, consume, contact excluded services, or rewrite state.

## Logs and common failures

The LaunchAgent writes standard output and error under the configured Prometheus log directory. Check the JSON doctor result first; it preserves individual failed checks instead of translating empty/error output into success.

Review [Security and trust boundaries](./security-and-trust.md) before changing identity, component trust, socket, or sandbox settings.

| Symptom | Meaning | Action |
| --- | --- | --- |
| `/health` works and `/ready` is `503` | Process is live but a required subsystem is not ready | Read the failed readiness subsystem and service log |
| `component-unauthorized` | Bytes, hash pin, active generation, or rollback state does not authorize the component | Verify the signed plugin generation and component descriptor |
| `request_hash_conflict` | One request ID was reused with different canonical content | Generate a new request ID or restore the original request |
| `artifact_not_found` | CAS does not contain the requested digest | Reconcile the run/receipt pins; do not fabricate an artifact |
| Tier P backend unavailable | The supported OS sandbox was not found or certified | Do not enable a direct-process fallback |
| remote queue omitted | Remote diagnosis was not selected or was excluded | This does not affect local health or offline verification |

## Recovery and rollback

After a crash, restart the same identity, state directory, socket, and plugin root. Reconciliation recovers logged receipts, requeues never-spawned work, and marks post-spawn interrupted work terminal without re-execution.

Binary backups are content-addressed under the install backup directory. Plugin rollback is separate and atomic:

```bash
node scripts/install-plugin-generation.js --rollback
node scripts/install-plugin-generation.js --verify
```

The pointer switch moves payload, component metadata, skill/component indexes, stable dispatchers, and target receipt authority together. It never rewrites a signed immutable generation.

Next: [Platform and evidence status](./platform-and-evidence-status.md).
