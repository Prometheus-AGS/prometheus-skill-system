---
type: SessionRecord
id: karpathy-session-3149a14101ed89ed
title: Karpathy session 3149a14101ed
tags:
- karpathy
- session-learning
sources:
- session:fa9e1cf3-c163-4f05-95c8-07b8c6aaa3f5
timestamp: 2026-08-04T11:35:31.611975+00:00
created_at: 2026-08-04T11:35:31.611975+00:00
updated_at: 2026-08-04T11:35:31.611975+00:00
revision: 0
---

## Delta

Everything is live and verified. Summary:

**Config wired (both models genuinely respond as themselves, confirmed via self-identification prompts):**
- `~/.config/liter-llm/liter-llm-proxy.toml` — added `k3` (Kimi K3, via the kimi.com/code subscription endpoint, `KIMI_CODING_KEY`) and `MiniMax-M3` (via the Token Plan subscription endpoint, `MINIMAX_KEY`)
- `~/.prometheus/kbd/models.toml` — `critic = "k3"`, `judge = "MiniMax-M3"`, gateway candidates reordered so liter-llm's own port (`:4000`) is checked before the unrelated `openai-proxy` (`:8181`)
- `~/.prometheus/kbd/secrets.env` — added `KIMI_CODING_KEY` and `MINIMAX_KEY`
- liter-llm's gateway is now a persistent launchd service (`ai.prometheus.liter-llm-api`, auto-starts at login), via a new wrapper script that sources secrets without putting them in the plist

**Two real bugs found and fixed along the way** (both would have silently produced fake "PASS" results otherwise):
1. `configure-models.sh`'s existing `kimi-coding`/`minimax` presets pointed at endpoints that don't actually draw on the subscription plans — fixed with the verified working endpoints, and added a `minimax-coding` preset.
2. liter-llm forwards your literal requested model name upstream unchanged (never substitutes `provider_model`), and any `base_url`-overridden entry always speaks OpenAI-compatible wire format regardless of prefix. Combined with `openai-proxy` silently answering *any* model name as ChatGPT, an early version of this config would have had adversarial review silently critiqued/judged by ChatGPT while believing it was Kimi/MiniMax. Documented as contract #4 in both `CLAUDE.md` and `model-configuration.md`.

Adversarial review's critic and judge now run on two genuinely different model families from the producer (Claude), which is the whole point of the gate.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: fa9e1cf3-c163-4f05-95c8-07b8c6aaa3f5
- Captured: 2026-08-04T11:35:30.684078Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-system

## Changed Paths

- CLAUDE.md
- skills/process/adversarial-review/references/model-configuration.md
- skills/process/liter-llm-bridge/scripts/configure-models.sh
- shared/launchagents/ai.prometheus.liter-llm-api.plist
- shared/scripts/liter-llm-api-launch.sh
