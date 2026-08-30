# 19 · Installation

This procedure builds, signs, installs, and locally certifies Prometheus 1.7.0. The canonical deployment is native user services on macOS or systemd user services on Linux; containers remain optional development packaging.

## Prerequisites

The skills profile requires Git and Node.js 18+. Clone normally; the root
installer initializes the exact required submodule pins:

```bash
git clone https://github.com/Prometheus-AGS/prometheus-skill-system.git
cd prometheus-skill-system
./install.sh
```

`./install.sh` interactively highlights `skills`, preselects all detected
clients, prints the exact mutation summary, and leaves deselected clients
untouched. Automation can use:

```bash
./install.sh --profile skills --targets detected --non-interactive --yes
./install.sh --verify --targets detected --non-interactive
```

Use `--profile full` on macOS or Linux to initialize all submodules, approve
prerequisites, build binaries locally, configure MCP for selected clients,
install user services, and run doctors. Full installation is rejected on
Windows; the skills profile is certified through Git Bash or WSL. `--best-effort`
is explicitly non-certifying. Release `1.7.0` is the minimum supported active
umbrella skill-system release.

The equivalent explicit full-system entrypoints are:

```bash
./install.sh --profile full
npm run setup:full
prometheus setup --full
```

Use the sharing profile only when this machine should exchange state with a
peer:

```bash
npm run setup:sharing
prometheus setup --full --sharing
```

The first two run the canonical full-profile installer. The
`prometheus setup --full` command extends CLI component setup with the same
managed-service installer while leaving the KBD control plane stopped and
disabled. KBD commands use the signed local `kbd-runtime` journal directly.
When cross-machine sharing is actually required, run
`prometheus setup --full --sharing`; that explicit profile loads
`ai.prometheus.sovereign-sync` on macOS or
`ai.prometheus.sovereign-sync.service` on Linux. The daemon passively replicates
local KBD state over a same-user Unix socket and is not required for ordinary
KBD work.

Keep Rust caches on an internal SSD:

```bash
export CARGO_HOME="$HOME/.cargo"
export CARGO_TARGET_DIR="/path/on/internal-ssd/prometheus-target"
```

Use a separate target directory for each workspace or worktree. Let `sccache`
share reusable compilation while each target directory keeps its own Cargo
lock. Before starting any Rust command, confirm that no other Cargo or `rustc`
process is active anywhere on the machine.

## Build order

```mermaid
flowchart TD
  Implement["Complete coherent implementation"] --> Server["Build affected Memory components"]
  Server --> Knowledge["Build affected knowledge/worker components"]
  Knowledge --> Root["Build affected prometheus CLI"]
  Root --> Binaries["Install + sign six binaries"]
  Binaries --> Plugin["Activate immutable plugin generation"]
  Plugin --> Services["Install allowed user services"]
  Services --> Integration["Run smallest full-integration gate"]
  Integration --> Doctors["Run local doctors"]
  Doctors --> Cert["Final local certification"]
```

The six release binaries are `surreal-memory-server`, `pk`, `pk-cherry`,
`prometheus-learning-worker`, `prometheus`, and `prometheus-exec`. Complete the
production implementation before authoring, modifying, or running tests. During
implementation, use inspection and static reasoning; if compiler feedback is
indispensable, wait for a coherent edit batch and run one package-scoped
`cargo check`. Do not run workspace-wide, all-target, Clippy, or per-edit Rust
loops. After the implementation is complete, run the smallest applicable full
integration target and then the final local certification. Unit and mock-only
tests are not acceptance evidence.

All six executables share the product release version. Verify the installed
artifacts before loading services:

```text
prometheus 1.7.0
pk 1.7.0
pk-cherry 1.7.0
prometheus-learning-worker 1.7.0
surreal-memory-server 1.7.0
prometheus-exec 1.7.0
```

Each line is the exact output of the corresponding `--version` command. The
Memory command must exit before logging, configuration, storage, embeddings, or
network initialization; `-V` is also supported.

`scripts/install-mcp-services.sh` installs and starts the execution service
(`ai.prometheus.exec`) along with the other managed daemons, delegating to
`install-prometheus-exec-service.sh` so the identity, version, hash, and
signature checks still run. Skip it with `--exclude exec`.

To build, sign, atomically install, and read back the execution service on its
own — the binary first, then the LaunchAgent:

```bash
bash scripts/install-prometheus-exec.sh --dry-run
bash scripts/install-prometheus-exec.sh
bash scripts/install-prometheus-exec-service.sh --dry-run
bash scripts/install-prometheus-exec-service.sh
```

`prometheus-exec` is a Unix-socket daemon with no HTTP port. It is pinned to the
stable toolchain by `crates/prometheus-exec/rust-toolchain.toml` because the
installer gates on the SHA256 in `config/prometheus-exec-binary.json`, and a
release binary's hash depends on the exact `rustc`. Build it from inside the
crate directory — `rust-toolchain.toml` is resolved from the current directory
and is **not** honored via `cargo build --manifest-path`.

See [Execution installation, doctor, and recovery](/docs/execution/installation-doctor-and-recovery) before loading the LaunchAgent.

## Plugin generation

```bash
./install.sh --profile skills --targets all --non-interactive --yes
./install.sh --verify --targets all --non-interactive
```

This validates the manifest, 14 target receipts, copy-versus-symlink modes, stable dispatchers, active/previous pointers, and stale-path absence.

## Refresh an existing machine after source changes

Build only affected native components first, then refresh services and all
detected harnesses from one generated distribution:

```bash
npm run build:distribution
npm run validate:harness-adapters
bash scripts/install-mcp-services.sh --restart
bash scripts/install-skills-flat.sh
npm run verify:skills
npm run validate:codex
npx tsc -p .opencode/tsconfig.json --noEmit --pretty false
```

The service installer disables Sovereign Sync unless sharing was explicitly
selected. The skill installer activates one immutable generation and configures
every detected supported client; an absent client configuration is reported as
skipped rather than silently created. Run the health checks and doctors below
after workers finish processing any accepted queue item.

## Services with explicit exclusions

Preview first:

```bash
bash scripts/install-mcp-services.sh --dry-run --exclude sovereign-sync
```

Then install only the reviewed services:

```bash
bash scripts/install-mcp-services.sh --exclude sovereign-sync
```

The managed deterministic-learning surface includes the native Memory server, `pk-cherry`, learning worker, and owner-only hook log rotation. An excluded service is not rendered, initialized, stopped, started, restarted, or rewritten.

## Verify

```bash
prometheus doctor --json \
  --exclude control.kbd-runtime \
  --exclude state.kbd-orchestrator \
  --exclude control.kbd-rollout \
  --exclude service:sovereign-sync

pk doctor --json
bash scripts/prometheus-services.sh doctor --exclude sovereign-sync
bash scripts/check-mcp-health.sh --json --exclude sovereign-sync
prometheus learning status --json
```

`/health` proves liveness; `/ready` proves durable ingestion readiness. Finally, run the intentionally mutating operation certification:

```bash
bash scripts/certify-memory-operations.sh --long-memory
```

It verifies exact replay, hash conflict, response-loss reconciliation, terminal receipts, and SSE resume. Archive redacted JSON reports and exact commands. Every warning needs a disposition; required checks must be green.

See [Installation and upgrades](/docs/operations/installation-and-upgrades) and [Doctors and Mac certification](/docs/operations/doctors-and-mac-certification).

---

*Previous: [← 18 · Plugins & Marketplace](18-plugins-and-marketplace.md) · Next: [20 · Updating →](20-updating.md)*
