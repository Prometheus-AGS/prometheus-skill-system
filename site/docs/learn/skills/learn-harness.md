---
id: learn-harness
title: /learn-harness
sidebar_label: learn-harness
---

# /learn-harness

Map the selected harness and inspect the collaborators needed for a learning session:

```text
/learn-harness --harness codex --map-only
```

Supported selections are `claude-code`, `opencode`, `codex`, `kimi` and `zed`. Without a selection, the procedure uses local detection hints.

The surface helper returns `tier0_text`, `tier1_structured` or `tier2_mcp_app`. Process and environment hints are not proof that a tool is exposed, a browser rendered the UI or a service completed a request. Confirm the actual session capabilities before choosing an interaction path. Cursor and Zed detection select the file-pair path; chat remains the fallback.

Binary presence, endpoint reachability, MCP discovery and successful user interaction are separate evidence. Report each observed result and any missing prerequisite. See [Surface Tier Detection](/docs/learn-internals/surface-tier-detection).
