# Skill Pack 1.10.0 local release certification

Date: 2026-09-19
Recovery task: `skill-pack-1-10-recovery/release-skill-pack-1-10-recovery/recovery-5`
Distribution behavior commit: `d6e5256`

## Delivered release state

- Umbrella release is 1.10.0; process skills are 1.6.0; research skills are 1.1.0; research server is 1.7.0; learn is 1.6.0.
- The distribution contains 166 canonical skills and installs identical payloads across all 14 configured targets.
- `karpathy-progress-memory` is generated into both Claude and Codex plugin distributions.
- The removed Sovereign Sync companion surfaces are absent from source, service manifests, generated plugins, and documentation generation.
- Windows drive, UNC, and verbatim paths use Windows containment semantics even when validation runs on a non-Windows host.
- Activation and pointer rollback remain contained when the generation store is reached through a symlinked ancestor.

## Commands and observed results

| Command | Exit | Observed result |
|---|---:|---|
| `npm run validate:strict` | 0 | Strict skill validation passed. |
| `npm test` | 0 | 18 of 18 deterministic suites passed. |
| `npm run test:control-plane` | 0 | Control-plane integration suite passed. |
| `node scripts/check-kbd-state.js` | 0 | 64 canonical phases, 5 legacy phases, 0 failures. |
| `node scripts/check-direct-writer.js` | 0 | Direct-writer gate passed. |
| `node scripts/check-progress-signals.js` | 0 | 57 process skills checked; 23 baselined. |
| `npm run validate:harness-adapters` | 0 | Claude 31 hooks, Codex 30 hooks; bundle `b852b4a2e0337ffb0a59a6c0abfbfe6464129ceb4a1a6cbb4b1d9fd9b0a7df9b`. |
| `node scripts/check-skills-index.js` | 0 | Generated skill index is current. |
| `npm run check:release-version` | 0 | Release version matrix verified for 1.10.0. |
| `openspec validate --specs` | 0 | 31 specs passed. |
| `openspec validate repair-kbd-progress-continuity --strict` | 0 | Change is valid. |
| `python3 skills/process/karpathy-progress-memory/tests/progress-memory-integration.py` | 0 | 12 success, outage, replay, idempotency, restart, and projection-integrity scenarios passed. |
| `env -u KBD_PRODUCER_MODEL bash skills/research/deep-research/tests/driver-contract.sh` | 0 | 128 passed. |
| `bash skills/research/deep-research/tests/scoring-graph.sh` | 0 | 45 passed. |
| `bash skills/learn/feynman-loop/tests/learn-coherence.sh` | 0 | 58 passed. |
| `bash shared/scripts/tests/test-companion-service-absence.sh` | 0 | Removed companion definitions and stale installer flags are absent. |
| `RUSTC_WRAPPER= cargo check --locked --manifest-path substrate/kbd-runtime/Cargo.toml` | 0 | `kbd-runtime` compiled. |
| `RUSTC_WRAPPER= cargo check --locked --manifest-path tools/prometheus-cli/Cargo.toml -p prometheus-cli` | 0 | `prometheus-cli` compiled. |
| `RUSTC_WRAPPER= cargo test --locked --manifest-path substrate/kbd-runtime/Cargo.toml --test position_continuity` | 0 | 1 passed. |
| `RUSTC_WRAPPER= cargo test --locked --manifest-path tools/prometheus-cli/Cargo.toml -p prometheus-cli --test kbd` | 0 | 8 passed. |
| `RUSTC_WRAPPER= cargo test --locked --manifest-path tools/prometheus-cli/Cargo.toml -p prometheus-cli --test contract` | 0 | 6 passed. |
| `RUSTC_WRAPPER= cargo test --locked --manifest-path tools/prometheus-cli/Cargo.toml -p prometheus-cli --test doctor` | 0 | 10 passed. |
| `npm run docs:check` | 0 | Mermaid, OpenAPI, documentation contracts, and the Docusaurus production build passed. |
| `npm run test:clean-install-parity` | 0 | 166 of 166 payloads matched across 14 targets. |
| `npm run test:install-policy` | 0 | Strict, best-effort, skills-only, and false-green policies passed. |
| `npm run check:distribution` | 0 | 166 canonical skills, payload parity, modes, pins, manifests, and marketplaces passed. |
| `npm run validate:codex` | 0 | Codex distribution validation passed. |
| `npm run docs:sync:check` | 0 | Documentation sync is clean. |
| `node scripts/tests/activation-pointer.test.mjs` | 0 | 18 checks passed; junction-only checks skipped on this symlink-capable volume. |
| `git diff --check` | 0 | No whitespace errors. |
| `git submodule foreach --recursive 'test -z "$(git status --porcelain)"'` | 0 | All initialized submodules are clean. |
| `git diff --cached \| gitleaks stdin --no-banner --redact` | 0 | No leaks found in the production behavior commit. |

Cargo commands ran serially. Before each command, the process check used `ps -axo pid=,comm=` filtered to exact `cargo`, `rustc`, and `sccache` executables, followed by `sysctl vm.swapusage`. No command reported here overlapped another repository writer.

## Isolated review

Artifact critic round 1: **BLOCK**. The reviewer found no test proving the Windows `path.win32` containment branch and no activation/rollback test through a symlinked store root.

Artifact critic round 2: **PASS**. The repaired artifact exercises drive, UNC, verbatim, cross-share, and cross-drive containment and performs activation plus both rollback pointer swaps through the symlinked root. The focused suite reports all 18 checks passing.

## Limits

This evidence certifies the local Skill Pack release inputs. Deployment receipts, native tool registrations, service health, the annotated tag, and the GitHub release are recorded by recovery task 6 after merge and redeployment. It does not certify Universal Agent Runtime. production-ready has no date.
