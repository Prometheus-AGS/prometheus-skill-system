# Verification — change-rah-004-daemon-job-execution

Repository: `prometheus-skill-pack`
Depends on: `change-rah-001-compile-baseline-and-timestamps`, `change-rah-003-stage-contract-driver`

## Acceptance criteria

- `cargo test -p prometheus-research --test job_execution` passes: complete path, blocked path, and export refusal.
- The three existing integration targets `job_lifecycle`, `mcp_tools`, and `sse_stream` (all confirmed present in `substrate/prometheus-research/tests/` on 2026-09-04) still pass; task 1 records the directory listing in Evidence.
- After `launchctl kickstart -k` of `com.prometheus.research`, `/health` answers `ok` within 5 seconds.

## Verify commands

Every acceptance criterion above maps to a command here; run from the repository root, locally, after the coherent edit batch. A command that cannot run (for example, another Cargo build is active) is recorded BLOCKED with the reason, never skipped silently.

```verify
test -z "$(pgrep -x cargo)"
ls substrate/prometheus-research/tests/job_lifecycle.rs substrate/prometheus-research/tests/mcp_tools.rs substrate/prometheus-research/tests/sse_stream.rs
cd substrate/prometheus-research && cargo test -p prometheus-research --test job_execution
cd substrate/prometheus-research && cargo test -p prometheus-research --test job_lifecycle --test mcp_tools --test sse_stream
launchctl kickstart -k gui/$(id -u)/com.prometheus.research && sleep 5 && curl -s http://127.0.0.1:7891/health | jq -e '.status == "ok"'
```

## Evidence

Run 2026-09-06 on the uncommitted working tree (branch `feat/cpc-001-002-integration-contract`, base `cfbc262`); no hosted CI. Every cargo command waited for the machine-wide build slot (`pgrep -x cargo`): another session was running flint-gate checks and tests, and the gate below started only after the slot was free (waits of 200 s and 10 s recorded).

| Gate | Command | Result |
|---|---|---|
| Test directory listing (task 1) | `ls tests/` | `job_lifecycle.rs mcp_tools.rs sse_stream.rs` present; `job_execution.rs` and `fixtures/fake-claude.sh` added |
| Compile | `cargo check -p prometheus-research --tests` | clean, no warnings (after `..Default::default()` on four literals in the two existing test files) |
| job_execution | `cargo test -p prometheus-research --test job_execution` | `1 passed` (6.5 s). Phase 1: job through `POST /api/v1/jobs` → real binary as `--daemon-job` → fake `claude` → real driver with the fixture runner; final checkpoint `complete`, `harness: claude`, `exit_code 0`, stage 10, progress 100, package under the output root with a manifest; driver checkpoint carries the job id, `citation_style APA`, `depth deep`; fake harness log shows `-p`, `KBD_HOOKS_DISABLED=1`, job id, output root, the query, depth, output root, and `--job-id` in the prompt; no `kbd-hook.marker`; hook log has `pre-research`, `post-stage 01`..`10`, `on-contradiction 06`, `post-export`; the SSE socket carried `event: agent.status` and the completing event; the broadcast carried an `agent.status` for every stage 1..10 and no `agent.error`; `research_export` returned the package with a manifest-derived status and verdict. Phase 2: empty PATH → `blocked`, reason `no harness binary on PATH (looked for claude, then codex)`, exactly one `agent.error`, `/health` ok, no panic in daemon.log. Phase 3: manifest without `query` → `validate_package` and `research_export` both refuse naming `query`; restored manifest validates |
| First run of the gate | same | BLOCKED at stage 10: `citation_style: 'apa' is not one of ['APA', ...]`. Every entry point defaults to lowercase `apa` (and REST/MCP to `depth: moderate`) while the driver and schema use `APA` and `shallow|deep|exhaustive`. Fixed by canonicalising both in the daemon before the prompt; documented in headless-execution.md |
| Existing targets | `cargo test -p prometheus-research --test job_lifecycle --test mcp_tools --test sse_stream` | `4 passed`, `3 passed`, `2 passed` |
| Release + install | `cargo build --release`; `cp` to `~/.local/bin`; `codesign --force --sign -`; `codesign --verify` | built (2 m 50 s), installed 13.7 MB, `signature ok`; `--daemon-job` hidden from `--help`, refuses without a value |
| Service | `launchctl kickstart -k gui/$(id -u)/com.prometheus.research && sleep 5 && curl /health` | `{"status":"ok","version":"0.1.0","pid":78314}` |
| Ingest route on the installed service | `POST /api/v1/jobs/probe/events` | first install: `202` with a matching job id, `400` with a mismatched one; after round 1 the route requires the per-server token: `401` without it on the reinstalled service, and the integration test asserts `401` without the token and `400` with a mismatched job id |
| Round-1 fixes under test | `--test job_execution` after the fixes | a depth of `deep; touch /tmp/rah004-injected` ends `blocked` naming `depth`, no prompt written, harness not invoked, file absent; `pid` and `harness_pid` recorded by the daemon; `spawn_job` writes nothing after a successful spawn |
| Final run (round-2 fixes in) | `cargo check -p prometheus-research --tests`; `cargo test --test job_execution --test job_lifecycle --test mcp_tools --test sse_stream`; `cargo build --release`; install + sign; `kickstart -k`; `/health`; ingest probe | check clean; `1 passed`, `4 passed`, `3 passed`, `2 passed`; release built (49 s), `signature ok`; `{"status":"ok","pid":63034}`; `401` without the token |
| Strict validation | `npm run validate:strict skills/research/deep-research` | PASS |
| Constraints | C-02 secrets grep over the diff; C-05 grep over the fixture script | 0 hits; none |

QA: C-01 no generator input touched (the reference is a skill file; skills-index reconciliation is rah-011); scope amendment in spec.md for `job/daemon.rs`, `http_server/mod.rs`, the two existing test files, and `Cargo.lock`. Adversarial review: disposition in spec.md.

Verdict: PASS WITH NOTES (review findings disposed in spec.md; `KBD_HOOKS_DISABLED` is a contract with the harness session, not an enforcement in the KBD hook library, as documented).

Verify gate: `kbd-apply verify` reported FAIL at archive time. Every command of the verify block above passes when run one by one (recorded on 2026-09-06, all five PASS). The driver's `verify` runs the per-task commands in `tasks.json`, and task 2's command grepped `src/job/mod.rs` for the event and blocked handling, where the spec expected the handler to live; the handler was implemented as `src/job/daemon.rs` (scope amendment in spec.md), so the literal grep missed. The task-2 verify now names `src/job/daemon.rs`; with that, all four per-task commands pass. No code was changed to satisfy the grep.
