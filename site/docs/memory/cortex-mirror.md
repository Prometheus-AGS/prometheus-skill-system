---
title: Optional Cortex mirror
description: Bounded optional mirroring after durable primary lesson publication.
---

# Optional Cortex mirror

`learning_write.py` can mirror a newly queued lesson to a separately installed Cortex MCP server. The primary queue/log is durable before mirroring. Cortex is optional and is not in the lookup chain.

Discovery uses explicit `PROMETHEUS_CORTEX_MCP`, otherwise the supported installed plugin path; `PROMETHEUS_LEARNING_CORTEX=0` disables it. No server is a normal outcome. Explicit command errors appear in the writer summary without discarding the lesson.

## Bounded admission and outcomes

`PROMETHEUS_CORTEX_MAX_FEEDERS` defaults to 4; 0 disables mirrors. Nonblocking process-shared admission skips a busy/full mirror rather than queuing unlimited processes. POSIX slot locks remain held through feeder/server lifetime, including a parent crash; unsupported lock inheritance fails closed for this optional path. Slot files are permanent while workers may be live.

The feeder holds stdin until the `cortex_remember` reply, process exit or finite deadline (`PROMETHEUS_CORTEX_FEED_TIMEOUT`, default 180 seconds). This prevents closing stdin while Cortex is still loading/saving. Capacity and lifetime are separate bounds.

Writer JSON reports `cortex_mirror` status/reason. `accepted` means the feeder started and received input, not remote storage completion. Disabled, absent, saturation and failure do not change the successful primary write. Cortex stores content/context/project identity according to its own contract; retrieval scopes are not an authorization guarantee.

The [memory tiers guide](/docs/guide/memory-tiers#optional-cortex-mirror) is canonical for slot locations, diagnostics, owner scope and historical server findings. Earlier Cortex 2.0.3 evidence does not certify this candidate or any installed plugin. Final integration uses isolated homes/data and a real configured server; it never writes into the plugin cache or live Cortex store.
