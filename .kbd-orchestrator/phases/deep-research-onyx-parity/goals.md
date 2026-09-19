# Goals

- G1 Thread execution: stages 02-04 run as a constrained research-director dispatching isolated research-workers with a deterministic merge, leaving the stage contracts, stage numbers, and driver validators unchanged
- G2 Lossless handoff: each thread persists a fact dossier with inline content-addressed citations and claims carrying verbatim quotes, so synthesis reads prose evidence instead of only compressed graph claims
- G3 Multi-pass report: stage 09 splits into outline, parallel section writers, deterministic assembly, and a coherence editor, bound by a claim-set invariant that the assembler enforces
- G4 Budgets and failure semantics: cycle and wall-clock budgets at job, director, and thread level with force-complete paths, and a ledger row for every dispatch including partial and failed threads
- G5 Measurement: a benchmark harness reporting RACE, effective citations, citation accuracy, and verified-claim ratio, so parity with Onyx is falsifiable rather than asserted

## Context

> Phase: `deep-research-onyx-parity`
> Created: 2026-09-08 by `/kbd-new-phase` (runtime revision 917)
> Predecessor: `research-agent-hardening`, closed 2026-09-08 with 5/5 goals MET,
> 11/11 changes implemented and archived (rah-007 descoped and archived later the
> same day). Its `reflection.md` recommends this phase.

**Intent:** deep-research functionality on par with the Onyx project
(https://github.com/onyx-dot-app/onyx) as a core feature of the skill system.

**Source analysis:** `docs/deep-research/deep-research-skill-vs-onyx-report.md`
(2026-09-07, revised 2026-09-08). Its thesis: graft Onyx's *execution topology*
onto the skill pack's *evidence contract*. Onyx wins on execution — a search-less
orchestrator dispatching up to three isolated agents per cycle, each returning a
deliberately long fact-only dossier, merged by a deterministic citation step.
The skill pack wins on everything after execution — stage contracts, claim
labels, verifier ordering, adversarial review, derived verification status,
schema-validated packages. The report's gap table G1–G10 is the work.

**Scope decisions already taken** (predecessor decision log, D-22 and D-23):

- This is a subsequent top-level phase, not a child and not an amendment. The
  predecessor's goals do not depend on threading, and this design depends on what
  the predecessor built: stage contracts, claim labels, checkpoint, verifier
  ordering, derived `verification_status`. The report instructs "resist
  renumbering" — stage numbers 02–04 and 09 stay stable so the daemon, the schema
  checker, and every existing fixture keep working.
- The report is **input to assess, not a settled plan**. Its change list
  (drt-000..drt-008) and its revised decisions D-1..D-4 are proposals.

**Assess must resolve these before any change is specced** (D-23). The report
could not complete them because local MCP servers hung; they are not inherited
as fact:

1. liter-llm Rust API availability, from UAR `Cargo.toml` / `versions.toml`.
2. UAR-REM-002 status for the three correctness defects that gate the native runner.
3. The nature of `universal-agent-runtime/crates/prometheus-skill-system` — submodule, worktree, or drifted copy.
4. The Onyx benchmark standing (RACE ~54, "reported as #1 at points in 2026") is
   **unverified** until `tests/bench/` runs. It must not enter the plan as
   established fact.

**Known defects carried in from the predecessor**, both install-surface, both
found by running the pipeline for real:

- **D-A:** `com.prometheus.research.plist` sets no `PATH`, so the launchd daemon
  cannot resolve any harness binary. Fix is a `__PROMETHEUS_PATH__` placeholder
  matching `shared/launchagents/*.plist` plus the substitution in
  `scripts/install-binaries.sh`.
- **D-B:** the installed plugin generation ships a pre-phase stub driver
  (1920 bytes vs 30718), so the daemon's `~/.claude` driver candidate resolves to
  a script with none of the stage contract. Fix is a reinstall; `RESEARCH_DRIVER`
  is the documented interim override.

**Resolved since this file was written:** rah-007 tasks 2–3 and the rah-008 eval
re-baseline were **descoped** on 2026-09-08 — nothing consumed that baseline (no
build target, no gate, no CI), and learn-grade's accuracy claim was downgraded
from empirical to provisional rather than left overstated. Neither blocks this
phase. See §4 question 8 of `assessment.md` for the caveat that travels with the
dataset if G5 reuses it.

**Trade-offs the report flags for early decision:** token cost (three parallel
workers is 3–6× today's linear 02–04; mitigate with model routing and a
`--max-parallel 1` mode); dossier length versus context; the director's
no-search rule being advisory on harnesses that ignore `tools:`, with the merge
script as the real gate; and the rule not to go three levels deep.
