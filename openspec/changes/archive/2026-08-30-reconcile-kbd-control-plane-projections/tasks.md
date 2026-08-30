## 1. Evidence Reconciliation

- [x] 1.1 Confirm the canonical nested child and duplicate compatibility paths, preserve the compatibility directory under `.kbd-orchestrator/backups`, and write a receipt with source, destination, canonical target, and hashes.
- [x] 1.2 Verify live phase discovery no longer sees the compatibility path while the canonical nested record and backup evidence remain readable.

## 2. Distribution and Installation Refresh

- [x] 2.1 Generate shared Codex/plugin distributions twice, prove identical tracked hashes, and run `npm run validate:codex` plus strict source validation locally.
- [x] 2.2 Refresh repository-managed user skill installations through the owned installer, verify source/install parity for touched skills, and prove user-owned collisions remain untouched.
- [x] 2.3 Build and install the tested release CLI binary with post-install version/hash verification.

## 3. Daemon-Free Local Certification

- [x] 3.1 Run live KBD memory hook-write and recall probes, verify a retrievable lifecycle entity plus a non-unreachable digest, and record bounded cleanup/retention evidence.
- [x] 3.2 Run registry prune dry-run, apply it once, verify its backup receipt and idempotent second run, and confirm retained project runtimes still exist.
- [x] 3.3 Keep sovereign-sync stopped and disabled, make its installation explicit to sharing workflows, then verify ordinary `prometheus kbd status` and typed mutations use the local signed runtime without unavailable-authority warnings.
- [x] 3.4 Run protected-test verification and `git diff --check`, then record all local commands/results in the phase evidence without using hosted CI.
