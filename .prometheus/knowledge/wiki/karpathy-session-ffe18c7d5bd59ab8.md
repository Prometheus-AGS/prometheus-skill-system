---
type: SessionRecord
id: karpathy-session-ffe18c7d5bd59ab8
title: Karpathy session ffe18c7d5bd5
tags:
- karpathy
- session-learning
sources:
- session:751c647e-97c4-4d8b-bb71-c2a55f99d776
timestamp: 2026-08-05T08:41:09.896129+00:00
created_at: 2026-08-05T08:41:09.896129+00:00
updated_at: 2026-08-05T08:41:09.896129+00:00
revision: 0
---

## Delta

Done. Both subscription models are wired in as adversarial judges and verified end to end.

## Configured

| Role | Model | Endpoint | Key |
|---|---|---|---|
| **judge** | `k3` (K3, 1M ctx) | `https://api.kimi.com/coding/v1` | `${KIMI_CODING_KEY}` |
| **critic** | `MiniMax-M3` | `https://api.minimax.io/v1` | `${MINIMAX_KEY}` |

Both OpenAI-compatible, both reached directly by liter-llm — no adapter needed.

## Verified, not assumed

Round trips through the gateway on `:4000` self-identify correctly: `k3` says **"Moonshot AI (月之暗面)"**, `MiniMax-M3` responds as MiniMax. That's the check that matters — CLAUDE.md contract #4 warns a wrong backend still returns HTTP 200, so I confirmed identity rather than trusting a 200.

`scripts/check-model-config.sh` → **✅ no findings, exit 0**.

Judge and critic are now different vendors from each other *and* from the producer (Claude). Previously both routed to the same OpenAI family via `:8181` — that was a self-grade.

## Four defects fixed along the way

1. **Env var mismatch** — existing Kimi entries referenced `KIMI_CODING_API_KEY`, your `.env` has `KIMI_CODING_KEY`. liter-llm expands unset vars to `""`, so those entries were silently keyless.
2. **Alias names violated contract #4** — entries were named `kimi-k3`, but a `base_url` override forwards the literal `name` upstream. Moonshot doesn't know `kimi-k3`. Renamed to real upstream IDs.
3. **My own error** — I first wrote roles as `[[aliases]]` with `target`, which liter-llm rejects (`unknown field target`). The service failed to start; I caught it in the logs and moved the bindings to `models.toml` by name.
4. **Gateway probe order** — `models.toml` probed `:8181` first, where neither model exists. Reordered so `:4000` wins.

## Also closed F5

The `liter-llm-api` plist is now rendered (0 placeholders), `plutil`-valid, loaded, and **wired into `install-mcp-services.sh`** so it survives a reinstall. Pushed as `ab9a0e6`.

Secrets live only in `~/.prometheus/kbd/secrets.env` (0600), referenced as `${VAR}`. Nothing sensitive entered any config file or the repo.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 751c647e-97c4-4d8b-bb71-c2a55f99d776
- Captured: 2026-08-05T08:41:07.512966Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
