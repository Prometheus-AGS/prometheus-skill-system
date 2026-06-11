---
id: change-003-pk-and-binary
title: pk health/ingest wiring, sycophancy binary CI build, karpathy decision
phase: memory-and-karpathy
gaps: [M3, M4, M5]
priority: P2
effort: M
agent: claude-code
evolver_item_id: null
status: done
scope:
  - shared/scripts/pk-health.sh
  - skills/process/kbd-process-orchestrator/hooks/hooks.json
  - hooks/hooks.json
  - package.json
  - .github/workflows/validate.yml
  - skills/rust/karpathy-tokenizer/SKILL.md
  - shared/scripts/tests/test-pk-health.sh
  - shared/scripts/tests/test-sycophancy-gate-e2e.sh # scope expansion: real-binary e2e test required by CA-4
---

# change-003 — pk wiring, binary CI, karpathy decision

## Context

Closes three smaller carry-forwards: pk health isn't checked, pk ingest only
runs at Stop (not reflect), pk-lint/mem0-compress are orphaned, the sycophancy
binary is never built in CI (artifact gate's real path untested), and
karpathy-tokenizer's role is re-litigated each session.

## Scope

In:

- New `shared/scripts/pk-health.sh` (SessionStart, `|| true`, 24h-throttled via
  `~/.prometheus/pk-health-last-run`): if `pk` on PATH, run `pk lint --check`
  and print a one-line health summary; missing pk → silent exit 0.
- `KBD/hooks/hooks.json`: add `pk ingest` to a `reflect:end` builtin entry
  (after the memory write from change-002; keep the Stop-hook ingest as-is).
- Root `hooks.json`: wire pk-health into SessionStart.
- `package.json`: `"lint:pk": "bash shared/scripts/pk-lint.sh"`,
  `"memory:compress": "bash shared/scripts/mem0-compress.sh"` so the orphaned
  scripts are runnable/documented.
- `.github/workflows/validate.yml`: extend (or add to) the rust job — build the
  sycophancy-correction binary (`cargo build --release` in
  skills/imported/sycophancy-correction) and run one real end-to-end gate test
  (feed a known-sycophantic reflection through the artifact gate and assert it
  sets reflect_gate). Skip gracefully if the submodule isn't checked out.
- `skills/rust/karpathy-tokenizer/SKILL.md`: add a short "Integration status:
  reference-only — intentionally not hook-wired" note so the decision is on
  record.
- New `shared/scripts/tests/test-pk-health.sh`: missing pk → exit 0 silent;
  fake pk → prints summary, respects 24h throttle.

## Tasks

- [x] 1. Write pk-health.sh; wire SessionStart
- [x] 2. Orchestrator reflect:end pk ingest; npm scripts for pk-lint/mem0-compress
- [x] 3. CI: build sycophancy binary + 1 real e2e gate test
- [x] 4. karpathy-tokenizer reference-only note; write test; run green

## Verification

Test green; CI workflow YAML valid; karpathy SKILL.md still validates strict;
pk-health no-ops without pk.
