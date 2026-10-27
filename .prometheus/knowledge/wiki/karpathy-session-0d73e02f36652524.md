---
type: SessionRecord
id: karpathy-session-0d73e02f36652524
title: Karpathy session 0d73e02f3665
tags:
- karpathy
- session-learning
sources:
- resource: session:3f374a11-0ce4-40d3-a1dc-4964cece6769
generated:
  by: pk/1.11.0
  at: 2026-10-05T17:27:29.888059+00:00
created_at: 2026-10-05T17:27:29.888059+00:00
updated_at: 2026-10-05T17:27:29.888059+00:00
revision: 0
---

## Delta

SurrealDB is now 3.3.0 and running, and `prometheus doctor` reports no required failures, only 4 warnings.

**SurrealDB upgrade**
- **Backup:** I stopped `surreal-memory-native` and `surrealdb-native`, then copied the RocksDB directory to `~/.prometheus/data/surrealdb-backup-3.2.0-202610051220`.
- **Binary:** I installed 3.3.0 at `~/.prometheus/bin/surreal`. It's the official darwin-amd64 release, but the project publishes no checksum (the `.sha256` URL returns 404), so I couldn't verify it. The SHA-256 is `915e91e5…`, and it reports `3.3.0 for macos on x86_64`.
- **Restart:** `install-mcp-services.sh --restart` re-rendered every plist and restarted the services. The datastore applied 3 data migrations cleanly and recorded version 3.3.0.
- **Health:** The endpoints on ports 28000, 23001, 8181, 8943, 8942, 7890 and 7891 all return 200, and the learning worker check passes.

**Repairs after the upgrade**
- **Interrupted generation swap:** the `current` symlink and the stable hook runner had moved to a newer generation (`f58037cc`) while the authoritative pointer still said `26bdb347`. That broke lifecycle-hook verification. I re-ran the generation installer to restore the runner, then pointed the `current` and `previous` symlinks at the pointer targets. The lifecycle-hook and Codex hook-graph checks now pass.
- **Stray bytecode:** a hook had written `__pycache__/*.pyc` into an installed immutable generation, which verification rejects. I removed it. I found no `.pyc` files in the active generation on the last check, but a hook could write one again.

**Open**
- **Embedding errors:** surreal-memory logs `Model forward pass failed: Metal error` for embeddings. The same error appears in the log since 2026-08-25, so the upgrade didn't cause it, but memory operations needing embeddings are still pausing. This machine is an Intel Mac with an AMD GPU, which is probably the reason. I haven't touched it.
- **Doctor warnings:** the KBD rollout gate is still in shadow mode, the KBD runtime authority check returns 404, no harness has a measured skill-discovery budget, and the plugin source tree has uncommitted changes.
- **Backups:** once you're happy, delete the SurrealDB backup directory and the `*.bak-20261005` files in `/usr/local/bin`.
- **Uncommitted changes:** nothing is committed or pushed. They are the exec pin change in `config/prometheus-exec-binary.json` and the wiki `index.md` and `log.md` edits.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 3f374a11-0ce4-40d3-a1dc-4964cece6769
- Captured: 2026-10-05T17:27:27.761449Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-system

## Changed Paths

- config/prometheus-exec-binary.json
- crates/prometheus-exec/.prometheus/
- history.txt
