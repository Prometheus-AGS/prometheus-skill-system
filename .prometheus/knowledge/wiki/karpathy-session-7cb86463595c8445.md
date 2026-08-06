---
type: SessionRecord
id: karpathy-session-7cb86463595c8445
title: Karpathy session 7cb86463595c
tags:
- karpathy
- session-learning
sources:
- session:cebcfbb5-668f-49de-8c67-ea6fae9d262c
timestamp: 2026-08-06T11:13:23.310594+00:00
created_at: 2026-08-06T11:13:23.310594+00:00
updated_at: 2026-08-06T11:13:23.310594+00:00
revision: 0
---

## Delta

Done. The judge models are accessible, and I found and fixed a real defect on the way.

## What was actually wrong

`~/.bash_profile` was already sourcing `secrets.env` correctly. The bug was inside `secrets.env`:

```bash
export LITER_LLM_MASTER_KEY=...   # exported ✅
export KIMI_CODING_API_KEY=...    # exported ✅
KIMI_CODING_KEY=...               # ❌ no export
MINIMAX_KEY=...                   # ❌ no export
```

Sourcing a file without `set -a` makes unprefixed assignments **plain shell variables** — invisible to any child process. So the judge (`k3`) and critic (`MiniMax-M3`) keys existed, were correct, and never reached `liter-llm`, `curl`, or the audit script.

## Fixes

1. **`secrets.env`** — added `export` to `KIMI_CODING_KEY` and `MINIMAX_KEY`. Still `0600`.
2. **`.bash_profile`** — wrapped the sourcing in `set -a` / `set +a`, so a future unexported entry can't silently reintroduce this. Also pinned `LITER_LLM_BASE_URL` to the `:4000` gateway (the one that registers the cross-vendor judges; `:8181` would answer as its own backend instead of failing loudly).

No key values were written to `.bash_profile` — it's `0644`, and per your choice the secrets stay in the single `0600` file. Timestamped backups of both files were taken first.

## Verified

| Check | Result |
|---|---|
| Fresh login shell (`bash -lc`) | All 6 vars exported |
| `scripts/check-model-config.sh` | **✅ no findings** (was: 2 UNSET) |
| `k3` through the gateway | `content: 'JUDGE-OK'`, `finish: stop` |
| `MiniMax-M3` through the gateway | responded, `finish: stop` |
| Collision check | producer / judge / critic **all three distinct** |

Both judges answered using only the environment — no keys passed by hand.

## One correction to my earlier diagnosis

Mid-investigation I reported the gateway was returning "Invalid API key" and inferred the config was broken. That was **my testing error** — I'd extracted the literal string `${LITER_LLM_MASTER_KEY}` from the TOML rather than the expanded value, so I was sending a bogus bearer token. The gateway was fine. The stale parse errors I found in the log were from 03:38, before a fix; the process started at 19:49 and had been healthy since.

The genuine defect was the missing `export`, and it's fixed.

Worth noting for the article: this is another instance of the pattern — the keys were present and correct, the sourcing line was right, and the whole thing still failed silently because of two missing words. The audit script *did* catch it; nobody had run it in a login shell.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: cebcfbb5-668f-49de-8c67-ea6fae9d262c
- Captured: 2026-08-06T11:13:19.124739Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- .kbd-orchestrator/phases/mobile-skill-portability/mobile-classification.json
- .prometheus/knowledge/wiki/index.md
- .prometheus/knowledge/wiki/kimi-desktop-extensibility-executor-completion-record.md
- .prometheus/knowledge/wiki/log.md
- .prometheus/knowledge/wiki/karpathy-session-01bf5edf5a4599ef.md
- .prometheus/knowledge/wiki/karpathy-session-06fb138df2cedd2f.md
- .prometheus/knowledge/wiki/karpathy-session-273f3645230ec04d.md
- .prometheus/knowledge/wiki/karpathy-session-75e1eb1f877c92a1.md
- .prometheus/knowledge/wiki/karpathy-session-807d6bc988e75156.md
- .prometheus/knowledge/wiki/karpathy-session-95cb4c69e3a13c3a.md
- .prometheus/knowledge/wiki/karpathy-session-b981029e4691b82f.md
- .prometheus/knowledge/wiki/karpathy-session-d36396632cc9ba19.md
- .prometheus/knowledge/wiki/karpathy-session-ed04ff580e446494.md
- .prometheus/knowledge/wiki/kimi-desktop-extensibility-executor-completion-unknown-change.md
- .prometheus/knowledge/wiki/kimi-desktop-extensibility-executor-session-completion.md
- docs/articles/OUTLINE-loops-that-lie.md
