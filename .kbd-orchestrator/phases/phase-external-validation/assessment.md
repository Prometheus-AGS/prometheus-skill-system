# Assessment — phase-external-validation

**Date:** 2026-06-30  
**Assessor:** Claude Code (Sonnet 4.6)  
**Previous phase:** phase-credibility-90 (92% readiness, 5/5 P1/P2 re-audit findings closed)  
**Sycophancy gate applied:** yes — detect_sycophancy at strict strictness

---

## Context

The production-readiness report (`docs/production-readiness-report.md`) scores the
prometheus-skill-pack at 92%. The remaining 8% is entirely in P5 (Strategic, weight 5%)
and the sycophancy-corrected ceiling for a single-maintainer project without external
deployment evidence. **No code change can close this gap.** External validation is the
only path forward.

This assessment maps each of the five goals in `goals.md` against the current state of
the codebase and project to identify what is in place, what is missing, and what is
actually feasible for a phase that does not write code.

---

## Current state — what is already in place

### Installation and onboarding infrastructure

| Artifact | Status |
|---|---|
| `scripts/install-skills-flat.sh` | Present and tested |
| `docs/guide/19-installation.md` | 24-page guide with step-by-step flow |
| `CONTRIBUTING.md` | Present in `docs/` |
| `.github/ISSUE_TEMPLATE/bug_report.md` | Present |
| `.github/ISSUE_TEMPLATE/feature_request.md` | Present |
| `.github/ISSUE_TEMPLATE/skill_proposal.md` | Present |
| `README.md` | Public, links to full guide |
| GitHub repo | **PUBLIC** at https://github.com/Prometheus-AGS/prometheus-skill-system |

The installation guide (`19-installation.md`) is detailed and covers prerequisites,
the clone + submodule flow, building tools, MCP service launch, and `forge init`.
An external user who follows it step-by-step has a path to a running system.

**Gap:** The guide assumes familiarity with Rust toolchain management, launchd/systemd,
and MCP concepts. There is no "5-minute quick-start" that gets to a first `/learn-goal`
invocation without prerequisite knowledge. A first-timer will hit friction at the
`check-prerequisites.sh --build-tools` step if Rust is not installed.

### Learning loop (G1) — Feynman loop

The 12 learn skills are present (`skills/learn/`): `learn-goal`, `learn-survey`,
`learn-plan`, `feynman-loop`, `learn-grade`, `learn-retain`, `learn-practice`,
`learn-certify`, `learn-kb`, `learn-about-system`, `learn-harness`, `ui-surface`.

The `surface-bridge` Axum service (`substrate/surface-bridge`) is built and
installed as a launchd service. The sycophancy gate is wired on the `learn-grade`
critical path.

**Gap:** No external user has run this loop. The mastery criterion (score ≥ 0.7,
`misconceptions_absent`, two transfer problems, 24h retention check) is correct but
only tested internally by the maintainer. There is no documented first-session log
from anyone else.

### Self-improving loop (G2) — forge enrich → reflect → enrich

`forge enrich` and `forge reflect` are built and tested (forge-rs, 44 tests passing).
The `resolve()` function in `forge-skills` now accepts the stale `HashSet<String>`
and deprioritizes stale skills. The drift report writes to `.forge/memory/drift/`.

**Gap:** No external user has run this loop. The wiring (`resolve()` → stale bucket
→ deprioritize) is unit-tested but not demonstrated end-to-end on a real project that
is not the maintainer's own.

### P2P sync (G3) — sovereign-sync

`sovereign-sync` crate is present with iroh 1.0 + Loro 1.13 backend. The
`IrohDocsAdapter` now has real share/import ticket support (added in
phase-sovereign-sync-hardening). The two-node sync regression test (`sync_roundtrip`)
passes but both nodes run on **the same host process** — not across distinct machines.

**Gap:** No two-machine validation exists. This requires access to two separate
machines (or two containers on different network namespaces). This is the hardest
goal in the phase to validate without external infrastructure.

### Sycophancy gate (G4) — independent validation

The `sycophancy-correction` MCP server is running. The gate is wired on `learn-grade`
and the PMPO reflector SubagentStop hook. The gate fires correctly (confirmed: gate
correctly detected and rejected sycophantic reflections during phase-credibility-90;
score 0.0 on strict honest reflection).

**Gap:** All validation to date has been done by the maintainer using the same Claude
Code session. No independent third party has tested the gate with a known-sycophantic
input and a known-honest input to confirm the gate fires correctly in both directions.

### Evidence artifact (G5) — public documentation

`docs/production-readiness-report.md` exists and is committed to the public repo.
The report is linked from `README.md`. It contains the honest 92% claim with the
P5 structural gap explanation.

**Gap:** The report is authored by the maintainer, not by external validators. There
is no GitHub discussion, issue, or external blog post that independently confirms
any of G1–G4. A follow-up report written after G1–G4 complete would be the artifact
that justifies a claim above 92%.

---

## Goal-by-goal gap analysis

| Goal | In place | Missing | Feasibility |
|---|---|---|---|
| **G1** First external user — Feynman loop | Install guide, 12 learn skills, surface-bridge launchd, issue templates | A willing external user; no "5-min quick-start" | MEDIUM — requires community outreach or a known collaborator |
| **G2** Self-improving loop validation | forge enrich + reflect + stale wiring tested | External user running on a real project; no quick start for forge | MEDIUM — same blocker as G1 |
| **G3** Sovereign-sync two-node P2P | iroh-docs share/import, same-host test passes | Two distinct machines or containers; no setup guide for two-node | HARD — requires infra or a second collaborator |
| **G4** Independent sycophancy gate validation | Gate wired, CI tested, binary present | Third-party tester with known-sycophantic + known-honest test cases | MEDIUM — can be automated; test cases need authoring |
| **G5** Public evidence artifact | production-readiness-report.md public | Post-validation update or external testimonial | EASY — write after G1–G2 complete |

---

## Blocking gaps

**BG-1 — No quick-start for external users.** The 24-page installation guide is
correct but not approachable for a first-time adopter who does not already know what
an MCP server is. A one-page quick-start (prerequisites → clone → install → first
`/learn-goal` in under 10 minutes) is the highest-leverage enabler for G1 and G2.

**BG-2 — No known external collaborator.** Both G1 and G2 require a second person.
Without outreach or a known collaborator, these goals cannot close. Code changes do
not fix this.

**BG-3 — No two-machine sovereign-sync setup guide.** G3 is blocked by infrastructure.
A setup guide for running sovereign-sync on two hosts (or two Docker containers) would
lower the barrier to testing. The guide does not exist.

**BG-4 — No known-sycophantic / known-honest test corpus for G4.** The gate fires
correctly on maintainer-authored inputs. An independent test requires a pre-authored
corpus of known-sycophantic and known-honest reflections that a third party can
reproduce. This corpus does not exist.

---

## What this phase will do

This phase cannot write production code. It will:

1. **Write a quick-start guide** (`docs/QUICK_START.md`) — one page, five steps,
   working example with `/learn-goal "explain recursion"`. Eliminates BG-1.

2. **Write a two-node sovereign-sync setup guide** (`docs/SOVEREIGN_SYNC_TESTING.md`)
   — Docker Compose or two-host setup with copy-paste commands. Addresses BG-3.

3. **Author the sycophancy gate test corpus** (`tests/sycophancy-corpus/`) — 3
   known-sycophantic reflections and 3 known-honest reflections, with expected gate
   verdicts. Provides the artifact G4 needs; enables independent reproduction.

4. **Create a GitHub discussion or issue** asking for first-user feedback — links the
   quick-start and explains what the maintainer needs from early adopters. Opens the
   community path for G1/G2.

5. **Write an updated evidence artifact** once any external validation occurs — a
   follow-up section in `docs/production-readiness-report.md` recording outcomes.

---

## What this phase will NOT do

- Rewrite forge-rs, sovereign-sync, or any Rust substrate crate
- Add more unit tests (44 is sufficient for the current internal scope)
- Change the 92% claim before external validation occurs

---

## Assessment verdict

**The phase is valid.** All five goals are achievable in principle. G1, G2, and G5 are
unblockable by code changes alone — they require human action. G3 and G4 can be
partially addressed by creating setup guides and test corpora. The net effect of this
phase is to remove all code-addressable barriers to external validation, leaving only
the human coordination gap.

**Recommended change count:** 4 (one per deliverable — quick-start, two-node guide,
sycophancy corpus, GitHub discussion/issue).

**Assessment complete.**
