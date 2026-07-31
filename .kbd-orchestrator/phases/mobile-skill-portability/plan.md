# Plan — mobile-skill-portability

**Phase:** `mobile-skill-portability` · **Planned:** 2026-07-31
**Backend:** OpenSpec · **Changes:** 9

## The blocking question dissolved

Assess ended on: *"does this phase have authorisation to write into
`universal-agent-runtime`?"* Further investigation answered it without needing a
scope decision.

**UAR consumes this repository as a submodule.**
`universal-agent-runtime/.gitmodules:4-6`:

```
[submodule "crates/prometheus-skill-system"]
	path = crates/prometheus-skill-system
	url = git@github.com:Prometheus-AGS/prometheus-skill-system.git
```

And `wasm_runtime.rs:115-121` discovers components **inside that submodule**:

```rust
/// falls back to `crates/prometheus-skill-system/skills`
/// (any `skill.wasm` discovered alongside a `SKILL.md`).
```

Verified: `git remote get-url origin` on this repo is that same URL, and UAR's
pin `8ddac9a` is an ancestor of our HEAD. **Producing skill components is
in-repo work.** UAR picks them up by bumping its submodule pointer — no code is
written into UAR to deliver them.

The host stub remains real (assess gap 1) and still needs UAR-side work, but it
is now an **independent** concern rather than a precondition. This plan is
ordered so every change lands in this repo; the stub only gates *end-to-end
execution proof*, which is isolated into a single final change.

> **These are cross-repo facts and this plan's ordering depends on them.**
> Adversarial review flagged that basing the ordering on files a single-repo
> reviewer cannot open is unsafe. **`change-msp-005` therefore begins by
> re-running these three commands and aborts if any disagrees:**
>
> ```bash
> grep -A2 'submodule "crates/prometheus-skill-system"' \
>   /Users/gqadonis/Projects/prometheus/universal-agent-runtime/.gitmodules
> sed -n '115,121p' \
>   /Users/gqadonis/Projects/prometheus/universal-agent-runtime/src/uar/runtime/skills/wasm_runtime.rs
> git remote get-url origin   # must equal the submodule URL above
> ```
>
> If discovery no longer reads `crates/prometheus-skill-system/skills`, the
> in-repo premise collapses and **005/006 must be re-planned**, not continued.

## A third incompatible target (new finding)

`skills/rust/librefang-wasm-skill/` already ships wasm templates — but they
target **core-wasm with an `extern "C"` `execute(ptr,len) -> i64` ABI and a
`host_call` import**, with **zero `.wit` files**. UAR loads
`wasmtime::component::Component`. A guest from these templates **cannot load in
UAR**. Evidence: [`evidence/component-targets.md`](evidence/component-targets.md).

So the divergence is not four WIT packages across two repos — it is **three
mutually incompatible guest targets**, one of them already in this repository.
This strengthens the unify-first ordering the previous phase recorded.

## Changes

Ordered so the cheapest verifiable work lands first and the least verifiable
last — the reverse of the seeded order, per assess.

### `change-msp-001-classify-script-skills`
**Goal:** scope · **In-repo** · No dependencies

Classify all 60 script-bearing skills (enumerated in
`evidence/skill-inventory.md`) into: **E0** (script is build/dev tooling a phone
never invokes), **E1** (needs Wasm), **E2** (needs a native binary), **R**
(remote execution covers it).

**Acceptance:** a committed `mobile-classification.json` with one verdict +
one-line rationale per skill, and a script that **fails** if any of the 60 is
unclassified or if the file drifts from the skill inventory. Counts must be
derived by the script, not typed.

*Why first: this sizes every other change. Porting is expensive; if most of the
60 are dev tooling, the phase is small.*

### `change-msp-002-zed-tier1-routing`
**Goal:** 4 · **In-repo** · No dependencies

Route `zed` to `_render_tier1_file_pair` in
`skills/learn/ui-surface/scripts/render.sh:174`, or record why not.

**Acceptance — exactly one of two outcomes, never a blend:**

- **Routed and verified.** The file-pair round trip **runs** under `zed` with an
  independent responder, confirmed under `bash -x` to reach
  `_render_tier1_file_pair`. Assertion-only is not acceptance.
- **Not routed.** A committed diagnostic recording *why* (e.g. zed does not poll
  the file pair), and `references/harness-delivery.md` continues to state zed is
  **Tier 0**. **No Tier 1 claim may be made in this outcome.**

Ambiguity between these two is the failure mode: "route it, or record why not"
without saying what each outcome requires is how an unverified claim slips in.

### `change-msp-003-verify-opencode-kimi`
**Goal:** 4 · **In-repo** · No dependencies

`opencode` and `kimi` share codex's verified path but have never been run.

**Acceptance:** both round trips executed, evidence appended to
`references/harness-delivery.md`. If either fails, record the failure — do not
quietly narrow the claim.

### `change-msp-004-fabric-integration-skill`
**Goal:** 2 · **In-repo** · No dependencies

Build `skills/devops/fabric-integration` converting the four version invariants
from prose to **enforced**.

**Acceptance — two modes, because one invariant is already violated.** A gate
that must fail on its first run cannot be merged into CI; that was a paradox in
the first draft of this plan, caught by adversarial review.

An audit-only script would not satisfy goal 2, which says **enforced**. So the
resolution is per-invariant, not per-mode:

- **Three invariants are ENFORCED immediately** — Loro minor, wasmtime major,
  iroh floor. All three hold today, so a gate on them lands green in CI and
  **exits non-zero** the moment one drifts. Proven by mutation: temporarily
  lower the iroh floor and watch CI fail.
- **The WIT invariant is QUARANTINED, not skipped.** It is already violated
  (`knowme:plugin` at 0.1.0 and 1.0.0), so gating on it would block every PR for
  a pre-existing condition. It is checked, **reported as VIOLATED**, and listed
  in a `known_violations` allowlist that the script itself prints on every run.

**The allowlist is the enforcement mechanism, not an escape hatch:** the script
**exits non-zero if a violation is NOT in the allowlist**, and also **exits
non-zero if an allowlisted violation has been FIXED** — forcing the entry to be
removed rather than lingering. A quarantine that never shrinks is a suppressed
check.

**Acceptance:** CI is green on merge; mutating any of the three enforced
invariants turns it red; the WIT violation is visible in output and in the
allowlist; and removing the WIT ambiguity (later, elsewhere) turns the run red
until the allowlist entry is deleted.

### `change-msp-005-prometheus-component-wit`
**Goal:** 1 · **In-repo** · Depends on 001

Author the `prometheus:component@0.1.0` WIT family
(`types`, `capabilities`, `skill`, `plugin`) per
`docs/decisions/wit-world-unification.md`. **Implement the decision; do not
reopen it.**

**Acceptance:** `wasm-tools component wit` parses every file; the `skill` world
is a superset of UAR's `run` contract; a mapping table records how each of
`uar:skill`, `uar:plugin`, `knowme:plugin`, and the **librefang core-wasm ABI**
relates to it — including any that cannot be expressed, stated as such.

### `change-msp-006-reference-component`
**Goal:** 1 · **In-repo** · Depends on 005

Build one real skill component against `prometheus:component/skill` — the
smallest skill from 001's E1 set.

**Acceptance:** `skill.wasm` builds, `wasm-tools validate --features component-model`
passes, and it sits beside its `SKILL.md` where UAR's discovery expects it.
**Executing it is change 008's job, not this one's** — this change proves the
artifact is well-formed, and must not claim more.

### `change-msp-007-ffi-pattern-decision`
**Goal:** 3 · **In-repo** · No dependencies

Decide **cbindgen vs uniffi** and record it via `decision-log.sh` (the tool the
previous phase shipped), with `--mode decision` adversarial review.

Inputs: liter-llm's in-tree cbindgen surface (767-fn C ABI, 46 JNI entry points,
150 Java files, **plus a Dart/Flutter bridge with a native loader** — directly
relevant to KnowMe's Flutter target) versus `frf-ffi`'s uniffi 0.31.2, which
generates Kotlin **and** Swift from one definition.

**Acceptance:** a decision record with alternatives, a stated falsifier, and
`outcome_status: pending`; review returns `verified-distinct`. **No FFI code is
written in this change** — deciding and building are separate.

### `change-msp-009-mobile-ffi-bindings`
**Goal:** 3 · **In-repo** · Depends on 005 **and** 007

Build the mobile FFI bindings the seeded goal asks for, using the pattern 007
chose.

> **Why this change exists.** The first draft of this plan replaced goal 3's
> *"Mobile FFI bindings"* with a decision-only change and deferred the build to
> a later phase. Adversarial review flagged that as **silently dropping a seeded
> goal** — a scope narrowing is the user's call, not the planner's. Restored.

**Why it depends on 005.** The carried-forward ordering constraint is that WIT
authoring blocks porting. The FFI boundary carries the **same skill-invocation
surface** the WIT `skill` world defines; building it against a shape that 005
then changes means building it twice — the exact duplication the
unify-before-porting rule exists to prevent.

**Acceptance:** a crate in this repo exposing the pack's skill-invocation
surface **as defined by `prometheus:component/skill` from 005**, across the FFI
boundary, with `crate-type` per 007's decision, that
**builds for at least one real mobile target** (`aarch64-apple-ios` or
`aarch64-linux-android`) — not just host. A round-trip test calls in across the
boundary and asserts on the returned value.

**If the chosen toolchain is unavailable** (no Android NDK / iOS SDK on this
machine), the change is recorded **BLOCKED with the missing prerequisite named**,
and goal 3 is reported **PARTIAL — pattern decided, bindings unbuilt**. It may
not be reported as MET on the strength of the decision alone.

### `change-msp-008-uar-execution-proof`
**Goal:** 1 · **CROSS-REPO — the only one** · Depends on 006

Prove the reference component actually executes.

**This is the one change that needs UAR-side work** (de-stubbing
`wasm_runtime.rs:92-111`). It is deliberately last and isolated so the other
eight deliver regardless.

> **Authorisation gate — the first task of this change, before any edit.**
> The carried-forward constraint is that cross-repo writes need the user's
> explicit agreement. This change therefore **starts by asking**, and:
>
> - **Unauthorised or unanswered → the change is BLOCKED and archived as such.**
>   No file outside this repo is touched. This is the default; silence is not
>   consent.
> - **Authorised → proceed**, and record the authorisation in the change.
>
> The plan may not assume the answer, and no later change may depend on 008.

**Acceptance, and the honesty constraint:** if cross-repo work is authorised,
the component runs under UAR and returns its own output rather than the
placeholder string. **If it is not authorised, this change is recorded as
BLOCKED with the reason** — and changes 005/006 are explicitly reported as
*"component well-formed, execution unproven"*. Neither may be reported as
end-to-end parity. Tier-1-by-assertion was the exact failure the previous phase
was built to stop.

## Ordering rationale

1–4 are in-repo with no dependencies and can proceed in any order; 001 first
because it sizes the rest. 005 needs 001's E1 set to pick a target. 006 needs
005's world. 007 is independent and can run any time; 009 needs **both** 005's
world (the surface it exposes) and 007's decision (how it exposes it).
008 is last because it is the only change whose completion is not fully within
this repo's control.

## What this plan does not do

- **No code into `flint-realtime-fabric`, `know-me-system`, or (except 008)
  `universal-agent-runtime`.** The previous phase touched zero files in all
  three; that holds here.
- **No claim of "100% parity on mobile."** 259 of 319 skills are manifest-only
  and portable today; the 60 script-bearing ones are what this phase is about,
  and 001 will likely show many need no port at all.

## Carry-forward, not absorbed

Two `sovereign-sync` control-token tests fail pre-existing
(`one_projects_token_is_rejected_by_another_project`,
`two_projects_mint_distinct_identities_and_tokens`). Not this phase's work
unless explicitly selected.

## Review record

Round 1 verdict **BLOCK** (3 CRITICAL, 1 WARNING), judge `kbd-judge` via
`rest-gateway:http://localhost:8181/v1`, `cross_model_check: verified-distinct`,
producer `claude-opus-5`.

| # | Severity | Finding | Response |
|---|---|---|---|
| 1 | CRITICAL | The plan drops the seeded "Mobile FFI bindings" goal, replacing it with a decision-only change | **Accepted — this was a real scope narrowing made unilaterally.** `change-msp-009` restored to build the bindings. Narrowing scope is the user's call, not the planner's. |
| 2 | CRITICAL | `change-msp-004` is impossible as written: it wires a check into CI that must fail on first run | **Accepted — a genuine paradox.** Split into `--audit` (exits 0, reports the violation, lands in CI) and `--enforce` (exits non-zero, mutation-proven, NOT wired to CI). Promotion deferred with a stated gate. |
| 3 | CRITICAL | The ordering rests on cross-repo facts a single-repo reviewer cannot verify | **Accepted.** `change-msp-005` now **begins** by re-running three named commands and aborting if any disagrees; if UAR's discovery path has changed, 005/006 are re-planned rather than continued. |
| 4 | WARNING | `change-msp-002` has contradictory scope ("route it, or record why not") and acceptance | **Accepted.** Rewritten as exactly two mutually exclusive outcomes, with "no Tier 1 claim" explicit in the not-routed branch. |

No finding was rejected. Finding 1 is the one worth flagging to the user: I had
quietly deferred a goal the phase was seeded with, which is precisely the failure
mode the producer≠judge separation exists to catch.

### Round 2 — `BLOCK` (1 CRITICAL, 2 WARNING)

| # | Severity | Finding | Response |
|---|---|---|---|
| 1 | CRITICAL | `--audit`-only does not meet goal 2's "enforced" | **Accepted — my round-1 fix over-corrected.** Resolving the paradox by shipping a non-failing check traded one defect for another. Now per-invariant: the three that hold are **enforced in CI immediately**; only the already-violated WIT one is quarantined, in a self-shrinking allowlist that fails both on un-allowlisted violations *and* on allowlisted entries that have been fixed. |
| 2 | WARNING | 008 writes cross-repo with no explicit authorisation step | **Accepted.** 008's first task is now the authorisation ask, with **BLOCKED as the default** on silence. No later change may depend on 008. |
| 3 | WARNING | 009 lacks a dependency on 005 despite the WIT-blocks-porting rule | **Accepted.** 009 now depends on **005 and 007** — the FFI boundary carries the same surface the WIT `skill` world defines, so building before 005 means building twice. |

**Stopping at the 2-round cap.** Both rounds found real defects and both were
fixed rather than argued with. Round 1's finding 1 (a silently dropped goal) and
round 2's finding 1 (an over-correction that defeated the goal it was meant to
serve) are the two worth remembering: **a planner correcting its own plan can
introduce a new defect while fixing the old one**, which is exactly why the judge
is a different model.

Per the skill's cap, this section is the required unresolved-findings
disclosure — though in this case no finding remains unresolved.
