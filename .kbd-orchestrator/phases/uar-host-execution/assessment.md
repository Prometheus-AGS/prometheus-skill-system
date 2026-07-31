# Assessment — uar-host-execution

**Phase:** `uar-host-execution` · **Assessed:** 2026-07-31
**Preflight:** `status: ok`, 2 distinct models
**Cross-repo authorisation:** **GRANTED** for `universal-agent-runtime`
([evidence](evidence/authorisation.md)). Not granted for
`flint-realtime-fabric` or `know-me-system`; both stay untouched.

## Scope correction — the seeded goals were nearly dropped

Adversarial review returned **BLOCK with 7 CRITICALs**, six of which were the
same mistake: I assessed the user's five new requirements and **silently
dropped the six goals this phase was seeded with**. New requirements *add to*
a phase's scope; they do not replace it, and narrowing is the user's call.

Restored below. **This phase has eleven items, not five.**

## Seeded goals — status

| # | Seeded goal | Status |
|---|---|---|
| S1 | De-stub UAR's Wasm runtime | **Now unblocked** — authorisation granted. `wasm_runtime.rs:92-111` still returns a placeholder without instantiating. |
| S2 | Decide the librefang ABI question | **Untouched.** `skills/rust/librefang-wasm-skill/` still ships core-wasm `extern "C"` templates with zero `.wit`; they cannot load in a component runtime. A real fork: port, keep both, or retire. |
| S3 | Close FFI falsifier 3 (marginal cost per added function) | **Untouched.** Needs a second function added to `substrate/skill-ffi` and the hand-written glue counted; reverse the pattern decision if >~20 lines. |
| S4 | Give CI the sibling repos | **Untouched.** The `fabric-invariants` job uses a bare `actions/checkout@v4`, so three of four invariants report SKIP. Verified: no sibling checkout step exists. |
| S5 | Fix waypoint `.phase` staleness in `kbd-reflect` | **Untouched, and it recurred again this session** — `next` was self-referential at the last transition too. Fix lives in an **installed** skill under `~/.claude/skills/`; patching from here is the plugin-cache mistake. |
| S6 | Exercise `cursor`, or state why it cannot reach Tier 1 | **Assessed by running it.** Resolves to `tier0_text`. Cause is identical to `zed`'s: `detect-surface-tier.sh:76-79` hardcodes `TIER="tier0_text"`, and `render.sh` omits `cursor` from both Tier 1 dispatch lists. No mechanism reason — the file-pair handshake is two files on disk. |

## Headline (new requirements)

**Far more exists than the goals assume.** UAR already has a skills table, a
builtin loader, `SkillOrigin::Builtin` delete-protection, an `enabled` flag, a
REST surface, and an admin UI with a toggle. Four of the five new requirements
are **partially met by code that already ships**.

The real gap is narrower and sharper than "build all this": **nothing knows
which version of the pack it loaded**, so drift is invisible. That is what
requirement 5 is really about, and it is now measured rather than suspected.

## Already done this session — submodule currency

UAR's pin on the skill pack was **359 commits and two months stale**
(`8ddac9a` 2026-06-01 → `e04bfa0` 2026-07-31), seeing **161 skills where the
pack has 220**. Nothing detected it.

Fast-forwarded after ancestry-checking every move; **9/9 builtin-loader tests
pass** and `cargo check --lib` is clean. `liter-llm` now pins to the identical
commit in both repos. Details, including six checkouts whose uncommitted
`AGENTS.md`/`CLAUDE.md` edits were **stashed rather than destroyed**:
[evidence/submodule-currency.md](evidence/submodule-currency.md).

## New requirements — requirement-by-requirement

> **Verification status.** Every claim in this section is **cross-repo** and
> cannot be checked from this repository's review packet — review flagged that
> correctly. Excerpts, SHA-256 hashes, and reproduction commands are committed
> to [evidence/uar-code-evidence.md](evidence/uar-code-evidence.md); plan must
> re-run them rather than accept these claims.

### R1 — skills auto-install and are recognised on every platform, embedded or not

**Status: partially met.**

- `builtin_loader.rs` walks `crates/prometheus-skill-system/skills` (override:
  `UAR_BUILTIN_SKILLS_DIR`), parses `SKILL.md` frontmatter, and registers each
  with `origin: SkillOrigin::Builtin`. **9 tests pass.**
- `wasm_runtime.rs` separately discovers `skill.wasm` beside `SKILL.md`.

**Gaps:** the loader populates the in-memory registry; whether every skill
reaches the **database** on every platform is unverified. `all_builtin_dirs()`
suggests multiple roots — the embedded-vs-server difference needs a run, not a
read.

### R2 — pack skills can never be deleted, only disabled

**Status: mostly met — and the strongest existing guarantee.**

- `service.rs:374` `delete_skill_permanent` **bails with
  `system_skill_immutable`** for `SkillOrigin::Builtin`, mapped to HTTP 409.
- `Skill.enabled: bool` exists in the domain model; the admin UI exposes
  `toggleSkillApi`.

**Gap, and it is load-bearing:** the **database schema has neither column.**
`migrations/20251225000000_init_uar.sql` defines
`skill_id, name, description, definition, embedding, created_at, updated_at` —
no `origin`, no `enabled`. So the protection lives in the in-memory registry
only. A skill absent from the registry (or a direct DB delete) is unprotected,
and `enabled` cannot survive a restart unless it is inside `definition` JSONB.
**Verify where `enabled` actually persists before trusting the toggle.**

### R3 — skills shown and administered in the UI

**Status: largely met.**

`frontend/src/services/skills-api.ts` exports `fetchSkillsList`,
`createSkillApi`, `updateSkillApi`, `toggleSkillApi`, `deleteSkillApi`,
`importSkillFromDisk`, plus a store, a hook, and `e2e/admin-skills.spec.ts`.

**Gaps:** unverified whether the UI *surfaces* builtin-ness (a delete button
that 409s is worse than one that is absent), and whether 220 skills render
usably. Needs a run.

### R4 — REST + embedded SDK, including dynamic skill creation

**Status: REST yes, SDK no.**

- REST: `/api/skills`, `/api/uar/skills`, `/api/uar/skills/reload`, agent–skill
  bindings — mounted in `server.rs`.
- **SDK: absent.** `src/lib.rs` exports `config, config_manager, llm, mcp,
  normalized, sandbox, server, session, uar` — no skill-facing convenience API.
  An embedder must reach through `uar::runtime::skills::*` internals.

**Not yet traced:** the path by which a skill *this pack generates* opts into
the UAR database. `createSkillApi` exists; whether a creator can call it, and
whether "optionally added" is expressible, is unknown.

### R5 — know when skills need updating; initiate from GitHub; update on mobile

**Status: NOT met. This is the phase's real work.**

- **No provenance.** `builtin_loader.rs` records no pack commit, tag, or
  version. Nothing can answer "which pack am I running?" — which is exactly why
  359 commits passed unnoticed.
- **No update check.** No code queries GitHub for a newer pack.
- **`/api/uar/skills/reload` exists but only re-reads the working tree** — it
  cannot fetch.
- **Mobile is hardest and least specified.** A phone has no git and no
  submodule. It needs a signed, versioned bundle over HTTP — a different
  mechanism from the desktop's `git submodule update`, not a port of it.

**Design constraint worth surfacing now:** desktop/server can pull via git;
mobile cannot. Trying to make one mechanism serve both is how this ends up
serving neither. Expect **two transports over one manifest**.

## Open questions for plan

1. **Where does `enabled` persist?** If only in registry memory or `definition`
   JSONB, R2's toggle is weaker than it appears. Determines whether a migration
   is needed.
2. **Does every builtin skill reach the DB, on every platform?** R1 hinges on
   it and it is unverified.
3. **What identifies a pack version?** Commit SHA, git tag, or a
   `pack.json` the pack itself emits. Cheapest correct option probably: the pack
   generates a version manifest, so UAR need not shell out to git (impossible on
   mobile).
4. **Mobile update transport?** Signed tarball over HTTPS, or the P2P sync
   substrate this stack already has (`sovereign-sync`, iroh)? The second reuses
   real infrastructure but couples skill updates to sync availability.
5. **Should dynamic skill creation default to on or off?** The requirement says
   "optionally". Default-on surprises users with a growing database; default-off
   means creators must know to opt in.

## Suggested shape

Eleven items is a large phase. The seeded goals split cleanly by cost:

**Cheap and in-repo (do first, they size nothing else):**
- **S6 `cursor`** — two one-line changes plus an executed round trip; the
  diagnosis is already done above.
- **S4 CI sibling repos** — a checkout step; turns 1-of-4 invariant coverage
  into 4-of-4.
- **S3 FFI falsifier 3** — add one function, count the glue.
- **S5 waypoint staleness** — a report, not a patch, unless the installed skill
  is regenerated from a source we own.
- **S2 librefang ABI** — a decision record, no code.

**Then the UAR work (cross-repo, now authorised):**

1. **Provenance** — record the pack version at load and expose it. Without this,
   nothing else in R5 is measurable, and it is the smallest change here.
2. **Persist `origin` + `enabled`** — a migration plus provider changes; makes
   R2's guarantee real rather than in-memory.
3. **De-stub the Wasm runtime** (the original goal 1) — now authorised.
4. **Embedded SDK** — a `uar::skills` facade over the existing service.
5. **Update check + GitHub-initiated update** — desktop path first, since it can
   reuse git.
6. **Mobile bundle transport** — last, because it is the only one with no
   existing substrate.

## Review record

Round 1 verdict **BLOCK** (7 CRITICAL, 2 WARNING), judge `kbd-judge` via
`rest-gateway`, `cross_model_check: verified-distinct`, producer `claude-opus-5`.

| # | Finding | Response |
|---|---|---|
| 1–6 | Assessed a different requirement set; missed the librefang ABI, FFI falsifier 3, CI sibling repos, `kbd-reflect` staleness, and `cursor` goals | **Accepted — one mistake, six findings.** I treated the user's five new requirements as *replacing* the six seeded goals. New requirements add to scope; narrowing is the user's call. All six restored, and `cursor` was assessed by **running** it (Tier 0, same two causes as `zed`). |
| 7 | UAR claims unsupported by the packet | **Accepted.** Excerpts + SHA-256 + reproduction commands committed to `evidence/uar-code-evidence.md`, and the section is labelled cross-repo with an instruction that plan must re-run them. |

The scope-drop is the one worth remembering: **a new instruction that arrives
mid-phase is additive unless the user says otherwise.** I had it backwards, and
a judge that was not the author caught it.

### Round 2 — `BLOCK` (3 CRITICAL, 2 WARNING)

| Finding | Response |
|---|---|
| The phase was expanded to eleven items by requirements not present in the goals packet | **Accepted — and this was the real cause.** The user's R1–R5 had never been written into `goals.md`, so the packet could not see them. Fixed at the source: `goals.md` now records R1–R5 verbatim, states that they **add to** rather than replace S1–S6, and totals the scope at eleven. |
| Cross-repo authorisation treated as resolved with evidence absent from the packet | **Accepted.** The grant is now quoted verbatim in `goals.md` alongside its scope limits, so it is in the packet rather than only in `evidence/`. |
| Load-bearing UAR claims from files missing from the packet | **Accepted as a limit** — see round 3. |

### Round 3 — `BLOCK` (1 CRITICAL, 2 WARNING) — stopping at the cap

Two of round 2's three CRITICALs cleared once `goals.md` carried the real scope.
The survivor is the **same structural limit this stack has hit before**: a
review packet built from *this* repository cannot resolve paths in
`universal-agent-runtime`, so every R1–R5 status is unverifiable from it —
**however much evidence is attached.** The packet builder does not descend into
sibling repositories.

Adding more hashes would not change what the reviewer can check. The honest
resolution is the labelling already applied:

> **R1–R5 are cross-repo findings carrying reproduction commands, not
> packet-verifiable claims. Plan must re-run them.** Every command is in
> [evidence/uar-code-evidence.md](evidence/uar-code-evidence.md), with the UAR
> commit SHA and per-file SHA-256 so a stale claim is detectable.

Per the skill's 2-round cap (exceeded by one round here because round 2 exposed
a fixable root cause rather than a limit), this section is the required
unresolved-findings disclosure.

**What was actually wrong, and worth remembering:** I let a mid-phase
instruction *replace* the phase's scope instead of adding to it, and I never
wrote the new requirements into `goals.md` — so the drop was invisible to every
tool that reads goals. The fix was not a better argument; it was updating the
artifact the packet is built from.
