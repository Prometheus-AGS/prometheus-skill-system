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

**Authority rule.** Only one machine may own a mutable KBD lease at a time.
Authored Markdown can be replicated or carried by Git, but signed KBD commands
must follow the ordered authority path.

**Today.** Use Git for reviewed project artifacts and explicitly hand off the
KBD lease/state. P2P pairing alone does not transfer the phase tree or KBD
authority in `0.1.0`.

## 2. One user, different AI harnesses on different machines

**Situation.** A developer uses Codex on a MacBook for implementation and
Claude Code on a larger workstation for research and review.

**What continuity should include.**

- a shared project identity;
- distinct device signing keys and harness/session actor identities;
- one fenced KBD command chain;
- task handoffs and committed revision;
- project-scoped context, not raw conversations.

**Benefit.** Each harness sees the same canonical state rather than maintaining
its own interpretation of `progress.json`. Audits show which device and harness
claimed, revised, paused, or released the work.

**Security rule.** The devices share `operator_id`, but never share signing
keys or control tokens. Actor identity remains attributable.

**Today.** The local KBD control plane already coordinates multiple harnesses
on one machine. Cross-process Raft transport is disabled, so two machines
cannot yet share one canonical command authority through Sovereign Sync.

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

**Today.** The learner model is stored as a Loro-backed document and exposes a
merge API, but the daemon does not open
`$HOME/.prometheus/learn/learner-model/` or send its deltas.

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
or a long-running migration. One person pauses and releases; the other claims
the next step from a different device.

**What should cross devices.**

- current revision and lifecycle;
- decision and evidence logs;
- active lease owner, expiry, and fencing token;
- pause/revise/resume/cancel events;
- exact next command and fallback;
- device enrollment/revocation state.

**Benefit.** The incoming operator does not act on a stale handoff or overlap
the outgoing operator. Duplicate commands are idempotent, stale revisions are
rejected, and the audit trail is preserved.

**Availability rule.** A production topology should use an odd number of
authenticated voters or an explicit witness policy. A two-node cluster alone
cannot preserve availability across one failure without weakening consistency.

**Today.** Embedded tests exercise quorum and fencing behavior, but normal
daemon startup supports one local voter. This use case is a target, not a
production-supported multi-machine workflow.

## 6. CI runner consumes public skill metadata

**Situation.** An ephemeral CI runner needs to know the canonical skill names,
versions, descriptions, and source hashes used by a project.

**What should sync.** A `Public` skill-index document containing metadata only.
The runner should install the actual versioned skill payload from the canonical
repository or package source.

**Benefit.** CI can compare the project’s declared skill catalog with the
operator machines without receiving private learning history, control tokens,
or local paths.

**Security rule.** Do not pair a disposable runner into a `Trusted` topic that
also carries learner or approved private project domains. Prefer separate
operator namespaces or domain-level peer authorization.

**Today.** MCP search builds a local index from `skills_dir`; no skill-index
CRDT producer is connected to P2P.

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

**Today.** Project/global Karpathy KBs are not daemon domains. Use a reviewed
Git commit or explicit secure transfer for approved entries.

## 8. Offline field work

**Situation.** A laptop works without internet for several days and later
rejoins the operator’s environment.

**Mergeable target.** Learner observations and approved knowledge can use CRDT
deltas after reconnection.

**Non-mergeable target.** KBD cannot accept two independent offline writers and
later combine their command histories. Only the lease-holding authority may
advance the canonical chain; the offline machine should work read-only or on an
explicit branch that is reviewed/replayed later.

**Benefit.** The system can preserve legitimate offline learning without
pretending that causal control conflicts are ordinary text conflicts.

**Today.** The current N0 endpoint-ID bootstrap is not suitable for an
air-gapped LAN, and the daemon does not transmit reconnection deltas. Use Git
branches/exported artifacts and an explicit KBD handoff.

## Choosing the right transport today

| Need | Use now |
|---|---|
| Source code, specs, reviewed docs, committed KBD artifacts | Git |
| Install the same skill payload/version | Canonical installer or package source |
| Copy one approved wiki entry | Reviewed Git commit or explicit secure transfer |
| Share raw secrets or device identities | Do not copy |
| Use one KBD authority across local harnesses | Local REST/MCP/CLI control plane |
| Prove iroh connectivity during development | Current pairing/log procedure |
| Automatically replicate project/global runtime state | Not yet available in `0.1.0` |

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
