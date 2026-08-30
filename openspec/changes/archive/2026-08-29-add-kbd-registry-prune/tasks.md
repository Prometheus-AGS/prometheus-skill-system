## 1. Registry Maintenance API

- [x] 1.1 Add structured missing-registration and prune-report types plus dry-run inventory to `ProjectRegistry`, and verify existing and missing paths are classified correctly in focused tests.
- [x] 1.2 Implement locked apply-time re-evaluation, timestamped registry backup, checksum, receipt, and atomic removal of missing entries, and verify runtime data plus existing registrations are preserved.
- [x] 1.3 Add tests for dry-run byte immutability, apply output, path reappearance, repeat-run idempotence, shared-project replicas, and rollback evidence.

## 2. CLI and Local Certification

- [x] 2.1 Add `prometheus kbd projects --prune-missing [--apply]`, reject invalid flag combinations, and verify JSON and human output through focused CLI tests.
- [x] 2.2 Update operator documentation with dry-run, apply, backup, and rollback behavior, and verify command examples match clap flag placement.
- [x] 2.3 Run Rust formatting, clippy with warnings denied, focused/full affected crate tests, release builds, protected-test verification, and `git diff --check`, recording exact local results.
