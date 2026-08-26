---
type: Reference
id: doctor-learning-worker-stall-expired-credential
title: Doctor learning-worker stall traced to an expired auth credential
tags:
- prometheus-doctor
- kbd-doctor
- learning-worker
- credential-expiry
- diagnosis
links:
- karpathy-gate-readiness
sources:
- stdin
timestamp: 2026-08-26T05:59:58.000000+00:00
created_at: 2026-08-26T05:59:58.000000+00:00
updated_at: 2026-08-26T05:59:58.000000+00:00
revision: 0
---

## Delta

`prometheus doctor` 1.7.0 was run in read-only diagnosis mode (no `--fix`, no `--refresh`).

The run was first issued from an unrelated project directory
(`~/Projects/references/liter-llm`, a Rust library) and reported **6 failing
required checks**. Re-run from the substrate root
(`~/Projects/prometheus/prometheus-skill-pack`), the same binary reported
**1 failing check, 5 warnings, 12 passing**.

| Metric | From `liter-llm` | From substrate root |
| --- | --- | --- |
| fail | 6 | 1 |
| warn | 2 | 5 |
| pass | 5 | 12 |
| skip | 7 | 2 |

Five of the six failures were artifacts of the working directory:
`skills.directory`, `hooks.lifecycle`, `hooks.harness-adapters`, and
`learning.snapshots` resolve substrate-relative paths (`skills/`,
`shared/harnesses/`, `scripts/install-plugin-generation.js`,
`.prometheus/knowledge/.prompt-snapshots/`) that do not exist in a library
checkout. No repair was warranted for any of them.

An intermediate hypothesis — that the substrate root was
`~/Projects/prometheus` — was tested and **disproved**: that directory
reproduced all 6 failures identically. The true root is one level deeper at
`prometheus-skill-pack`, located by finding `install-plugin-generation.js`.

## Root Cause

The single genuine failure is `learning.worker`, and it is **not** a missing
or unloaded worker. Doctor reported the worker installed and the service
loaded, with 1057 jobs completed and a run as recent as the same session.

The queue at `~/.prometheus/learning-queue` is **stalled, not idle**:

- jobs 23 pending / 0 processing / 1057 completed / 0 rejected
- memory 41 pending / 3 submitting / 16 accepted / 1017 completed
- legacy retry/dead: 0/0

`status.json` carries the proximate cause:

```json
"lastError": "operation submission returned 503 Service Unavailable: {\"error\":\"Connection reset\"}"
```

The worker is alive and draining into a backend refusing writes.
`ai.prometheus.surreal-memory-native` shows `last exit code = 1` under
keepalive.

The upstream cause surfaced via the mandated `pk focus` step:

```
401 Unauthorized: "Provided authentication token is expired." (code: token_expired)
```

Meanwhile `http://127.0.0.1:23001/health` returns
`{"service":"surreal-memory-server","status":"ok","version":"1.7.0"}`.

**The service is healthy; an expired auth credential is producing the 503s and
the resulting backlog.** The same expiry explains the `review.judge-gateway`
warning (HTTP 401 on `http://localhost:4000/v1`).

## Corrective Actions

### Taken

None. Read-only diagnosis only; no repair mode was requested and none was run.
No files, services, or credentials were modified.

### Deferred — manual only

Doctor offers `services.install-mcp-services`
(`bash scripts/install-mcp-services.sh --restart`) for `learning.worker`,
flagged `safe: true`.

**This action should not be run for this failure.** It reinstalls and restarts
a worker that is already installed and running correctly, and cannot mint or
refresh a credential. It would perturb the symptom without addressing the
expired token, and the queue would re-stall on the next submission.

Credential rotation is manual-only under the skill's safety policy
(`references/repair-policy.md`). Required sequence:

1. Rotate the expired token behind the memory/gateway auth path.
2. Re-run `prometheus doctor --check learning.worker` from
   `prometheus-skill-pack` and confirm the pending counts (23 jobs /
   41 memories) drain.
3. Only if the failure persists, consider the supervised service restart.

### Gate verification

`~/.prometheus/repair/karpathy-ready.json` was verified before any repair
consideration: `ready: true`, generated 2026-07-17. Its recorded
`surreal_memory` endpoint probe still passes, consistent with credential
expiry rather than substrate regression.

### Backups / rollback

None written — no mutating action was taken, so no rollback artifact exists.
No `install-refresh-manifest.json` was produced (that is written only by a
successful `--refresh --yes` run).

## Non-blocking warnings

Expected outside a live control plane; no action taken:

- `control.kbd-runtime` — KBD daemon diagnostics unreachable on `127.0.0.1:7892`
- `state.kbd-orchestrator` — revision 5, projections mismatched
- `control.kbd-rollout` — shadow evidence collection not started
- `skills.discovery-budget` — 146 skills / 46134 discovery characters, 0/4 harnesses measured

## Operational note

`prometheus doctor` resolves several checks relative to the current working
directory and does not warn when run outside a substrate checkout. Running it
from an unrelated repository produces five false failures whose suggested
repairs target a healthy system. **Always run doctor from the substrate root.**
