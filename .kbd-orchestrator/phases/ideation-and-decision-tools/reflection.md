# Reflection — ideation-and-decision-tools

**Phase:** `ideation-and-decision-tools`
**Closed:** 2026-07-31 · **Implementation:** 9/9 changes archived

## Goal achievement

| # | Goal | Verdict | Evidence |
|---|---|---|---|
| 1 | Judge with a find-problems mandate, not a persona round-table | **MET** | `assets/reviewer-mandate-decision.md` — single isolated reviewer, explicit "find the reasoning that will make this decision wrong", novelty scoring forbidden. No debate loop was built. |
| 2 | Every decision artifact carries `cross_model_check: verified-distinct` | **MET** | `findings.schema.json` top-level `required` includes `cross_model_check`; decision mode additionally requires the literal `verified-distinct`. Live: both fixtures recorded it, `isolation_mode: rest-gateway:http://localhost:8181/v1`. |
| 3 | Automation-bias countermeasures | **MET** | `commit-before-reveal.sh` exits 2 until a judgement is recorded; schema requires `confidence`, `what_would_change_this`, non-empty `disconfirming`. 11 assertions incl. 5 bypass attempts. |
| 4 | Persist decisions **and outcomes** into the wiki | **MET** | `decision-log.sh` record/outcome/revisit over OKF `type: Decision`; `revisit` returns both halves and marks unchecked ones PENDING. 15 assertions; mutation-tested. Exercised on this phase's own four fabric decisions. |
| 5 | Coach and reflector as separate roles that cannot grade themselves | **MET** | `agents/kbd-coach.md` holds a read-only tool grant, so it cannot persist a verdict; reflector hook byte-unchanged. Enforced structurally — the test fails if the coach gains a write tool or matches its own grading hook. |
| 6 | Claude Code **plus one non-Claude harness** via `ui-surface` | **MET** | Executed as `codex`: intent written, an independent blind responder answered, flow consumed it and continued in 2 s. Confirmed under `bash -x` that dispatch reached `_render_tier1_file_pair`, not a Tier 0 fallback. |
| 7 | Committed fixtures prove weak BLOCKs and sound PASSes, cross-model | **MET** | `weak-idea` BLOCK 4/4, `sound-idea` PASS 6/6, both `verified-distinct`. Inverting expectations turns the suite red; `--groups Z` exits 2 rather than reporting success. |

**7/7 MET.** Every verdict above rests on something executed this session, not on
a description of what the code intends to do.

## What actually happened vs. what was planned

### The plan was wrong about facts, twice — and the corrections matter

Change 009 was written to *record* decisions the analyze stage had already made.
Verifying those decisions against disk and crates.io before writing them down
found two errors:

1. **`iroh-webrtc-transport`'s repository was recorded as 404.** It returns HTTP
   200. The rejection still holds, but it now rests on the disqualifying fact —
   a `^0.98.2` pre-1.0 iroh pin against our 1.0.2 floor — rather than on a claim
   that is no longer true.
2. **The two WIT worlds were recorded as `uar:skill@0.1.0` and
   `knowme:plugin@0.1.0`.** UAR's second world is `uar:plugin@0.1.0`; KnowMe's
   `knowme:plugin` exists at **two versions simultaneously** (0.1.0 and 1.0.0).
   Four packages across two repos, not two — which *strengthens* the
   unify-before-porting ordering rather than weakening it.

**Root cause:** the analyze stage recorded conclusions without re-checking the
facts underneath them at write time. **Corrective action already applied:** every
claim in the four decision records was verified against a file path or a live API
response before being written, and the corrections are stated in the records
themselves rather than silently fixed.

### Change 008 exposed a live vulnerability the task list did not anticipate

Raising the iroh floor to ≥1.0.2 broke `sovereign-sync`'s build — because its
lockfile pinned **1.0.0**, the exact version with the relay DoS where one
malformed datagram from any client crashes an entire relay. The task said
"raise the floor"; the floor could not be raised without discovering that a
shipped crate was on the vulnerable version. Both it and `learner-model` now
resolve to 1.0.3.

### The sound fixture was wrong, and the judge was right about it

Change 007's `sound-idea` fixture passed only 4 of 6 runs. The tempting reading
was judge non-determinism. It was not: on each BLOCK the judge named a real
defect — an overstated "removes the compliance surface entirely", a payment
falsifier with no threshold, pilot criteria that did not match the decision's own
stated wedge, and a commitment to *build* before its own falsifiers had run.

Fixing the fixture rather than loosening the assertion took it to 6/6 with zero
criticals. **Non-determinism was tracking real internal inconsistency.** The
generalisable lesson: when a judge is inconsistent on a fixture, suspect the
fixture first.

One honest note on process here: two of my four "fixes" made the fixture *worse*
(1/6) by adding specificity that created new inconsistencies. What finally worked
was making the decision internally consistent — narrowing what it commits to —
not making it longer.

## Deltas — what was planned and not delivered

- **Decision mode was built but never documented.** Found during reflect: the
  `## Modes` section of `adversarial-review/SKILL.md` described `diff`,
  `artifact`, `skill`, and `agent`, and omitted `decision` entirely — the mode
  this whole phase exists to add. Fixed during reflect rather than deferred.
  **Root cause:** no change in the plan owned user-facing documentation; each
  change documented its own script but not the skill's entry surface.

- **`zed` resolves to Tier 0, not Tier 1.** `render.sh` detects `zed` but its
  Tier 1 dispatch routes only `opencode|codex|kimi` to the file-pair branch.
  Recorded as a stated limit in `references/harness-delivery.md` rather than
  described as delivery. Not a regression — pre-existing, and outside goal 6's
  "one non-Claude harness".

- **`opencode` and `kimi` were not exercised.** They share the identical code
  path as `codex` and are expected to behave the same. Expected is not verified;
  the reference says so.

- **Version invariants are documented, not enforced.** Three of four hold today
  and **nothing checks any of them**. Mechanical verification belongs to the
  `fabric-integration` skill, deferred with the WIT authoring it depends on.
  Recorded in the decision record itself so it cannot read as enforced.

## Technical debt introduced

| Item | Where | Note |
|---|---|---|
| `opencode` / `kimi` Tier 1 unverified | `references/harness-delivery.md` | same code path as the verified `codex`; run to confirm |
| `zed` Tier 1 routing absent | `ui-surface/scripts/render.sh:174` | one-line dispatch addition; pre-existing |
| Version invariants unenforced | `docs/decisions/fabric-version-invariants.md` | needs `fabric-integration` skill |
| Idea fixtures are calibrated, not arbitrary | `tests/fixtures/README.md` | any edit requires re-running determinism |

## Pre-existing issues found, not introduced

- **2 failing tests in `sovereign-sync`** (`one_projects_token_is_rejected_by_another_project`,
  `two_projects_mint_distinct_identities_and_tokens`). Confirmed by `git stash`
  that both fail identically without this phase's changes. Control-token
  derivation, unrelated to iroh. **Not fixed here** — outside scope, and
  silently absorbing them into this phase would hide them.

## Lessons

1. **Verify the plan's facts at write time, not just its reasoning.** Two of the
   plan's factual claims had decayed between analyze and execute. Records that
   restate a plan inherit its staleness.

2. **When a judge is inconsistent on a fixture, suspect the fixture.** The
   instinct to blame model non-determinism would have produced a loosened
   assertion and a worse fixture.

3. **"Demonstrated" is not "enforced."** Goal 5 asserts the coach *cannot* grade
   itself; goal 6 ran the round trip rather than asserting it; goal 7 fails on
   inversion and on zero assertions.

4. **A timeout is not a response.** `emit-ui-intent.sh` exits 3 and says so,
   because a caller treating "some text appeared" as delivery cannot distinguish
   a working round trip from a silent fallback.

## Recommended next phase — `mobile-skill-portability`

Deliberately deferred here, with dependencies already settled:

- **Author the `prometheus:component/*` WIT family** — the decision and its
  ordering constraint are recorded; the authoring is not done. This blocks
  porting, by design.
- **Build the `fabric-integration` skill** — makes the four version invariants
  enforced rather than documented.
- **Mobile FFI bindings** — this pack has no cdylib/staticlib and no uniffi;
  `frf-ffi` (uniffi 0.31.2) is the pattern to copy.
- **Verify `opencode` and `kimi` Tier 1**, and route `zed` to the file-pair
  branch or state why not.

## Sycophancy gate

Routed through `analyze_reflect_phase` at `strict` (the gate's own fixed level):

| Field | Result |
|---|---|
| `sycophancy_score` | **0.018** (rejection threshold 0.4) |
| `s08_detected` (Reflect Phase Inversion) | **false** |
| Patterns | 1 × S-07 (Low) — length |

S-08 is the one that matters for a reflection: it fires when the output
summarises success instead of naming deltas. It did not fire.

The gate's suggested rewrite proposed deleting the goal table, the evidence
column, and the technical-debt inventory to satisfy a three-section template.
**Not applied** — the evidence column is what makes a 7/7 claim checkable, and
removing it to score better on a formatting heuristic would trade the substance
for the appearance of rigor. The S-07 length flag was addressed instead by
cutting the two Lessons entries that restated the delta narrative verbatim.
