# Scope decisions for the 2026-08-20 architecture review

**Companion doc to** [`2026-08-20-skill-pack-architecture-review.md`](2026-08-20-skill-pack-architecture-review.md).
**Read §18 of that file first** — it records the corrections this document acts on.

The review proposes a 6-pillar roadmap plus a cross-cutting Pillar 7. Read
without qualification it looks like an unstarted backlog for *this* repository.
It is not. This document records what was taken on, what was deliberately not,
and — for each exclusion — the verified reason.

## In scope, and done

| Review area | What landed |
|---|---|
| §1 Reliability | `ThrottleInterval` on all five restarting LaunchAgents; PID-aware bootstrap lock (60 s stall → 0.036 s); `config/defaults.env` as the single identity source |
| §6 Hooks | Observability extended to all 17 contract-wired hooks; four remedies withdrawn on measurement (§18.6) |
| §7 Skill discovery | Authoring guide + `checkDescriptionQuality()` validator check; three competing descriptions rewritten |
| §5 Build time | `sccache` wired (0 → active; clean build 8.62 s → 3.33 s); `lld`; `config/nextest.toml` |
| §8/§9 misc | Plugin schema, `strict: false` documentation, install-script drift |

## Not in scope: Pillars 5 and 6

**Pillar 5 (Prometheus Companion — Tauri tray + dashboard, ~10 engineering days)
and Pillar 6 (Prometheus Mobile — Flutter + Rust FFI, ~9 engineering days) are
not skill-pack work.** They are new applications in a different repository.

Verified state of `/Users/gqadonis/Projects/prometheus/prometheus-companion` at
the time of this decision: **two commits** (`19e2f08`, `773aa5e`), containing
`src-tauri/src/{lib,main}.rs`, a stub `src/app.tsx`, an audit-script suite, and a
4405-line specification. No `health.rs`, no `plugin.rs`, no Tauri command
surface, no tray. The specification is thorough; the implementation has not
started.

Nothing in the skill-pack should wait on, or call into, the Companion. Where the
review routes a skill-pack weakness through a Companion feature (W1.3, W1.4,
W1.7, W7.4, W7.6, W9.4, W9.7), the skill-pack either fixed it locally — as with
W9.4, now `config/defaults.env` — or recorded it as deferred with a reason.

## Not in scope: the six HMA skills

§16.1 states that six new skills "ship in the HMA `v0.2.0` release" and that the
Companion enforces them by running HMA-provided verifier scripts. **None of them
exist.**

Verified in `/Users/gqadonis/Projects/hybrid-mobile-architecture-src`
(version `2.0.0-alpha.2`, **30 skills**):

| Skill §16.1 says ships | Present |
|---|---|
| `connected-skill-packages` | No |
| `tauri-tray-app` | No |
| `launchagent-supervisor` | No |
| `realtime-skill-refiner` | No |
| `claude-hooks-reliability` | No |
| `auto-skill-package-integration` | No |

The verifier scripts §1.4 and §6.10 tell the Companion to run —
`verify-supervisor.sh`, `render-supervisor-plist.sh`,
`install-launchagent-supervisor.sh`, `verify-hooks-reliability.sh`,
`install-hooks-reliability.sh`, `verify-skill-manifest.sh` — do not exist either.

**Consequence for this repo:** the "HMA ships the scripts, the Companion runs
them" chain in §1.4, §6.10, and §15 is an *architectural intention*, not a
dependency that can be called. The reliability and hook fixes above were
therefore implemented directly in the skill-pack, using the patterns already
proven in-repo (`ai.prometheus.sovereign-sync.plist`,
`shared/scripts/lib/hook-log.sh`) rather than waiting for an external skill.

Authoring those six skills in the HMA repo remains reasonable future work. It is
simply not this repository's work, and the skill-pack must not assume it.

## Deferred inside the skill-pack, with reasons

| Item | Why deferred |
|---|---|
| **R7.4** semantic router (`fastembed-rs` + vector index in `skill-index`) | `substrate/skill-index` is a 142-line crate doing literal substring matching, with no embedding dependency. This is a genuine multi-day feature. The description backfill is cheaper, and may reduce the need for it — do it after, and measure first |
| **R7.6** lazy progressive disclosure | Same reasoning; depends on the router landing first |
| **R7.3** forced-eval `UserPromptSubmit` hook | Taxes *every* prompt to compensate for description quality. Fix the descriptions first, then re-measure whether this is still needed |
| **R5.3.2** superworkspace (merge 46 lockfiles) | The review's own "M effort, M risk". Most `[workspace]` roots are git submodules that must keep building standalone. `sccache` already recovers much of the benefit — verified: a *separate* workspace picked up 51 cache hits from another workspace's compilation |
| **R1.6** self-healing watchdog, **R1.7** desktop notifications | Both add a new always-on process, which works against the review's own service-consolidation goal (§3). Revisit if a service is actually observed being removed by launchd |
| **R6.8** `prom-hook-dispatch` Rust binary | Adds a binary to the install path for a bug class that §18.3 shows cannot occur, since the hook manifests are generated from one template |

## Standing rule

Everything above was validated locally, per `CLAUDE.md` → *Local-Only Validation
(MANDATORY)*. No GitHub Actions run was used as evidence for any claim in this
document.
