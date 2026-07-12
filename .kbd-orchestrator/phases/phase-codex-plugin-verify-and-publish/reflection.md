# Reflection — phase-codex-plugin-verify-and-publish

_Reflected 2026-07-12. 6/6 changes DONE. Closed the three caveats carried from `phase-codex-plugin-distribution-and-ci`._

## Summary

Set out to "verify" three prior-phase caveats and found the verification itself
had a prerequisite nobody had checked: **`main`'s CI had been red for days**,
pre-dating this work, silently swallowing the `validate:codex` gate shipped last
phase. Fixing that (cowork Progress Signals + prettier ignores) turned out to be
the actual G-01 deliverable. From there: real (non-probe) plugin hooks were
confirmed to fire and resolve paths correctly under Codex, a remote marketplace
resolved all 11 plugins from a GitHub URL, and the env-provisioning helper was
delivered — with one honest gap remaining on live-value forwarding.

## Goal achievement

| Goal | Status | Evidence / caveat |
|---|---|---|
| G-01 validate:codex verified in real CI | **MET** | Actions run `29195543794`: **success**, `✓ Validate Codex plugin artifacts are in sync` executed and passed. Required fixing 2 pre-existing failures (cowork `## Progress Signals`, 32-file prettier gap) first — CI went red→green |
| G-03 real hooks run under Codex (not just probe) | **MET** | `codex exec --dangerously-bypass-hook-trust`: real `SessionStart`/`Stop` hooks fired; zero path-resolution errors; `${CLAUDE_PLUGIN_ROOT:-$PLUGIN_ROOT}` confirmed working with actual hook commands, not a synthetic probe |
| G-04 git-subdir / remote resolution | **MET (scope reduced by user choice)** | User chose the non-destructive quick test over a full publish: `codex plugin marketplace add <github-url>` cloned the pushed repo and resolved **all 11 plugins** remotely. The `git-subdir` source-type path itself (vs `local`) was **not** exercised — deferred, not broken |
| G-02 MCP env round-trip | **PARTIAL** | Helper works (bash 3.2, idempotent, no secrets, writes `inherit="all"`); plugin's `tavily` server registers with a `TAVILY_API_KEY` env entry. But the throwaway test home had no Codex auth, so `codex doctor` couldn't evaluate the MCP section — **live-value forwarding from `inherit="all"` to a spawned server was not confirmed**, only the literal-env path (proven last phase) |

**3.5/4 MET** — I'm not rounding G-02 up to MET; it's the one real gap this phase leaves open.

## Delivered changes

001 cowork-progress-signals · 002 format-fix · 003 ci-green-verify · 004 real-hooks-codex · 005 env-roundtrip · 006 git-subdir-publish (scoped to quick remote test per user choice). All archived under `.kbd-orchestrator/changes/archive/2026-07-12-*`.

## Artifact Quality Summary

| Metric | Value |
|---|---|
| Changes with QA (artifact-refiner) | 0/6 |

No `.refiner/artifacts/` logs — each change was small/doc-centric (size-rule skip)
or its verification *was itself* a live-system check (CI run, codex exec), which
is a stronger signal than a constraint-gated static QA pass would have been here.

## Delta → Root Cause → Corrective Actions

**Delta 1 — G-02 env round-trip stayed partial.**
I built a throwaway `CODEX_HOME` to avoid mutating the user's real config (correct
caution), but that same isolation removed Codex auth, so `codex doctor`'s MCP
section couldn't evaluate the live-forwarding claim.
*Root cause:* the safety mechanism (throwaway home) and the thing being tested
(auth-dependent doctor output) were in tension; I didn't have a way to test both
safely and completely in one pass.
*Corrective:* either (a) accept a *scoped* mutation to the real `~/.codex` with a
before/after diff and restore, or (b) directly inspect the spawned MCP server
process's env (`ps eww` on the child) rather than relying on `codex doctor`'s
summary. Flag this as the one open thread if anyone builds on G-02.

**Delta 2 — G-04 scope was reduced by design, not oversight.**
The user explicitly chose the non-destructive quick test over a full git-subdir
publish. This is a correct call, not a shortfall — recording it as a delta only so
future readers don't assume git-subdir itself was verified.
*Corrective:* none needed unless/until an external publish is actually wanted;
generator support already exists and was validated for idempotency last phase.

## Technical debt introduced

- None new. `.prettierignore` grew by 6 lines (2 submodules, 2 generated-artifact
  dirs, vendored JS) — additive, matches the existing pattern
  (`marketplace/marketplace.json` was already there).

## Lessons captured

1. **A shipped CI gate is unverified until you watch it actually run.** Last
   phase's `validate:codex` step existed in `validate.yml` but had *never executed*
   — two unrelated pre-existing failures upstream of it in the same job silently
   swallowed it. "I added the CI step" ≠ "the CI step ran." Check `gh run list`
   after any CI change, not just that the YAML parses.
2. **Testing in isolation can remove the very thing you're testing.** The
   throwaway `CODEX_HOME` for G-02 was the right instinct (don't mutate the user's
   real config) but also stripped auth, which made the test inconclusive. Isolate
   the *side effect* you're worried about, not the *feature* you're verifying.
3. **Verifying with the real artifact (not a probe) catches what synthetic tests
   miss.** G-03's synthetic probe (last phase) proved *a* hook could fire; this
   phase's run of the *actual* 39 hooks proved the fix generalizes — different
   confidence levels, both worth doing.
4. **When a user offers a menu of verification depths, the "quick/non-destructive"
   option is usually still a real verification** — G-04's quick remote test
   genuinely confirmed remote resolution; it just didn't exercise every source type.

## Recommended next phase

No urgent Codex-plugin work remains — G-02's live-forwarding gap is minor and has
a documented, proven fallback (inline env). Options, none forced: (a) a tiny
follow-up to definitively close G-02 (test against the real authed `~/.codex` with
a diff+restore, or inspect the spawned process env directly); (b) move on to a
different area of the pack — the UAR Codex App Server / `codex mcp-server`
orchestration frontier remains open in the *UAR* repo, not this one. Confirm
direction with the user before opening a new phase.
