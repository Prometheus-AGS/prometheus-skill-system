---
id: use-cases
title: Real-World Use Cases
sidebar_label: Real-World Use Cases
---

# Real-World Use Cases

Sovereign Sync is valuable when the state needed to resume work is richer than
a Git commit. These examples show the intended operator/user benefit, the data
boundary that makes the scenario safe, and what is possible in the current
release.

## 1. Desktop-to-laptop project handoff

**Situation.** An operator spends the morning on a Linux workstation running a
KBD phase. Before traveling, they switch to a laptop and want to resume at the
exact next change rather than reconstruct the session from chat history.

**What continuity should include.**

- immutable project ID;
- current phase/path and source revision;
- phase plan, progress, evidence, decisions, and handoffs;
- active OpenSpec change and task state;
- outer/evolver checkpoints relevant to the same project;
- approved project Karpathy entries produced by the morning reflection.

**Benefit.** The laptop can answer “what is next, why is it next, and what has
already been proven?” without replaying a transcript. Repeated discovery and
duplicate work decrease, and the decision log survives a harness change.

**Authority rule.** The currently deployed runtime accepts KBD writes through
one local journal authority. Authored Markdown can be replicated or carried by
Git, but signed KBD commands must follow that ordered path.

**Today.** Signed `kbd-control:<project-id>` updates can move the authoritative
Loro project document between enrolled peers. Pause remains advisory: use a
deliberate handoff and confirm the destination's applied receipt/frontier
before resuming writes. Git still carries reviewed authored artifacts.

## 2. One user, different AI harnesses on different machines

**Situation.** A developer uses Codex on a MacBook for implementation and
Claude Code on a larger workstation for research and review.

**What continuity should include.**

- a shared project identity;
- distinct device signing keys and harness/session actor identities;
- one ordered KBD command chain;
- task handoffs and committed revision;
- project-scoped context, not raw conversations.

**Benefit.** Each harness sees the same canonical state rather than maintaining
its own interpretation of `progress.json`. Audits show which device and harness
claimed, revised, paused, or released the work.

**Security rule.** The devices share a random group secret through pairing, but never share signing
keys. Actor identity remains attributable.

**Today.** Each replica writes through one fsynced journal lock and imports into
the shared project document. Signed cross-machine Loro pushes require the same
group secret plus an endpoint-to-signing-key allow-list; receipt replay and
frontier checks prevent ambiguous duplicate execution.

## 3. Feynman study continuity between work and home

**Situation.** A user practices Rust ownership at work, then reviews the topic
at home. The home device should know which misconceptions were detected and
when the next retention review is due.

**What the learner domain should include.**

- the same logical learner ID;
- mastery per concept;
- scored observations and source skills;
- open/resolved gap records and grounding evidence;
- FSRS stability, difficulty, due date, state, repetitions, and lapses;
- session summaries identifying concepts touched.

**Benefit.** Review scheduling follows the learner rather than one device. A
gap closed at work does not reappear as an unknown at home, and the home
session’s observation can improve the next work-session plan.

**Privacy rule.** Sync the typed learner model, not the user’s raw prompts,
private notes, entire graph memory, or credentials.

**Today.** The learner model is stored as immutable, uniquely keyed Loro
evidence. The daemon adapter opens the configured learner-model directory and
sends deltas only after an explicit signed push. Folding remains commutative,
associative, and idempotent after local writes and imports.

## 4. Home workstation and travel laptop on different networks

**Situation.** One machine is behind a home router and the other is on hotel or
cellular internet.

**Network behavior.** Both machines use an exchanged endpoint ID and the same
operator topic. Iroh attempts a direct encrypted path after relay-assisted
contact and can remain on the relay when NAT/firewall policy blocks direct
QUIC.

**Benefit.** The user does not need to expose a home HTTP server, configure
dynamic DNS, or forward the KBD API port.

**Operational rule.** TCP `7892` stays on loopback. Public relay/discovery
metadata and availability must be acceptable for the deployment.

**Today.** Connectivity development is possible, but endpoint identity is
ephemeral and application data is not wired to the gossip path. Re-pair after
the anchor restarts.

## 5. Pair-programming or on-call handoff

**Situation.** Two trusted operators need a precise handoff during an incident
or a long-running migration. One person pauses and records the next step; the
other resumes from a different device.

**What should cross devices.**

- current revision and lifecycle;
- decision and evidence logs;
- actor and device provenance;
- pause/revise/resume/cancel events;
- exact next command and fallback;
- device enrollment/revocation state.

**Benefit.** The incoming operator does not act on a stale handoff or overlap
the outgoing operator. Duplicate commands are idempotent, stale revisions are
rejected, and the audit trail is preserved.

**Availability rule.** A receiving operator must be able to verify the durable
checkpoint and command history before resuming.

**Today.** Normal daemon startup supports one local writer. This use case is a
target, not a production-supported multi-machine workflow.

## 6. CI runner consumes public skill metadata

**Situation.** An ephemeral CI runner needs to know the canonical skill names,
versions, descriptions, and source hashes used by a project.

**What should sync.** A `Public` skill-index document containing metadata only.
The runner should install the actual versioned skill payload from the canonical
repository or package source.

**Benefit.** CI can compare the project’s declared skill catalog with the
operator machines without receiving private learning history, device keys,
or local paths.

**Security rule.** Do not pair a disposable runner into a `Trusted` topic that
also carries learner or approved private project domains. Prefer separate
operator namespaces or domain-level peer authorization.

**Today.** MCP search and `skill-index` pushes use the same deterministic index
implementation as generated agents, plugin generations, and mobile FFI. A
signed explicit push sends the manifest-approved public metadata domain.

## 7. Share distilled knowledge, keep raw memory private

**Situation.** Two machines should share a reviewed lesson—“run Cargo
formatting after clippy edits”—without copying the entire Surreal graph,
Memory Palace, embeddings, or session transcripts.

**What should sync.**

- an approved Markdown wiki entry;
- provenance, revision, tags, and links;
- no raw prompt, secret, personal note, or graph embedding.

**Benefit.** A durable operational lesson follows the operator while sensitive
episodic memory remains local. Human-readable Markdown keeps the shared claim
auditable.

**Privacy rule.** `surreal-memory` remains `Local`. A future
`approved-kb:<project-id>` or approved global-knowledge domain must filter and
classify content before export.

**Today.** Project/global Karpathy KBs remain outside the daemon's automatic
domains. Use a reviewed Git commit or explicit secure transfer for approved
entries; do not relabel raw Memory data as learner evidence.

## 8. Offline field work

**Situation.** A laptop works without internet for several days and later
rejoins the operator’s environment.

**Mergeable target.** Learner observations and approved knowledge can use CRDT
deltas after reconnection.

**Current limitation.** Two offline writers may produce causally conflicting
control decisions even though signed grow-only events can converge. Stale
frontiers are rejected and surfaced for explicit reconciliation rather than
silently choosing a winner.

**Benefit.** The system can preserve legitimate offline learning without
pretending that causal control conflicts are ordinary text conflicts.

**Today.** Iroh reconnects enrolled peers and explicit signed pushes reconcile
Loro deltas. Fully air-gapped deployments still need an approved discovery or
ticket-transfer design; use exported artifacts and an explicit KBD handoff
when the configured discovery fabric is unavailable.

## Choosing the right transport today

| Need | Use now |
|---|---|
| Source code, specs, reviewed docs, committed KBD artifacts | Git |
| Install the same skill payload/version | Canonical installer or package source |
| Copy one approved wiki entry | Reviewed Git commit or explicit secure transfer |
| Share raw secrets or device identities | Do not copy |
| Use one KBD authority across local harnesses | Local REST/MCP/CLI control plane |
| Prove iroh connectivity during development | Pairing plus live peer/status and signed receipt evidence |
| Replicate a supported runtime domain | Explicit signed v2 push with terminal per-peer receipt |

## Acceptance criteria for claiming a use case works

For any future release, validate the scenario at the application-data layer:

- both devices report the expected, distinct peer identities;
- the domain and project/learner IDs match;
- the source revision/version is known;
- the destination reports an applied revision/version;
- the expected record is present after apply;
- `Local` domains, credentials, raw transcripts, and service logs are absent;
- restart and reconnect behavior is tested;
- direct and relay paths are distinguishable in diagnostics.

Without those assertions, a green health check or queue acknowledgement proves
only local process behavior.
