---
id: migration-and-rollout
title: Migration & Rollout
sidebar_label: Migration & Rollout
---

# Migration and Rollout

The canonical runtime can inventory and import legacy KBD ledgers while
preserving a recoverable copy. Rollout evidence is stored separately from the
authoritative project document so measurements can block promotion but cannot
grant write authority.

## Inventory legacy state

```bash
prometheus kbd --path "/path/to/project" migrate --check | jq .
```

The report includes:

- whether a top-level v1 journal still requires replica-layout migration;
- discovered and migrated progress files;
- uncertain legacy rows;
- invalid files;
- alias conflicts;
- phases marked `legacy-read-only`;
- stale compatibility projections;
- unreplayable history;
- backup paths when applying.

## Apply migration

```bash
prometheus kbd --path "/path/to/project" migrate --apply | jq .
```

Apply:

1. establishes `.prometheus/project.json` if needed;
2. re-signs each old journal event into the registered initial replica while
   preserving source event IDs and hashes as migration provenance;
3. fsyncs `replicas/<replica-id>/events.jsonl` and `project.loro`;
4. renames the old journal to `events.v1.jsonl.archive` and writes its SHA-256;
5. writes `JOURNAL-MIGRATION-ROLLBACK.md` without deleting any runtime data;
6. creates a checksummed backup of legacy projection inputs;
7. imports recoverable state and labels uncertain rows instead of inventing certainty;
8. writes atomic, frontier-stamped compatibility projections.

Never change a copied project manifest to “make migration fit.” A mismatched
project identity is rejected.

## Verify migration

```bash
prometheus kbd --path "/path/to/project" status --json | jq .
prometheus kbd --path "/path/to/project" audit --json | jq 'length'
prometheus kbd --path "/path/to/project" migrate --check | jq .
```

The final check should report no unexplained stale projections or unreplayable
history.

## Shadow and canary evidence

```bash
prometheus kbd --path "/path/to/project" rollout status | jq .
```

Record an idempotent observation:

```bash
prometheus kbd --path "/path/to/project" rollout observe \
  --observation-id "shadow-2026-07-28T120000Z" \
  --real-mutations 12 \
  --synthetic-replay-mutations 1500 \
  --unexplained-projection-mismatches 0 \
  --harness claude-code \
  --device workstation
```

Mark a failed observation with `--failed`. Advance only when all thresholds
for the current stage pass:

```bash
prometheus kbd --path "/path/to/project" rollout promote
```

The current production-convergence gates require seven shadow days, at least
100 real mutations, at least 10,000 synthetic replay mutations, and zero
unexplained projection mismatches before staged canaries. There is no voter or
quorum mode: one exclusive journal transaction is the write authority.
Disposable cross-process tests still exercise partitions, stale frontiers,
crash windows, and replay without turning those tests into consensus.
