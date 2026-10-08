# Issue 160: hook activation and immutable payloads

Scope: [issue 160](https://github.com/Prometheus-AGS/prometheus-skill-system/issues/160),
patch release identity **1.12.1**. No release tag or broader publication.

## Execution plan

1. Implement: prevent implicit downgrades; journal and recover activation; protect
   shell and compiled dispatch; provide native-cache retirement and migration docs.
2. Validate locally: exercise activation/recovery, packaged learning hooks and
   historical native caches; review the complete change; check generated outputs,
   release identities and committed protected-test integrity.
3. Deliver and repair: merge the reviewed PR, back up and quiesce the affected
   machine, install the merged release, refresh native registrations, quarantine
   superseded caches, restart sessions and verify installed hooks. Close the issue
   only after this final machine verification. A blocked restart leaves it open.

The implementation preserves explicit rollback and bundle-bound dispatch. It
does not alter historical signed generations, dependency pins or protected BDD.

## Local evidence

Execution directory:
`/Users/gqadonis/.codex/worktrees/issue-160-hook-activation/prometheus-skill-pack`.
Detailed command/stdout/stderr records are retained under
`/tmp/issue160-evidence`. No hosted testing was used.

The compiled runtime was built once, with no competing Cargo build:

```sh
env -u CARGO_TARGET_DIR \
  PATH=/Users/gqadonis/.cargo/bin:/Users/gqadonis/.nvm/versions/node/v24.21.0/bin:/opt/homebrew/bin:/usr/bin:/bin \
  RUSTUP_TOOLCHAIN=stable cargo build \
  --manifest-path crates/prometheus-hook/Cargo.toml --locked
```

Result: PASS. Log: `cargo-build.log`. Binary SHA-256:
`0315c102a864e6abaa77f8835021316264416835921406db91ade9ff494ddbd1`.

The following commands use `node` from
`/Users/gqadonis/.nvm/versions/node/v24.21.0/bin`. The shell lookup order puts
`/usr/bin:/bin` before `/usr/local/bin`; the packaged Python harness uses
`/Library/Frameworks/Python.framework/Versions/3.12/bin/python3`.

```sh
ISSUE160_EVIDENCE=/tmp/issue160-evidence/final-activation \
  node scripts/tests/hook-activation-recovery.integration.mjs

/bin/bash scripts/tests/test-hook-bytecode-integration.sh \
  --full /Users/gqadonis/.codex/worktrees/issue-160-hook-activation/prometheus-skill-pack \
  --evidence /tmp/issue160-evidence/final-bytecode \
  --scratch /tmp/issue160-scratch \
  --hook-runtime-bin /Users/gqadonis/.codex/worktrees/issue-160-hook-activation/prometheus-skill-pack/crates/prometheus-hook/target/debug/prometheus-hook \
  --legacy-payload /Users/gqadonis/.claude/plugins/cache/prometheus-skill-pack/prometheus-skill-pack/1.11.2

node scripts/tests/stale-cache-retirement.integration.mjs \
  --scratch /tmp/issue160-cache-scratch \
  --evidence /tmp/issue160-evidence/final-cache \
  --claude-bin /Users/gqadonis/.local/bin/claude \
  --codex-bin /opt/homebrew/bin/codex \
  --legacy-codex-cache /Users/gqadonis/.codex/plugins/cache/prometheus-skill-pack/prometheus-skill-pack/1.11.3

node scripts/generate-harness-adapters.js --check
node scripts/generate-skill-system-distribution.js --check
node scripts/check-release-version-matrix.mjs
git diff --check
```

Results:

- Packaged learning: PASS, 74 real processes, both clients and both runners,
  unset/conflicting bytecode environment, real isolated write/recall and reviewed
  coverage, unchanged payload bytes/modes and deliberate contamination rejection.
  Receipt: `final-bytecode/hook-bytecode-result.json`.
- Cache retirement: PASS, 12 scenarios. Receipt: `final-cache/report.json`.
  The original 1.7.0 native cache lacks Git metadata required by its installer.
  The initial prerequisite failure is retained. Supplying external Git provenance
  from the exact historical tag, without modifying cached source, reproduces the
  signed downgrade and pointer/link/runtime disagreement; the original installer
  verifies the resulting historical generation. Corrected repair, real native
  registration refresh, live-session blocking, intact quarantine, idempotence and
  interrupted/corrupted inventory handling all pass.
- Activation/recovery: PASS, 24 production processes. Receipt:
  `final-activation/result.json`; exact commands: `final-activation/commands.json`.
  An earlier full concurrent bootstrap
  exposed the 60-second shared-lock timeout. Both installer and shell bootstrap
  now allow five minutes; the final activation gate covers that correction.
- The bytecode and cache gates passed after reconciling upstream reviewed-closure
  installation changes. The only later production change lengthens the shared
  lock timeout; the final activation gate exercises it.
- Generated adapters, both native distributions, release-version matrix and
  whitespace checks: PASS. Final read-only implementation review and timeout-delta
  review reported no blockers. No protected BDD edits are included.

Protected-test certification runs after committing the candidate and before push:

```sh
node scripts/verify-protected-tests.mjs \
  --base 53e3e4b42574a8a17cc7e1d62d35cdb6fc93b62e --candidate HEAD
```

## Rollout boundary

Use [the recovery procedure](../guide/hook-activation-recovery.md). The initial
machine inventory found a signed 1.12.0 generation with 1.11.3 native caches and
live Claude sessions holding the old cache. Local acceptance does not certify
those existing sessions. Merge identity, backups, repair commands and final
restart/verification results belong in the issue's rollout record before closure.
