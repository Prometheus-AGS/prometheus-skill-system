# Analysis — ideation-and-decision-tools

> **Date:** 2026-07-30 · **Tool:** Claude Code (claude-opus-5)
> **Mode:** stack-specified (iroh + wasmtime named by the user)
> **Scope added at invocation:** iroh as fabric-wide transport across
> `flint-realtime-fabric`, `universal-agent-runtime`, and `know-me-system`; WASM
> for skill parity on mobile; how to populate UAR with all skills.
>
> **Decided with the user before research:** *design and record only* — no
> cross-repo code this stage; `/kbd-plan` schedules the work. Plus a
> **fabric-integration skill** as a first-class deliverable, so "all projects use
> this pack's skills" is mechanical rather than convention.

---

## §1 The finding that constrains everything else

**iroh cannot be one transport everywhere. The browser is a relay-only client, by
architectural necessity — not a temporary gap.**

The browser sandbox has no UDP, so every browser connection flows through a relay
over WebSocket-over-TLS. Not WebTransport, not WebRTC. iroh's own docs:
*"All connections from browsers to somewhere else need to flow via a relay server."*
The API confirms the degradation — `watch_addr()`, `bound_sockets()`, and
`dns_resolver()` are all marked *"Available on non-`wasm_browser` only"*.

A browser tab gets a stable Ed25519 identity, E2E encryption, and reachability. It
does **not** get hole punching, direct addresses, or relay independence. Every byte
pays relay bandwidth and latency, permanently.

**Do not architect around `iroh-webrtc-transport`.** It is a personal project
(`SuddenlyHazel`, not n0), pinned to **iroh ^0.98.2** (pre-1.0), at
`0.1.0-alpha.2` with **33 total downloads** and a repo that 404s. n0's own
`TRANSPORTS.md` registry has **no WebRTC entry at all**.

**This does not sink the plan — it sharpens it.** Browser is one of four targets, and
it is the one where a permanently-relayed node is acceptable: a browser is already
online, already talking to servers, and already latency-tolerant. The tiering below
takes that as a given rather than fighting it.

## §2 Where iroh is strong — and it is strong

| Target | State | Evidence |
|---|---|---|
| Desktop / server | **Production** | iroh 1.0.3, wire-stable, >200M endpoints created in 30 days |
| **Mobile** | **Production, better than expected** | `iroh-ffi` v1.1.0: 5-target iOS xcframework with **simulator-booted CI tests**; all 4 Android ABIs via `cargo-ndk` with **16 KB page-size verification** (Android 15+); ships via SwiftPM/CocoaPods and Maven Central |
| Browser | **Relay-only** | §1 |

Mobile is the target this phase cares most about, and it is the best-supported
non-desktop story. Delta Chat has shipped iroh on iOS/Android/desktop for ~2 years and
runs its own relays. A telling detail: Mainline DHT discovery is **off by default**
partly *"so that mobile apps don't look like BitTorrent clients and get flagged by the
OS"* — that is real field experience, not a datasheet claim.

**Version compatibility verified against the registry** (not assumed):

| Crate | Version | iroh 1.0 compatible | wasm |
|---|---|---|---|
| `iroh` | 1.0.3 | — | ✅ (relay-only) |
| `iroh-gossip` | 0.101.0 | ✅ (`^1`) | ✅ |
| `iroh-blobs` | 0.103.0 | ✅ (`^1.0.0`) | ✅ |
| `iroh-docs` | 0.101.0 | ✅ (`^1`) | ❌ **not wasm-compatible** |

This pack already pins `iroh 1.0` + `iroh-gossip 0.101` in `sovereign-sync` — both
current and both wasm-clean.

> ⚠️ **Relay operations are a real security responsibility.** iroh **1.0.2 was a
> critical relay fix**: a missing length check let any client crash an entire relay
> with one malformed datagram. Relays accept unauthenticated frames by design. Pin
> **≥1.0.2** (current 1.0.3). n0's public relays are explicitly *"suitable for
> development and testing. For production, use dedicated relays."* Self-hosting is
> practical — ACME TLS is built in — and dedicated relays run ~$0.27/hr (~$195/mo
> continuous).

## §3 In-house state — what already exists (verified, not assumed)

This is the part that changes the plan most: **far more is built than the assessment
assumed**, across three repos.

### universal-agent-runtime — the skill substrate is already there

| Asset | Detail |
|---|---|
| `SkillStorageProvider` trait | `list_skills` / `refresh` / `save_skill` / `delete_skill` |
| Backends | **filesystem, database, builtin** — all three implemented |
| `SkillKind` enum | `Native` · `Manifest` · `Wasm` |
| `wasm_runtime.rs` | **wasmtime 46**, Component Model, loads `.wasm` (JIT) **or `.cwasm` (AOT)** |
| `wit/uar-skill.wit` | **`uar:skill@0.1.0`** — `run(input: string) -> result<string, string>`, pinned, additive-only until 1.0 |
| `pack_detection.rs` | **4-level resolution** ending at the embedded submodule `crates/prometheus-skill-system/skills` |

**`SkillKind::{Manifest, Wasm, Native}` maps exactly onto the assessment's E0/E1/E2
tiers — and it is already implemented.** The tiering is not a new design; it is
naming something UAR already does.

**This pack is a submodule of UAR** at `crates/prometheus-skill-system`. Changes here
propagate there by construction.

### know-me-system — the capability-gated host

`knowme_plugin_host`: wasmtime 46 `component-model` + `async` + `runtime`,
deliberately **without `cranelift`** — the no-JIT shape iOS needs. Ten ratified
requirements in `openspec/specs/plugin-sandbox-host`, including deny-by-default WASI,
fuel/memory/epoch limits, SSRF-guarded `net_fetch`, a typed `HostDispatch` seam, and
a **precompiled `.cwasm` cache keyed by content hash + engine version**.

Still **orphaned**: no crate depends on it, and `gen_ui_ffi` (the mobile FFI crate)
does not include it.

### flint-realtime-fabric — the fabric, and it is well-shaped for this

23 crates in a clean hexagonal architecture (`frf-ports` defines traits;
`frf-bridge-*`, `frf-media-str0m`, `frf-store-*` are adapters).

| Asset | Relevance |
|---|---|
| `frf-crdt` → **Loro 1.13.1** | **Same CRDT and same minor as `sovereign-sync`'s Loro 1.13** — interop is already possible |
| `frf-ffi` | **`crate-type = ["cdylib", "staticlib"]` + uniffi 0.31.2** — the mobile FFI this pack lacks entirely |
| `frf-ports::FederationBridge` | `send` / `subscribe` over a `FederationProtocol` — **the exact seam an iroh adapter implements** |
| `frf-media-str0m` | WebRTC already present, as a *media* adapter — orthogonal to iroh, not competing |
| `frf-wasm` | `wasm-bindgen` **browser** glue — not a WASM *host*. FRF has no host; UAR and KnowMe do |
| **iroh** | **absent** |

**FRF is the right place for iroh, and the adapter shape already exists.** An
`frf-transport-iroh` crate implementing `FederationBridge` sits beside
`frf-bridge-matrix` and `frf-bridge-atproto` — no architectural change, one new
adapter.

## §4 Build-vs-adopt calls

| Gap | Verdict | Rationale |
|---|---|---|
| P2P/QUIC transport | **ADOPT iroh 1.0.3** | Production, wire-stable, excellent mobile FFI, self-hostable relays. Already in this pack. |
| Browser peer transport | **ADOPT iroh relay-only** — do *not* add a second stack | libp2p is the only option offering browser↔server WebRTC, but adding it means two transport stacks for one degraded leg. Relay-over-WSS is what iroh gives the browser anyway. |
| WASM skill host | **ADOPT wasmtime 46** (already in UAR *and* KnowMe) | Same major in both repos; `.cwasm` AOT is the no-JIT path. |
| WASM skill contract | **ADAPT `uar:skill@0.1.0`** | Exists and is pinned. See §5 — it must reconcile with `knowme:plugin@0.1.0`. |
| Mobile FFI bindings | **ADOPT FRF's `frf-ffi` pattern** (uniffi 0.31.2) | This pack has **no** cdylib/staticlib and no uniffi anywhere. FRF has solved it. |
| CRDT | **ADOPT Loro 1.13** | Already aligned across FRF and this pack. |
| Skill delivery to mobile | **ADOPT UAR's `SkillStorageProvider`** | The database backend needs no filesystem walk — precisely the mobile path. |
| Ideation methodology encoding | **BUILD**, selectively | §6 |
| Fabric-integration skill | **BUILD** | §7 — nothing exists; it is the mechanism the user asked for. |

## §5 The central architectural decision: two WIT worlds

There are **two** component contracts, and they must reconcile before "100% parity"
means anything:

| World | Owner | Shape |
|---|---|---|
| `uar:skill@0.1.0` | UAR | Minimal: `run(input: string) -> result<string, string>` |
| `knowme:plugin@0.1.0` | KnowMe | Rich: capability-gated host imports (`log`, `kv`, `net`, `uar-invoke`, `infer`, `memory-recall`, `a2ui-push`) |

KnowMe's own extensibility spec already proposes the resolution — a layered
`prometheus:component/*` family (`core`, `agent`, `data`, `llm`, `ui`, `settings`)
where *"a native skill is a component exporting the skill world"* and hosts advertise
which interfaces they provide, with **capability negotiation at install time, before
any code runs**.

**Recommendation:** adopt that family as the single contract. `uar:skill@0.1.0`
becomes the `agent` world's minimal export; `knowme:plugin@0.1.0`'s host imports
become `prometheus:component/core` + `llm` + `data`. **Do not** ship a third world.

**This is the highest-leverage decision in the phase.** Two worlds means every skill
gets ported twice and the parity claim is false by construction.

## §6 Ideation methodology — encode by evidence grade

Carried from assessment §3. Encode **brainwriting** (strong meta-analytic support:
parallel independent generation beats interacting groups, and the gap *grows* with
group size). Encode **pre-mortem** with an honest confidence note (moderate; the
~30% figure traces to essentially one 1989 study). **Do not** encode design thinking
as an evidence-backed method — Roth et al. (2020) found *"no quantitative empirical
evidence"* it improves project performance. Shipping folklore as a skill would
contradict this pack's own anti-sycophancy posture.

Combined with the Diversity Collapse finding, the mechanism is settled: **enforce
diversity structurally** (independent generation, then pool, then judge) rather than
prompting personas to disagree.

## §7 The fabric-integration skill (BUILD)

The user's framing: *"all projects use THIS project's skills to do everything — this
is the key."* Today that is convention. UAR resolves this pack through 4 levels;
KnowMe carries its own `.claude/skills/`; FRF has no link at all.

**Proposed `fabric-integration` skill** — one documented way for a consuming repo to
adopt the pack:

1. **Detect** the consumer (UAR submodule, KnowMe FFI, FRF adapter, plain repo)
2. **Install/verify** the pack at the right level for that consumer
3. **Verify contracts** — Loro minor alignment, iroh ≥1.0.2, wasmtime major match,
   WIT world version
4. **Report drift** — the pack's own `check-model-config.sh` already proves this
   pattern catches stale installed copies

That makes "every project uses these skills" checkable, and it is the natural home for
the version-alignment invariants in §2 and §3.

## §8 Open questions

| # | Question | Blocks |
|---|---|---|
| OQ-1 | Adopt `prometheus:component/*` as the single WIT family, or keep two worlds? | §5 — everything downstream |
| OQ-2 | App Store **4.7.2** on a WASM host exposing native APIs (no precedent either direction) | E1 on iOS |
| OQ-3 | Self-hosted relay vs n0 dedicated (~$195/mo continuous) | Browser + mobile budget |
| OQ-4 | `storage-provider` pins `iroh-docs` **unconditionally** (`Cargo.toml:19`, `fs-store`) — it cannot compile for wasm32. Feature-gate it, or accept native-only? | Any browser target for this pack |
| OQ-5 | Port the 5 `adversarial-review` scripts to one WASM component, or call them via remote invoke? | 29 of 38 script-skills have ≤2 scripts and are tractable; the 15 toolchain skills never are |
| OQ-6 | Does the fabric-integration skill install, or only verify-and-propose? | §7 scope; the pack's own rule is never auto-install third-party code |

## §9 Goal-serving substrate (the seven actual goals)

An adversarial round caught a real defect in the first draft of this analysis: it
researched the *added* scope thoroughly and under-served the **seven phase goals**.
Four pieces of in-repo substrate went unevaluated, and three of them are close fits.

| Goal | Substrate found | Verdict |
|---|---|---|
| 1 · judge, not round-table | `agents/kbd-idea-critic.md` — scores candidates on a 4-dimension rubric, explicitly *"the idea that proposed the idea should never also grade it"* | **adopt** the role separation; revise the rubric to weight executability over novelty (F-1) |
| 1 · ideation onramp | `skills/process/ideation-mindmap` — 6-branch concept mindmap via surreal-memory | **adapt** — single-pass generation inherits diversity collapse; enforce independent-then-pool |
| 1/4 · staged gates | `skills/process/pmpo-evolver/skills/validate-idea` — three gates plus an Archive of Stepping Stones | **adapt** — the archive persists *attempts*, not outcomes |
| 2 · cross-model artifacts | `build-review-packet.sh` supports `diff\|artifact\|skill\|agent`; `dispatch-judge.sh` stamps `cross_model_check` | **adapt** — add `--mode decision`; reuse judge, findings schema, retry loop, sycophancy screen unchanged |
| 4 · persistence | `pk` wiki — focus-on-prompt + Stop-hook ingest, 5,260 recorded fires; OKF v0.1 needs only a non-empty `type` | **adapt**, not build-from-zero — add a decision entry type and a revisit query |
| 5 · **reflector** role | **`hooks/hooks.json:170` already has a `reflector` SubagentStop matcher** routing reflection output through the sycophancy gate at `strict`, with a 2-rejection cap | **adopt** — the reflector half of goal 5 is *already wired*; only the **coach** role is missing. No coach agent exists in `agents/`. |
| 5 · role separation | `kbd_require_producer_model` fail-closed guard + `kbd-idea-critic` | **adopt** — proven and live |
| 6 · harness delivery | `skills/learn/ui-surface` — `detect-surface-tier.sh` + `render.sh`, tier logic in **one** place | **adopt**, with a caveat below |

**Goal 6 caveat — Tier 0 alone is not a delivery claim.** Tier 0 text is a genuine
universal floor, but "works on Codex/Kimi" means those harnesses reach at least
**Tier 1**, which outside Claude Code is a **file-pair handshake**: write
`~/.prometheus/learn/ui/__ui_intent__.json`, then poll `__ui_response__.json` every
2 s with a 30 s timeout (`ui-surface/SKILL.md:96–104`). That mechanism exists but has
never been exercised by an ideation flow, and it only works if the harness actually
polls. **Plan must verify Tier 1 on the chosen non-Claude harness, not assume the
floor suffices.**
| 7 · fixtures | `tests/run-fixture-suite.sh` — flawed *and* clean, inversion fails the suite | **copy the pattern**; idea fixtures do not exist |

Genuinely build-required for the goals: **`--mode decision` + idea fixtures** and
**automation-bias countermeasures** (commit-before-reveal, calibration, friction) —
the latter absent from this pack *and* from all 21 surveyed tools.

## §10 Recommendation — goals first

**Ordering correction.** The first draft put WIT reconciliation and mobile fabric work
ahead of the ideation goals. That inverted the phase: it would have produced a
transport/plugin phase with the ideation capability as a trailing hope. Goals first.

**A. Ship the ideation capability (this phase, goals 1–7)**

1. Add **`--mode decision`** to the packet builder — the smallest change that makes
   goal 2 true, reusing machinery proven live last phase.
2. **Adopt `ui-surface` unchanged** for goal 6. Emitting `UiIntent` rather than
   rendering means Claude Code *and* a non-Claude harness are satisfied by
   construction, with Tier 0 as the floor.
3. **Adapt `ideation-mindmap` + `validate-idea` + `kbd-idea-critic`** rather than
   writing new ideation skills — enforce diversity structurally (independent
   generation → pool → judge) instead of prompting personas to disagree.
4. **Build automation-bias countermeasures** (goal 3) — the only fully-new work, and
   the one no competitor has.
5. **Extend the `pk` wiki** with a decision type + revisit query for goal 4.
6. **Commit weak/sound idea fixtures** (goal 7) using the proven inversion pattern.

**B. Record the fabric decisions now, build them next phase**

7. **Adopt iroh 1.0.3** for desktop/server/mobile; accept **relay-only in the
   browser** and say so plainly. Pin **≥1.0.2** for the relay DoS fix.
8. **Feature-gate `iroh-docs`** in `storage-provider` (`Cargo.toml:19`) — a one-line
   change that unblocks any future wasm target.
9. **Resolve the two WIT worlds** into `prometheus:component/*` **before** any skill
   is ported, or every skill gets ported twice.
10. **Add `frf-transport-iroh`** as a `FederationBridge` adapter — additive to FRF.
11. **Build the fabric-integration skill** so "all projects use this pack" is
    checkable rather than conventional.

**C. Keep the two-phase split** from assessment §8. This analysis makes the second
phase *more* concrete, not less necessary: WIT reconciliation, FFI bindings, and
on-device wasmtime proof are their own body of work — and none of them is a
prerequisite for shipping the ideation capability in A.

### Prerequisite blockers — state at analyze time

`goals.md` carries two blockers. Both were cleared earlier in this session and
**re-verified now**, not assumed:

| Blocker | State | Evidence |
|---|---|---|
| Commit the previous phase | **CLEARED** | `c88a05f` + `ffc0bea` on `main`, pushed; `origin/main` behind 0 / ahead 0 |
| Refresh stale plugin caches | **CLEARED** | `check-model-config.sh` → **0 DRIFT** (was 6); `kbd_require_producer_model` present in both `.claude` and `.codex` 1.6.0 caches |

This matters for the recommendation above: goals 1, 2 and 5 depend on the
producer≠judge guard being live *in installed copies*. Before the cache refresh it
was not, so any creator running from a plugin cache would have used a resolver
without the fail-closed guard.

---

## §11 Unresolved review findings

Two adversarial rounds ran against a cross-model judge (`verified-distinct` both
times). Round 1 returned three CRITICALs and all were **correct**: the first draft
researched the added iroh/WASM scope thoroughly while under-serving the seven phase
goals, omitted `cross_model_check` and `ui-surface` from the candidate set entirely,
and ordered WIT/mobile work ahead of the ideation goals. §9 and §10 are the fix —
six goal-serving candidates added (`cand-014`…`cand-019`), and the recommendation
re-ordered goals-first.

Round 2 returned three more. Two were valid and are now incorporated:

| Finding | Disposition |
|---|---|
| Goal 5 reduced to generic producer/judge separation | **Fixed.** `hooks/hooks.json:170` already wires a `reflector` SubagentStop matcher through the sycophancy gate — the reflector half is *live*; only the **coach** role is missing. Now stated in §9. |
| Non-Claude delivery treated as satisfied by Tier 0 | **Fixed.** Tier 0 is a floor, not a delivery claim. §9 now names the Tier 1 file-pair handshake (`ui-surface/SKILL.md:96–104`), notes it is unexercised, and requires plan to verify it on the chosen harness. |
| "Blockers dropped from the analysis" | **Fixed** by the table above. |

The two WARNINGs are recorded rather than actioned, with the reason:

- *"The 21-tool survey is not auditable from the packet."* Correct — the survey is
  recorded in `assessment.md` §3 with URLs and an explicit vendor-bias caveat, but a
  packet scoped to this phase's artifacts cannot re-run it. The build decision for
  automation-bias countermeasures does not actually rest on the survey: it rests on
  the harm literature (Microsoft Research 2025; PubMed 41076923) plus the fact that
  **nothing in this pack implements commit-before-reveal**, which is checkable here.
- *"Cross-repo claims are not verifiable from the packet."* Structural, and the same
  limitation recorded in `assessment.md` §9: `build-review-packet.sh` builds
  `file_tree` with `find -maxdepth 2`, so files under `skills/<domain>/<skill>/…`
  (depth 3+) are invisible, and files in *other repositories* are invisible by
  definition. All cross-repo claims carry file-and-line citations so they are
  checkable in those repos. **This remains open technical debt on the review tooling
  itself, not on the analysis.**
