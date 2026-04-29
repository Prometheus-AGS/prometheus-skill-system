---
id: change-005-package-and-upload
title: forge package-librefang + /upload-to-bossfang + /start-business-build
phase: phase-compliance-and-power-multiplier
gaps: [G3, H4, G4, G5]
priority: P0
effort: M
agent: architect
evolver_item_id: null
status: DONE
completed: 2026-04-29
security_review: "passed (7 findings, all addressed before completion — see threat-model.md)"
forge_subcommand_status: "spec queued at tools/forge-rs/.forge/changes/forge-package-librefang/proposal.md for phase-librefang-wasm-onramp"
---

# change-005 — Packaging, Upload, and Headline Orchestrator

## Context

This is the change that *closes the loop*: it takes the WASM-target output from
change-004 and makes it deployable to a running LibreFang/bossfang instance.
It also adds the headline `/start-business-build` orchestrator that strings every
existing pipeline stage together — the single onramp that makes the whole pack
feel like one product instead of a kit.

## Scope

In:

- New `forge` subcommand `forge package-librefang <path>`:
  1. Validate the project layout (must contain `agent-skill` crate + root
     `skill.toml`).
  2. Run `cargo build --target wasm32-wasip2 --release -p agent-skill`.
  3. Validate the emitted `.wasm` against the ABI (re-use change-003's
     `validate-wasm-abi.sh`).
  4. Produce `<agent-name>.lf-skill.zip` containing `<name>.wasm`, `skill.toml`,
     `README.md`, and any declared `assets/`.
  5. Print the absolute path to stdout, JSON-formatted for downstream piping.
- New slash-command `/upload-to-bossfang <url>`:
  - Looks for a `*.lf-skill.zip` in the current directory (or accepts a `--zip`
    flag).
  - POSTs the zip to `<url>/skills/install` with `Content-Type: application/zip`.
  - Calls `<url>/skills/reload`.
  - Reports the installed manifest by GETting `<url>/skills/<name>`.
  - SSRF guard: validates the URL hostname against an allow-list pattern (loopback
    plus user-configured allowed hosts from `~/.config/prometheus-skill-pack/bossfang-allowlist.toml`),
    refuses unknown destinations unless `--insecure` is passed.
- New slash-command `/start-business-build "<concept>"`:
  1. (Stub) call ideation-mindmap if available, otherwise pass concept directly.
  2. Run `zeespec-interrogator` with the concept → constraint manifest.
  3. Run `iterative-evolver assess + plan` → OpenSpec change set (or KBD
     equivalent if openspec/ is absent).
  4. For each change: `forge enrich` → dispatch to AI implementer (Claude/Codex)
     → `forge reflect` → `pk ingest`.
  5. After acceptance: offer `forge package-librefang` + `/upload-to-bossfang`
     as a final step (interactive prompt).
- New marketplace sub-package `prometheus-librefang-skills` in
  `marketplace/marketplace.json` (Gap G4) so users can install just the WASM
  capability.

Out:

- Full ideation-mindmap implementation — that's H1, deferred to a later phase.
  This change uses a stub that simply forwards the concept text.

## Deliverables

1. `forge` CLI with `package-librefang` subcommand (in `tools/forge-rs/crates/forge-cli`).
2. New skills under `skills/process/native-agent/skills/`:
   - `upload-to-bossfang/SKILL.md` + `scripts/upload.sh`
   - `start-business-build/SKILL.md` + `scripts/orchestrate.sh`
3. `marketplace/marketplace.json` with the new sub-package.
4. SSRF allowlist config example at `references/bossfang-allowlist.example.toml`.

## Acceptance Criteria

- `forge package-librefang ./test-agent` produces `test-agent.lf-skill.zip` with:
  - `unzip -l` showing `*.wasm`, `skill.toml`, `README.md`.
  - `unzip -p test-agent.lf-skill.zip skill.toml | tomlq '.runtime.type'` returns
    `"wasm"`.
- `/upload-to-bossfang http://localhost:4545` successfully installs the skill
  into a running librefang instance and the GET-back returns the manifest.
- `/upload-to-bossfang https://attacker.example.com` is rejected with a clear
  SSRF-guard error message unless `--insecure` is passed.
- `/start-business-build "track competitor pricing"` produces a chain of
  artifacts ending in (a) a runnable WASM skill and (b) an upload prompt.

## Security Review Required

This change MUST be reviewed by `security-reviewer` before merge. The upload
command is a classic SSRF surface. The implementation must:

- Default-deny non-loopback URLs.
- Require explicit `--insecure` to allow public URLs (and warn loudly).
- Honor `bossfang-allowlist.toml` for known-good production instances.
- Refuse `file://`, `gopher://`, `dict://` and any non-`http(s)` scheme.
- Never echo the auth token (if any) into stdout/stderr/logs.

## Files to Touch

- `tools/forge-rs/crates/forge-cli/src/main.rs` (add subcommand)
- `tools/forge-rs/crates/forge-cli/src/cmd/package_librefang.rs` (new)
- `skills/process/native-agent/skills/upload-to-bossfang/` (new)
- `skills/process/native-agent/skills/start-business-build/` (new)
- `marketplace/marketplace.json`
- `scripts/register-slash-commands.sh` — register the two new commands

## Test Plan

- Unit: forge CLI test against a fixture project.
- Integration: full end-to-end with a local librefang spawned via `librefang start`.
- Security: a dedicated test file `tests/ssrf-guard.rs` confirms each blocked
  scheme/host pattern is rejected.
- Smoke (the §9 verification plan): runs from `npm run doctor` through to
  `unzip -l <zip>`.
