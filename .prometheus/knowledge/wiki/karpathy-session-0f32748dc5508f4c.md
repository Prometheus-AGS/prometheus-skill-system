---
type: SessionRecord
id: karpathy-session-0f32748dc5508f4c
title: Karpathy session 0f32748dc550
tags:
- karpathy
- session-learning
sources:
- session:751c647e-97c4-4d8b-bb71-c2a55f99d776
timestamp: 2026-08-04T16:13:19.020746+00:00
created_at: 2026-08-04T16:13:19.020746+00:00
updated_at: 2026-08-04T16:13:19.020746+00:00
revision: 0
---

## Delta

## Assessment: what needs rebuilding after the latest pull

**Short answer: nothing needs a rebuild. Binaries are already current. There is one unwired new service, and two dirty submodules that carry uncommitted local fixes.**

### The pull

`8464db7` — *Prometheus-exec code execution engine scaffolding + liter-llm adversarial-review model fix* (2026-08-04 06:38). ~10.7k lines added.

### Binaries: current, no rebuild required

All 18 pack binaries in `~/.local/bin` are dated **2026-08-04 07:37–07:41** — after the HEAD commit at 06:38. Versions check out (`pk 1.7.0`, `liter-llm 1.12.0`, `cowork 0.2.0`).

The new commit adds **no new installable binary**. `install-binaries.sh:105` builds substrate binaries from a fixed list (`learner-model surface-bridge sovereign-sync`) — the new `substrate/exec-*` crates aren't in it, and `crates/prometheus-exec` isn't referenced anywhere in the installer. Their `target/` dirs are all absent.

That absence is correct for now, not a gap. This commit is scaffolding: `exec-service` and `exec-tier-p` are 10-line `lib.rs` stubs, and `prometheus-exec` is a CLI with tests but no install path. Per the phase progress file, only `change-exec-001` (contracts + verification) is landed; 002/003/004 are unstarted. **Nothing to install until the tiers are real.**

### The one actual gap: `ai.prometheus.liter-llm-api` is not wired

The commit adds `shared/launchagents/ai.prometheus.liter-llm-api.plist` + `shared/scripts/liter-llm-api-launch.sh`, but:

- it is **not installed** to `~/Library/LaunchAgents/`
- **no install script references it** — the only hit for `liter-llm-api` outside the plist is a comment inside the launch script itself
- it has unsubstituted `__PROMETHEUS_ROOT__` placeholders, so it can't be copied verbatim

**Impact today: none.** Port 8181 is served by `openai-proxy` (PID 894, `dev.prometheusags.openai-proxy` loaded, `/v1/models` returns HTTP 200), and `scripts/check-model-config.sh` exits 0 with no findings. The adversarial-review routing works. The new plist looks like an intended *alternative* gateway that was committed without an installer hook.

### Two dirty submodules — local fixes, not drift

Both are **at** their recorded commits (no rewind), with uncommitted working-tree edits:

| Submodule | Local edits |
|---|---|
| `tools/surreal-memory-server` | `Cargo.toml`: adds `surreal-memory/cuda` + `surreal-memory/metal` feature propagation. `src/operations.rs`: drops `working` heartbeat events from the durable journal (the SurrealDB flooding fix) |
| `tools/prometheus-knowledge` | `pk-learning-worker/src/main.rs` modified |

These match the "local fixes preserved" from the 1.7.0 update and the durable-heartbeat-flooding fix in memory. The installed `surreal-memory-server` (07:40:47) is newer than its source (05:40:08), so **the fixes are already compiled into the running binary**.

The real risk here isn't staleness — it's that these edits exist only in your working tree. A `git submodule update` or a fresh clone silently loses them.

### Recommendation

| Item | Action |
|---|---|
| Binaries | None — current |
| `substrate/exec-*`, `crates/prometheus-exec` | None — scaffolding, correctly not installed |
| `liter-llm-api` plist | Decide: wire into `install-mcp-services.sh` with placeholder substitution, or leave dormant since `openai-proxy` already serves 8181 |
| Dirty submodules | Push those local fixes upstream before they get clobbered |

I've only assessed — nothing changed. Want me to trace the `liter-llm-api` plist's intent against the change-exec docs, or check whether those submodule edits exist on any remote branch?

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 751c647e-97c4-4d8b-bb71-c2a55f99d776
- Captured: 2026-08-04T16:13:15.457576Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- tools/prometheus-knowledge
- tools/surreal-memory-server
