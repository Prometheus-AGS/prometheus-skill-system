---
type: Reference
id: prometheus-skill-pack-1-11-2-hook-activation-closure
title: Prometheus skill pack 1.11.2 hook activation closure
tags:
- prometheus-skill-system
- hook-activation
- skill-pack
- codex-hooks
- doctor-checks
- plugin-sources
- release-1-11-2
links:
- karpathy-session-b56c000bebddc25f
sources:
- id: session
  resource: /private/tmp/claude-501/-Users-gqadonis-Projects-references-open-design/b39f7933-4014-479a-b258-793615c54f5b/scratchpad/karpathy-hook-activation-closure.md
generated:
  by: pk/1.11.0
  at: 2026-10-05T10:56:42.106394+00:00
created_at: 2026-10-05T10:56:42.106394+00:00
updated_at: 2026-10-05T10:56:42.106394+00:00
revision: 0
content_hash: c9274cb8c202be4a3c1bf14a592db5bf2152092e7714bd99b4abdd27cb674bde
---

## Release outcome

Prometheus skill pack **1.11.2** was released in PR #157 to close issue #156. The agent prepared the change but did **not** merge it.[^session]

This work addressed hook activation failures by testing hooks from copied payloads under empty `HOME`, improving bootstrap diagnostics, adding doctor checks, and guarding plugin refreshes from unsafe registered sources.[^session] It is related to prior Codex hook packaging investigation in [Karpathy session b56c000bebdd](/karpathy-session-b56c000bebddc25f.md), which found Codex hooks were not being shipped/wired in generated packages.

## Key changes

- Added a **payload activation test** for Claude and Codex: every shipped hook must execute from a copied payload with an empty `HOME`.[^session]
- Updated `scripts/hook-entry.mjs` so failed bootstrap emits one classified, actionable JSON line instead of a Node stack trace.[^session]
  - `PAYLOAD_INCOMPLETE`
  - `BOOTSTRAP_FAILED`
  - `NOT_ACTIVATED`
- Added `PROMETHEUS_HOOK_DEBUG=1` escape hatch to preserve raw output for debugging.[^session]
- Extended `prometheus doctor` with checks for:[^session]
  - `plugins.source-topology`
  - `plugins.native-cache-skew`
- Ensured `scripts/check-plugin-source.js` ships in the payload and is never resolved from the current working directory.[^session]
- Updated refresh scripts to refuse registered topic-branch sources unless explicitly overridden:[^session]
  - `scripts/update-skill-pack.sh`
  - `scripts/refresh-native-plugin-installs.sh`
  - default refusal exit code: `3`
  - override flag: `--allow-topic-branch`

## Root cause analysis

1. Skill pack versions **1.10.0** and **1.11.0** shipped `install-plugin-generation.js` without five required `scripts/lib` imports. This was fixed in **1.11.1**.[^session]
2. The recurring Stop-hook failure was caused by a registered marketplace source pointing at a topic-branch worktree that was later removed.[^session]
3. The harness refuses every hook when the plugin directory is missing, before any hook runs; therefore hook-level changes cannot fix this failure mode. The required controls are registration discipline plus detection.[^session]
4. Correction: sessions pinned to a 1.10.0 bundle continue working while that bundle remains registered. They were not “still failing”; the earlier clean-`HOME` inference was incorrect, and issue #156 records the correction.[^session]

## Corrective actions

### Prevention

- Added an activation-closure test into `check:distribution`.[^session]
- Included a mutation case to verify distribution completeness failures are caught.[^session]

### Degradation behavior

- Converted hook-entry bootstrap failures into classified diagnostics rather than raw Node stack traces.[^session]
- Kept raw output available via `PROMETHEUS_HOOK_DEBUG=1`.[^session]

### Detection and operations

- Added read-only doctor checks using `--no-optional-locks`.[^session]
- Added installer guards for unsafe refresh sources.[^session]
- Added operations documentation at `site/docs/operations/plugin-sources-and-hook-failures.md`.[^session]

## Review findings

The change underwent 12 rounds of independent cross-model review. Rounds 1–10 each found real defects; rounds 11–12 passed.[^session]

Defects found during review included:[^session]

- symlink aliases
- TOML spelling variants
- untracked files
- null paths
- current-working-directory-relative script execution
- `EACCES` being read as “missing”
- git probe failures hiding a branch
- Codex payload not being tested

## Engineering lessons

- A hand-written TOML reader tends to accumulate edge cases. The safer approach is to fail safe with `REGISTRATION_UNREADABLE` and bound the review loop rather than chasing every spelling variant.[^session]
- Doctor checks must never execute a same-named script from the current working directory. Tools should be resolved from an explicit root or from the installed generation.[^session]

## Verification

Verification was local only and covered:[^session]

- distribution checks
- matrix checks
- adapter checks
- docs-sync checks
- 3 Node test suites plus payload activation
- Cargo tests:
  - 33 unit tests
  - 25 doctor integration tests

## Known gaps

Documented PR gaps:[^session]

- Cold-start generation bootstrap is approximately 30 seconds, but `SessionStart` declares a 10-second timeout.
- Only the first `installed_plugins.json` record is compared.
- `refresh-native-plugin-installs.test.sh` is red on `main`.
- Old plugin caches were intentionally not pruned due to user constraint.

## Changed paths

```text
scripts/hook-entry.mjs
scripts/check-plugin-source.js
scripts/lib/plugin-source-topology.js
scripts/update-skill-pack.sh
scripts/refresh-native-plugin-installs.sh
scripts/generate-skill-system-distribution.js
scripts/generate-harness-adapters.js
scripts/tests/payload-activation.test.mjs
scripts/tests/hook-entry-diagnostics.test.mjs
scripts/tests/plugin-source-topology.test.mjs
tools/prometheus-cli/crates/prometheus-cli/src/commands/doctor.rs
tools/prometheus-cli/crates/prometheus-cli/src/commands/doctor/plugin_source.rs
tools/prometheus-cli/crates/prometheus-cli/tests/doctor.rs
site/docs/operations/plugin-sources-and-hook-failures.md
CHANGELOG.md
version files for 1.11.2
```

[^session]: Session notes from `karpathy-hook-activation-closure.md`.