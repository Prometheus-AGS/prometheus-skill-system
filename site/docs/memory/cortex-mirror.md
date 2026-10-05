---
title: Cortex mirror
description: The optional Cortex mirror for learning writes, verified against the real Cortex 2.0.3 MCP server.
---

# Cortex mirror

When a Cortex MCP server is discoverable, `learning_write.py` mirrors each new lesson with one detached `cortex_remember` call. Cortex is optional: when it is absent the write is unchanged, exits 0 and prints nothing.

## Verified against Cortex 2.0.3

The mirror is verified against the real Cortex 2.0.3 server, not only a stub. `shared/scripts/tests/test-cortex-mirror.sh` runs a `real` case that:

1. locates the installed plugin under the real home directory, then switches `HOME` to a scratch directory and passes the server explicitly through `PROMETHEUS_CORTEX_MCP`;
2. points `CORTEX_DATA_DIR` at a scratch directory, so `~/.cortex` is never opened for writing;
3. writes a lesson through `learning_write.py` with the mirror enabled;
4. calls `cortex_recall` on the real server over stdio JSON-RPC for the lesson's `projectId` and asserts the lesson text and its `team/role:<team>/<role>` tag (stored by Cortex inside the memory content as `[Context: ...]`) come back;
5. asserts `~/.cortex/memory.db` (mtime and size) and `~/.claude/plugins/cache/cortex` are unchanged.

The case records the Cortex version it ran against and fails if it is not 2.0.3. Set `CORTEX_EXPECTED_VERSION` to verify another version deliberately.

The case needs the embedding model that ships inside the plugin (`node_modules/@xenova/transformers/.cache`). If it is missing the case exits 2 (BLOCKED); the test never downloads into the plugin cache. `CORTEX_REAL=skip` runs only the stub cases.

## What the real service exposed

Cortex 2.0.3 exits the instant its stdin closes, even with a request in flight (model load, embedding, save). Closing stdin right after writing the requests therefore lost the memory. The mirror now runs a small detached `--cortex-feed` worker that holds the server's stdin open until the `cortex_remember` reply arrives (or `PROMETHEUS_CORTEX_FEED_TIMEOUT`, default 180 seconds), so the write still never blocks the caller.

Cortex stores only `content`, `context` and `projectId`; a `global` lesson is saved without a `projectId`.

## Environment

| Variable | Effect |
|---|---|
| `PROMETHEUS_LEARNING_CORTEX=0` | Disable the mirror. |
| `PROMETHEUS_CORTEX_MCP` | Explicit server command, e.g. `node <plugin>/dist/mcp-server.js`. |
| `PROMETHEUS_CORTEX_FEED_TIMEOUT` | Seconds the worker waits for the reply. |
