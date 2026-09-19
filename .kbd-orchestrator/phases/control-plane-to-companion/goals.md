# Goals

> Phase: `control-plane-to-companion`
> Created: 2026-09-01 by `/kbd-new-phase` (runtime revision 386)
> Amended: 2026-09-02 by operator decision D-02 (final model, see below)
> Predecessor: `kbd-control-plane-recovery` (9/9 changes, reflect not yet run)
> Next command: `/kbd-spec control-plane-to-companion`

## Phase objective

Move the shared control plane, decentralized functionality, CRDT replication
and P2P sync, discovery, and cross-machine orchestration, monitoring, and
run-guarantee out of `prometheus-skill-pack` and into `prometheus-companion`,
which owns the Axum-based node that provides all of that plus management,
configuration, and monitoring of the skills and their services in a single
manageable way.

## Governing model (operator decision D-02, 2026-09-02)

- **The pack is open source and complete on its own.** Skills, hooks, the
  `prometheus` CLI, kbd-runtime as the local signed KBD authority, and every
  functional service stay in the pack. Every kbd-* skill works fully with no
  Companion present. The pack must certify fully without the Companion.
- **The Companion is the paid control and extension framework.** Sync, CRDT
  replication, P2P, discovery, orchestration, monitoring, and the supervisor
  that guarantees services are running move there and become its product.
  Third parties extend the pack the same way the Companion does: from their
  own repositories, through a contract the pack publishes.
- **The pack never depends on the Companion**, at build time or install time.
  The Companion consumes the pack's open crates by git rev; nothing flows the
  other way. No pack crate may import a paid crate.
- **Optional capability is discovered at runtime, never assumed.** With the
  Companion present, kbd-* gains sync and remote signed commands through the
  discovered endpoint. Absent, nothing changes and nothing warns.
- kbd-runtime uses Loro as the local storage format for the signed journal;
  that is function the CLI needs and stays. CRDT *replication* moves.

## Source inventory (verified 2026-09-01, dispositions final 2026-09-02)

| Today in `prometheus-skill-pack` | Kind | Disposition |
|---|---|---|
| `substrate/sovereign-sync` (daemon, REST/MCP/SSE, multi-project router, remote signed commands, iroh + iroh-gossip P2P, Loro replication) | Rust crate + daemon | **moves** in full, including `--mode mcp` |
| `substrate/sovereign-client` (reqwest + SSE SDK) | Rust crate | **moves** |
| `substrate/storage-provider` `iroh_docs.rs` adapter | module | **moves**; trait, `local_dir`, `loro_adapter`, `sync_manifest` stay |
| `substrate/kbd-mobile` (mobile sync wire) | Rust crate | **moves**; `skill-ffi` drops or feature-gates its dependency without importing a paid crate |
| `shared/launchagents/ai.prometheus.sovereign-sync.plist`, `shared/systemd/ai.prometheus.sovereign-sync.service`, `--sharing` install path | service wiring | **removed** from the pack; the Companion installs its own node |
| `skills/learn/sync-status`, `sync-peers`, `sync-push` | manifest skills | **move** to the Companion's skill plugin (first worked example of third-party extension) |
| `substrate/kbd-runtime` (local signed authority, journal, projections, device key via keyring) | Rust crate | **stays**; consumed by the Companion by git rev |
| `tools/prometheus-cli` (`prometheus` binary, `kbd` commands, transport chain) | CLI | **stays**; its control-endpoint discovery becomes part of the published contract |
| `substrate/skill-index`, `learner-model`, `skill-ffi`, `exec-*`, `prometheus-research`, `surface-bridge` | Rust crates | **stay** |
| exec, forge-mcp, liter-llm-api, pk-cherry, surface-bridge, surreal-memory-native, surrealdb-native, research, learning-worker, codex-skills-sync, hooks-logrotate, prometheus-nudge and their LaunchAgent/systemd templates | services | **stay** where they are, launchd/systemd-managed; the Companion adopts and manages them through the contract |
| kbd-* skills, hooks, `runtime/v1/run-hook` bundles | skills + hooks | **stay**, fully functional without the Companion |

Destination: `prometheus-companion` is a Tauri 2 scaffold with a spec
(`docs/00-architecture-and-implementation-plan.md` §14, §16, §18) that already
commits to sovereign-sync + iroh, an Axum boundary, a service supervisor, and
connected skill packages (§18.5, §18.6). It has no `crates/` and no Axum
code yet. Kademlia-style discovery exists in neither repo today; iroh's
Mainline DHT address lookup supplies it (analysis D-01).

## Goals

- Inventory every control-plane, CRDT sync, P2P, and discovery capability now in prometheus-skill-pack and record an explicit ownership boundary per the governing model above: what moves to prometheus-companion, what stays as the open-source operational core, what is deleted
- prometheus-companion owns a single Axum-based node that provides the shared control plane (multi-project router, remote signed commands, REST/MCP/SSE) over the pack's local signed KBD authority, Loro CRDT replication, and iroh QUIC P2P transport, replacing the sovereign-sync daemon as the authoritative shared process
- Cross-machine discovery and membership run inside the companion node via iroh Mainline DHT address lookup (Kademlia) plus mDNS and iroh-gossip membership, with device identity and pairing as the admission rule, so peers are found and managed without hand-maintained peer lists
- The companion node exposes unified management, configuration, and health monitoring of every skill-pack service (exec, forge-mcp, learning-worker, liter-llm-api, pk-cherry, surface-bridge, surreal-memory, surrealdb, research, codex-skills-sync) as one manageable surface consumed by the Companion dashboard and tray, by adopting the services where they are rather than relocating them
- prometheus-skill-pack stays fully capable without the Companion: kbd-* skills, the prometheus CLI, hooks, and every service work unchanged; sovereign-sync, sovereign-client, kbd-mobile, the iroh-docs adapter, the sharing LaunchAgent/systemd unit, the `--sharing` install path, and the three sync-* skills leave the pack; existing signed KBD state (project.loro, receipts, run history) migrates intact
- The pack publishes an open, versioned integration contract that the Companion and third-party repositories use to extend it: control-endpoint discovery in the CLI transport chain, hook bundle extension points, a service manifest the supervisor can adopt, and the connected-skill-package declaration; the Companion is its first consumer and the sync-* skills its first worked example
- Full-integration evidence: two-node runs prove discovery, CRDT convergence, a signed KBD command delivered over P2P, and service management through production entry points, and the pack certifies fully with the Companion absent; install scripts, plugin manifests, and docs updated on both repos

## Constraints carried into this phase

- Implementation-first, integration-only evidence (CLAUDE.md highest-precedence
  policy). No unit tests as delivery evidence; two-process integration is the gate.
- One Cargo build at a time machine-wide; separate `target/` per workspace.
- Existing signed KBD state (`project.loro`, receipts, run history, revision 386+
  frontier) must migrate intact. A migration that discards evidence is a failure.
- The Companion is a renderer for A2UI and a sovereign-sync node, never an
  authority over agent execution (spec §13, §14). The pack's kbd-runtime remains
  the local KBD authority; the node provides the shared control plane over it.
- Companion's `versions.toml` `[pins]` is empty and `Edit(versions.toml)` is
  denied to agents there; new crate pins are a human edit.
- HMA `compatibility/prometheus-control-plane.json` (`lifecycle: prometheus`)
  stays valid because kbd-runtime stays in the pack.
- sovereign-sync's MIT history is already public and remains public after the
  move; licensing of the moved code in the Companion is the operator's call.

## Out of scope

- Relocating any functional service, the CLI, or kbd-runtime (governing model).
- Companion UI work beyond what the node's management surface requires
  (dashboard/tray polish belongs to the Companion's own phases).
- Mobile FFI parity beyond keeping `skill-ffi` compiling without `kbd-mobile`.
- Re-litigating iroh over libp2p; the Companion spec already settles this.

## Open questions for the assessment (status after analyze)

1. Cargo workspace: resolved, one root workspace `crates/<name>` + `src-tauri` (D-04).
2. DHT in v1: resolved, iroh Mainline address lookup, off by default with per-operator opt-in (D-01, D-13).
3. Which repo's KBD run owns the cross-repo work: open for spec (assessment Q3).
