# Decision Log — control-plane-to-companion

---

### 2026-09-02 — Build-vs-adopt decisions (kbd-analyze, revision 2 after adversarial review round 1)

| Decision | Verdict | Status | Provenance |
|---|---|---|---|
| D-01: Discovery | `iroh-mainline-address-lookup` (Kademlia DHT) + `iroh-mdns-address-lookup` + existing bootstrap list; admission stays Ed25519 pairing; libp2p-kad rejected (Companion spec L183 forbidden table, §25.2 tradeoff) | resolved | research (iroh 1.0.3 registry source, Companion spec, HMA peer-crdt.md) |
| D-02: Crate ownership and authority location | Options scored: A crates stay/Companion consumes by git rev (19/25), C third shared repo (16/25), B crates move/CLI repoints (10/25). A–C gap under 15%: contested, routed to operator | **resolved: option D, product boundary (operator, 2026-09-02; see final entry)** | research; scoring table in analysis.md §2.2 retained as a framing-error record |
| D-03: Node surface | Unix-socket REST/MCP/SSE stays external (TCP optional); Tauri IPC for the Companion UI only; Companion spec §15 must be amended at spec stage | resolved, spec amendment owed | research (CLI transport chain, sync-* skills, clawdesk precedent) |
| D-04: Workspace layout | one root workspace `crates/<name>` + `src-tauri`; hand-mirrored `[patch]` workspace rejected | resolved | research (HMA scaffold, graph-explorer, know-me failure mode) |
| D-05: Service kinds | v1: InProcess for pack axum libraries (sovereign-sync, skill-index, surface-bridge after router merge); ExternalSidecar for liter-llm, forge, pk-cherry, surreal-memory, surrealdb, and initially prometheus-exec and research; Companion spec §16.4/§25.2 must be amended | resolved, spec amendment owed | research (Companion spec §4/§16, pack tools/ submodule layout) |
| D-06: Device key store | incumbent `keyring = "=3.6.3"` in kbd-runtime, entry `prometheus-kbd-device`; keyring 4.2.0 rejected for this phase; keychain-ACL continuity between daemon and Companion binaries is build work | resolved | research (kbd-runtime Cargo.toml:29-35, lib.rs:2973/3419) |
| D-07: Operator credentials | tauri-plugin-stronghold 2.3.2 per Companion spec §17, scoped to operator/provider keys only | resolved | research (spec mandate, registry health) |
| D-08: State paths | node keeps `dirs_next` + `PROMETHEUS_DATA_DIR`; Tauri path resolver never used for shared state; device-key file fallback is `~/.config/sovereign-sync/device-key.json` (XDG), canonical is the credential store | resolved | research (Tauri path docs, kbd-runtime lib.rs:303-308, 2979-2982) |
| D-09: Outer supervision | launchd via rendered plist (HMA launchagent-supervisor) restarts the Companion; the node supervises children per D-05; tauri-plugin-autostart rejected | resolved | research (HMA skill, Companion spec §4.1/§25.1) |
| D-10: Pairing | iroh EndpointId auth + tickets; Noise attribution in Companion spec corrected; spake2 short code deferred to spec | resolved, spec correction owed | research |
| D-11: Harness | one node hosted via the Companion headless preset, one via desktop; extends domain_sync.rs | **superseded by D-14** (desktop node withdrawn; domain_sync.rs is not the gate) | research (Companion spec §4.2, Appendix A.2) |

One contested decision (D-02). It is a goal amendment, not a stack choice, so `pmpo-elicit` was not invoked; it is routed to the operator through the analyze handoff and must be answered before `/kbd-plan`.

---

### 2026-09-02 — Revision 1 errors corrected (kbd-analyze)

Revision 1 narrowed Goal 5 and reworded Goal 2 unilaterally, missed the incumbent keyring dependency in kbd-runtime, recommended external supervision of ten processes against a spec that specifies in-process children, cited a nonexistent HMA libp2p prohibition, misplaced the device-key file under `dirs_next::config_dir()`, left the iroh lookup crate names open when the lockfile and registry answered them, and claimed numeric score gaps with no scoring data. All corrected in revision 2; see `review/analyze/findings.round1.json`.

---

### 2026-09-02 — Budget-bounded item (kbd-analyze)

Whether `iroh-mdns-address-lookup` / `iroh-mainline-address-lookup` 0.5.0 (per review: crates.io, 2026-08-18) target iroh 1.0.x or require 1.1.0 was not confirmed; Tier 3 cap reached. Spec must confirm before any Cargo change. The pack's manifest floor is `iroh = "1.0.2"`, resolved 1.0.3.

---

### 2026-09-02 — Revision 3 after adversarial review round 2 (kbd-analyze)

| Decision | Verdict | Status | Provenance |
|---|---|---|---|
| D-12: MCP surface | keep a thin `sovereign-sync --mode mcp` stdio shim in the pack proxying to the node socket (recommended) or drop MCP mode and update installers and sync-* skills | open, spec | research (main.rs:11/244, install-skills-flat.sh:541-552, Companion spec §4.1 L269) |
| D-13: DHT publication default | Mainline lookup off by default, per-operator opt-in; Companion spec §25.2 row amended to state it | open, plan | research (exposure of signed pkarr records on a public DHT) |
| D-14: G6 gate | `domain_sync.rs` reclassified as library regression; the gate is a two-process harness through production entry points with a headless-hosted Companion node, run locally | resolved | research (domain_sync.rs:113-121, 183-185; CLAUDE.md integration-only policy) |
| D-06 amended | keychain continuity remedies are ACL trust (GUI), `PROMETHEUS_DEVICE_KEY_FILE` (headless), or re-enrollment; "access group" withdrawn | resolved | research (keyring-3.6.3/src/macos.rs) |
| Q2 closed | iroh address-lookup crates 0.5.0 depend on iroh ^1.0.0; no bump above 1.0.2 floor for discovery | resolved | per review (crates.io), Tier 3 cap reached |

Round-2 BLOCK accepted under the two-round cap; both CRITICALs appended verbatim to analysis.md. Review isolation was harness-native for all four rounds across assess and analyze; a cross-model re-run is owed when gateway quota resets.

---

### 2026-09-02 — D-02 interim: option B with the CLI, phased (SUPERSEDED the same day by the final entry below)

Options: A (crates stay, Companion consumes by git rev) vs B (crates move, CLI repoints) vs C (third shared repo) | Analyze scores: A 19, C 16, B 10 of 25
Decision: **B, extended to include the `prometheus` CLI, delivered in phases** | Provenance: **user** (Travis James, 2026-09-02) | Elicitation ID: inline (Claude Code chat, no pmpo-elicit checkpoint)

Operator rationale, recorded verbatim in substance: a skill pack is a skill collection, not a software component; it has no structure for owning, configuring, monitoring, or guaranteeing the 13-plus services the stack needs; the Companion is software with libraries, SDKs, and control mechanisms and is the right substrate for all control structures; keeping libraries in the skill package is the wrong substrate.

Scoring critique accepted: two of the five analyze criteria ("dependency direction preserved" and "HMA contract holds without revision") measured conformance to the status quo in which the pack owns software. Under the operator's architecture those criteria invert, so B's 10/25 was a verdict on the criteria's prior, not on B. The assessment evidence (seven fragmented health scripts, no aggregator, installer aborted by a submodule build, one-Cargo-build rule, fail-open hooks, script-bearing skills inert on mobile) supports the premise.

Consequences now binding on spec and plan:
- **Boundary:** the Companion owns everything with a build or a process; the pack ships skills, hooks, thin client scripts, and a declared minimum Companion version (a client contract, not a library).
- **This phase moves:** the sovereign-sync path-dependency closure (sovereign-sync, kbd-runtime, kbd-mobile, storage-provider, learner-model, skill-index, sovereign-client, skill-ffi) **and `tools/prometheus-cli`** (the `prometheus` binary and its workspace), plus supervision of all services (G4). Goals 2 and 5 stand as written; Goal 5 is amended to name the CLI.
- **Follow-on phase moves:** the remaining substrate crates (exec-*, prometheus-research, surface-bridge, learner-model consumers), the `tools/` submodule pins, `scripts/install-*.sh`, and `shared/launchagents`; the Companion becomes the installer (2026-08-20 review §2.1). Named here so the boundary is deliberate, not drift.
- **Product decision recorded:** the Companion becomes a hard prerequisite for kbd-* and sync-* skills; manifest-only skills still run without it, kbd-* degrade to inert with a clear message rather than a hidden fallback.
- **HMA:** `compatibility/prometheus-control-plane.json` (`upstream.repository`, `minimumPackageVersion`) must be revised to name the Companion as the control-plane upstream; owner is the operator's HMA repo, not this phase, but the plan must carry the dependency.
- Unchanged by this decision: D-01, D-03..D-14 (keyring incumbent, paths, MCP shim, harness, DHT default, service kinds) because they concern running the node in a different binary, not who owns the source.
- cand-016 (git deps by rev, pack→Companion) is withdrawn: after B the pack has no Cargo consumers. cand-017 (hand-mirrored `[patch]`) stays rejected for the Companion's own workspace.

---

### 2026-09-02 — D-02 FINAL by operator: option D, the product boundary

Options considered: A (crates stay, Companion consumes), B (crates move, CLI repoints), C (third shared repo), and the operator's D | Score gap: not applicable, D reframes the question
Decision: **D** | Provenance: **user** (Travis James, 2026-09-02, three clarifications in Claude Code chat) | Elicitation ID: inline

Operator's model, in substance: the skill pack is open source and must remain fully capable and functional without the Companion; the CLI and every functional service stay where they are because they are needed for the pack and its services to be operational; what moves is all sync, CRDT replication, P2P, discovery, and the purely control mechanisms that orchestrate, monitor, and guarantee services are running, which do not themselves provide function; the Companion is a framework and a releasable, eventually paid, piece of software, and is also the way third parties extend the pack from their own repos with smooth integration.

Binding consequences:
- **Stays (open-source operational core):** skills, hooks, `runtime/v1/run-hook` bundles, `tools/prometheus-cli`, `substrate/kbd-runtime` (local signed authority; Loro is its journal format, which is function), skill-index, learner-model, storage-provider trait + `local_dir` + `loro_adapter` + `sync_manifest`, skill-ffi, exec-*, prometheus-research, surface-bridge, and every service with its LaunchAgent/systemd template.
- **Moves (paid control and extension framework):** sovereign-sync in full including `--mode mcp`, sovereign-client, storage-provider `iroh_docs.rs`, kbd-mobile, the sharing plist and systemd unit, the `--sharing` install path, and the three `sync-*` skills (as the Companion's skill plugin and first third-party-extension example).
- **Dependency direction:** the pack never depends on the Companion at build or install time; the Companion consumes pack crates by git rev. skill-ffi drops or feature-gates its kbd-mobile dependency without importing a paid crate. HMA `prometheus-control-plane.json` stays valid as written.
- **Runtime discovery, never assumption:** kbd-* gains sync and remote signed commands when the Companion's endpoint is discovered; absent, nothing changes and nothing warns. The "degrade to inert" wording from the interim B entry is withdrawn.
- **New goal G7:** the pack publishes an open, versioned integration contract (CLI transport discovery, hook bundle extension points, service manifest, connected-skill-package declaration); the Companion is its first consumer.
- **Supervision model (amends D-05):** the Companion adopts existing services where they run (Companion spec §18.5.9), runs only its own node in-process; spec §16.4/§25.2 amended to "adopted external services plus one in-process node".
- **MCP (amends D-12):** `--mode mcp` moves with sovereign-sync and the Companion owns its registration; no shim in the pack.
- **Certification gate added to G6:** the pack certifies fully with the Companion absent.
- Unchanged: D-01, D-03, D-04, D-06 through D-11, D-13, D-14.
- Fact recorded, not advice: sovereign-sync's MIT history is already public and remains so.
