---
id: service-operations
title: Services, ownership and recovery
sidebar_position: 26
---

# Services, ownership and recovery

Start with skills and the signed local KBD runtime. Add services for searchable
memory, a knowledge library, provider routing, execution or learning interfaces.
Installing a skill or registering MCP does not prove a service is running.

Full owns its local service templates. Mini supplies a smaller Compose stack
and can discover an existing full installation. Companion is a separate optional
extension for connected control and synchronization. Neither pack requires it
to create a team or manage local work.

## Choose services for team work

Begin with the operation the team needs. The team creator records local
coordination; the native harness supplies worker execution. Optional services
extend that workflow independently.

| Team operation | Required capability | Service boundary |
| --- | --- | --- |
| Create/adopt a team, manage tasks or accept a local handoff | Complete creator payload and Node.js 22+ | No memory, gateway or Companion service required |
| Execute an assigned role | An authorized native harness with the role available | Export, installation and ledger `start` do not spawn a worker |
| Compare a declared model catalog | Creator model-selection runtime and explicit metadata | No inference endpoint required; comparison does not establish availability |
| Discover gateway models or run routed inference | Separately configured model endpoint and credentials | liter-llm routes inference; a tool-enabled worker still supplies execution |
| Retrieve shared lessons or knowledge | The selected memory or knowledge integration | Full and mini have different delivery paths and stores; local handoff remains usable without them |
| Request work from another repository | Authorized request channel and destination intake | Full's issue route uses `gh`; mini uses manual delivery; neither requires Companion |
| Use connected control or replication | The separately installed Companion/sovereign-sync host and its enrolled identity | Optional extension with its own source, installation and acceptance evidence |

Use the [agent-team handbook](/docs/guide/agent-teams#find-your-responsibility)
for ownership, dispatch and acceptance. Continue below when operating one of
the services the work actually uses. Team names, memory scope labels and a
reachable service do not confer credentials or permission to act for another
project.

## Source ownership

| Component | Repository | Purpose |
|---|---|---|
| Skills, hooks, CLI, KBD and learning/execution substrate | [prometheus-skill-system](https://github.com/Prometheus-AGS/prometheus-skill-system) | Local workflows, signed state and hook delivery |
| Mini distribution | [prometheus-skills-mini](https://github.com/Prometheus-AGS/prometheus-skills-mini) | Smaller cross-platform packaging, local team files and optional memory publication |
| `surreal-memory-server` | [surreal-memory-server](https://github.com/Prometheus-AGS/surreal-memory-server) | Scoped memory REST/MCP API and embedding-backed retrieval |
| `pk`, `pk-cherry`, learning worker | [prometheus-knowledge-rs](https://github.com/Prometheus-AGS/prometheus-knowledge-rs) | Markdown knowledge library, MCP access and asynchronous writeback |
| `liter-llm` | [liter-llm](https://github.com/GQAdonis/liter-llm) | Authenticated routing to configured model providers |
| `openai-proxy` | [openai-proxy](https://github.com/GQAdonis/openai-proxy) | Optional separately configured provider gateway |
| SurrealDB | Upstream SurrealDB, pinned by the pack | Memory database |
| Companion | Separate `prometheus-companion` repository | Optional connected control and replication; owns its installer and services |

Components have independent versions. Use mini's approved `versions.toml` and
the full pack's existing release matrix, gitlinks and artifact manifests together;
the full pack has no root `versions.toml`. Companion's publication destination belongs in
its release evidence; a local directory name does not establish a remote.

## Full service templates

`shared/services.manifest.json` is generated from LaunchAgent and systemd
templates. These are shipped defaults, not a machine's installation receipt.
All labels below have the prefix `ai.prometheus.`.

| Label | Interface and purpose | State/configuration | Platform templates |
|---|---|---|---|
| `surrealdb-native` | Loopback `28000`, memory database | Native `~/.prometheus/data/surrealdb/database.db` | macOS, Linux |
| `surreal-memory-native` | Loopback `23001`; MCP `/mcp/sse`, REST `/api/v1/memory` | Native namespace `memory`, database `mcp`; `~/.cache/huggingface` model cache | macOS, Linux |
| `pk-cherry` | Knowledge MCP `8942/mcp` | Template's `PK_KB_DIR` selects install source `.prometheus/knowledge` | macOS, Linux |
| `forge-mcp` | Code enrichment MCP `8943/mcp`; consumes pk on `8942` | Template project and skills roots | macOS, Linux |
| `surface-bridge` | Learning UI bridge on loopback `7890` | Learner state belongs to the learner store | macOS, Linux |
| `liter-llm-api` | Configured gateway, normally `4000/v1` | `~/.config/liter-llm/liter-llm-proxy.toml`; private `.prometheus/kbd/secrets.env` | macOS; other platforms need their own gateway setup |
| `exec` | Execution daemon over Unix socket | `~/.prometheus/exec`, private identity, `.prometheus/run/prometheus-exec.sock` | macOS |
| `learning-worker` | Scheduled reconciliation/writeback, no port | Learning queues and delivery index under selected `.prometheus` | macOS, Linux |
| `hooks-logrotate` | Scheduled log maintenance, no port | Selected `.prometheus/logs` | macOS, Linux |
| `prometheus-nudge` | Optional periodic memory heartbeat, no port | Configured memory URL and nudge log | macOS, Linux |
| `codex-skills-sync` | Compatibility catalog refresh, no port | Selected Codex home, curated catalog and install source | macOS |

On-demand components are separate. `learner-model` reads JSON-RPC from stdin
and defaults to `~/.prometheus/learn/learner-model` for state. The harness starts
optional sycophancy-correction MCP. Deep research has a workflow driver and an optional `prometheus-research` daemon. The binary installer conditionally builds it and registers its own macOS plist; this is separate from the core service manifest. Cortex mirroring integrates a separately installed plugin; the
pack does not own a Cortex service.

## Conditional research job

`substrate/prometheus-research` is a separate pack-owned optional process outside the 11-label generated service-template inventory. `install-binaries.sh` builds it only when its Cargo source exists, installs `prometheus-research`, copies static UI assets, and on macOS registers `com.prometheus.research` from the crate's plist. Other hosts can run its actual `--mode mcp` stdio path or explicitly configured server; no Linux research service template is shipped by this conditional installer.

Server mode binds loopback `7891` by default (`--port` selects it), with job REST, per-job SSE and component/static UI routes. Configuration defaults to the platform config directory's `prometheus-research/config.toml`. Job packages/checkpoints default to `~/.prometheus/research` (`RESEARCH_OUTPUT_DIR` overrides). The plist writes diagnostics to `/tmp/prometheus-research.log` and `/tmp/prometheus-research.error.log` and grants an explicit PATH for its headless Claude/Codex child.

`RESEARCH_EVENT_TOKEN` authorizes child event ingest; absent server configuration creates a per-process token before the runtime starts. This token is not general authentication for job creation/read/delete routes. Keep the listener local and use an authorized caller. The daemon delegates stages to a real harness/driver; health or child exit zero alone does not prove a research package passed verification.

Inspect/unload/reload this separate plist through its actual macOS service definition rather than assuming `install-mcp-services.sh` owns it. Preserve packages/checkpoints, source identity and logs during upgrades; restore an isolated package before resuming a failed job. See [research runtime](/docs/substrate/prometheus-research) for source-backed supervision and package evidence.

## Installation and operation

Use a clean checkout matching the release; preserve it while marketplace or
service registrations refer to it. Full binary and service installation are
separate:

```bash
bash scripts/install-binaries.sh
bash scripts/install-mcp-services.sh --dry-run
bash scripts/install-mcp-services.sh
```

The native service installer requires Bash 4 or newer. macOS `/bin/bash` 3.2
cannot run its associative arrays. Select a modern Bash for this installer;
LaunchAgent wrappers have their own platform requirements. The skill installer
builds learner/bridge binaries; service installation starts the bridge.

The service installer accepts `--restart`, `--unload`, `--learning-recovery`,
`--render-only <directory>` and repeatable `--exclude <service>`. Unloading
preserves definitions and data. Learning recovery targets pk, the worker and
hook rotation. Inspect selections before applying a plan.

On macOS, the legacy manager's `status`, `doctor` and `logs` commands show its
managed subset: `bash scripts/prometheus-services.sh status`. Inspect the
specific LaunchAgent for other labels. On Linux, use the shipped
`systemctl --user` units. No platform template means no automatic service
installation for that platform, even when the binary runs there.

The execution binary installer verifies atomic replacement and artifact
hash/signature. Its service installer checks version, creates the private
identity, validates the plist and registers the job. Functional execution is
separate evidence. The general service installer can warn about execution
service failure and continue; exit zero alone is not all-service acceptance.

## Data and credentials

Full native memory uses `memory/mcp`; mini Compose uses `memory/main_local_384`.
The same port or namespace does not merge their records. Record actual endpoint,
namespace, database, model identity and data location before upgrade/recovery.
Query-cache invalidation after a model change does not convert stored vectors.

Native database templates use loopback development credentials. Their memory
REST/MCP configuration has no application authentication. Team/project scopes
partition retrieval; they do not authorize network clients.

The liter-llm wrapper loads the private secrets file and supplies an explicit
config path. `/v1/*` requires gateway credentials. An unauthenticated `401`
proves listener reachability, not inference readiness. Align real provider/model
IDs with team policy; a declared alias is not proof of the model used.

Preserve database, knowledge, signed KBD store, device identity, learner state,
queues and plugin receipts separately. Stop relevant writers for a consistent
backup. Restore to an isolated destination first and confirm identity,
permissions and schema/model compatibility. Do not discard queues or replace
identity keys to clear a health failure. Reconcile uncertain remote writes
before another publication attempt.

Full hook failures retain the durable primary learning record; optional Cortex
mirror saturation does not discard it. Codex parent SessionStart reads metadata
digests only, while role and Claude lead recall use their own scopes. Mini keeps
its smaller file tier/outbox rather than the full Python/worker pipeline.

## Optional Companion

Local KBD needs no always-on control daemon. Companion owns optional connected
control and replication through its separate recovered source repository. The
pack never builds or installs it. The [integration contract](/docs/kbd/integration-contract)
defines the one-way extension boundary. Companion has no public remote or
certified release yet; its `docs/installation.md` and `docs/control-api.md`
describe the current source candidate, not installed delivery.

Headless and windowed Companion share the same-user Unix control socket.
Standalone `sovereign-sync --mode daemon` is an alternative host; choose one
socket owner. `--mode server` disables P2P, while `--mode mcp` connects stdio
sync tools to an existing host. The MCP bridge has no TCP fallback and does not
create a second P2P identity. Local control starts before optional P2P transport;
a failed or disabled transport need not stop local authority.

Select the same socket, data root, configuration and enrolled signing key on
host and clients. `SOVEREIGN_SYNC_SOCKET` selects the socket;
`PROMETHEUS_DATA_DIR` selects journal/registry data but does not relocate socket
or identity files. `PROMETHEUS_PACK_ROOT/shared/services.manifest.json` is the
explicit optional service-adoption source. A missing manifest leaves local
control independent; Companion must not guess or take over pack services.

Companion's macOS installer owns only `ai.prometheus.companion`, accepts a
separately approved headless binary and existing private enrolled key, and
preserves a sibling plist backup. Its separate connected-skill installer
supports Claude Code user-scope registration and receipt-owned replacements.
Registration does not start a host. Linux has no supplied Companion service
installer; native Windows is outside this Unix/launchd contract. See
[Companion installation](/docs/sovereign-sync/installation) for source procedures.

Connected MCP `sync-push` persists the exact signed request and selected socket
before POST in `<data-root>/prometheus/companion/mcp-push-outbox/<request-id>.json`.
It returns `intentPath`; failed intent persistence means `not-submitted` and no
send. A transport/response failure can be uncertain: recover the original ID,
body and endpoint, then query `sync-push-receipt` on that host before replay.
This outbox is intent, not acceptance. Local `broadcast` is not peer application.
See [signed pushes and recovery](/docs/sovereign-sync/signed-pushes-and-receipts).

`prometheus contract show --json` reports discovery; endpoint absence is normal.
Discovery, `/health`, `/ready`, service loading and peer reachability each have
separate meanings. Desktop/mobile plans and local source convergence are not
shipped skill-pack features or cross-device acceptance. Start with the
[connected source contracts](/docs/sovereign-sync/overview).

## Update evidence

A usable update records source commits and installed artifact identities,
verifies the selected plugin generation/configuration and exercises the real
operation. Version strings, listening ports, registration and build success are
individual observations, not the complete functional proof.

Maintainers finish production before the consolidated local gate. Tests use
isolated homes, data, keys, sockets and ports, never the live memory listener.
Serialize Cargo/rustc machine-wide and keep each worktree's target directory.
Hosted test runners are not release evidence.
