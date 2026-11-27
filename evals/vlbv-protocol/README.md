# VLBV protocol evals

Per-harness evidence that an agent follows the VLBV method (change request CR-31,
phase `vlbv-r4-portability`). The cases are the 22 acceptance tests of
`skills/testing/vlbv/references/requirements.md` plus the §18 fresh-session check
of `supersession.template.md`.

These scripts run **only at a phase boundary** (B4/B5,
`skills/process/kbd-process-orchestrator/references/boundaries.md`), never as a
development loop. Each harness run is a real model session and costs money.

## Cases

`cases/vlbv-protocol.json` holds 23 cases.

- **17 agent cases** run in every installed harness. Each sample builds a fresh
  scratch project (a small KBD-managed Node package with the pack's VLBV policy
  block and an open phase without B4 authorization). Marker scripts record when
  tests, builds, generated code, publishing or sending actually ran, so
  deterministic checks read file effects rather than the agent's self-report.
- **6 machinery cases** (AT 05, 06, 09, 10, 15, 22) test pack scripts, not agent
  behaviour. `machinery.mjs` runs them once against the real `prometheus` runtime
  and `kbd-phase-close-eval.sh`. Every harness report carries their verdicts,
  marked harness-independent.

Cases with a `capabilityFields` entry, and AT 04 (early execution, rare-critical),
need 3 of 3 passing samples; the rest need 2 of 3. A sample passes only when its
deterministic check passes **and** the cross-family grader says PASS.

## Memory recall corpus

`cases/kbd-memory-recall.json` holds four agent cases for project memory during
Execute (issue #256, `references/execute-memory.md` in the orchestrator): a
recalled lesson changes the action at task start; `pk` is consulted on a
material blocker; with every source unavailable the agent proceeds; and a
conflicting lesson cannot authorize the boundary run or widen scope (critical,
3 of 3).

It is a separate corpus because a report of the default corpus must carry
exactly that corpus's case ids. Run it with `--corpus kbd-memory-recall`;
reports go to `evals/reports/kbd-memory-recall/` and transcripts to
`evals/runs/kbd-memory-recall/`. These runs are opt-in, billed and not a release
gate.

A case with a `memory` entry also gets the Execute skills, the recall script and
the recall library, and a recording stand-in for `pk` that lives outside the
project. Every eval session has surreal-memory pointed at an address that
refuses connections, so no session can read or write the user's store. That
also means these cases never exercise the store itself: store-first ordering is
covered without a model by `shared/scripts/tests/test-kbd-memory-loop.sh`, and
the lookup mechanics by `shared/scripts/tests/test-kbd-task-recall.sh`.

## Scripts

| Script | Purpose |
|---|---|
| `machinery.mjs [--negative --at ATnn]` | Machinery cases; `--negative` removes one AT's trigger and requires the real script to accept |
| `run.mjs --harness <id[,id]\|all> --samples 3` | Agent cases per harness; writes `evals/reports/<harness>.{json,md}` and validates them |
| `run.mjs --corpus kbd-memory-recall --harness <id> --samples 3` | The memory recall corpus; writes `evals/reports/kbd-memory-recall/<harness>.{json,md}` |
| `run.mjs --stub exit0` | Negative control: a no-output harness must give BLOCKED, never PASS |
| `run.mjs --harness claude-code --samples 1 --keep-scratch --expect-leftover` | Negative control: the leftover-auth check must notice kept auth material (one case only; kept scratch is removed afterwards) |
| `validate-report.mjs --harness <id> [--negative]` | Re-derives every verdict from retained artifacts; `--negative` plants a PASS with a missing transcript and a PASS contradicted by its effects |
| `apply-capabilities.mjs --reports evals/reports [--negative]` | Promotes `shared/harnesses/capabilities.json` fields from PASS cases only; `--negative` checks the validator rejects three bad promotions |
| `scan-fixture.mjs --case mixed\|all-after` | Drives the closure packet's pre-authorization artifact scan (CR-33) |
| `scan-secrets.mjs <path>...` | Refuses credential-shaped text in reports and transcripts (AT 22) |

A negative-control command exits 0 when the expected rejection was observed.

## Isolation

Each sample runs in a scratch root under the system temp directory with its own
HOME and harness config home. Only the minimum auth material is copied in
(owner decision D-R4-03). Configs are reduced to provider and model settings, with
no MCP servers, hooks or plugins. The scratch root is deleted after each sample.
Real harness files and the harness-related login-keychain items (attributes only)
are hashed before and after each harness; any change stops further harnesses.
Claude Code keeps its interactive login in the keychain under a name tied to the
default config directory, so nothing is copied for it and a relocated config
directory is not logged in. Isolated runs authenticate with
`CLAUDE_CODE_OAUTH_TOKEN` (from `claude setup-token`), read from the environment
or `~/.prometheus/kbd/secrets.env` and passed only through the child environment.
Without it the harness is reported as not run.

Transcripts and effects are kept under `evals/runs/` as local evidence. That
directory is gitignored; the reports in `evals/reports/` are not.

## Grading

The grader goes through the liter-llm gateway (as `kbd_complete` does). It is
MiniMax-M3, or glm-5.3 when the harness under test is MiniMax. A reply from any
other model (for example a gateway that ignores the requested model) is BLOCKED.
