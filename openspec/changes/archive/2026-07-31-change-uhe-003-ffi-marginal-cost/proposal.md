# Close FFI falsifier 3 by measuring marginal cost

**Change:** `change-uhe-003-ffi-marginal-cost`
**Phase:** uar-host-execution
**Goal:** S3

## Why

See `.kbd-orchestrator/phases/uar-host-execution/plan.md` for full rationale,
acceptance criteria, and the two-round adversarial review record.

## Outcome: threshold not exceeded — the decision STANDS

Task 4 was conditional ("if over threshold, reverse the decision"). **The
threshold was not reached, so no reversal was made** — the task is closed as
not-applicable rather than left looking like a reversal happened.

### The measurement

`list_skills` was added to `substrate/skill-ffi` — a genuine need, not a probe:
a mobile client cannot invoke a skill it cannot enumerate.

| Category | Hand-written lines |
|---|---|
| FFI attributes / annotations (`#[...]`) | **0** |
| `extern "C"` / `no_mangle` / `unsafe` | **0** |
| `Cargo.toml` | **0** |
| `lib.rs` | **0** |
| `build-mobile.sh` | **0** |
| Dart | **0** |
| **Total glue** | **0** |

The 21 lines added to `api.rs` decompose as 9 doc comments, 3 inline comments,
1 blank, and **8 lines of ordinary Rust function body**. That is the function,
not the cost of exposing it.

**Threshold: >~20 lines reverses. Actual: 0.**

`flutter_rust_bridge` generates from the plain signature — there is no
annotation to write, which is exactly the property the pattern was chosen for.

### Why this matters beyond the arithmetic

The decision was recorded **provisional** precisely so it could be tested by
building rather than ratified by argument. All three falsifiers are now closed
by measurement, and the decision-log entry carries the outcome, so a later reader
sees what happened rather than only what was intended.

Both mobile targets still build (iOS 16,408 B dylib; Android 454,856 B `.so`)
and **8/8 tests pass**, including a new one asserting `list_skills` reports
`no host bound` rather than an empty catalog — an empty `Ok(vec![])` would read
as "no skills exist" when the truth is "nothing can answer yet".
