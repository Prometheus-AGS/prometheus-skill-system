# Maintenance, upgrades and recovery

This contract covers the full Prometheus skill system: its skills, hooks, CLI,
local KBD authority and declared learning services. The current
`phase-learning-deploy-and-debt` source is a release candidate. Compilation,
generated-package reconciliation, local integration, independent review, owner
approvals, publication and installed functional acceptance are still separate
requirements. This document does not certify that candidate or authorize a
machine update.

The optional Prometheus Companion owns its installation, services and release.
Its absence is normal. The pack's local KBD path must remain usable without an
extension. Read [service operations](guide/26-service-operations.md) for service
ownership and [the integration contract](integration-contract.md) for the
versioned extension seams. Change 14's Companion source handoff now supplies the
installation, connected-control and recovery contracts below. Its initial
origin is a private local mirror, not a public release source. Publication-bound
links and accepted artifacts remain release-owner work.

## Bounded support matrix

`skill-system.json` declares the finite installation profiles below. A declared
target is source support intent. The release manifest must additionally name the
tested OS version, architecture, harness version and artifact for every supported
combination. No architecture, OS release or harness version is inferred from a
successful build on another host. Those exact release bounds remain pending the
freeze and final local gate.

| Platform/profile | Source prerequisites and boundary | Release acceptance |
|---|---|---|
| macOS, skills | Git, Node 20.19+, Bash launcher; selected harness targets | Pending exact host/harness matrix and installed invocation |
| macOS, full | Skills prerequisites plus component toolchains; Bash 4+ for the native service installer; only shipped LaunchAgent templates | Pending binary, service, hook and real operation receipts |
| Linux, skills | Git, Node 20.19+, Bash launcher; selected harness targets | Pending exact host/harness matrix and installed invocation |
| Linux, full | Skills prerequisites plus component toolchains; Bash 4+; native supervision only for services with supplied Linux management | Pending native installation and real operation receipts |
| Windows through Git Bash, skills | Node 20.19+, Git Bash; no full-profile native services | Pending native Windows evidence for the approved harness versions |
| Windows through WSL, skills | Node 20.19+, Git and Bash in WSL; no Windows full profile | Pending separate WSL evidence; Linux evidence does not certify Windows |
| Windows `cmd.exe`/PowerShell, full | Outside the full-pack installation matrix | Use the separately maintained portable mini where its install-scope rule permits |

The full manifest has exactly 14 target IDs. This is a directory matrix, not 14
claims of equivalent native orchestration:

| Target ID | Directory relative to the selected home | Projection mode |
|---|---|---|
| `claude` | `.claude/skills` | Symlink to immutable generation |
| `opencode` | `.opencode/skills` | Symlink |
| `kimi-code` | `.kimi-code/skills` | Symlink |
| `minimax` | `.minimax/skills` | Verified copy |
| `cursor` | `.cursor/skills` | Symlink |
| `codex` | `.codex/skills` | Verified copy; explicit `CODEX_HOME` takes precedence |
| `gemini` | `.gemini/skills` | Symlink |
| `roo` | `.roo/skills` | Symlink |
| `devin` | `.devin/skills` | Symlink |
| `codeium-windsurf` | `.codeium/windsurf/skills` | Symlink |
| `agents` | `.agents/skills` | Symlink |
| `zed-config` | `.config/zed/skills` | Symlink |
| `zed` | `.zed/skills` | Symlink |
| `cline` | `.cline/skills` | Symlink |

The lifecycle capability source, `shared/harnesses/capabilities.json`, declares
Claude Code, Codex, OpenCode and Kimi adapters. Skill discovery in the other
targets does not establish hook activation, MCP configuration or native team
binding. Every advertised adapter needs a real installed-harness entry-point
receipt. Prometheus Exec, provider routes, Docker and optional extensions have
their own narrower platform/artifact matrices; this table does not extend them.

## Version and compatibility policy

Freeze one compatible graph before distribution: exact pack source commits,
all gitlinks, component versions, generated bundle identity, binaries and
signatures, optional container digests, service-template versions and harness
versions. Retain the previous accepted graph. Never substitute a moving branch,
`latest` image, configured model alias or older version-only receipt for that
graph. New upstream commits receive an explicit compatibility disposition.

The owner-selected full skill-system baseline is
`ba5c4516fd72e60c21ab3bb1d09a63105aeb3c61`. It is not a memory-server SHA and
does not identify the pending phase edits. The candidate manifest currently says
`releaseVersion: 1.11.2` and `minimumActiveVersion: 1.11.0`; these are observed
source declarations, not an approved next tag or proof that every older version
has the current features. Change 03 owns the final graph and explicit approval
for protected version edits and tags.

Extension contract `1.0.0` is versioned independently from the pack. Its policy
is editorial PATCH, additive compatible MINOR and breaking MAJOR; validation
must reject an unsupported declared contract requirement with both versions
identified. A discovered endpoint is only a capability candidate until identity,
compatibility, authorization and the actual operation are established. Legacy
discovery names such as `SOVEREIGN_SYNC_SOCKET` do not make the pack the owner of
the answering daemon. Companion source requires pack contract `1.0.0`; signed
push schema `1.7` is a separate version domain. Installed interoperability is
pending the exact compatible freeze and final local gate.

For every other persisted or process contract, retain its explicit schema or
contract version in the release graph. Changes to learning envelopes, doctor
outcomes, task identities, signed-event schemas or installation receipts require
consumer reconciliation and a documented migration when behavior breaks. A
package patch number alone cannot make incompatible state safe to read.

## Reproducible installation and update

Obtain the approved immutable source/artifact set from its recorded origin;
verify commit, gitlink, artifact hash and signature/trust policy. Preserve lockfiles
and the approved build toolchain. A fresh clone must resolve the same dependency
graph; a locally dirty checkout is provenance for implementation, not a release
artifact. Generated packages and sites must come from that frozen source at the
final boundary. Keep source, bundle, signed receipt and installed generation IDs
together so an operator can establish what actually ran.

After release approval and backup, the canonical selected-target skills path is:

```bash
./install.sh --profile skills --targets detected --dry-run --non-interactive
./install.sh --profile skills --targets detected --non-interactive --yes
./install.sh --profile skills --verify --targets detected --non-interactive
```

Use an explicit comma-separated target list when that is the intended install
scope. `--targets all` is an explicit broader selection. Absent clients must not
be silently treated as installed. The root installer initializes the exact
profile-required imports and displays its mutation summary. `--best-effort`
never establishes certification. A verifier or installer exit code still needs
the installed generation and actual harness invocation evidence.

For a full native installation, use the approved full-profile installer on macOS
or Linux. Review native-service mutations separately:

```bash
bash scripts/install-mcp-services.sh --dry-run
bash scripts/install-mcp-services.sh --restart
```

These examples require Bash 4+; macOS's default Bash 3.2 is insufficient for the
service installer. Only templates actually shipped for the host are managed.
The installer can report an execution-service warning while returning success,
so capture each required service's artifact and real operation rather than
treating one exit status as all-service acceptance. Optional sharing/control
services belong to their owner and require explicit selection and compatibility.

For Codex, the full source policy sets all three values to false:
`[features].memories`, `[memories].generate_memories` and
`[memories].use_memories`. `shared/scripts/codex-memories-config.sh` preserves
unrelated TOML and archives the known v1/v2 summaries; it preserves raw memories,
`MEMORY.md` and databases. Python 3.11+ with `tomllib` is required for this editor.
An already running session can retain prior settings. Confirm the installed
configuration and start a new session before acceptance. The full Codex root
context uses local metadata with `digest_only`; lesson bodies, REST/pk retrieval
and knowledge gaps are excluded there. Role/Claude recall retains its own scoped
contract. Source edits alone do not prove installed isolation.

## Upgrade sequence and ownership

1. Record the current accepted graph, installed generation/previous pointer,
   selected target paths, service owners and private configuration locations.
   Keep credentials out of the public receipt.
2. Stop or coordinate writers for a consistent backup. Retain queued/uncertain
   operations, signed authority, identities, registry and prior artifacts as
   described below. An upgrade must not erase pending work.
3. Complete production changes, reconcile generated outputs and run the final
   local integration/review boundary in disposable homes and data roots.
   Only one Cargo/rustc process may build on the machine; use each worktree's
   normal target directory. Hosted test runs are not release evidence.
4. Obtain the protected-edit/tag, publication/merge and machine-rollout approvals
   required by that release. Install only the accepted graph into the selected
   surfaces; restart the affected services and harness sessions.
5. Record installed hashes/generation/configuration and exercise the advertised
   user path across its real collaborators. A port response, service load,
   `--version`, package listing or successful compilation alone is insufficient.

Change 12 owns this phase's consolidated gates, independent cumulative review,
approved publication, machine rollout and historical reconciliation. This
maintenance draft does not replace that boundary.

## Rollback

The full plugin installer has an explicit generation rollback:

```bash
node scripts/install-plugin-generation.js --rollback --targets <selected-IDs>
node scripts/install-plugin-generation.js --verify --targets <selected-IDs>
```

It requires existing current and previous generations, a trusted signature and a
previous version satisfying `minimumActiveVersion`. It restores selected target
projections and signed receipts, verifies dispatchers, then exchanges the
pointers. Keep both generations and their evidence until rollback acceptance.
If no eligible previous generation exists, restore the recorded accepted bundle
through its normal installer; do not invent receipts or lower the minimum to
force an unsafe rollback. Reload harnesses after the switch.

Services/binaries and optional containers are separate rollback units. Stop
affected writers, restore the previous approved artifact/configuration and use
that component's installer. Do not restore an old database over new accepted
writes just because a binary was downgraded. A backward-incompatible schema needs
its documented recovery path or isolated restore; preserve the newer state and
queues for reconciliation. Plugin rollback does not roll back signed KBD history,
learner state, provider-side writes or an independently installed Companion.

`./install.sh --uninstall --targets <selected-IDs> --non-interactive --yes`
removes only receipt-owned selected surfaces. Before deliberate removal, archive
generation/trust/receipt/recovery history. Do not delete user collisions,
unrelated skills, databases or identity material as a repair.

## Optional Companion control and recovery

Companion has separate `docs/installation.md`, `docs/control-api.md` and
`docs/maintenance.md` in its selected source checkout. They are the source
references for its service/Claude installers, signed push protocol and recovery.
No public Companion remote or download URL is available for this candidate;
change 14's publication owner supplies those links after the approved freeze.
The pack does not install or require Companion. Its local signed KBD authority
and service owners remain independent.

Choose one host for a control socket: `prometheus-companion-headless`, the
Tauri-hosted node, or `sovereign-sync --mode daemon`. Server mode disables P2P;
MCP mode is a stdio client of the selected existing Unix host, not another host.
Its sync client does not fall back to TCP, use proxies/redirects or create a
second identity. The pack's legacy endpoint discovery is a separate contract;
it does not grant this client additional transports.

| Selected resource | Companion source resolution |
|---|---|
| Journal/registry and MCP intent data | `PROMETHEUS_DATA_DIR`, otherwise platform data-local root |
| Unix socket | Standalone `--socket`, then `SOVEREIGN_SYNC_SOCKET`, then `<data-local-dir>/prometheus/run/sovereign-sync.sock` |
| Sync config | Standalone `--config`, then `SOVEREIGN_SYNC_CONFIG`, then `XDG_CONFIG_HOME/sovereign-sync/config.toml`, then `~/.config/sovereign-sync/config.toml` |
| Device signing key | `PROMETHEUS_DEVICE_KEY_FILE`, then managed XDG-style `sovereign-sync/device-key.json`; credential-store fallback belongs to interactive contexts |
| P2P identity | Selected config's `node.p2p_identity_file`, default XDG-style `sovereign-sync/p2p-identity.json` |
| Headless/windowed skills | `PROMETHEUS_SKILLS_DIR`, then `CLAUDE_CONFIG_DIR/skills`, then `~/.claude/skills`; standalone uses config `node.skills_dir` |
| Pack service supervision | Explicit `PROMETHEUS_PACK_ROOT/shared/services.manifest.json`; absent supervision reports `[]` |

Changing the data root does not relocate the socket or keys. Preserve matching
host/client overrides, project identity, enrolled signer and peer membership.
An invalid explicit key/config fails instead of selecting another identity.
Headless key preflight requires a readable regular non-symlink private file;
access preflight alone does not establish key validity, enrollment or authority.

The supplied service installer owns macOS `ai.prometheus.companion` only and
retains the prior plist privately. The Claude installer uses user-scope MCP CLI
registration, with an explicit installed binary and private ownership receipt
under `<Claude-config-dir>/.prometheus-companion/`. It preserves unrelated
configuration and edited/unowned entries. Dry-run contracts describe no writes
or CLI calls; their real acceptance remains pending. There is no supplied Linux
service installer, native Windows packaging or other-harness installer. Linux
Unix headless source needs its own accepted artifact/platform gate. The Tauri
tray observes five service-health states; the main window is a starter, with no
dashboard, pairing UI or mobile application promised. Desktop Drop cancellation
does not establish awaited shutdown; headless has explicit bounded shutdown
whose SIGTERM/crash recovery still needs final evidence.

Before MCP submits `POST /api/v2/sync/pushes`, it privately fsyncs the exact
signed request and selected endpoint to
`<data-root>/prometheus/companion/mcp-push-outbox/<request-id>.json`. Preserve
that intent with the original request ID/body, host and receipts in backups.
Failed intent recording is `not-submitted`; response loss after submission is
uncertain. Query `GET /api/v2/sync/pushes/{request_id}` on the original host
before electing an authorized exact replay. Do not create a new ID, signature,
frontier or body to settle uncertainty. Intent is not acceptance; a local
`broadcast` receipt is not peer application. Preserve newer signed history and
successful receipts when rolling back the host binary, and stop the one socket
owner before replacement. Network pairing and status metadata do not authorize
canonical mutation. Surreal-memory remains local-only and must not be pushed.

## Data backup and recovery

Record the selected data roots rather than assuming everything lives in the
repository. Full KBD authority uses `<data-root>/prometheus/kbd`, where
`PROMETHEUS_DATA_DIR` or the platform-local data directory selects the root.
Retain its `registry.json`, `projects/<project-id>/project.loro`, replica
`events.jsonl`, archive segments, receipts, checkpoints and complete run history.
Retain the repository's `.prometheus/project.json` identity and the original
device enrollment/key references. OS credential-store keys require their own
secure recovery/export procedure; copying the public reference is not a backup
of a private key. A legacy file-key installation must preserve the existing
protected key file without exposing it in evidence.

Also preserve native SurrealDB data, knowledge state, learner-model state,
learning queues/outboxes, plugin generations/receipts and private service/provider
configuration. The service guide identifies their owners and default paths.
Record memory endpoint, namespace, database and embedding model/dimension. Full
native `memory/mcp` and mini Compose `memory/main_local_384` are different stores.
An embedding cache is not a database backup; changing a cache/model identity does
not migrate stored vectors.

Stop writers and take a consistent snapshot with an inventory/hash receipt,
timestamp, source/artifact graph, schema identities, revision/frontier and key
references. Keep private data in private storage. Restore into an isolated
destination first, retain original bytes, then check permissions, identity,
schema/model compatibility and actual read/write/recall behavior at the final
local gate. Recover/enroll through the supported identity path; never generate
replacement keys merely to make a signed journal open. Do not rewrite signatures,
task IDs, boundary starts, accepted events or missing receipts.

### Projection-only migration

The new production CLI source exposes:

```bash
prometheus kbd --path <existing-project> migrate --projections --dry-run
prometheus kbd --path <existing-project> migrate --projections
```

The first is an inventory; the second explicitly applies projection migration.
`--projections` conflicts with legacy `--check`/`--apply`; `--dry-run` requires
`--projections`. The legacy migration modes keep their separate behavior. These
new entry points remain uncompiled/unaccepted until the final gate.

The new opener reads existing identity/registration and authority without normal
startup. Dry-run creates no identity, registry, lock or directory and performs no
checkpoint or persisted recovery. It reports each planned disposition and any
pending archive transaction. Ordinary snapshot/startup paths do not provide this
same no-write promise.

Apply is Unix-only in the current source. It requires the existing initialized
replica `runtime.lock` and `project.loro.lock`, holds the replica exclusively and
the authority document shared, and refuses a journal ahead of or disagreeing with
Loro authority. Missing initialization or authority recovery must be handled
separately and explicitly. Do not run projection migration as an initialization,
journal migration or speculative legacy import.

For each safely inventoried unmarked `progress.json`, matching canonical content
is adopted. Comparison ignores only six top-level ownership metadata fields:
`generatedBy`, `projectionContractVersion`, `sourceRevision`, `derivedRevision`,
`last_updated`, `last_updated_by`. IDs, schema, frontier, task status, counts and
unknown content must otherwise match. A known phase with different or malformed
content is archived before installing its actual canonical projection. An unknown
phase is `unknown-phase-archived`: exact original bytes are retained with no
invented phase or replacement. Already owned unregistered files are preserved.
Symlink/path-escape or ambiguous provenance failures preserve data and require
operator resolution.

Archives live under
`.kbd-orchestrator/archives/projection-migration/<transaction-UUID>/`.
`original.json` and the durable `receipt.json` precede displacement of the live
file; `displaced.json` retains the moved inode. Receipts retain original/canonical
hashes, canonical revision/frontier, disposition, archive path/hash and transaction
state. Unknown-phase receipts have no canonical phase/hash. Keep the entire
transaction directory, including both original and displaced data.

After interruption, repeat dry-run to see pending transactions. Apply finalizes
completed work or restores the actual displaced bytes before replanning against
current authority. It never blindly installs stale canonical content, overwrites
a concurrent writer or discards modified temporary data. Preserve archive receipts
even after recovery. A manual restoration of historical projection bytes must be
an explicitly documented private recovery action with the archive retained;
routine projection overwrite guards remain strict. Neither adoption nor archival
creates a canonical event, advances revision/frontier or closes a historical task.

## Security, dependency maintenance and support

The release maintainer owns the full compatible graph and both source/public
documentation surfaces; each repository owner owns its binary/service contract.
Memory and knowledge maintainers own their stores and schemas, liter-llm owns
provider routing, upstream SurrealDB owns its pinned engine, and Companion owns
its optional endpoints/installer. Maintainers must refresh pin dispositions and
security advisories before every release and when a relevant vulnerability or
behavioral regression is reported. No SLA or unverified automatic updater is
promised here.

Use loopback bindings unless the component supplies an explicitly accepted
remote security model. The current memory REST/MCP endpoints do not provide
application authentication; scoped `user_id`/`agent_id` fields are not a network
authorization boundary. Native database development credentials and private
provider/identity files must not be published. Dependency fixes need the same
compatible-graph, complete-implementation and final-integration discipline as
features. Retain original upstream revisions, licenses and attribution.

For a regression, retain the exact source/artifact/generation IDs, platform and
harness versions, entry-point command, configuration shape, operation/receipt ID,
expected/observed result and uncertainty. Redact tokens, lesson bodies and keys.
Route installation/projection failures to the pack owner, service/store failures
to the owning repository and optional endpoint failures to the extension owner.
Use the original canonical blocker/task ID. A partial result, unavailable
provider, missing receipt or failed independent-review route remains pending.

Known limits include the unaccepted candidate matrix, Unix-only projection
migration, narrower service-template/Exec coverage, unavailable native harness
proof, uncertain remote writes without guaranteed exactly-once delivery, and
unpublished Companion artifacts/unaccepted operations. The full and mini packs must not shadow each
other in one native home. Maintenance readiness requires all shipping requirements
and the final phase evidence; source convergence or a completed change flag does
not establish it.
