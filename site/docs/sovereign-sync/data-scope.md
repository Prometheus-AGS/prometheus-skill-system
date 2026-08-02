---
id: data-scope
title: Exactly What Syncs
sidebar_label: Exactly What Syncs
---

# Exactly What Syncs

Sovereign Sync is domain-based, not directory-based. A domain owner must:

1. name the domain;
2. assign a privacy class and storage-key prefix;
3. serialize its state into a CRDT document;
4. export a snapshot or delta;
5. send a domain/version envelope;
6. import and persist the delta on the peer.

Nothing is included because it happens to be under a familiar directory.
There is no recursive scan of a repository, home directory, Karpathy wiki, or
KBD phase tree.

:::warning Current answer in one sentence

In `sovereign-sync 0.1.0`, **no project or global application data is
automatically synced by the daemon**. The domain and CRDT building blocks exist,
but the daemon’s P2P sender/receiver is not connected to any real data producer.

:::

## Implemented domain primitives versus daemon behavior

| Domain or family | Code that exists | Intended/recommended classification | Automatically transmitted by `0.1.0`? |
|---|---|---|---:|
| `skill-index` | Example manifest entry and local MCP search index | `Public` metadata | **No** |
| `learner-model` | Local CRDT documents, typed store, merge API, manifest tests | `Trusted` | **No** |
| `surreal-memory` | Manifest/privacy rejection tests | `Local` | **No; rejected in tested CRDT path** |
| `kbd-control:<project-id>` presence | Project-scoped Loro presence document | `Trusted` after peer authorization | **No** |
| KBD authoritative commands/events | Flocked, fsynced journal + signed event runtime | Single writer during project-document migration | **Local only until authoritative Loro sync is enabled** |
| `open-spec` | Advertised in the REST scaffold response | Trusted project data would be appropriate | **No adapter or daemon registration** |
| `kbd-orchestrator` | Advertised in the REST scaffold response | Split authored artifacts from command authority | **No adapter or daemon registration** |
| `kb:<name>` | Generic custom-domain model in `storage-provider` | Explicit `Public`, `Trusted`, or `Local` decision | **No daemon adapter** |

`POST /api/v1/sync/push` accepts any domain string and echoes
`{"status":"queued"}`. It does not validate the domain against a live manifest,
read a source, export a delta, call `P2PNode::broadcast`, or confirm delivery.

## Project identity and isolation

Each canonical KBD project has an immutable UUID in:

```text
<project>/.prometheus/project.json
```

Clones of the same logical project must keep the same `projectId`. The canonical
local KBD runtime is keyed by that value:

```text
<platform-data-root>/prometheus/kbd/projects/<project-id>/
```

The operator gossip topic is broader: every project using one `operator_id`
would use the same topic. A completed replication protocol therefore needs a
domain envelope that includes the project identity and rejects cross-project
payloads. The current raw gossip layer has no such daemon-wired envelope.

Do not create two different `.prometheus/project.json` files for two clones and
assume a matching operator ID will join their KBD state. Conversely, do not
assume a shared project ID makes two different operator topics discover each
other.

## Project-scoped data inventory

The table below inventories the main Prometheus state families an operator
normally means by “sync this project.”

| Data family | Representative paths/content | Current automatic sync | Correct ownership boundary |
|---|---|---:|---|
| Immutable project identity | `.prometheus/project.json` | No | Distribute once through Git or reviewed setup; never merge two generated IDs |
| Canonical KBD authority | platform data root: `events.jsonl`, `runtime.lock`, signed events | No cross-process sync yet | Journal is write-ahead ingestion; project Loro document becomes converged authority |
| KBD credentials | control token, device signing key | Never | Local secret/identity |
| KBD resume projections | `.kbd-orchestrator/current-waypoint.json`, `.md`, `position.json`, `position-reminder.txt` | No | Derived from canonical KBD revision or authored summary |
| KBD phase lifecycle | `phases/<phase>/progress.json`, goals, assessment, analysis, plan, execution, reflection, tasks, evidence, handoffs, decision logs | No | Project state; future adapter must separate authored artifacts from authoritative commands |
| KBD changes | `.kbd-orchestrator/changes/<change>/change.md`, `tasks.md`, `tasks.json`, execution evidence | No | Project work products; often also tracked by Git |
| KBD goals | `.kbd-orchestrator/goals/<goal>/goal.json`, `STATE.md`, `CONTROL.md` | No | Project loop state |
| Outer loops | `.kbd-orchestrator/loops/<name>/loop.json`, `journal.md`, `decision-log.md`, elicitations | No | Project standing-loop state |
| OpenSpec | `openspec/specs/`, active `changes/`, archived changes | No | Reviewed project specs; Git is the current transport |
| Iterative evolver | `.evolver/registry.json`, `evolutions/<name>/state.json`, plans, reports, checkpoints, history, model-routing logs, learning signals | No | Project strategic-loop state |
| ZeeSpec | `.zeespec/registry.json`, subject state, manifests, coverage scores, checkpoints, history, pending requests, model-routing logs | No | Project constraint/interrogation state |
| Skill creator | `.creator/registry.json`, per-skill state, checkpoints, workflow triggers | No | Project generation-loop state |
| Forge/Karpathy reflection | `.forge/enriched/`, `.forge/memory/iterations/`, `.forge/memory/drift/`, constitution and project skill overrides | No | Project enrichment/reflection state |
| Project Karpathy KB | `.prometheus/knowledge/raw/`, `wiki/`, `wiki/index.md`, `wiki/log.md` | No | Human-readable project knowledge; requires filtering before any future `Trusted` domain |
| Karpathy librarian events | `.prometheus/events.jsonl`, `events-kg.jsonl`, `events-episodic.jsonl`, `events-unsorted.jsonl` | No | Project learning/audit events |
| Project traces | `.prometheus/traces/<skill>/<timestamp>.json` | No | Potentially sensitive execution telemetry |
| Source, tests, docs, skills | ordinary repository files | No | Git/package distribution, not Sovereign Sync |

Some project paths are intentionally committed and some are ignored. That Git
choice does not change their Sovereign Sync classification.

## All loop levels

Prometheus uses several nested loops. “Sync the loops” must be broken down by
authority and merge behavior.

### Session/task execution

Raw prompts, model transcripts, tool payloads, temporary files, and agent
conversation history are not declared sync domains. Session summaries may feed
the Karpathy pipeline, but the raw conversation is not a substitute for a
reviewed learning artifact.

**Current sync:** none.

### KBD tactical loop

KBD covers global phase, OpenSpec change, and artifact QA granularity. Its
canonical command history is ordered, signed, revisioned, leased, and fenced.
Compatibility projections and authored Markdown live under
`.kbd-orchestrator/`.

Authoritative events move through signed project-scoped Loro deltas once the
project-document migration is enabled. The local journal remains the fsynced
write-ahead ingestion log.

**Current sync:** local single-writer journal authority only; authoritative P2P
sync is not yet enabled in the deployed service.

### Iterative evolver and strategic loops

The L2 evolver records assessment, plan, execution results, reflection,
checkpoints, history, reports, model routing, and learning signals in
`.evolver/`. It can bridge results into KBD through
`evolver-bridge.json`.

**Current sync:** none. If the files are committed, Git carries the reviewed
version.

### Standing outer loops

The L3 outer loop records the definition, cadence, tick journal, decision log,
feedback digests, and elicitation checkpoints under
`.kbd-orchestrator/loops/`.

**Current sync:** none. Two machines ticking one loop concurrently can create
semantic conflicts even if Markdown could be merged; future coordination needs
one lease/fence owner.

### ZeeSpec, creator, and artifact QA loops

ZeeSpec’s interrogation state, constraint manifest, coverage scores, and
history live under `.zeespec/`. Skill-creator checkpoints live under
`.creator/`. Artifact-refiner state can be provider-backed or project-local.

**Current sync:** none.

### Feynman learning loop

The learner model is a real CRDT-backed data model. For each learner it stores:

- learner ID and timestamps;
- concept IDs, labels, and mastery estimates;
- append-only scored observations with source skill and vector clocks;
- knowledge-gap records, severity, evidence, and resolution timestamps;
- learning-session IDs, time bounds, skills called, and concepts touched;
- per-concept FSRS stability, difficulty, due date, state, repetitions,
  lapses, and last review.

The default local directory is:

```text
$HOME/.prometheus/learn/learner-model/
```

The storage key is `learner/<learner-id>/model.crdt`.

**Current sync:** local storage and merge API exist; Sovereign Sync does not
open this directory or send its deltas.

### Karpathy knowledge and reflection loop

The Karpathy-pattern path spans more than the Markdown wiki:

1. `pk focus` reads project/global wiki context;
2. Forge enrichment records focused context with an iteration;
3. `forge reflect` produces project reflection/drift records;
4. `pk ingest` compiles durable wiki entries;
5. librarian events record compile, focus, lint, and update activity;
6. session evaluation writes structured global learning logs;
7. the skill-update proposer writes a human-reviewed candidate.

Project wiki, event, and Forge paths are listed above. The device/global paths
are listed below.

**Current sync:** none of these paths are connected to the daemon.

## Device/global data inventory

| Data family | Representative location | Current automatic sync | Policy |
|---|---|---:|---|
| Global Karpathy KB | `$HOME/.prometheus/knowledge/{raw,wiki}/` | No | Cross-project local knowledge unless explicitly curated |
| Shared Karpathy KB | `$HOME/.prometheus/knowledge/shared/` | No | Deliberately shared across local projects; not automatically shared across machines |
| Session learning log | `$HOME/.prometheus/learning-log/*.jsonl` | No | Can include mistakes, paths, and operator context |
| Skill-update candidates | `$HOME/.prometheus/skill-updates/` | No | Human-gated diffs; distribute only after approval |
| Last session summary | `$HOME/.prometheus/last-session-summary.txt` | No | Device-local working summary |
| Hook log | `$HOME/.prometheus/hooks.log` | No | Device-local diagnostic data |
| Global traces | `$HOME/.prometheus/traces/<skill>/<timestamp>.json` | No | Potentially sensitive telemetry |
| Learner model | `$HOME/.prometheus/learn/learner-model/` | No | `Trusted` is appropriate only after explicit user/device policy |
| `surreal-memory` | local SurrealDB graph, vectors, tasks, Memory Palace | No | `Local`; do not export through P2P |
| Installed skill trees | tool-specific global skill directories | No | Reinstall/update from the canonical skill pack |
| Sovereign config | `$HOME/.config/sovereign-sync/config.toml` | No | Copy only the chosen `operator_id`; preserve machine-specific settings |
| Device signing key | platform credential store or `device-key.json` | Never | Unique secret per machine |
| KBD control tokens | one token per canonical project runtime | Never | Local loopback API credential |
| Service logs | `$HOME/.prometheus/logs/` | No | Local diagnostics |

Global scope does not mean “safe to send to every device.” It means the data is
owned by the device/user rather than one repository. Every future global domain
still needs an explicit privacy and peer policy.

## What a completed scope should look like

A safe implementation should use narrow domains rather than one “everything”
payload:

| Proposed domain shape | Content | Merge/authority model |
|---|---|---|
| `skill-index` | names, versions, descriptions, source hashes | Public CRDT/index rebuild |
| `learner-model:<learner-id>` | typed learner CRDT only | Trusted CRDT merge |
| `approved-kb:<project-id>` | reviewed, sanitized project wiki entries | Trusted CRDT or content-addressed docs |
| `kbd-presence:<project-id>` | device/harness/session/revision presence | Trusted ephemeral CRDT |
| `kbd-authority:<project-id>` | signed commands, replicas, project document | Authenticated Loro deltas |
| `openspec:<project-id>` | reviewed specs/change state | Trusted, project-scoped adapter |
| `loop:<project-id>:<loop-id>` | definition plus single-writer tick results | Lease/fence plus structured merge |

`surreal-memory`, secrets, raw transcripts, unreviewed prompts, and service logs
should remain outside those domains.

## How to verify actual replication when it is implemented

An end-to-end proof must include all of the following:

1. live peer identity and authorized topic membership;
2. named domain and project/learner identity;
3. source version vector or committed KBD revision;
4. bytes exported and transmitted;
5. destination manifest/trust decision;
6. destination import/commit result;
7. destination version/revision after apply;
8. content-level assertion on the expected record;
9. negative assertion that `Local` and secret data did not move.

The current health, peer, status, and push responses do not provide this proof.
Until they do, use Git or another explicit reviewed transfer for project
artifacts and treat global state as device-local.
