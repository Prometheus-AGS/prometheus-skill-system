---
title: Apple Foundation Models
description: Use the Mac's built-in on-device model as a local, private endpoint that the pack sets up during installation.
---

# Apple Foundation Models

Apple Silicon Macs running macOS 26 or newer with Apple Intelligence enabled ship an
on-device language model (about 3B parameters). The pack exposes it as a local
OpenAI-compatible endpoint, so pack scripts can send small private or offline work to
it without any cloud key.

## What installation does

`scripts/update-skill-pack.sh` and `scripts/install-mcp-services.sh` run
`shared/scripts/apple-foundation-models.sh --install --quiet`. On an eligible Mac it:

1. Installs [`apfel`](https://github.com/Arthur-Ficial/apfel) from Homebrew core if
   it is absent. apfel wraps Apple's `LanguageModelSession` as a CLI and an
   OpenAI-compatible server.
2. Installs and starts the LaunchAgent `ai.prometheus.apple-fm`
   (`apfel --serve --host 127.0.0.1 --port 11437`). The port is not apfel's default
   11434, which Ollama uses.
3. Registers `apple-foundationmodel` with liter-llm
   (`configure-models.sh add-provider apple-fm`) and restarts the gateway.
4. Adds `[roles] local = "apple-foundationmodel"` to `~/.prometheus/kbd/models.toml`
   when no `local` role exists.

On any other machine it prints nothing and exits 0. It is an optional capability:
absence is the normal case and never fails an install.

## Use it

```bash
bash shared/scripts/apple-foundation-models.sh --check     # JSON health
curl -s http://127.0.0.1:11437/v1/models                   # lists apple-foundationmodel
```

From pack shell scripts, resolve the role and send a completion through the gateway:

```bash
. shared/scripts/lib/kbd-model-resolve.sh
kbd_complete "$(kbd_resolve_role local)" "Answer in one line." "Summarize this changelog entry: ..." 256
```

The context window holds input and output together: **4,096 tokens** on macOS 26
(8,192 on macOS 27 with an M3 or newer). Use it for summarization, classification
and short extraction. It is never eligible as the adversarial-review judge or critic;
the resolver refuses that and falls back to the configured cross-vendor model.

## Controls

| Variable | Effect |
|---|---|
| `PROMETHEUS_APPLE_FM=0` | Skip it entirely |
| `PROMETHEUS_APPLE_FM_PORT` | Serve on another port (default 11437) |
| `PROMETHEUS_APPLE_FM_FORCE_INELIGIBLE=1` | Simulate an ineligible machine |

Apple Intelligence is a system setting (System Settings → Apple Intelligence & Siri).
The helper reports when it is off and never changes it. Remove the service with
`--uninstall` (the apfel formula is kept).
