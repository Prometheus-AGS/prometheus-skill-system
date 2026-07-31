# Goals: uar-host-execution

Seeded from: `mobile-skill-portability/reflection.md`
Created: 2026-07-31T12:47:03Z

## Seeded Goals

Small, well-specified, and gated on one decision:

- **De-stub UAR's Wasm runtime** — needs the cross-repo authorisation the last
  phase did not get. Everything else is ready: the world is authored, a
  validated component sits where discovery looks, and the FFI boundary is built.
- **Decide the librefang ABI question** — port to `prometheus:component`, keep
  both targets, or retire the templates.
- **Close falsifier 3** by adding a second FFI function and measuring the
  marginal cost.
- **Give CI the sibling repos** so `fabric-integration` verifies four invariants
  instead of one.
- **Fix waypoint `.phase` staleness in `kbd-reflect`.** It hit twice on
  2026-07-31 and every skill instructs agents to trust that file for their
  position. The fix is one line beside the existing `.status` write — but it
  lives in an installed skill, not this repo.
- **Exercise `cursor`, or state a mechanism reason it cannot reach Tier 1.**
  Raised by the sycophancy gate. It is detected and left at Tier 0 with no
  stated cause — the same shape as `zed` before the last phase found two.

## Added requirements (user, 2026-07-31)

Given **after** this phase was seeded. They **add to** the six goals above —
they do not replace them. Adversarial review caught me assessing these five and
silently dropping the seeded six; recording them here is what makes the packet
show the real scope.

- **R1 — Skills install and are recognised in the UAR skill database on every
  platform**, embedded or not.
- **R2 — Pack skills can never be deleted in UAR** — only turned off or
  excluded.
- **R3 — The UAR user interface shows these skills and can administer them.**
- **R4 — UAR exposes skill installation and query REST APIs and an SDK (for
  embedded)**, including adding more skills, and handling the case where this
  skill set *creates* skills that should be added to the UAR — dynamic skill
  creation should be able to result, **optionally**, in DB registration.
- **R5 — Know when skills need updating**, be able to initiate that from the
  GitHub repository, and have a way for them to be updated **for mobile use**.

Plus a standing instruction: **keep submodules current.** Acted on during
assess — UAR's pin on this pack was 359 commits stale; see
[evidence/submodule-currency.md](evidence/submodule-currency.md).

**Total scope: eleven items** (six seeded S1–S6, five added R1–R5).

---

## The gate this phase turns on

**Goal 1 cannot be delivered from this repository.** De-stubbing
`universal-agent-runtime/src/uar/runtime/skills/wasm_runtime.rs:92-111` means
writing into a repo the previous phase deliberately did not touch — verified:
**zero files** modified in `flint-realtime-fabric`, `universal-agent-runtime`,
or `know-me-system` across the whole of `mobile-skill-portability`.

`change-msp-008` asked for that authorisation and was archived **BLOCKED**,
because silence is not consent. **Assess must resolve this before planning**,
since it changes the phase's shape rather than one change's scope:

| Answer | Phase shape |
|---|---|
| **Authorised** | Goal 1 is deliverable; the other five are in-repo and proceed alongside. |
| **Not authorised** | Goal 1 stays PARTIAL indefinitely. The phase narrows to the in-repo goals — and that narrowing is the user's call to make explicit, not something to infer. |

> **RESOLVED 2026-07-31: AUTHORISED.** The user, verbatim: *"yes, you are
> authorized to write to universal-agent-runtime because I own it"*.
> Recorded in [evidence/authorisation.md](evidence/authorisation.md).
>
> Scope is exactly `universal-agent-runtime`. **Not** granted for
> `flint-realtime-fabric` or `know-me-system` — both stay untouched. R1–R5 land
> in UAR under this grant.

> Goal 5 (`kbd-reflect` staleness) has the **same shape**: the fix lives under
> `~/.claude/skills/`, outside this repo. Editing an installed skill from here
> is the same class of mistake as editing a plugin cache — the next install
> overwrites it and git never sees it. Either the fix goes upstream to whatever
> produces that skill, or the goal is a documented report, not a patch.

## What is already done — do not redo it

Verified at the close of `mobile-skill-portability`:

- `wit/prometheus-component/` — `types`, `capabilities`, `skill`, `plugin` at
  `@0.1.0`, parsing as one package. `MAPPING.md` records what does **not** map.
- `skills/react/prometheus-entity-skills/entity-graph-optimize/skill.wasm` —
  a real component, validated, sitting where UAR discovery looks. **Never
  executed.**
- `substrate/skill-ffi` — builds for `aarch64-apple-ios` **and**
  `aarch64-linux-android`, 7 round-trip tests.
- `skills/devops/fabric-integration` — three invariants enforced, the fourth
  quarantined in a self-shrinking allowlist.
- All four file-pair harnesses verified by **executed** round trips.

## Constraints carried forward

### 1. The premise check is mechanical — run it, don't assume it

`skills/devops/fabric-integration/scripts/check-uar-discovery.sh` verifies that
UAR still declares this repo as a submodule and still discovers `skill.wasm` at
`crates/prometheus-skill-system/skills`. **Exit 2 means the in-repo premise has
collapsed** and the component work must be re-planned. Exit 3 (UAR absent) is
*unverifiable* — deliberately **not** a pass.

### 2. The librefang question is a real fork, not a formality

`skills/rust/librefang-wasm-skill/` ships templates producing **core wasm with
an `extern "C"` `execute(ptr,len) -> i64` ABI and a `host_call` import**, with
zero `.wit` files. A component and a core module are different binary formats —
**no adapter bridges them.** Those guests cannot load in the runtime this phase
de-stubs. Porting, keeping both, or retiring are all defensible; picking one
silently is not.

### 3. "Well-formed" is not "working"

The component validates and has never run. Until a host instantiates and invokes
it, nothing may be reported as end-to-end parity. That distinction is what kept
goal 1 honest last phase, and it is the standing bar here.

### 4. CI verifies 1 of 4 invariants

`fabric-integration` reports `SKIP` for the three cross-repo invariants because
the runner has no sibling repos. SKIP is honest; it is **not** coverage. Fixing
it means giving CI the repos or accepting the limit explicitly.

### 5. Pre-existing, not this phase's to absorb

Two `sovereign-sync` control-token tests fail
(`one_projects_token_is_rejected_by_another_project`,
`two_projects_mint_distinct_identities_and_tokens`). Confirmed via `git stash`
that they predate this work. Fix deliberately or leave deliberately — do not let
them quietly become this phase's failure.

### 6. Two lessons that cost time last phase

- **Ask what the consumer already uses before comparing options.** Twice I
  evaluated the alternatives in front of me instead of establishing the
  incumbent — and the incumbent was in neither column.
- **Run it, don't read it.** `zed`'s Tier 0 fallback had *two* independent
  causes; fixing the one visible in the code left it still broken. Goal 6
  (`cursor`) is the same shape and should be run, not reasoned about.

---

## Instructions

Review and refine the goals above before running `/kbd-assess`.
Add, remove, or clarify as needed. When ready:

```
/kbd-assess uar-host-execution
```
