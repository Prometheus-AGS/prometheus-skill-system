# Reflection — mobile-skill-portability

**Phase:** `mobile-skill-portability`
**Closed:** 2026-07-31 · **Implementation:** 9/9 changes archived (one BLOCKED by design)

## Goal achievement

| # | Goal | Verdict | Evidence |
|---|---|---|---|
| 1 | Author the `prometheus:component/*` WIT family | **PARTIAL** | The family is authored and parses as one package; a real component was built against it and validates. But **nothing has executed it** — UAR's Wasm tier is still a stub, and `change-msp-008` was BLOCKED for want of cross-repo authorisation. Well-formed ≠ working. |
| 2 | Build `fabric-integration` — invariants enforced, not documented | **MET** | Three of four invariants fail CI on drift; the already-violated fourth is quarantined in an allowlist that fails **both** on un-allowlisted violations and on allowlisted entries that have been fixed. Five mutation tests, all executed. |
| 3 | Mobile FFI bindings | **MET** | `substrate/skill-ffi` builds for `aarch64-apple-ios` (16,408 B dylib) **and** `aarch64-linux-android` (454,856 B .so), 7 round-trip tests asserting on returned values. Both falsifiers from the pattern decision were tested and passed. |
| 4 | Verify `opencode`/`kimi` Tier 1; route `zed` or state why not | **MET** | All four file-pair harnesses now have **executed** round trips with an independent blind responder, each confirmed under `bash -x` to reach `_render_tier1_file_pair`. |

**3 MET, 1 PARTIAL.** Goal 1 is PARTIAL for a reason recorded before the work
began, not discovered at the end: its final proof needs a repository this phase
was not authorised to write to.

## What the plan got wrong, and what fixed it

### The blocking question dissolved on investigation

Assess ended by asking whether cross-repo writes were authorised, treating it as
a precondition for the whole phase. It was not. **UAR consumes this repository
as a submodule** (`crates/prometheus-skill-system`) and discovers `skill.wasm`
beside `SKILL.md` inside it. Producing components is in-repo work; UAR picks
them up by bumping a pointer.

That reordered everything: 8 of 9 changes needed no authorisation at all, and
the one that did was isolated to last. **Root cause of the original framing:** I
treated "the host is broken" and "we cannot deliver components" as the same
statement. They are not.

### A third guest target nobody had counted

`skills/rust/librefang-wasm-skill/` ships wasm templates targeting **core wasm
with an `extern "C"` pointer ABI** and zero `.wit` files. UAR loads
`wasmtime::component::Component`. Guests from those templates **cannot load
there** — different binary formats, no adapter. The pack already ships wasm
skill tooling that cannot run in the host this phase targets. Recorded in
`MAPPING.md`; unresolved by design.

### The classifier's residual class was wrong for every member

`change-msp-001` classified E1 as "pure text/JSON transformation" — the residual
after E0/E2/R matched. `change-msp-006` went looking for a pure skill to port
and found **all 18 E1 members touch the filesystem or clock**. The residual had
silently absorbed every skill no other rule matched.

**Root cause:** a residual class is a guess wearing a verdict's clothes. I even
wrote that risk into the script's header ("the class most likely to be wrong,
which `--check` cannot detect") and still shipped it. **Corrective action
applied:** E1 now carries `needs_capabilities`, so "portable" states its price.

### I compared two patterns and never asked what the target already used

`change-msp-007` chose uniffi over cbindgen on a maintenance-cost measurement.
Adversarial review returned CRITICAL: the stated target is Flutter. One command
against `know-me-system` showed **`flutter_rust_bridge` 2.12.0 already in
production there** — a third pattern, in neither column.

**Root cause:** I compared the two options I had found rather than establishing
what the consumer used. Choosing uniffi would have added a second FFI toolchain
to an app that already had one, and made the actual delivery surface the one
platform needing a bridge to the bridge.

## Deltas — planned and not delivered

- **`change-msp-008` BLOCKED.** Cross-repo authorisation was requested and not
  granted; silence blocks by the change's own contract. **Zero files** were
  modified in `flint-realtime-fabric`, `universal-agent-runtime`, or
  `know-me-system` — verified by `git status` on all three.
- **Falsifier 3 of the FFI decision is open.** Marginal cost per added function
  cannot be measured at authoring; it needs a second function over time.
  Recorded as open rather than quietly closed.
- **No generated Dart committed.** `flutter_rust_bridge_codegen` output belongs
  with the consuming app; generating it here would commit bindings nothing
  imports.
- **`cursor` remains Tier 0.** Not exercised, not claimed.

## Technical debt introduced

| Item | Where | Note |
|---|---|---|
| Component never executed | `entity-graph-optimize/component/` | unblocks with `change-msp-008` |
| `knowme:plugin` dual version still quarantined | `fabric-integration/assets/known-violations.json` | allowlist shrinks when something adopts the unified world |
| librefang templates target an incompatible ABI | `skills/rust/librefang-wasm-skill/` | port, keep both, or retire — undecided |
| CI verifies 1 of 4 invariants | `.github/workflows/validate.yml` | runner has no sibling repos; SKIP is honest but not coverage |
| Waypoint `.phase` goes stale at reflect | `~/.claude/skills/kbd-reflect/` (installed, not this repo) | **recurring — hit twice today.** Only `kbd-next-phase` ever writes `.phase`; reflect does not, so it names a phase from two transitions ago. Evidence: [`evidence/waypoint-phase-staleness.md`](evidence/waypoint-phase-staleness.md) |

## Pre-existing, not introduced

Two `sovereign-sync` control-token tests still fail
(`one_projects_token_is_rejected_by_another_project`,
`two_projects_mint_distinct_identities_and_tokens`). Confirmed last phase by
`git stash` that they fail identically without any of this work. **Deliberately
not fixed** — out of scope, and absorbing them would hide them.

## Lessons

1. **Ask what the consumer already uses before comparing options.** Two of this
   phase's four significant errors were the same shape: I evaluated the
   alternatives in front of me instead of establishing the incumbent.

2. **A residual class is not a verdict.** E1 was wrong for 18 of 18 members and
   `--check` could never have caught it, because drift detection compares a file
   to itself.

3. **Run it, don't read it.** zed's Tier 0 fallback had *two* causes; fixing the
   one visible in `render.sh` left it still broken, and only executing the round
   trip revealed the hardcoded tier in the detector.

4. **A fix can introduce the defect it was meant to prevent.** My repair for the
   CI paradox in `change-msp-004` produced an audit-only check that no longer
   met the goal's word "enforced"; my new WIT package exposed that the
   allowlist could leak onto packages it was never granted for. Both were caught
   by a judge that was not the author.

## Recommended next phase — `uar-host-execution`

Small, well-specified, and gated on one decision:

- **De-stub UAR's Wasm runtime** — needs the cross-repo authorisation this phase
  did not get. Everything else is ready: the world is authored, a validated
  component sits where discovery looks, and the FFI boundary is built.
- **Decide the librefang ABI question** — port to `prometheus:component`, keep
  both targets, or retire the templates.
- **Close falsifier 3** by adding a second FFI function and measuring the
  marginal cost.
- **Give CI the sibling repos** so `fabric-integration` verifies four invariants
  instead of one.
- **Fix waypoint `.phase` staleness in `kbd-reflect`.** It hit twice today and
  every skill instructs agents to trust that file for their position. The fix is
  one line beside the existing `.status` write — but it lives in an installed
  skill, not this repo.
- **Exercise `cursor`, or state a mechanism reason it cannot reach Tier 1.**
  Raised by the sycophancy gate. It is detected and left at Tier 0 with no
  stated cause — the same shape as `zed` before this phase found two.

## Sycophancy gate

Routed through `analyze_reflect_phase` at `strict`:

| Field | Result |
|---|---|
| `sycophancy_score` | **0.018** (rejection threshold 0.4) |
| `s08_detected` (Reflect Phase Inversion) | **false** |
| Patterns | 1 × S-07 (Low) — length |

S-08 is the one that matters: it fires when a reflection summarises success
instead of naming deltas. It did not fire.

**One point from the gate's rewrite is worth keeping.** Its corrective-actions
list includes *"Exercise and promote `cursor` beyond Tier 0"* — which this
reflection had recorded only as a passive "not exercised, not claimed". That is
fair: `cursor` is detected by `_detect_harness` and left at the text floor for
no stated mechanism reason, exactly as `zed` was before `change-msp-002` found
two independent causes. It is added to the next phase rather than left as a
footnote.

The rest of the rewrite proposed replacing the goal table and evidence column
with a three-section template. **Not applied** — the evidence column is what
makes a 3-MET/1-PARTIAL claim checkable.
