# Plan — uar-host-execution

**Phase:** `uar-host-execution` · **Planned:** 2026-07-31
**Backend:** OpenSpec · **Changes:** 16
**Scope:** eleven items — six seeded (S1–S6) plus five added (R1–R5)
**Cross-repo:** `universal-agent-runtime` **authorised**. `flint-realtime-fabric`
and `know-me-system` are **not**, and nothing here touches them.

## Premises re-verified at plan time

The assessment labelled its cross-repo claims "re-run these before planning".
Done:

| Check | Result |
|---|---|
| `check-uar-discovery.sh` | **exit 0** — UAR still declares this repo as a submodule and still discovers `skill.wasm` under `crates/prometheus-skill-system/skills` |
| Schema has no `origin`/`enabled` column | **confirmed** — no migration defines either |

> **These are cross-repo and NOT verifiable from this repository's review
> packet.** Review flagged it, and it is the same structural limit the last two
> phases hit: the packet builder does not descend into sibling repositories.
> Every UAR claim below carries a reproduction command; the file excerpts and
> SHA-256 hashes are in
> [`evidence/uar-code-evidence.md`](evidence/uar-code-evidence.md).
> **Each change re-runs its own command before acting** — no change proceeds on
> a claim inherited from this document.

**And one assessment finding was corrected by re-verifying.** R2's gap is *not*
"protection lives in memory only": `postgres.rs:77` serialises the whole `Skill`
into `definition` JSONB, and both `origin` and `enabled` carry
`#[serde(default)]` without `skip`, so they persist across restarts. The actual
gap is that **`DELETE FROM skills` has no guard** — the `Builtin` check exists
only in `service.rs:374`, so any caller reaching the storage provider directly
bypasses it. That changes change-uhe-004 from "add persistence" to "add a
DB-level constraint", which is a different and smaller change.

## Ordering principle

**In-repo and cheap first, cross-repo after.** The five seeded goals that need
no UAR access size and de-risk nothing else, so they land first and fast. The
UAR work is ordered so that **provenance comes before anything that depends on
knowing which pack version is loaded** — without it, R5 is unmeasurable and the
359-commit drift recurs.

## Wave 1 — in-repo, no dependencies (S2, S3, S4, S6)

### `change-uhe-001-cursor-tier1`
**Goal:** S6 · **In-repo**

`cursor` resolves to `tier0_text` with the **identical two causes `zed` had**:
`detect-surface-tier.sh:76-79` hardcodes the tier, and `render.sh` omits
`cursor` from **both** Tier 1 dispatch lists.

**Acceptance — exactly one of two outcomes:**
- **Routed and verified:** the file-pair round trip **runs** under `cursor` with
  an independent responder, confirmed under `bash -x` to reach
  `_render_tier1_file_pair`.
- **Not routed:** a committed diagnostic saying why, `cursor` stays documented
  as Tier 0, and **no Tier 1 claim is made.**

### `change-uhe-002-ci-sibling-repos`
**Goal:** S4 · **In-repo**

The `fabric-invariants` CI job uses a bare `actions/checkout@v4`, so three of
four invariants report `SKIP`.

**Target:** `.github/workflows/validate.yml`, job `fabric-invariants` (verified
present at plan time). Today it runs a bare `actions/checkout@v4`.

**Acceptance:** that job checks out the sibling repos (or sets `FRF_ROOT`/
`UAR_ROOT`/`KNOWME_ROOT`) and `check-invariants.sh` reports **4 of 4 verified,
0 SKIP** —
proven by the workflow log, not by local runs. **If the sibling repos cannot be
checked out in CI** (private-repo credentials), record that as a stated limit
and keep SKIP — do **not** silently claim coverage.

### `change-uhe-003-ffi-marginal-cost`
**Goal:** S3 · **In-repo**

Close falsifier 3 of the FFI pattern decision.

**Acceptance:** a second function is added to `substrate/skill-ffi`, and the
hand-written Dart + Rust-annotation + build-config lines it required are
**counted and recorded**. Per the decision, **>~20 lines reverses the pattern
choice** — so this change may end in a recorded reversal, and that is a valid
outcome, not a failure.

### `change-uhe-004-librefang-abi-decision`
**Goal:** S2 · **In-repo** · Decision only

`skills/rust/librefang-wasm-skill/` ships core-wasm `extern "C"` templates with
zero `.wit`; they **cannot** load in the component runtime S1 de-stubs.

**Acceptance:** a decision record via `decision-log.sh` with alternatives
(port / keep both / retire), a stated falsifier, `outcome_status: pending`, and
`--mode decision` review returning `verified-distinct`. **No code.**

## Wave 2 — UAR provenance (R5 foundation, blocks R5 delivery)

### `change-uhe-005-pack-provenance`
**Goal:** R5 · **UAR + pack** · No dependencies

Nothing records which pack version is loaded — the root cause of 359 commits of
undetected drift.

**Two halves, deliberately:** the pack **emits** a version manifest
(`skills-index` already exists — extend it, do not invent a parallel file), and
UAR **reads and exposes** it. UAR must not shell out to `git`: that is
impossible on mobile, which is the whole point.

**Acceptance:** a `GET` endpoint returns the loaded pack's version/commit and
skill count; a test asserts it changes when the manifest changes. **The 359-commit
drift would have been visible through this endpoint** — that is the bar.

## Wave 3 — UAR skill lifecycle (R1, R2, R3, R4)

### `change-uhe-006-origin-enabled-columns`
**Goal:** R2 · **UAR** · Depends on nothing, but 007 depends on it

**Why columns at all, given the premise says JSONB already persists both?**
Review caught the apparent contradiction. The answer is that **007 cannot be
built without them**: a database-level guard needs a column to constrain — you
cannot put a `CHECK` or trigger on a value buried in a JSONB blob without
extracting it first. Columns are the enabling change for the guard, not a fix
for a persistence gap that does not exist.

**The cheaper path is tested INSIDE this change, not as a reason to skip it.**
An earlier draft said 006 "may be dropped" if the guard works against
`definition->>'origin'` — which left 007–012's declared dependency on 006
pointing at nothing. Review caught that.

So task 1 of this change is the probe: **can a DB constraint be expressed
against `definition->>'origin'`?**
- **Yes** → this change delivers exactly that expression and adds **no
  columns**. It still completes, so every dependency on it stays valid.
- **No** → it adds the columns.

Either way `change-uhe-006` completes and downstream ordering is unchanged. The
outcome changes *what* it delivers, never *whether* it does.

**Acceptance:** a migration adds both with values backfilled from `definition`;
the provider round-trips them; existing rows survive with correct values.

### `change-uhe-007-db-level-delete-guard`
**Goal:** R2 · **UAR** · Depends on 006

**Acceptance — enforced, not demonstrated:** `DELETE FROM skills` on a
`Builtin` row **fails at the database**, proven by a test that calls the
storage provider **directly, bypassing `SkillService`**. A guard in one call
path is one refactor from being bypassed; the test must prove the bypass route
is closed.

### `change-uhe-008-builtin-db-registration`
**Goal:** R1 · **UAR** · Depends on 006

**Acceptance — all three providers, because R1 says "no matter what platform
… or whether embedded or not".** Verifying one provider and naming the rest as
unexercised was the first draft's weakness; review flagged it, and it would have
left the embedded path (the one this phase exists for) unproven.

UAR ships exactly three: `postgres.rs`, `surreal.rs`, `memory.rs` —
`memory` **is** the embedded/no-database case, so it is the most important, not
the one to skip.

After startup, every discovered builtin skill must be present with
`origin='builtin'` in **each** provider, verified by **row count == loader's
discovered count** (equality, not "some rows exist"). A provider that genuinely
cannot be exercised here is recorded **BLOCKED with the missing prerequisite
named**, and R1 is reported PARTIAL — never MET on one provider.

### `change-uhe-009-embedded-sdk`
**Goal:** R4 · **UAR** · Depends on 006

`src/lib.rs` exposes no skill-facing API; an embedder must reach into
`uar::runtime::skills::*` internals.

**Acceptance:** a `pub` facade (list / get / install / toggle / query) usable
from an external crate, proven by a test **in `tests/`** (integration, so it
consumes the public API exactly as an embedder would). Internals stay private.

### `change-uhe-010-rest-api-completeness`
**Goal:** R4 · **UAR** · Depends on 006

R4 says UAR must **expose skill installation and query REST APIs**. The
assessment found `/api/skills`, `/api/uar/skills`, and `/api/uar/skills/reload`
already mounted — and the first draft of this plan treated that as done without
verifying it covers *installation* and *query*. Review flagged the gap.

**Acceptance:** an enumerated table of every skill endpoint with method, path,
and a passing request/response test — covering **install** (add a new skill),
**query** (list + get + search), and **toggle**. Any verb R4 names that has no
endpoint is either added or recorded as a stated gap. **"Endpoints exist" is not
acceptance; a passing test per verb is.**

### `change-uhe-011-dynamic-skill-registration`
**Goal:** R4 · **UAR + pack** · Depends on 009

A skill this pack *generates* should be able to register itself in the UAR DB —
**optionally**.

**Acceptance:** an explicit opt-in (flag or env), **off by default**, with a
test proving that **without** the opt-in nothing is written. Default-on would
silently grow a user's database; the requirement says "optionally", and the
default has to encode that.

### `change-uhe-012-ui-surfaces-builtin`
**Goal:** R3 · **UAR** · Depends on 006

The admin UI already lists, creates, updates, toggles, deletes, and imports.

**Acceptance:** builtin skills are **visually distinguishable** and their delete
affordance is **absent or disabled** — a button that 409s is a worse experience
than no button. Verified in `universal-agent-runtime/frontend/e2e/admin-skills.spec.ts` —
confirmed present (1,861 bytes) at plan time, not assumed.

### `change-uhe-013-github-update-check`
**Goal:** R5 · **UAR** · Depends on 005

Provenance alone answers *"which pack am I on?"*. R5 also requires knowing
**that an update exists** and being able to **initiate it from GitHub** — which
the first draft of this plan omitted entirely. Review caught it.

**Acceptance:** an endpoint compares the loaded manifest version against the
GitHub repository's latest and reports `up-to-date` / `behind by N` / `unknown`
(network failure is `unknown`, **never** `up-to-date` — a check that reports
current when it could not reach the network is worse than no check). A second
endpoint initiates the update on desktop/server. **Test against a fixture
manifest, not live GitHub**, so the test does not depend on the network.

### `change-uhe-014-mobile-update-transport`
**Goal:** R5 · **UAR + pack** · Depends on 005, 013

A phone has no git and no submodules. **This was deferred in the first draft;
review correctly called that scope-dropping**, since "updated for mobile use" is
half of R5's plain text.

**Decision first, then implementation.** A decision record picks the transport —
signed versioned bundle over HTTPS, or the existing `sovereign-sync` P2P
substrate — with a stated falsifier, since the second reuses real infrastructure
but couples skill updates to sync availability.

**Acceptance:** the decision is recorded via `decision-log.sh` and passes
`--mode decision` review; **then** a mobile-reachable path fetches a versioned
bundle and the provenance endpoint reflects the new version. **If the chosen
transport cannot be exercised on this machine, the change is archived BLOCKED
naming the missing prerequisite, and R5 is reported PARTIAL — never MET on the
decision alone.**

## Wave 4 — execution proof (S1)

### `change-uhe-015-uar-wasm-execution`
**Goal:** S1 · **UAR** · Depends on 005

De-stub `wasm_runtime.rs:92-111`. The component from `change-msp-006` is built,
validated, and sitting where discovery looks — it has never executed.

**Acceptance:** the reference component **returns its own output**, not the
placeholder string, proven by a test asserting on the returned value. This is
the change that turns goal 1 from PARTIAL to MET, and nothing may be described
as end-to-end parity until it passes.

### `change-uhe-016-waypoint-staleness-report`
**Goal:** S5 · **In-repo** · No dependencies

S5 is a **seeded goal** and the first draft covered it only under "Deferred" —
review correctly called that leaving a goal unplanned. A goal whose right answer
is "do not patch here" still needs a change that produces something.

Two deterministic defects, both hit twice on 2026-07-31:
1. `kbd-reflect` never writes `.phase`, so it names a phase from two
   transitions ago.
2. `kbd-next-phase.sh:270` writes a **self-referential** `next`
   (`/kbd-next-phase <phase>`) while `exactNextCommand` in the same file
   correctly says `/kbd-assess`.

**The fix does not belong in this repo.** Both live in installed skills under
`~/.claude/skills/`; editing them here is the plugin-cache mistake — overwritten
by the next install, invisible to git.

**Acceptance — a report plus a guard, not a patch:**
- A committed report with both root causes, the exact lines, and the one-line
  fix each needs, ready to apply wherever those skills are authored.
- **A check script in this repo** that reads `current-waypoint.json` and **exits
  non-zero** when `.phase` disagrees with the active phase directory or when
  `next` is self-referential. That way the bug is *detected* here even though it
  is *fixed* elsewhere — and the next occurrence is caught, not discovered
  after the fact for a third time.

## Deferred, with reasons

*(The mobile transport was listed here in an earlier draft **and** planned as
`change-uhe-014` — a contradiction review caught. It is planned. Nothing is
deferred from this phase's eleven items.)*

## What this plan does not do

- **No code into `flint-realtime-fabric` or `know-me-system`.** Not authorised.
- **No claim of end-to-end parity before `change-uhe-015` passes** — that is
  the Wasm-execution change. (An earlier draft said "012", a stale number from
  before the renumbering; review caught it, and reporting parity one change
  early is exactly the mistake this line exists to prevent.)
- **No mobile update path.** Deferred above, with the reason stated.

## Carry-forward, not absorbed

Two `sovereign-sync` control-token tests fail pre-existing
(`one_projects_token_is_rejected_by_another_project`,
`two_projects_mint_distinct_identities_and_tokens`). Confirmed by `git stash` in
an earlier phase that they predate this work. Not this phase's unless selected.

Six submodule checkouts carry stashed `AGENTS.md`/`CLAUDE.md` "Phase-Gated
Testing" edits. **That policy belongs in this pack's own files**, where git can
see it — an edit in a consumer's checkout dies at the next update. Applying it
is not planned here; it is recorded so it is not lost.

## Review record

Round 1 verdict **BLOCK** (4 CRITICAL, 2 WARNING), judge `kbd-judge` via
`rest-gateway`, `cross_model_check: verified-distinct`, producer `claude-opus-5`.

| # | Severity | Finding | Response |
|---|---|---|---|
| 1 | CRITICAL | R5 omits GitHub-initiated updating **and** the mobile path | **Accepted — this was scope-dropping, not deferral.** "Updated for mobile use" is half of R5's plain text and I had pushed it out with a rationale. Added `change-uhe-013` (update check, network failure reports `unknown` not `up-to-date`) and `change-uhe-014` (mobile transport, decision-first, BLOCKED-if-unexercisable). |
| 2 | CRITICAL | R1 verifies one persistence provider, not "every platform" | **Accepted.** UAR ships three (`postgres`, `surreal`, `memory`) and **`memory` IS the embedded case** — the one this phase exists for. Now all three, with row-count equality per provider. |
| 3 | CRITICAL | R4's REST coverage was never planned | **Accepted.** I read "endpoints exist" in the assessment and treated it as done. Added `change-uhe-010`: a per-verb passing test for install/query/toggle. "Endpoints exist" is not acceptance. |
| 4 | CRITICAL | The R2 correction cites code the packet cannot resolve | **Accepted as a limit.** Labelled cross-repo with reproduction commands; each change re-runs its own before acting. |
| 5 | WARNING | `change-uhe-006` contradicts the premise it just corrected | **Accepted — a real inconsistency.** Columns are the *enabling* change for 007's DB-level guard (you cannot constrain a value inside JSONB), not a fix for a persistence gap that does not exist. And if 007 works against `definition->>'origin'` directly, 006 is **dropped** — tested first. |
| 6 | WARNING | UI acceptance cites an e2e file absent from the packet | **Accepted.** Verified present at plan time (1,861 bytes) and the path is now given from the UAR root. |

Three of four CRITICALs were the same failure in different places: **I let the
assessment's "already partially met" readings stand in for planned, tested
work.** Partially-met is a starting position, not an acceptance criterion.

### Round 2 — `BLOCK` (3 CRITICAL, 2 WARNING) — stopping at the cap

Every round-2 CRITICAL was a **contradiction I introduced while fixing round 1**:

| Finding | Response |
|---|---|
| `change-uhe-006` "may be dropped" while 007–012 declare hard dependencies on it | **Accepted.** The cheap path is now a **probe inside 006** (task 1: can a constraint target `definition->>'origin'`?). Either branch **completes** the change, so downstream ordering never dangles. The outcome changes what it delivers, never whether it does. |
| S5 has no planned change — the plan declines to patch it | **Accepted.** A goal whose right answer is "do not patch here" still needs a change that *produces something*. `change-uhe-016` delivers a report **plus a check script in this repo** that exits non-zero when `.phase` disagrees or `next` is self-referential — detected here, fixed elsewhere, and caught on the next occurrence rather than found a third time by accident. |
| Mobile transport both planned (014) and listed as deferred | **Accepted.** A leftover from the round-1 fix. It is planned; the deferral text is removed. **Nothing is deferred from the eleven items.** |
| WARNING — wrong change number for the parity gate | **Accepted.** "012" was stale from renumbering; the gate is `change-uhe-015`. Reporting parity one change early is precisely what that line prevents. |
| WARNING — CI change had no concrete target | **Accepted.** Named: `.github/workflows/validate.yml`, job `fabric-invariants`, verified present. |

Stopping at the 2-round cap. Nothing remains unresolved. The pattern across both
rounds is worth stating: **round 1 caught scope I had dropped; round 2 caught
inconsistencies created by the repair.** A plan edited under review pressure
needs re-reading as a whole, not just at the edited lines — which is what a
second reviewing model is for.
