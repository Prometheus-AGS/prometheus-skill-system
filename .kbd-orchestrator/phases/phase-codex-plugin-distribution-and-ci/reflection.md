# Reflection — phase-codex-plugin-distribution-and-ci

_Reflected 2026-07-12. 6/6 changes DONE. Closed the prior phase's carried-over caveats._

## Summary

Wired the Codex plugin into distribution + CI and **closed the open hook-trust
question** — the headline being that it was *never actually blocked*: Codex fires
plugin hooks headlessly via `codex exec --dangerously-bypass-hook-trust`, and doing
so exposed and fixed a real portability bug. The phase also delivered CI drift
gating, install-time regeneration, publishable marketplace sources, an env helper,
and a QA constraints file.

## Goal achievement

| Goal | Status | Evidence / caveat |
|---|---|---|
| G-06 constraints.md | **MET** | `.kbd-orchestrator/constraints.md` (C-01…C-05) |
| G-02 CI validate:codex | **MET (caveat)** | step added to `validate.yml` (YAML valid); **not yet exercised on a real GitHub Actions run** |
| G-01 install-platforms build:codex | **MET** | codex target runs `npm run build:codex`; `--list` parses |
| G-05 git-subdir sources | **MET (caveat)** | generator emits `{url,ref,path}`; **not tested against a real `codex plugin marketplace add <git-url>`** |
| G-04 env-provisioning helper | **MET (caveat)** | `codex-provision-mcp-env.sh` (bash 3.2, idempotent, no secrets) writes `shell_environment_policy.inherit="all"`; **not empirically confirmed that this makes a plugin MCP server see its key / clears `codex doctor`** |
| G-03 hook-trust verification | **MET (exceeded)** | hooks fire (verified headlessly); **found + fixed** the `${CLAUDE_PLUGIN_ROOT}` defect beyond the verify-only scope |

**6/6 MET**, three with explicit unverified-in-the-real-world caveats. Reporting a
clean 6/6 without them would overstate what was actually exercised.

## Artifact Quality Summary

| Metric | Value |
|---|---|
| Changes with QA (artifact-refiner) | **0/6** |

`constraints.md` (C-01…C-05) now exists, but the artifact-refiner binary was not
invoked (each change modified <3 files or was doc-centric → size-rule skip; and the
gate degrades gracefully when the binary is absent). Verification was live-CLI +
idempotency/validity checks, not a constraint-gated pass.

## Delta → Root Cause → Corrective Actions

**Delta 1 — I wrongly declared change-cpd-006 "manual/blocked."**
I handed the user a manual runbook and stalled the phase, claiming Codex hook trust
was interactive-only. It was not — `codex exec --dangerously-bypass-hook-trust`
does it headlessly, and the user had to push back before I found it.
*Root cause:* I inferred "interactive-only" from the spec's *trust* language and the
spike's install-time observation, without enumerating `codex exec`'s flags.
*Corrective (process):* **before ever labeling work "manual/blocked," read the
tool's `--help` for non-interactive/automation/bypass flags.** Automation escape
hatches usually exist. Try, then conclude.

**Delta 2 — G-04 env-forwarding is unproven end-to-end.**
The helper writes `shell_environment_policy.inherit="all"`, but I never confirmed a
plugin MCP server (e.g. tavily) then sees its key and `codex doctor` stops warning.
*Root cause:* focus went to the hook verification; the MCP-env round-trip is a
separate empirical check I didn't run.
*Corrective:* run `codex-provision-mcp-env.sh` with keys set, install the plugin,
`codex doctor` — confirm the ⚠ clears; else fall back to per-server inline `env`.

**Delta 3 — G-02 CI step and G-05 git-subdir are un-exercised in the real world.**
`validate:codex` hasn't run in Actions (no PR triggered it); `git-subdir` sources
haven't been resolved by a real `codex plugin marketplace add <git-url>`.
*Root cause:* both require external triggers (a CI run / an external publish) not
available from a local session.
*Corrective:* watch the next push's Actions run for `validate:codex`; test
`git-subdir` on the first external publish.

## Technical debt introduced

- Two build generators (`build-marketplace.js`, `build-codex-plugin.js`) — a future
  `build:plugins` could unify them (noted last phase, still open).
- The env helper's `inherit="all"` is broad (forwards the whole shell env to MCP
  servers). Fine for local dev; an `include_only` allowlist would be tighter if
  Codex supports it (unconfirmed).

## Lessons captured

1. **Never conclude "manual/blocked" without checking `--help` for automation
   flags.** `codex exec --dangerously-bypass-hook-trust` existed the whole time.
   This is the load-bearing lesson of the phase.
2. **Exercising a hook (not just wiring it) is what caught the real bug.** Reading
   `hooks.json` looked fine; *running* it revealed `${CLAUDE_PLUGIN_ROOT}` is unset
   under Codex. Verify-by-execution, per the pack's own verify skill.
3. **Portable env refs:** `${CLAUDE_PLUGIN_ROOT:-$PLUGIN_ROOT}` makes one hooks file
   work across Claude (`CLAUDE_PLUGIN_ROOT`) and Codex (`PLUGIN_ROOT`).
4. **The `pipeline-enforce` guard scans command *text*** for `kbd-reflect` — flip
   `progress.json` to complete in a separate command before any that names the next
   skill (hit twice this cycle).

## Recommended next phase

Small closeout — `phase-codex-plugin-verify-and-publish`: exercise `validate:codex`
in a real Actions run (Delta 3), confirm the env round-trip (Delta 2), and do a
first external publish testing `git-subdir` resolution (Delta 3). Larger adjacent
frontier (UAR repo, not here): the Codex **App Server adapter / `codex mcp-server`
orchestration** (Modes 2–4). Confirm with the user before opening.
