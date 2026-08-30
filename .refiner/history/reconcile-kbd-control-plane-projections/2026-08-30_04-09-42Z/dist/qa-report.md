# Artifact-refiner QA — reconcile-kbd-control-plane-projections

## Specification

- Preserve the duplicate compatibility projection outside live discovery without changing the canonical nested child.
- Generate distributions twice, prove byte identity, validate them, and refresh repository-owned user installations without touching foreign content.
- Certify live memory write/readback/recall and recoverable registry pruning.
- Keep sovereign-sync stopped and disabled by default while making explicit sharing installation available.
- Prove ordinary status, doctor, and typed mutations use the signed local runtime without unavailable-authority warnings.

## Deterministic constraint evaluation

| Constraint | Status | Evidence |
|---|---|---|
| C-01 generated surfaces | Satisfied | Two repository-owned generation passes produced the same aggregate SHA-256 `2a1e7eed67212a4af4661a4a2a928a75f5b896107bf42971e9c07c152240ec17` over 2,339 tracked outputs. Codex and strict validators passed. Installation verified 2,296/2,296 placements. |
| C-02 no committed secrets | Satisfied | A value-suppressing scan of added diff lines found zero recognized credential or private-key patterns. Receipts explicitly avoid secret-bearing hook output. |
| C-03 documentation parity | Satisfied after refinement | Public installation docs already covered `--sharing`; this iteration added the missing Codex-specific contract to `docs/codex-plugin.md` and the CLAUDE.md Codex section, refreshed both generated runtime references, and proved `docs:sync:check` clean. |
| C-04 generator idempotence | Satisfied | Pass-one and pass-two aggregate digests, tracked-file counts, output counts, byte totals, and modes are identical. Generated validation passed without hand editing. |
| C-05 Bash compatibility | Satisfied | `/bin/bash -n` passed for modified installer/service shell paths; inspection found no unsupported associative-array or `mapfile` use on a launchd-executed path. |
| Reconciliation specification | Satisfied | Ten valid JSON receipts cover compatibility relocation/readability, distribution, user installation, CLI installation, live memory, recoverable registry pruning, daemon-free KBD, and protected-test/diff certification. |

## Integration and runtime evidence

- Compatibility evidence was relocated under the backup namespace with matching source/destination hashes; live discovery now exposes only the canonical child.
- Managed installation activated signed generation `4158951e90c675b037a01b401a8a6c299e9b684b0e13f21c47bd8cc8a0eaef8d`; foreign user content remained byte-identical.
- Live memory health and readiness returned HTTP 200, a unique lifecycle entity was retrieved exactly once, and recall produced a 15-line non-unreachable digest.
- Registry prune removed 28 absent registrations after writing rollback evidence, preserved all 42 runtime directories, and a second apply was byte-idempotent with zero candidates.
- `ai.prometheus.sovereign-sync` is currently unloaded, disabled, and absent from the process table. Default rendering emits zero sovereign definitions; explicit `--sharing` emits launchd and systemd definitions.
- Installed status and typed decision mutation emitted no unavailable-authority warning; the mutation advanced signed revision 309 to 310 and folded back correctly.
- The local protected-test verifier reported zero protected changes, and `git diff --check` passed. Final committed-state protection is intentionally repeated before push.

## Regression and recovery review

- Canonical KBD history and retained project runtimes were not rewritten or deleted.
- Registry cleanup has a checksum-verified backup and structured receipt.
- Compatibility evidence remains readable outside the live phase namespace.
- The optional sharing daemon was never started during daemon-free certification.
- No hosted workflow or runner was used as validation evidence.

## Verdict

PASS. The change converged after correcting the Codex documentation omission and stale generated documentation projections. Proceed to bounded adversarial/sycophancy review, strict OpenSpec verification and archive, then parent-phase committed-state certification.
