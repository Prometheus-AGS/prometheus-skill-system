---
id: learn-harness
title: /learn-harness
sidebar_label: learn-harness
---

# /learn-harness

Harness detection and capability map. Identifies which UI tier the current
harness supports and what capabilities are available.

## What it detects

| Capability | How detected |
|-----------|--------------|
| Surface tier (0/1/2) | MCP tool availability check |
| AskUserQuestion | Claude Code-specific tool presence |
| surface-bridge | HTTP health check at 127.0.0.1:7890 |
| sovereign-sync | HTTP health check at 127.0.0.1:7892 |
| surreal-memory | REST health check |
| sycophancy-correction | Binary presence check |

## Surface tiers

| Tier | Description |
|------|-------------|
| 0 | Plain text / markdown (always works) |
| 1 | AskUserQuestion (Claude Code) or file-pair UI protocol |
| 2 | Tier 2 MCP App iframe via surface-bridge |

## Usage

```
/learn-harness
```

Output example:

```
Harness: Claude Code
Surface tier: 1 (AskUserQuestion available)
sovereign-sync: running on :7892
surreal-memory: running
sycophancy-correction: binary found
```
