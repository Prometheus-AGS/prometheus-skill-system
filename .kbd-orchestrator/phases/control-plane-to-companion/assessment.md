ASSESSMENT: control-plane-to-companion
Project: prometheus-skill-pack (package `prometheus-skill-system` 1.8.0; KBD project 6ac090a4-3656-4d83-8eb6-2891508196d5)
Date: 2026-09-01
Codebase baseline: skill-pack `main` at 1dc5a67 (release 1.8.0), runtime revision 387; prometheus-companion at 773aa5e with 735 uncommitted lines in its architecture plan. Reference repos cited below: `/Users/gqadonis/Projects/hybrid-mobile-architecture-src` at 153f20c (2026-08-24), `/Users/gqadonis/Projects/know-me/know-me-system` at 68f7ab8 (2026-07-31), `/Users/gqadonis/Projects/graph-explorer` at cf24cda (2026-09-01).
Cross-tool progress: none

Scope note. This phase spans two repositories and three reference
architectures. The assessment inspected: prometheus-skill-pack (source of
the control plane), prometheus-companion (destination), and, as the user
requested, hybrid-mobile-architecture-src (HMA, the doctrine), know-me-system
and graph-explorer (worked HMA examples). Findings are grouped by the six
assessment dimensions, then rolled up per goal. Every count below carries
the command that produced it; the Companion's own prior handoff warns that
counts in assessments were wrong at least once each, and revision round 1 of
this document proved the same.

IMPLEMENTATION STATUS

Source side — what exists today in prometheus-skill-pack

- Axum control-plane node: [DONE in the daemon; portability UNASSESSED] —
  `substrate/sovereign-sync` 1.8.0 (`cat src/*.rs | wc -l` = 9,360;
  `rest_api.rs` 3,834, `p2p.rs` 1,313, `kbd_control.rs` 927, `mcp_server.rs`
  640, `health_check.rs` 652) is an axum 0.8 + loro 1.13 + iroh 1.0.2 +
  iroh-gossip 0.101 + rmcp 1.8 + redb 2 node with modes
  `init|mcp|daemon|server|status|pair-export|pair-import|openapi`. REST covers
  `/health`, `/ready`, `/openapi.json`, skills search, sync status/peers/push
  (v1 + signed v2), the per-project KBD surface (replicas, status, events,
  audit, submodules, diagnostics, conflicts, claims, SSE `events/stream`,
  signed `commands`), and AG-UI `/api/v1/stream`. Default transport is a
  Unix socket at `data_local_dir()/prometheus/run/sovereign-sync.sock`; TCP
  on 7892 only with `--tcp` plus a mode-0600 bearer `--token-file`.
  What exists is a daemon written for launchd. Hosting it inside the
  Companion is not a copy: the Companion's AGENTS.md §17 bans `unwrap()` in
  Rust repo-wide (L518, unscoped; the `constraints.md:62` check greps only
  `src-tauri/src/` and must widen to `crates/`), §15 requires exact pins, §4 places iroh in the
  Infrastructure layer, §17 of the spec allows credentials only through
  Stronghold (the daemon uses a bearer token file), and the external surface
  (REST/MCP/SSE versus Tauri IPC) is undecided (gap 6). Those are rework
  classes, not relocation.
- KBD runtime authority: [DONE] — `substrate/kbd-runtime` (`find src -name
  '*.rs' | xargs cat | wc -l` = 14,877, of which top-level `src/*.rs` =
  14,540 and `bin/control-plane-recover.rs` 337). Owns `KbdStateV2`
  (lib.rs:1141), ed25519 `SignedCommandEnvelope` sign/verify, the append-only
  journal `events.jsonl` (the WAL), `runtime.lock`, checkpoints/archives, the
  Loro `ProjectDocument`, the registry (`registry.rs` 1,958), rollout and
  migration proofs, and the compatibility projections (`current-waypoint.json`,
  `position-reminder.txt`, `progress.json`, `position.json`, `tasks.md`).
  Zero substrate path deps. `sovereign-sync::kbd_control::submit_signed`
  requires `schema_version == "2"` and verifies the envelope before submit,
  behind one exclusive journal lock.
- CRDT sync: [DONE] — Loro 1.13 in sovereign-sync, kbd-runtime, kbd-mobile,
  storage-provider, learner-model. `kbd_sync.rs` handles `project.loro`. No
  openraft in any Cargo.toml (the memory index entry naming OpenRaft is
  stale; redb is present).
- P2P transport: [DONE] — iroh QUIC with the n0 preset (pkarr DNS discovery +
  relay) plus a static `[peers].bootstrap` list from
  `~/.config/sovereign-sync/config.toml`; blake3-derived gossip topic from a
  32-byte `group_secret`; membership from `NeighborUp/Down`; explicit
  pairing via `P2PIdentity::export_ticket/import_ticket/authorize_endpoint`;
  `P2PSupervisor` retry/backoff with a status snapshot.
- Kademlia / DHT discovery: [MISSING] — absent from every crate in every repo
  inspected. See gap 1: two recorded decisions reject it.
- Thin client SDK: [DONE] — `substrate/sovereign-client` (355 LOC, reqwest +
  SSE, path dep on kbd-runtime). `SovereignClient::new(base_url)` takes a
  URL; 7892 appears only in a doc comment (client.rs:19) and `#[cfg(test)]`
  (client.rs:249, 261), so it is not an active fixed client.
- Mobile wire compatibility: [DONE] — `substrate/kbd-mobile` (761 LOC, iroh +
  loro) and `substrate/skill-ffi` (2,067 LOC, `flutter_rust_bridge = "=2.12.0"`)
  with test `mobile_wire_is_byte_compatible_with_sovereign_sync`.
- Storage adapter: [DONE] — `substrate/storage-provider` (1,161 LOC):
  `traits.rs`, `local_dir.rs`, `loro_adapter.rs`, `sync_manifest.rs`,
  `iroh_docs.rs` behind default feature `iroh-docs-backend`.
- Learner model: [DONE, previously omitted] — `substrate/learner-model`
  (1,443 LOC, path dep on storage-provider) is a path dependency of
  sovereign-sync (`Cargo.toml:72`) for the `learner-model` sync domain.
- Skill index: [DONE, previously omitted] — `substrate/skill-index`
  (`prometheus-skill-index`, 142 LOC) is a path dependency of sovereign-sync
  (`Cargo.toml:73`) and skill-ffi.
- Service supervision: [PARTIAL] — see the service table under gap 4. Health
  checking is fragmented across `detect-toolchain.sh`, `service-probe.sh`,
  `check-mcp-health.sh`, `check-model-config.sh`, `kbd-doctor.sh`,
  `prometheus doctor`, and `sovereign-sync --mode status`. No unified
  aggregator exists; the 2026-08-20 architecture review names this net-new
  (§2.4, R3.7).
- CLI transport: [DONE] — `tools/prometheus-cli` `commands/kbd.rs` (2,363 LOC)
  and `control_transport.rs` (288 LOC): `PROMETHEUS_CONTROL_ENDPOINT` →
  `SOVEREIGN_SYNC_SOCKET` → `data_local_dir()/prometheus/run/sovereign-sync.sock`
  → fallback `http://127.0.0.1:7892`. Every kbd-* skill reaches the runtime
  through `prometheus kbd`, never HTTP; `runtime-authority.sh` refuses direct
  writes and shells out to the CLI.
- Daemon-identity footprint, the surface Goal 5 must repoint or remove
  (`grep -rl -E 'sovereign-sync|sovereign_sync'` over scripts, shared,
  skills/learn, skills/process, tools/prometheus-cli/crates, hooks, and the
  manifests, excluding target/ and Cargo.lock): 33 files, plus 42 under
  `docs/` and `site/docs`. By role:
  - install scripts (7): `scripts/install-binaries.sh`,
    `scripts/install-skills-flat.sh`, `scripts/install-mcp-services.sh`,
    `scripts/install-sovereign-sync-sharing.sh`, `scripts/install-system.js`,
    `scripts/docs-sync.mjs`, `scripts/certify-prometheus-exec-use-cases.py`
  - CLI (5): `tools/prometheus-cli/.../commands/{control_transport,doctor,kbd,setup}.rs`,
    `.../src/main.rs`
  - CLI tests (2): `tools/prometheus-cli/.../tests/{doctor,kbd_device_authority}.rs`
  - probes and health (3): `shared/scripts/detect-toolchain.sh`,
    `shared/scripts/service-probe.sh`, `scripts/check-mcp-health.sh`
  - service templates (7): `shared/launchagents/ai.prometheus.{sovereign-sync,forge-mcp,liter-llm-api,pk-cherry,surface-bridge}.plist`
    (the four non-sovereign plists reference it in ordering or env),
    `shared/systemd/ai.prometheus.sovereign-sync.service`
  - shell tests (4): `shared/scripts/tests/test-{detect-toolchain-sovereign-sync,kbd-registry-service-install,learning-service-install,service-exclusions}.sh`
  - skills (3): `skills/learn/sync-{status,peers,push}/SKILL.md`
  - top-level docs and manifest (3): `CLAUDE.md`, `README.md`, `package.json`
  The narrower port footprint (`grep -rln 7892`, 13 files) is a subset and
  was the wrong measure; the socket path is the default transport, so most
  callers never name the port.
- Hook runtime path: [DONE, separate from the daemon] — `hooks/hooks.json`,
  `.mcp.json`, `.claude-plugin/marketplace.json`, and `skill-system.json`
  contain zero references to sovereign-sync (`grep -c`). Hooks invoke
  `$HOME/.prometheus/plugins/prometheus-skill-pack/runtime/v1/run-hook
  --bundle <kbd-control|kbd-open|kbd-receipt|…>`; the bundles reach the
  runtime through the signed local `prometheus` CLI path, not the daemon.
  This is a Goal 5 surface with its own transport that the plan must
  inventory before any repoint.

Destination side — what exists today in prometheus-companion

- Rust node, crates/, Axum, iroh, loro, tokio, supervisor: [MISSING] —
  `src-tauri/src/lib.rs` is 14 lines (one `greet` command); Cargo deps are
  tauri, tauri-plugin-opener, serde, serde_json. No `crates/` directory.
- Frontend CLEAN layers: [MISSING] — `find src -type f` = 5 (app.tsx,
  main.tsx, app.css, vite-env.d.ts, assets/react.svg); the four layer
  directories declared in AGENTS.md §4 are not created, so `audit-layer.sh`
  no-ops.
- Governance: [DONE] — `templates/rules/*.tmpl` → generated AGENTS.md/CLAUDE.md,
  10 `audit:*` scripts, `audit-all.sh`, pre-push hook (commit 773aa5e).
- Specification: [PARTIAL] — `docs/00-architecture-and-implementation-plan.md`
  (4,405 lines, 735 uncommitted) covers §4 process model (one in-process
  substrate library, not a sidecar; `crates/prometheus-substrate/` with
  `supervisor.rs`, `platform_id.rs`, `services/*.rs`), §14 P2P pairing
  (Ed25519, 6-digit code + QR, Noise, capability manifest, operator
  approval), §15 AG-UI consumed via Tauri events rather than HTTP SSE, §16
  supervisor actor (dependency-ordered start, backoff 1s→60s, 10 failures →
  disabled), §17 credentials (Stronghold only), §18 storage lanes, §18.5
  connected skill packages with a Supervisors panel, §18.6 auto-integration,
  §23 ten weekly phases, §25 risks, Appendix A crate plan, C target tree,
  D 40 checkpoints. `versions.toml` `[pins]` and `[decisions]` are empty.
  `openspec/specs/` and `openspec/changes/` are empty. Companion
  `test_command` is null.
- Companion's own KBD phase: `docs-review-for-build-assessment` has assess
  artifacts on disk (`assessment.md` and `handoffs/assess.handoff.json`,
  both dated 2026-08-23) but the Companion runtime still projects
  `Stage: ready` with next command `/kbd-assess`; the assess transition was
  never recorded. Its goal G3 ("Appendix A is missing") is stale: the
  uncommitted edit added Appendix A at L3884–4084.

Reference architectures — what the requested examples actually contain

- HMA (hybrid-mobile-architecture-src, last commit 2026-08-24): a skill and
  reference repo with no runnable app. Prescribes for tray apps exactly the
  Companion's shape, `crates/<name>` + `src-tauri` joined by a root workspace
  (`scripts/scaffold-tauri-tray.sh:53-54, 95-98`), and names
  `prometheus-companion/crates/prometheus-companion/src/{tray,health}.rs` as
  the canonical health-aggregator reference (`docs/06-tauri-tray-app-spec.md`),
  a file that does not exist yet. Prescribes `launchagent-supervisor`
  (ThrottleInterval ≥ 15, KeepAlive dictionary form, PID-file lock, one
  installer, self-check watchdog) and a single five-state health aggregator.
  Loro 1.13 is the only permitted CRDT (ADR-LFS-5). `versions.toml` pins
  loro_crdt 1.13, flutter_rust_bridge 2.12.0, flutter_webrtc 1.4; no iroh,
  no libp2p, no Kademlia pin.
- know-me-system (last commit 2026-07-31): 22 crates + a separate
  `desktop/src-tauri` workspace with a hand-mirrored `[patch]` table. Its Axum
  node `gen_ui_server_axum` (382 LOC) has `/health`, `/ready`, providers,
  control-plane catalog, and an AG-UI run/events SSE split with keep-alive,
  terminal-event break, and `Lagged → transport_error`. No auth middleware.
  The Tauri desktop never starts Axum; it reaches the host through Tauri
  IPC via `tauri_plugin_gen_ui`. Axum is bound only by the separate
  `knowme-web-server` binary. Peer sync crate `knowme_ofp` is a 135-LOC stub;
  real sync is server-centric (Electric-shaped shapes over
  flint-realtime-fabric). No loro, iroh, discovery, supervisor, or two-node
  test. FRB pin drift: versions.toml 2.13.0-beta.5 vs Cargo/pubspec =2.12.0.
- graph-explorer (last commit 2026-09-01): single clean workspace with
  `desktop` as a member, 8 lib crates, `flutter_rust_bridge = "=2.12.0"`.
  No Axum, no sync, no P2P, no supervisor; MCP appears only as an outbound
  health probe. Most transferable: `tauri-plugin-gen-ui/tests/surface_parity.rs`
  (desktop and mobile answer identically through one core) and the
  `Arc<dyn EntityStore>` port + conformance-suite pattern.

CROSS-TOOL PROGRESS
- NONE — `progress.json` has `changes: []`; no other tool has touched this
  phase.

SPEC GAP SUMMARY

1. Goal 3 (Kademlia-style DHT discovery) contradicts two recorded decisions.
   Companion spec §25.2 (L3702) chose iroh over libp2p explicitly for having
   "no DHT gossip tax". HMA `references/sync/peer-crdt.md:80-85` states
   "Admission = device pairing, not open discovery" with an Ed25519 roster
   kept inside the Loro doc. sovereign-sync already implements pkarr DNS
   discovery + relay + static bootstrap + ticket pairing. Adopting a DHT
   needs an ADR that names the failure of the current mechanism; none is
   recorded.
2. Goal 2 (Companion owns KBD authority) conflicts with HMA's external
   contract `compatibility/prometheus-control-plane.json`: `"lifecycle":
   "prometheus"`, `"canonicalState": "signed-loro-event-map"`,
   `"compatibilityProjections": "read-only"`, minimum package 1.7.0 /
   contract 2.0.0. The contract binds authority to the prometheus-skill-system
   package. Either the Companion becomes that package's shipping vehicle for
   the control plane, or the contract is revised in HMA first.
3. HMA rejects iroh for its browser-capable vault peer lane
   (`peer-crdt.md:55-56`: "rules out QUIC/iroh as the primary lane (browsers
   cannot join iroh gossip)") and prescribes webrtc-rs inside `gen_ui_core`.
   The rejection is scoped to a lane where browsers are peers. The Companion
   spec §14.3 and skill-pack `docs/decisions/fabric-transport-iroh.md` both
   commit to iroh for machine-to-machine sync. Not a direct conflict, but the
   analyze stage must record which lane the control-plane sync belongs to so
   the doctrine is applied, not sidestepped.
4. Goal 4 says "every skill-pack service" and Goal 5 says "delete the
   LaunchAgent", but no single inventory existed and the counts in circulation
   disagree (the 2026-08-20 review says 7, goals.md names 10, this repo ships
   12 macOS templates and 13 systemd units). The Companion spec §4.1 (L263)
   and §25.1 (L3690) also keep launchd as the outer restart authority for the
   Companion itself, and §18.5.9 has the Companion observe and manage plists.
   The reconciled inventory, from `ls shared/launchagents shared/systemd
   shared/scripts/scheduled ~/Library/LaunchAgents` (installed count excludes
   `.bak`/`.deprecated`):

   | Template (repo) | Installed today | In goals.md G4 | Disposition to decide |
   |---|---|---|---|
   | ai.prometheus.sovereign-sync (plist + .service) | yes | yes (the node) | replaced by the Companion-hosted node; launchd then supervises the Companion |
   | ai.prometheus.exec (plist) | yes | yes | node-supervised |
   | ai.prometheus.forge-mcp (plist + .service) | yes | yes | node-supervised |
   | ai.prometheus.learning-worker (plist + .service/.timer/.path) | yes | yes | node-supervised timer |
   | ai.prometheus.liter-llm-api (plist) | yes | yes | node-supervised |
   | ai.prometheus.pk-cherry (plist + .service) | yes | yes | node-supervised |
   | ai.prometheus.surface-bridge (plist + .service) | yes | yes | node-supervised |
   | ai.prometheus.surreal-memory-native (plist + .service) | yes | yes | node-supervised |
   | ai.prometheus.surrealdb-native (plist + .service) | yes | yes | node-supervised |
   | com.prometheus.research (plist in substrate/prometheus-research) | yes | yes | node-supervised |
   | ai.prometheus.codex-skills-sync (plist) | not installed | yes | node-supervised timer or stays launchd |
   | ai.prometheus.hooks-logrotate (plist + .service/.timer) | yes | no | undispositioned |
   | ai.prometheus.prometheus-nudge (plist + .service/.timer) | yes | no | undispositioned |
   | ai.prometheus.pk-lint, ai.prometheus.mem0-compress (shared/scripts/scheduled) | not installed | no | undispositioned |
   | ai.prometheus.ferrox (no template in this repo) | yes | no | out of scope, owned elsewhere |
   | com.prometheus.universal-agent-runtime (no template here) | yes | no | out of scope, owned elsewhere |
   | dev.prometheusags.openai-proxy (no template here) | yes | no | out of scope, owned elsewhere |

   Installed prometheus-family plists today: 15. What collapses under Goal 4
   is the ten node-supervised rows; the four undispositioned rows need a
   decision; the three out-of-scope rows must be named as such in the plan so
   "every service" is bounded.
5. Companion Appendix A is stale by a major version and by path. It pins
   `iroh = "0.30"` (actual 1.0.2, the security floor per
   `substrate/sovereign-sync/Cargo.toml:47-48`) and `rmcp = "0.1"` (actual
   1.8), and consumes skill-pack crates as `{ path = "../<name>" }` from a
   superworkspace that does not exist: the skill pack has no root
   `Cargo.toml`, every `substrate/*` crate is its own workspace with its own
   `target/`, and `crates/` holds only `prometheus-exec`. The spec's Phase 1
   "initialize the superworkspace" has not happened.
6. Goal 2 fixes the node's external surface as REST/MCP/SSE, and two
   specifications conflict with that. Companion spec §15 consumes AG-UI via
   Tauri events, and HMA's desktop prescription is Tauri commands + events
   over an in-process core, with Axum reserved for the hosted web profile and
   the UAR gateway boundary. No reference example implements in-process Axum
   inside Tauri: know-me's desktop uses Tauri IPC and graph-explorer has no
   server. The goal stands; the open questions are only whether the
   Companion UI additionally consumes the node over Tauri IPC, and which spec
   section (§15, and HMA's desktop guidance) must be amended to admit an
   external REST/MCP/SSE surface for the `prometheus` CLI, other machines,
   and the Codex/Kimi harnesses. The CLI's socket-first transport chain and
   the sync-* skills' `curl --unix-socket` contract are existing consumers
   of that surface.
7. Cross-repo dependency direction, full path-dependency closure. From
   `grep -n 'path = ' substrate/*/Cargo.toml tools/prometheus-cli/Cargo.toml`:
   sovereign-sync → storage-provider, kbd-runtime, learner-model,
   prometheus-skill-index (dev: kbd-mobile); learner-model → storage-provider;
   kbd-mobile → kbd-runtime; sovereign-client → kbd-runtime; skill-ffi →
   kbd-mobile, kbd-runtime, skill-index, exec-embedded, exec-contracts,
   exec-core, exec-service; prometheus-cli → kbd-runtime. Moving sovereign-sync
   drags learner-model, skill-index, and storage-provider with it or turns
   them into cross-repo dependencies. Moving kbd-runtime makes the skill
   pack's own CLI depend on the Companion repo, inverting "skill pack is a
   consumer" at the build level. The Companion spec's `optional = true` path
   deps point the other way. The analyze stage must choose per crate: stays
   in the pack as a library the Companion consumes, moves with a git or
   registry dependency back, or is feature-gated out of the node.
8. Shared filesystem contracts survive any code split. From
   `substrate/kbd-runtime/src/lib.rs`: data root resolves
   `PROMETHEUS_DATA_DIR` → `dirs_next::data_local_dir()` →
   `temp_dir()/prometheus-data` (lines 2979-2982 and 3157-3160), then
   `canonical_runtime_root_at(data_root, project_id)`; the device key is
   `config_dir/sovereign-sync/device-key.json` (line 307) and per-root
   `device-key.json` (line 3248); the control socket is
   `data_local_dir()/prometheus/run/sovereign-sync.sock`. Goal 5's "state
   migrates intact" is therefore a path-resolution requirement: whether a
   Tauri-hosted process (app-scoped data dir, sandbox entitlements, signed
   bundle) resolves `data_local_dir()` and `config_dir()` to the same paths
   as the launchd daemon is unassessed and must be tested, not assumed.
9. Companion spec allocates no port for the substrate, no Axum listen
   address, and no QUIC port. Only surreal :28000 and openai-proxy :8181 are
   named.
10. Companion governance gaps that bind a node: `versions.toml` pins are
    empty and `Edit(versions.toml)` is denied to agents; AGENTS.md §0.2
    "observed problems only" requires every supervisor retry/backoff branch to
    trace to a named failure; §17 bans `unwrap()` in `src-tauri/src/`;
    `openspec/` is empty so the node would be the first change; the handoff
    for the Companion's open phase records AG-UI specified twice with zero
    payload agreement and A2UI specified twice with zero ids in common.
11. Existing integration evidence that must survive the move
    (`grep -c '#\[tokio::test'` and `grep -c '^#\[test\]'`):
    `substrate/sovereign-sync/tests/integration_tests.rs` has 18 async + 4
    sync = 22 tests across the axum boundary, anchored by
    `unsigned_v1_sync_push_is_rejected_without_same_user_unix_transport` and
    `kbd_command_requires_device_signature_and_claim_surface_reports_commit`;
    `tests/domain_sync.rs` has 4 async two-node tests
    (`skill_index_replicates_end_to_end_between_two_nodes`,
    `learner_model_replicates_end_to_end_between_two_nodes`,
    `surreal_memory_is_rejected_before_any_bytes_are_prepared`,
    `signed_kbd_authority_updates_replicate_claims_between_two_nodes`) plus
    one single-process `#[test]` at line 44,
    `mobile_wire_is_byte_compatible_with_sovereign_sync`, which round-trips a
    `MobileProject` delta through one runtime in a tempdir with no iroh
    endpoint. That last test is the kbd-mobile compatibility gate, not
    two-node evidence, and lives in sovereign-sync, not kbd-mobile. Goal 6's
    two-node evidence has the four async tests as its baseline; a plan
    acceptance criterion should name them, not a total.
12. HMA's canonical health-aggregator reference is circular: it points at
    Companion files that do not exist. The template crate
    `assets/templates/tauri-tray/health-aggregator/` in HMA is the actual
    starting point.
13. Two incompatible pairing designs are in play and the Companion spec
    misattributes one of them. Companion spec §14 (L66, L122, L222,
    L2239–2298) describes "sovereign-sync (Noise + Iroh)" with a 6-digit
    code, QR, Noise handshake, capability manifest, and operator approval.
    `grep -il noise substrate/sovereign-sync/src/*.rs Cargo.toml` returns
    nothing: sovereign-sync pairs by ticket export/import, a blake3-derived
    topic from a shared `group_secret`, and an endpoint allowlist
    (`authorize_endpoint`) over iroh's QUIC/TLS 1.3. Goal 3's "device
    identity and pairing" therefore does not simply exist; the analyze stage
    must choose one design, and the spec's Noise attribution must be
    corrected either way.

OPEN QUESTIONS FROM goals.md — what the evidence bounds

- Q1 (Cargo workspace with `crates/`, or node inside `src-tauri`): the
  evidence favors a root workspace `members = ["crates/<name>", "src-tauri"]`.
  HMA prescribes it for tray apps (`scaffold-tauri-tray.sh:95-98`), the
  Companion spec §4.2 already names `crates/prometheus-substrate/`, and
  graph-explorer's single workspace with `desktop` as a member avoids the
  hand-mirrored `[patch]` table that know-me's split workspace carries.
  Decision owner: analyze stage; no contrary evidence found.
- Q2 (DHT in v1): gap 1 shows two recorded decisions against a DHT and a
  working pkarr + relay + bootstrap + pairing path in sovereign-sync. The
  evidence bounds this to "not v1 without an ADR naming a failure of the
  current discovery". Decision owner: analyze stage, with the operator's
  intent in the phase request weighed against the recorded tradeoffs.
- Q3 (which repo's KBD run owns the cross-repo work): this phase lives in
  the skill pack's run `sovereign-sync-service-reliability-20260829` at
  revision 387. The Companion has its own runtime with one open phase whose
  assess artifacts exist on disk (2026-08-23) but whose assess transition was
  never recorded, so its runtime still says `ready`; that runtime rejected
  commands for this phase as "phase not found" during this assessment, which
  confirms the two runtimes are independent. Any Companion child phase must
  first reconcile that on-disk-versus-runtime discrepancy. The evidence supports: this phase owns the
  boundary decision and the skill-pack side of the move; a Companion child
  phase, linked by a handoff, owns Companion-side code so its audit gate and
  constraints apply natively. Decision owner: plan stage.

BUILD HEALTH
- build check (skill-pack): [PASS, prior evidence] — runtime
  `completion.certification` for `kbd-control-plane-recovery` is `complete`
  ("Final local integration, release, protected-test, OpenSpec, harness,
  documentation, and receipt-identity certification passed", 2026-08-30).
  Not re-run this session; no Cargo build was started per the one-build rule.
- build check (companion): [PASS] — `pnpm run build` exit 0 (191.99 kB JS,
  built in 158 ms); `bash scripts/audit-all.sh` exit 0 with PASS 7 / FAIL 0 /
  SKIP 3 (tokens, skill-desc, ui are stubs). Rust side is a 14-line scaffold.
- known violations: companion `docs/00-architecture-and-implementation-plan.md`
  has 735 uncommitted lines; companion `constraints.md` "declared-but-absent
  enforcement" table is stale (773aa5e landed the scripts); seven skill-pack
  phase `progress.json` files lack the top-level `generatedBy: "kbd-runtime"`
  marker (`jq -e '.generatedBy == "kbd-runtime"'` fails for
  ideation-and-decision-tools, kimi-desktop-extensibility,
  mobile-skill-portability, openspec-mirror-drift-cleanup,
  prometheus-exec-code-execution-engine, uar-frontend-workspace-repair,
  uar-host-execution), and the runtime's refusal loop runs over every phase
  on every transition (kbd-runtime `lib.rs:5210-5211`). These seven are part
  of the state Goal 5 must migrate; each needs an adopt-or-archive decision.
- test coverage: [PARTIAL] — full-integration coverage exists for
  sovereign-sync REST and two-node P2P; none for kbd-runtime as hosted by
  anything but the daemon; companion has no test runner.

CONSTRAINT CHECK
- AGENTS.md violations (skill-pack CLAUDE.md policies): NONE observed. No
  unit-test evidence cited; no Cargo build run; one-build rule honored.
- constraints.md violations: NONE for C-01..C-05 at assess time. C-01 binds
  during execution for the files that actually change: plugin generations
  and the Codex mirror regenerate from `.claude-plugin/*`, and `npm run
  validate:codex` must pass in the same change. No install path touches the
  repo `.mcp.json` for sovereign-sync; `--sharing` writes the user-level
  `~/.claude/mcp-servers.json` (`install-skills-flat.sh:533`), which is
  outside C-01 but must still be cleaned by the removal change. `hooks/hooks.json`,
  `.mcp.json`, `.claude-plugin/marketplace.json`, and `skill-system.json`
  contain no sovereign-sync references today, so the hook bundle runtime
  (`runtime/v1/run-hook`) is the surface to inventory, not those files.
- Companion AGENTS.md: §0.2, §0.3 (security traces to a named boundary), §4
  layer rule (Iroh is Infrastructure), §15 exact pins, §17 no unwrap, §19
  audit gate all apply and are currently unexercised by any node code.

GOAL PROGRESS
- G1 inventory and ownership boundary: NOT MET — the inventory now exists
  (this document: crate closure in gap 7, service table in gap 4, transport
  footprint above) but the disposition per crate and per service is still a
  proposal. Gaps 2 and 7 mean the boundary cannot be declared until
  authority location and dependency direction are decided.
- G2 Companion owns one Axum node: NOT MET — the node exists as
  sovereign-sync 1.8.0 in the skill pack, written for launchd with a bearer
  token file and no Companion governance applied; nothing is hosted by the
  Companion, which has no crates/, no Axum, and an empty versions.toml. The
  rework classes (surface decision, unwrap removal, exact pins, credential
  model, superworkspace) are listed under IMPLEMENTATION STATUS and gaps 5,
  6, 10.
- G3 Kademlia + gossip discovery in the Companion: NOT MET — iroh-gossip +
  pkarr + bootstrap + pairing exist in sovereign-sync; no DHT exists anywhere,
  and two recorded decisions reject one (gap 1).
- G4 unified management, configuration, and monitoring of every service:
  NOT MET — 15 installed plists, 12 templates, 13 systemd units, seven health
  scripts, no aggregator; the Companion spec §16 and HMA
  `launchagent-supervisor` prescribe the design; no code. The scope of
  "every service" is now bounded by the table in gap 4.
- G5 skill pack becomes a consumer with state migrating intact: NOT MET —
  the CLI transport chain is endpoint-agnostic, which lowers the cost, but
  33 non-doc files and 42 doc files name the daemon, the hook bundle runtime
  is a separate transport,
  the cross-repo dependency direction (gap 7) and the path-resolution
  contract (gap 8) are unresolved.
- G6 two-node full-integration evidence on both repos: NOT MET — a
  four-test two-node baseline exists for the daemon; nothing exercises a
  Companion-hosted node.

Sizing note for analyze and plan. Rust in the sovereign-sync path-dependency
closure (`cat src/*.rs | wc -l` per crate): sovereign-sync 9,360;
kbd-runtime 14,877; storage-provider 1,161; learner-model 1,443; skill-index
142; kbd-mobile 761; sovereign-client 355; skill-ffi 2,067. Total roughly
30.2k LOC whose home must be decided. Rust that stays and repoints:
prometheus-cli 14,116. The Companion side starts from 14 lines. The largest
risks are not code volume but the three decisions in gaps 1, 2, and 7, each
of which changes what moves.

UNRESOLVED REVIEW FINDINGS

Adversarial review ran in artifact mode for two rounds. The configured
cross-model judges were unavailable (k3: weekly quota, HTTP 401; MiniMax-M3:
token plan limit, HTTP 429, both at the liter-llm gateway on 2026-09-02), so
both rounds used the contract's harness-native fallback: a fresh-context
artifact-critic subagent of the producer's own model family. That is the
weaker isolation guarantee and is recorded as `isolation_mode:
harness-native`, `cross_model_check: false` in `review/assess/findings.json`
(round 2) and `findings.round1.json`. Round 1 returned one CRITICAL (omitted
learner-model and skill-index path deps) and six WARNINGs; all were verified
and corrected above. Round 2 returned the CRITICAL below plus five WARNINGs
and three SUGGESTIONs; all were verified and folded in above, but under the
two-round cap the round-2 CRITICAL is carried verbatim so the next stage
inherits it explicitly rather than trusting this document's own fix:

- CRITICAL (round 2, assessment.md:385): "The Goal 1 inventory is declared
  to exist, but the repoint/removal footprint it hands to the plan is
  measured by port number (13 files with `7892`) rather than by the daemon's
  identity, so a plan built from it will leave most of the sovereign-sync
  surface untouched." Evidence: `grep -rl 'sovereign-sync\|sovereign_sync'`
  over scripts, shared, skills/learn, skills/process, tools/prometheus-cli/crates,
  hooks, and manifests = 33 files, plus 41 under docs/ and site/docs; files
  absent from the artifact included scripts/install-binaries.sh,
  scripts/install-system.js, scripts/docs-sync.mjs,
  scripts/install-sovereign-sync-sharing.sh,
  scripts/certify-prometheus-exec-use-cases.py, prometheus-cli
  commands/{doctor,setup}.rs, src/main.rs, tests/{doctor,kbd_device_authority}.rs;
  sovereign-client/src/client.rs has 7892 only in a doc comment and
  `#[cfg(test)]`. Disposition: the name-based footprint now appears under
  IMPLEMENTATION STATUS ("Daemon-identity footprint") and was not re-vetted.
  The analyze stage must treat that list as its input and re-derive it with
  the same command before planning.

ASSESSMENT COMPLETE
