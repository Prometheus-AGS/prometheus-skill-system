# 19 · Installation

Install from approved source and its pinned dependencies. Skills, native binaries and services have independent versions and evidence. Installation is a mutation; preserve existing ownership receipts and data before changing a machine.

## Select a profile

```bash
git clone https://github.com/Prometheus-AGS/prometheus-skill-system.git
cd prometheus-skill-system
./install.sh --profile skills
```

Git and compatible Node are required; the configured development line is Node 22 and the root engine requirement is in `package.json`. The root installer initializes the exact imports for the profile. It offers detected clients and prints its mutation plan. Automated selection is explicit:

```bash
./install.sh --profile skills --targets detected --non-interactive --yes
./install.sh --verify --targets detected --non-interactive
```

Release and minimum-active versions come from `skill-system.json`. `--best-effort` permits partial development outcomes and is not certification. Full skills on Windows require Git Bash or WSL; mini is the Node-only alternative. Full profile is supported on macOS/Linux only:

```bash
./install.sh --profile full
```

Full installs offer a provider for each service. The native supervisor is launchd on macOS and user systemd on Linux. Full's scheduled maintenance scripts require Bash 4+. Windows users can manage Docker services with Node from PowerShell and install the skills profile from Git Bash or WSL.

### Choose service providers

Use the interactive Full installer, or supply one choice per service. A skills-profile install also accepts explicitly requested services. The standalone service command works independently of skill distribution.

```bash
./install.sh --profile full --service surrealdb=reuse --endpoint surrealdb=http://127.0.0.1:28000 \
  --service surreal-memory=native --service prometheus-research=off --dry-run

# Start from an empty service selection on a fresh home.
node scripts/service-providers/cli.mjs plan --enabled surface-bridge --service surface-bridge=docker --port surface-bridge=17892
node scripts/service-providers/cli.mjs apply --enabled surface-bridge --service surface-bridge=docker --port surface-bridge=17892
node scripts/service-providers/cli.mjs status --json
node scripts/service-providers/cli.mjs stop --only surface-bridge
node scripts/service-providers/cli.mjs uninstall --only surface-bridge
```

In PowerShell use, for example, `./scripts/services.ps1 plan --enabled surface-bridge --service surface-bridge=docker`. Use `--non-interactive` for automation. Set `PROMETHEUS_SERVICE_MODE` for the default, or set `PROMETHEUS_SERVICE_PK_CHERRY_MODE=docker` and `PROMETHEUS_SERVICE_PK_CHERRY_PORT=18942` for one service. The corresponding `_ENDPOINT` variable accepts an explicit URL. Command-line choices take precedence over service environment variables and saved choices. A global default applies to enabled services that have no saved choice.

### Reuse a database started by another tool

Suppose The Boss already owns the database listening on port 28000. Run a read-only plan first, using its actual endpoint:

```bash
node scripts/service-providers/cli.mjs plan --enabled surrealdb --service surrealdb=reuse --endpoint surrealdb=http://127.0.0.1:28000
```

Look for the selected endpoint and a compatible readiness result. If the service is unreachable or its identity is incompatible, stop and check the owning tool and URL. A listening port alone cannot identify the database. If you need a separate pack-owned instance, choose a free port and review its persistent data location before applying. When reuse succeeds the original owner remains responsible for startup, upgrades and backups.

| Mode | Installer behavior |
| --- | --- |
| `auto` | Probe configured and known endpoints, reuse one compatible service, otherwise choose native where supported. |
| `reuse` | Require a running compatible instance. The Boss, another pack, and manually installed services can supply it. |
| `native` | Install the selected native binary and register the user job. |
| `docker` | Pull the image digest published for this checkout and the local Linux container architecture. Run Compose with a loopback host binding. |
| `off` | Disable this service for this pack. User-owned processes and data stay with their owner. |

The Full service IDs are `surrealdb`, `surreal-memory`, `liter-llm`, `openai-proxy`, `pk-cherry`, `forge-mcp`, `surface-bridge`, `prometheus-research`, `prometheus-exec`, and `learning-worker`. Native surface-bridge binds port 7890. A Docker mapping can publish another port. The released exec daemon requires macOS Seatbelt for Tier P readiness. Its Linux images can start the process, but its Unix socket reports an unavailable Tier P backend. The installer rejects the Docker selection during planning. Select native on macOS, reuse a local exec socket that passes readiness, or choose off. The learning worker uses a scheduled queue and exec uses a Unix socket. Neither has an HTTP port to publish.

Docker Desktop on macOS or Windows must use Linux containers. Docker Engine needs Compose v2. The images for linux/amd64 and linux/arm64 package the checked Linux release artifacts. The `service-images.json` asset on the versioned GitHub release identifies each image and its source. SurrealDB uses a separately pinned upstream image. An absent or incompatible service image is reported by the plan. You can explicitly select another provider. For Docker configuration:

- Supply `SURREAL_ROOT_USERNAME` and `SURREAL_ROOT_PASSWORD` for a new database, and `MEMORY_USERNAME` and `MEMORY_PASSWORD` for the memory server's database account. Full's Gate-enabled memory image also requires `AUTH_JWT_ISSUER`, `AUTH_JWT_AUDIENCE`, `AUTH_JWT_ALGORITHM`, and `AUTH_JWT_KEY` or `AUTH_JWT_KEY_FILE`.
- Set `LITER_LLM_CONFIG` to an existing gateway configuration. Set `OPENAI_PROXY_AUTH_PATH` to a Codex authentication file or provide `OPENAI_API_KEY` for openai-proxy. Configure gateway model routes for the chosen backend.
- Set `FORGE_MCP_TOKEN` for authenticated Forge access and supply that bearer token to clients. Research needs `PROMETHEUS_RESEARCH_HARNESS_DIR` with the Linux harness executable and its runtime dependencies. The learning worker needs `PROMETHEUS_LEARNING_PROJECT_ROOT` for paths referenced by its queued jobs.

Full's Docker memory API requires a Gate-issued bearer token for MCP clients. Export `PROMETHEUS_MEMORY_BEARER_TOKEN`. For a native or container learning worker consuming this API, set `PROMETHEUS_WORKER_MEMORY_GATEWAY` to the address of a gateway that authenticates its requests. Supply `FORGE_MCP_TOKEN` in the client environment for protected Forge. The installer writes environment references for Claude, OpenCode and Codex. Configure the authorization header directly in clients that use a different credential mechanism.

Auto and reuse can inspect local Docker labels to identify an existing sidecar. Set `PROMETHEUS_DISCOVER_DOCKER=0` to skip that optional inspection, or set it to `1` to enable it during a native-only plan. A busy port with an unknown service, multiple matching instances, or an incompatible memory namespace, database or embedding profile requires an explicit decision. Pass `--endpoint` for an existing compatible instance, select a different port for an owned service, or select `off`. The provider receipt at `~/.prometheus/service-providers/full.json` records the mode, endpoint and ownership for status, stop and removal. Stop and uninstall operate on recorded pack-owned services. Provider operations preserve service data.


## Selected home and immutable payload

A nonempty inherited `CODEX_HOME` selects Codex's root, otherwise the selected `--home` user's `.codex` is used. Preserve this choice through installation, verification, rollback and removal. Logical plugin-store receipts and the selected Codex projection are different boundaries.

```bash
CODEX_HOME="/absolute/custom-codex" ./install.sh --profile skills --targets codex
CODEX_HOME="/absolute/custom-codex" ./install.sh --verify --targets codex --non-interactive
```

The installer verifies signed payloads and selected projections; copy targets retain ownership evidence. Unknown copies and unrelated skills are preserved and diagnosed. Never edit generated packages, immutable installed generations or harness caches by hand.

## Services and optional components

The [service operations guide](26-service-operations.md) is the canonical runbook for binary/service install, transport, namespace/database, credentials and recovery. Building learner/bridge binaries does not start their service. Companion owns cross-device sync and is not installed by this pack.

Codex memory policy disables `features.memories`, `memories.generate_memories` and `memories.use_memories`, archives only known v1/v2 summary files, and preserves other memory data. Existing sessions retain configuration; start a new session and account for explicit overrides. See [memory tiers](memory-tiers.md).

## Versioned native binaries

The binary publisher uses the pack version as its Git tag. Full `v1.17.0` is a published example. To
obtain the exact source, clone with `git clone --branch v1.17.0 --recurse-submodules
https://github.com/Prometheus-AGS/prometheus-skill-system.git` (one command).
The release contains a platform archive and a file manifest for each of macOS,
Linux and Windows on AMD64 and ARM64. SHA256SUMS covers both. See the
[GitHub releases](https://github.com/Prometheus-AGS/prometheus-skill-system/releases).

- On macOS and Linux, run `bash scripts/install-binaries.sh` for the managed
  binaries or use `./install.sh --profile full` for the full setup.
- On Windows, run `.\scripts\install-binaries.ps1` in PowerShell from the
  checkout. Add the destination to your user PATH to use the tools in a new shell.
  The Windows native binary installer is separate from Unix service setup.

The installer requests the exact tag and platform. A missing asset permits the
source-build path. A corrupt download or a manifest mismatch stops installation.
The representative local download of the published 1.17.0 macOS ARM64 archive verified 23 files and ran the CLI. Clean-machine installation and Docker pulls remain separate checks.
Apple MLX resources are included on Apple Silicon, and the Darwin execution
binary must also match its independently pinned hash.

## Local evidence

Maintainers finish every phase production change before tests, formatting, builds or review. Use isolated homes, data, keys and ports. Serialize Cargo/rustc machine-wide and keep separate worktree target directories; use `sccache` for reuse.

After source completion, exercise the real installed production entrypoints and archive exact local results. Version strings, configured endpoints, listening ports and build success each prove only their own observation. Hosted testing is forbidden; documentation and binary release packaging are separate from local certification.

Previous: [Plugins and Immutable Distribution](18-plugins-and-marketplace.md) · Next: [Updating](20-updating.md).
