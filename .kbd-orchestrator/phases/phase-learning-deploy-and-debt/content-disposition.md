# Candidate content disposition

This is production source bookkeeping for change15, not acceptance or a release. Every inventoried path has an explicit decision, source owner, candidate hash, currentness boundary and route disposition in [content-inventory.json](content-inventory.json). No tests, compiler, checks, generation, build, runtime probe or independent reviewer ran. Final evidence belongs to change12.

## Coverage and source authority

The candidate contains **8,519 reader-source records**: 6,156 full, 2,363 mini. All Git-tracked Markdown, MDX and README surfaces are included, together with new working-tree reader pages and four explicitly handed site landing/navigation sources. Gitlinks and vendor repositories were not initialized or traversed. The inventory excludes its own disposition document to avoid recursive fingerprinting.

Full uses owner-selected baseline `ba5c4516fd72e60c21ab3bb1d09a63105aeb3c61`; mini uses refreshed `ce893fb89bb20c250ff222a6d12c2e0f4820eef0`. Candidate hashes include admitted dirty source. Neither identity selects a new release. Full has no root `versions.toml`: source declarations, the release matrix and pinned Git dependencies are its authorities. Mini retains its owner-controlled version file.

This replaces rough `content-scope-preflight` for source coverage; it does not inherit its filename-only classifications or old mini baseline. Per-page authored decisions use actual topics, purpose excerpts, observed claims and owner handoffs. Retaining unrelated skill/reference guidance is an explicit source disposition, not independent certification of external-library examples. Confirmed pack-contract conflicts have exact owners; no blanket scope assigns hundreds of skill references to a docs/site writer.

| Classification | Candidate records |
|---|---:|
| engineering-instructions | 32 |
| generated-or-managed-mirror | 4,110 |
| history-or-design-record | 2,650 |
| mixed-source-and-generated-index | 1 |
| owned-source | 1,044 |
| owned-upstream-adaptation | 81 |
| protected-test-or-fixture-reference | 52 |
| upstream-provenance-or-vendor | 549 |

## README families

Full task15/1 rewrote 31 source READMEs and retained four inspected source contracts. Mini task15/2 rewrote its root onboarding and UAR schema-family README; four other owned source READMEs remain useful and accurate within their stated boundaries. Every retained fixture, upstream and generated README has its own preserve/source-regeneration decision in the JSON.

| Pack and owned source | Decision | Source boundary |
|---|---|---|
| full: `.opencode/README.md` | rewrite | Rewrote "Full-pack OpenCode adapter" using declared source behavior and explicit installation/evidence boundaries; no runtime or acceptance claim. |
| full: `README.md` | rewrite | Rewrote "Prometheus Skill Pack" using declared source behavior and explicit installation/evidence boundaries; no runtime or acceptance claim. |
| full: `crates/prometheus-exec/README.md` | rewrite | Rewrote "prometheus-exec" using declared source behavior and explicit installation/evidence boundaries; no runtime or acceptance claim. |
| full: `docs/README.md` | rewrite | Rewrote "Documentation" using declared source behavior and explicit installation/evidence boundaries; no runtime or acceptance claim. |
| full: `docs/deep-research/README.md` | rewrite | Rewrote "Native research client" using declared source behavior and explicit installation/evidence boundaries; no runtime or acceptance claim. |
| full: `docs/guide/README.md` | rewrite | Rewrote "Prometheus Skill Pack guide" using declared source behavior and explicit installation/evidence boundaries; no runtime or acceptance claim. |
| full: `examples/prometheus-exec/README.md` | rewrite | Rewrote "Prometheus Exec runnable examples" using declared source behavior and explicit installation/evidence boundaries; no runtime or acceptance claim. |
| full: `policies/README.md` | rewrite | Rewrote "Cedar Policies" using declared source behavior and explicit installation/evidence boundaries; no runtime or acceptance claim. |
| full: `shared/scripts/README.md` | rewrite | Rewrote "Shared scripts" using declared source behavior and explicit installation/evidence boundaries; no runtime or acceptance claim. |
| full: `shared/scripts/scheduled/README.md` | rewrite | Rewrote "Scheduled maintenance templates" using declared source behavior and explicit installation/evidence boundaries; no runtime or acceptance claim. |
| full: `skills/imported/README.md` | rewrite | Rewrote "Imported skills" using declared source behavior and explicit installation/evidence boundaries; no runtime or acceptance claim. |
| full: `skills/learn/README.md` | rewrite | Rewrote "learn — Feynman-Spine Learning Domain" using declared source behavior and explicit installation/evidence boundaries; no runtime or acceptance claim. |
| full: `skills/process/agent-team-creator/schemas/uar/README.md` | rewrite | Rewrote "UAR collaboration schema snapshots" using declared source behavior and explicit installation/evidence boundaries; no runtime or acceptance claim. |
| full: `skills/process/delivery-cadence/examples/README.md` | keep | Editable admission example deliberately lacks an entry point; it does not authorize operation execution. |
| full: `skills/process/iterative-evolver/README.md` | rewrite | Rewrote "Iterative Evolver" using declared source behavior and explicit installation/evidence boundaries; no runtime or acceptance claim. |
| full: `skills/process/kbd-process-orchestrator/shared/openspec/README.md` | keep | Managed CLI source declares refresh/run, explicit pin/offline/disabled outcomes, scoped backups and exit contracts. |
| full: `skills/process/prometheus-context-bootstrap/references/rules-src/project/README.md` | keep | Explains authored path-scoped rule source, rather than generated target edits. |
| full: `skills/process/spec-gate/README.md` | rewrite | Rewrote "spec-gate" using declared source behavior and explicit installation/evidence boundaries; no runtime or acceptance claim. |
| full: `skills/react/prometheus-entity-skills/entity-graph-optimize/component/README.md` | rewrite | Rewrote "`entity-graph-optimize` as a WASM component" using declared source behavior and explicit installation/evidence boundaries; no runtime or acceptance claim. |
| full: `skills/rust/librefang-wasm-skill/references/example-echo/README.md` | rewrite | Qualify the actual 'Echo Skill — LibreFang WASM Example' source/host prerequisites and distinguish an API/manifest/build from live execution acceptance. |
| full: `skills/testing/bdd-cucumber-js/references/examples/README.md` | rewrite | Rewrote "JavaScript BDD example source" using declared source behavior and explicit installation/evidence boundaries; no runtime or acceptance claim. |
| full: `skills/testing/bdd-cucumber-rs/references/examples/README.md` | rewrite | Rewrote "Rust BDD example source" using declared source behavior and explicit installation/evidence boundaries; no runtime or acceptance claim. |
| full: `substrate/exec-contracts/README.md` | rewrite | Rewrote "`prometheus-exec-contracts`" using declared source behavior and explicit installation/evidence boundaries; no runtime or acceptance claim. |
| full: `substrate/exec-core/README.md` | rewrite | Rewrote "`prometheus-exec-core`" using declared source behavior and explicit installation/evidence boundaries; no runtime or acceptance claim. |
| full: `substrate/exec-embedded/README.md` | rewrite | Rewrote "`prometheus-exec-embedded`" using declared source behavior and explicit installation/evidence boundaries; no runtime or acceptance claim. |
| full: `substrate/exec-remote/README.md` | rewrite | Rewrote "`prometheus-exec-remote`" using declared source behavior and explicit installation/evidence boundaries; no runtime or acceptance claim. |
| full: `substrate/exec-service/README.md` | rewrite | Rewrote "`prometheus-exec-service`" using declared source behavior and explicit installation/evidence boundaries; no runtime or acceptance claim. |
| full: `substrate/exec-tier-p/README.md` | rewrite | Rewrote "`prometheus-exec-tier-p`" using declared source behavior and explicit installation/evidence boundaries; no runtime or acceptance claim. |
| full: `substrate/exec-tier-w/README.md` | rewrite | Rewrote "`prometheus-exec-tier-w`" using declared source behavior and explicit installation/evidence boundaries; no runtime or acceptance claim. |
| full: `substrate/learner-model/README.md` | keep | Distinguishes recorded FSRS-5 scheduler dependency from older FSRS-6 terminology and real RPC target from generic unit evidence. |
| full: `substrate/skill-ffi/README.md` | rewrite | Rewrote "`skill-ffi` — the mobile FFI boundary" using declared source behavior and explicit installation/evidence boundaries; no runtime or acceptance claim. |
| full: `substrate/surface-bridge/README.md` | rewrite | Rewrote "surface-bridge" using declared source behavior and explicit installation/evidence boundaries; no runtime or acceptance claim. |
| full: `tools/bench-automation/README.md` | rewrite | Rewrote "Research benchmark tooling" using declared source behavior and explicit installation/evidence boundaries; no runtime or acceptance claim. |
| full: `tools/forge-rs/README.md` | rewrite | Rewrote "forge-rs" using declared source behavior and explicit installation/evidence boundaries; no runtime or acceptance claim. |
| full: `tools/prometheus-cli/README.md` | rewrite | Qualify the actual 'Prometheus CLI in The Boss' source/host prerequisites and distinguish an API/manifest/build from live execution acceptance. |
| mini: `README.md` | rewrite | Current entry point: Node doctor and receipt-preserving copy fix; separate payload registration, first KBD workflow, teams/models, optional scoped publication, Compose/Companion, recovery and local contribution boundaries. |
| mini: `docker/README.md` | keep | Read source operations against current compose and service runner: three containers, loopback ports, optional external endpoints, volume retention, prebuilt images and migration/export boundary. |
| mini: `lib/platform/openspec/README.md` | keep | Retain actual managed CLI contract: pinned baseline, explicit latest discovery, offline/migration/pending outcomes, shared deadline, locks and generated-integration ownership; Node22 mini minimum still applies. |
| mini: `rules/src/project/README.md` | keep | Retain rules-authoring path-scoped reference instructions; this README is not rendered into project rules. |
| mini: `skills/agent-team-creator/schemas/uar/README.md` | rewrite | Aligned full/mini UAR schema-family explanation and actual consumer-receipt authority; removed stale separately duplicated provider commit. Provider schemas and receipt bytes preserved. |
| mini: `skills/delivery-cadence/examples/README.md` | keep | Retain examples as editable declarations, intentionally absent operation entry point, and non-authorizing evidence; no live project changes promised. |

Mini onboarding now documents Node22, the install-scope restriction, actual doctor/copy fix and receipt ownership, separate plugin registration, first workflows, teams/model selection, optional project-scoped publication, Compose service ownership, optional Companion, update/uninstall limits and local contribution gates. It does not advertise the full installer or rollback API. Identity filters do not establish server authorization. Compiled team runtime and generated packages are reconciled at final12.

The obsolete original mini README is removed from current onboarding. Its exact body remains in Git at `ce893fb89bb20c250ff222a6d12c2e0f4820eef0:README.md`; relevant existing `COMPARE.md` and `TOOL_ANALYSIS.md` records are retained under their historical dispositions. No duplicate archive copy was added. Existing history, provider and license bytes were not rewritten by this task. No public site route is deleted by the README replacement.

## Coordinated source dispositions

Task17 handed 219 exact authored page decisions in `/tmp/ldd-task17-2-page-dispositions.json`, with `/tmp/ldd-task17-2-source-handoff.json` retaining `taskComplete:false` and no acceptance. The inventory overlays those semantic notes and source evidence, then fingerprints actual candidate bytes. Later root-owned corrections can advance a source beyond its earlier editorial hash; both the handoff identity and current hash remain explicit.

Current docs/site prose is source-reconciled. Existing sovereign-sync routes remain relocation stubs. Historical/proposal/evidence pages keep their bodies with non-onboarding boundaries. Generated skill catalog sections, mini routing docs and rule mirrors stay at their canonical generation boundary. New service pages, full/mini maintenance drafts, and site landing/navigation source are included even when untracked.

Task18 maintenance source is reconciled with the completed14/2 Companion operational handoff. Both `docs/maintenance.md` files and `maintenance-readiness.md` retain `maintenanceReady: false`; approved03 graph values,14/3 publication links and12 evidence remain pending. Migration, team-memory and Companion handoffs establish source only, never executed migration or compatible installed artifacts.

Root narrow task17 source corrections cover the actual `native-agent` guide, BDD lifecycle skill, visual-baseline reference and spec-gate who-dimension reference. These now describe real local integration, signed protected-test approval where applicable and human merge; hosted CI and labels do not replace acceptance. Provider schemas and protected scenarios remain preserved.

## Remaining source reconciliation

All admitted reader-source families now have a per-page source disposition. Task16’s handoff distinguishes 10 authored paths from 21 source-provenance input references. The two authored models-memory references retain later learning-contract changes; unchanged references are kept with their actual domain source evidence. The six creator profile/parity conflicts discovered during15 reconciliation were corrected under explicitly authorized task17 scope, not retroactively credited as task16 edits. New authoring uses selected draft.2 receipt identity; draft.1 migration remains supported. Shared portable contracts do not promise identical full/mini source or emitted bytes.

Exact handoffs are `/tmp/ldd-task16-source-handoff.json` and `/tmp/ldd-task17-team-source-corrections.json`; current candidate source hashes and completed canonical16 source receipts are recorded in the inventory. Neither handoff establishes generated, installed, provider or platform acceptance. No pending source owner decision is hidden as accepted content.

Root completed the exact mini rule-source truth corrections: README is current onboarding; the supplied stack is three Docker containers for two optional capabilities on supported hosts; reuse of full services retains full ownership and its distinct dataset. Generated AGENTS/CLAUDE/rule mirrors remain deferred to final12.

Remaining release deltas are concrete: owner-approved source/version/image/schema identities under change03; actual Companion publication or explicit unpublished state; final runtime compilation and generated output; final installed-harness/service/migration evidence; and refresh of these candidate hashes after those approved changes. No fictional public Companion URL, future release number, runtime success or historical obligation closure is inserted.

## Retention and public routes

Upstream skills, provider schemas, attribution/license records, protected fixtures and historical receipts retain their byte-authority. Generated copies are kept for final reconciliation from source, never edited as canonical prose. Historical records remain outside normal onboarding or behind their stated historical boundary; `archive` in this inventory is a semantic disposition, not an unreported file move. Public docs routes remain present or explicitly retained as relocation stubs. Any later removal needs a deliberate route/redirect decision.

The lead alone owns canonical KBD transitions, source-pin approval, final regeneration, certification and copying candidate artifacts to MAIN.

## Mechanical affected-source refresh

At `2026-10-05T19:33:23.720225+00:00`, refreshed 222 existing reader rows from 219 editorial dispositions, 20 operations handoff paths and 3 maintenance source paths. These overlap: the 20 operation paths are part of the 219 editorial records. Added 23 prior-hash history records; no new inventory scope or product audit occurred.

The latest operations source describes actual selected Unix socket/one host, separate key/config/data scope, durable signed intent and original-host receipt/replay; source does not establish peer application or accepted installed behavior. Approved release fields/publication links remain lead-owned and final12 owns evidence.

The hosted-workflow source disposition retains exact before-hashes and Git history for removed mini CI and memory image workflows; the memory Pages workflow retains permitted packaging/deployment after removal of typecheck. These YAML/memory-repository paths are outside this full/mini reader inventory, so they are annotated as external owner evidence without adding new rows or runnable archive copies. The memory container-publication document is likewise external release-method evidence. No hosted workflow was executed or used as acceptance.
