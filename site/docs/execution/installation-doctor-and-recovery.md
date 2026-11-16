---
title: Installation, doctor, and recovery
description: Strict local installation, LaunchAgent operation, non-mutating diagnosis, logs, and rollback.
---

# Installation, doctor, and recovery

The installer is strict by default. It builds or accepts the independently pinned Exec release binary, checks its version, stages it on the destination filesystem, applies and verifies the platform signature, atomically replaces the destination, reads back the installed hash/version/signature, and writes a mode-`0600` installation receipt. Any failure restores the prior binary or removes the failed first install.

## Build and install locally

Finish the phase's production changes and local integration gates before installing.
The installer invokes the canonical reproducible build helper by default:

```bash
bash scripts/install-prometheus-exec.sh
prometheus-exec --version
```

`scripts/build-prometheus-exec.sh` stages a workspace, uses the crate's pinned
toolchain and remaps source paths. A plain Cargo release build may embed a checkout
path and cannot be assumed to match the certified hash. Keep the source build,
expected version and the `config/prometheus-exec-binary.json` hashes aligned through
the authorized final certification process; do not bypass a mismatch. Exec's version
is independent of the skill pack and other services.

## The certified build hash

The installer refuses to install a binary whose SHA-256 differs from a committed
value. That gate is what lets a receipt say which exact bytes produced a result. It
only works if the same source gives the same bytes everywhere, so the build is made
reproducible and the hash is pinned per platform.

### One hash per target

A release binary for Apple Silicon and one for Intel are different files. A single
hash can match only the architecture that certified it, so the manifest pins one
value per Rust host target:

```json
{
  "expectedVersion": "prometheus-exec 1.7.0",
  "expectedBuildSha256ByTarget": {
    "aarch64-apple-darwin": "<64 hex characters>",
    "x86_64-apple-darwin": "<64 hex characters>"
  }
}
```

The installer asks `rustc -vV` for the host target from inside the crate directory,
because `rust-toolchain.toml` is resolved from there and an x86_64 toolchain on
Apple Silicon builds x86_64 binaries. It then reads that target's entry. When
`rustc` is absent it falls back to the kernel's architecture. A manifest that still
has only the older single `expectedBuildSha256` field keeps working: that value is
used for every target.

Keep one map entry per line. The manifest is read with `awk`, not a JSON parser.

Check what a machine would require without building or installing:

```bash
bash scripts/install-prometheus-exec.sh --dry-run
```

```text
[dry-run] would build crates/prometheus-exec --release via scripts/build-prometheus-exec.sh
[dry-run] would require source sha256:fe2f2c5e… (target x86_64-apple-darwin), then verify, stage, sign, atomically install, and read back …/prometheus-exec
```

The installation receipt records the same `target` beside the expected, built and
installed hashes.

### What makes the build reproducible

The helper removes three machine-specific inputs:

| Input | Why it changes the hash | What the helper does |
|---|---|---|
| Checkout path | `rustc` embeds absolute source paths in panic locations and `file!()` | Remaps the repository, Cargo home and build directory with `--remap-path-prefix` |
| Path dependencies outside the crate | Cargo hashes an out-of-workspace dependency's absolute path into every symbol | Builds in a staged workspace that contains the crate, `substrate/exec-*` and `wit/` |
| The `rust-src` component | When it is installed, `rustc` rewrites standard-library paths from `/rustc/<commit>/…` to the local toolchain directory, which embeds the home directory | Remaps the toolchain's source directory back to `/rustc/<commit>` |

The third one is easy to trigger without noticing. Installing rust-analyzer adds
`rust-src`, and from then on a plain build produces a different binary from the
same source. With the remap in place, the set of installed toolchain components no
longer affects the hash.

Flags are passed through `CARGO_ENCODED_RUSTFLAGS` and `RUSTFLAGS` is unset, so
ambient compiler flags cannot alter the certified binary.

### When to recertify

Recertify when one of these changed **on purpose**:

- source under `crates/prometheus-exec`, `substrate/exec-*` or `wit/`;
- the crate's `Cargo.lock`;
- the pinned channel in `crates/prometheus-exec/rust-toolchain.toml`;
- `scripts/build-prometheus-exec.sh` itself.

Do not recertify to make a mismatch go away. A mismatch with none of those changes
means the build is not reproducible on that machine, and that is the defect to
find.

Print the hash for the machine you are on:

```bash
bash scripts/build-prometheus-exec.sh --print-hash
```

Write the value to that machine's entry in `expectedBuildSha256ByTarget`. A hash is
specific to its target, so **every** pinned target must be recertified, each on a
machine of that target, and all entries updated in the same commit. Until that is
done, the installer on the other architecture correctly refuses the build.

A no-op Cargo build proves nothing about a pin. If Cargo reports the crate as
fresh, the hash you read belongs to an earlier build. The integration test has a
real-build case that copies the working tree to a second directory, builds there
from scratch and requires the pinned hash:

```bash
PROMETHEUS_EXEC_TEST_REAL_BUILD=1 bash scripts/tests/install-prometheus-exec.test.sh
```

Run it at the phase boundary, with no other Cargo build active on the machine.

### Overrides

| Variable | Use |
|---|---|
| `PROMETHEUS_EXEC_EXPECTED_SHA256` | Require this hash instead of the manifest's. For a controlled one-off, such as installing an approved prebuilt binary |
| `PROMETHEUS_EXEC_SOURCE_BIN` | Install this file instead of building |
| `PROMETHEUS_EXEC_HASH_MANIFEST` | Read the pin from another manifest file |
| `PROMETHEUS_EXEC_EXPECTED_VERSION` | Require a different `--version` string |

Exporting `PROMETHEUS_EXEC_EXPECTED_SHA256` in a shell profile defeats the gate on
that machine: any build it names installs, whatever the committed pin says. Keep it
for single commands.

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

The shipped service definition is macOS-only; a binary build on another platform
does not provide a matching managed service. On macOS, the service installer creates a private identity if absent, renders the checked LaunchAgent, validates it with `plutil`, installs it atomically, and optionally loads `ai.prometheus.exec`. Use `--no-load` when only the definition should be installed.

## Non-mutating doctor

```bash
prometheus-exec doctor \
  --socket "$HOME/.prometheus/run/prometheus-exec.sock" \
  --state-dir "$HOME/.prometheus/exec" \
  --identity "$HOME/.prometheus/exec/identity.json" \
  --plugin-root "$HOME/.prometheus/plugins/prometheus-skill-pack" \
  --service-definition "$HOME/Library/LaunchAgents/ai.prometheus.exec.plist" \
  --mcp-schema ./docs/reference/api/prometheus-exec.mcp.json \
  --exclude remote-queue \
  --format json
```

Doctor reads binary identity, the service definition, the LaunchAgent label's loaded state, socket health/readiness and permissions, receipt identity, Tier P/W availability, signed component trust, ledger/receipt/CAS reconciliation, MCP schema parity, and an optional remote queue. A supplied service definition fails diagnosis if its LaunchAgent is not loaded. Exclusions are applied before check construction. Diagnosis does not install, start, stop, compile, consume, contact excluded services, or rewrite state.

## Logs and common failures

The LaunchAgent writes standard output and error under the configured Prometheus log directory. Check the JSON doctor result first; it preserves individual failed checks instead of translating empty/error output into success.

Review [Security and trust boundaries](./security-and-trust.md) before changing identity, component trust, socket, or sandbox settings.

| Symptom | Meaning | Action |
| --- | --- | --- |
| `source hash mismatch for target <triple>; expected …, got …` | The built binary is not the certified one for this architecture | If exec source or the toolchain changed on purpose, recertify every pinned target. Otherwise find what made the build differ |
| `no valid certified build hash for target '<triple>'` | The manifest has no entry for this machine's target | Run `build-prometheus-exec.sh --print-hash` on this machine and add the entry through the certification process |
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
