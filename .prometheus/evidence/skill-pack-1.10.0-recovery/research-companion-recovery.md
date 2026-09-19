# Research and Companion recovery evidence

Date: 2026-09-19
Recovery task: `skill-pack-1-10-recovery/release-skill-pack-1-10-recovery/recovery-3`
Research commit: `ae09a5a05983d4a67f999d164ccdaf20af49e665`
Companion-removal commit: `15c874e4f5e79c05ae8ba36021400de946106312`

## Delivered behavior

- Research and learning changes from `research-agent-hardening` are committed as
  one production group, including daemon execution, durable stage contracts,
  source scoring, learner-model writes, FSRS updates, coherent artifact paths,
  OpenSpec capabilities, and local evidence.
- The pack no longer builds, installs, starts, stops, or publishes the relocated
  Sovereign Sync service or its three sync skills. Current operational docs point
  to the Companion-owned installers.
- Optional control-plane discovery is silent when absent. The versioned contract
  identifiers remain for compatibility, while doctor/setup use the shared
  discovery path.
- The stale `--sharing` paths in both binary and service installers are rejected
  by process tests.

## Commands and observed results

| Command | Exit | Observed result |
|---|---:|---|
| `env -u KBD_PRODUCER_MODEL bash skills/research/deep-research/tests/driver-contract.sh` | 0 | 128 passed, 0 failed. |
| `bash skills/research/deep-research/tests/scoring-graph.sh` | 0 | 45 passed, 0 failed. |
| `bash skills/learn/feynman-loop/tests/learn-coherence.sh` | 0 | 58 passed, 0 failed. |
| `KBD_PRODUCER_MODEL=codex bash skills/process/adversarial-review/tests/run-fixture-suite.sh` | 0 | 28 passed, 0 failed; four live judge calls. |
| `bash shared/scripts/tests/test-companion-service-absence.sh` | 0 | Companion definitions absent; both removed `--sharing` flags rejected. |
| `bash shared/scripts/tests/test-detect-toolchain-control-plane.sh` | 0 | Missing optional endpoint reported disabled with no stale daemon row. |
| `bash shared/scripts/tests/test-learning-service-install.sh` | 0 | Learning worker and hook rotation definitions rendered correctly. |
| `bash shared/scripts/tests/test-service-exclusions.sh` | 0 | Service exclusion fixtures passed. |
| `npm run check:services-manifest` | 0 | Generated service manifest is current. |
| `openspec validate --specs` | 0 | 31 specs passed, 0 failed. |
| `npm run validate:strict` | 0 | Strict validator exited 0; existing advisory description warnings remain visible. |
| `cargo test --locked --manifest-path tools/prometheus-cli/Cargo.toml -p prometheus-cli --test doctor` | 0 | 10 passed, 0 failed. |
| `cargo test --locked --manifest-path tools/prometheus-cli/Cargo.toml -p prometheus-cli --test contract` | 0 | 6 passed, 0 failed. |
| `cargo test --locked --manifest-path crates/prometheus-exec/Cargo.toml --test cli` | 0 | 9 passed, 0 failed after a 7m14s cold build. |
| `git diff --check` and staged variants | 0 | No whitespace errors. |

Before every Cargo command, `pgrep` was checked for `cargo`/`rustc` and
`sysctl vm.swapusage` was captured. One unrelated `aso-web-server` writer was
allowed to finish before these commands began. The passing runs were serial.
Swap remained high at 26,155.75 MiB used of 26,624 MiB.

## Review status

The research phase retains its isolated per-change critic records under
`.kbd-orchestrator/phases/research-agent-hardening/review/`. The Companion
migration defects repaired here were observed integration failures: a stale
binary-installer `--sharing` path and stale daemon-specific tests after the
crate relocation. Their process regressions passed above.

## Open release work

Generated indexes, manifests, documentation projections, and `dist/` are still
deferred to the single release-generation pass after integration onto current
`origin/main`.
