# Prometheus 1.7.0 local release evidence

This directory records local evidence for the 1.7.0 remediation release. GitHub
workflow output is not certification evidence.

## Scope and safety boundary

- Certified base: `dc22ae5092f6e852e9eea8116dc6bcab9186940c`.
- Work was performed in an isolated Git worktree; the dirty `main` worktree was
  not modified.
- KBD did not orchestrate the work and no KBD skill, wrapper, installed service,
  live state, or KBD-backed memory was invoked.
- The installed Sovereign Sync service was not stopped, restarted, or
  rewritten. One read-only `cowork toolchain status` run unexpectedly probed
  its health endpoint because Cowork exposes no service-exclusion flag. That
  boundary violation is recorded below and the command was not rerun.
- Disposable product fixtures are permitted only with isolated directories,
  sockets, ports, identities, and data.
- Applicable doctors use these exclusions:
  `control.kbd-runtime`, `state.kbd-orchestrator`, `control.kbd-rollout`, and
  `service:sovereign-sync`.

## Implemented commit chain before documentation certification

| Commit | Local result |
| --- | --- |
| `7dfdd3b` | RED voter/config removal fixtures established |
| `ef12667` | voter/quorum facade removed; single-writer behavior retained |
| `f9bf392` | deterministic learner-folding fixtures established |
| `79695c2` | Loro immutable evidence and non-destructive migration implemented |
| `6cad86d` | signed checkpoints and hash-linked journal archives implemented |
| `82e9998` | private Unix transport and durable P2P identity implemented |
| `0c41992` | signed sync pushes, receipts, REST/MCP service, and OpenAPI implemented |
| `4e1f3f6` | Git-state protected-test certification implemented |
| `2051042` | cumulative local review receipts and waiver policy implemented |
| `2d144be` | strict installer and explicit development modes implemented |
| `48a4565` | signed plugin generations and shared deterministic index implemented |

## Documentation gate

Final command:

```bash
CARGO_TARGET_DIR=<internal-ssd-target> RUSTUP_TOOLCHAIN=stable npm run docs:check
```

Result: **exit 0**.

The single entry point proved:

- `docs:sync` was clean across two consecutive runs and unchanged source input
  selection returned `relevant=false`;
- hosted workflow policy and its negative fixtures passed;
- the public documentation sanitizer passed;
- Memory OpenAPI 3.1 validation and deterministic examples passed;
- Sovereign Sync OpenAPI 3.1 validation passed and the checked-in document
  exactly matched fresh Rust route/type output;
- release metadata, sidebars, authored semantic drift rules, ADR presence, and
  OpenAPI parity passed;
- the catalog deterministically generated 147 skills across 17 categories; and
- Docusaurus client and server production bundles compiled successfully with
  broken links treated as errors.

The first attempt stopped because the isolated worktree had no site dependency
tree. `npm --prefix site ci` installed exactly `site/package-lock.json`; rerunning
the same gate passed. This is a resolved environment prerequisite, not a waived
warning.

## Targeted API contract proof

```bash
CARGO_TARGET_DIR=<internal-ssd-target> RUSTUP_TOOLCHAIN=stable \
  cargo test --quiet --manifest-path substrate/sovereign-sync/Cargo.toml \
  generated_openapi_tracks_v2_route_constants_and_rust_schemas -- --exact
```

Result: **1 passed, 0 failed** in the Sovereign integration-test target. The
test binds v2 route constants, the 1.7.0 contract, generated Rust schemas,
request/receipt examples, ordered receipt events, all documented POST statuses,
and response-loss scenarios.

## Rust and deterministic product tests

All commands used the stable toolchain and an internal-SSD Cargo target. Format,
check, warnings-denied Clippy, unit, integration, and property tests passed for
the affected workspaces:

| Workspace | Test result |
| --- | ---: |
| `kbd-runtime` | 61 passed; 6 operator-only tests ignored by the ordinary suite |
| `learner-model` | 27 passed |
| `skill-index` | 2 passed |
| `skill-ffi` | 11 passed |
| `kbd-mobile` | 1 passed |
| `sovereign-sync` | 44 library, 5 binary, and 22 integration tests passed |
| Prometheus CLI workspace | 8, 9, and 1 tests passed across affected crates |

The allowed root aggregate ran with `PROMETHEUS_SKIP_KBD=1` and passed all 12
deterministic suites, including 145 strict skill payloads, cross-tool parity,
PK false-green fixtures, canonical Karpathy hooks, signed plugin generations,
and learning queues. The KBD state validator was not used as certification
evidence. An early default aggregate invocation entered that checked-in
validator and failed on two pre-existing ledgers; it contacted no service and
performed no repair. The run was stopped and repeated with the required KBD
exclusion.

## Disposable runtime evidence

Two Sovereign peers ran with distinct temporary data roots, Unix sockets,
ports, identity files, endpoint IDs, and signing identities. The complete
pairing tickets and group secret were held only in process variables and were
never printed or archived.

- socket creation was atomic and mode `0600`;
- bind completed in 23–27 ms and authority readiness in 27–119 ms;
- the endpoint ID remained identical across restart;
- reciprocal pairing produced matching group-secret fingerprints and distinct
  allow-list bindings;
- both peers reported the other endpoint with transport ready;
- signed exact replay, hash conflict, ordered SSE resume, response-loss
  reconciliation, signer/group/replay rejection, and terminal receipts passed
  the integration matrix;
- 100 warm `/health` samples had p50 0.233 ms, p95 0.287 ms, p99 0.297 ms,
  maximum 0.328 ms, and zero timeouts; and
- `/ready` returned 200 in 0.735 ms.

A separate disposable KBD product fixture—not the installed KBD service—proved
journal-first SIGKILL recovery. Startup reconciled the fsynced event into Loro,
preserved two events, exported a hash-addressed audit receipt without mutating
the Git worktree, archived a deliberately torn 57-byte tail with a SHA-256
sidecar, and reopened with the same two valid events.

## Local integrity, installer, and plugin gates

- Protected-test fixtures proved Bash and Python remained unrestricted while
  content, rename, deletion, and mode changes failed final certification unless
  covered by an SSH-signed canonical approval manifest.
- Cumulative local-review fixtures proved small and docs-only diffs cannot skip
  review; `pending_review` fails certification and signed waivers verify.
- Strict, `--best-effort`, `--skills-only`, and false-green installer fixtures
  passed.
- Clean-install parity verified 145 payloads across 14 targets.
- Plugin signature, tampering, trust-store rejection, collision rejection,
  bundle pinning, transactional activation, rollback, uninstall, shared-index
  parity, and 14 signed target receipts passed.
- Candidate-only Gitleaks scanning covered 692 KiB and found no secrets.
- Root and site `npm audit --audit-level=high` reported zero vulnerabilities.

The pre-publication installed candidate generation was content-addressed and
signature-valid:
`703faac729fef755c81cd809e05ca6ac250935c80142e9e8f99b706039c3f6a6`.
Its signed manifest records source version `1.7.0`, bundle
`e39e2f6b7cf1515c7f4db423f6a336a428d28430394b56c7f48b3084ccc02be0`,
145 indexed skills, 14 target payloads, and nine external source commit pins;
all 14 signed target receipts were present. Codex was upgraded from its 1.6.1
local marketplace cache to the 1.7.0 isolated release worktree, and the cached
release manifest was byte-identical to the locally certified source manifest.
The final clean-commit generation is installed after the documentation commit;
its non-circular hash is recorded in the release PR description.

## Doctor and installed-host evidence

The canonical Prometheus diagnosis, fix dry-run, and refresh dry-run each
reported 11 passed, 1 warning, 3 skipped, 0 failed, exit 0. Selection occurred
before constructing excluded checks; the negative fixtures prove excluded KBD
and Sovereign scopes make zero requests and cannot install, restart, or rewrite
those services.

Additional results:

| Surface | Result |
| --- | --- |
| `npm run doctor` | parity pass; no required failures |
| `pk doctor --json` | 6 passed, 0 warned, 0 failed |
| `check-mcp-health.sh --json --exclude service:sovereign-sync` | Memory health/ready 200, Knowledge MCP 200, Forge auth boundary 401 as expected |
| `prometheus-services.sh doctor --exclude service:sovereign-sync` | definitions valid; Memory/Knowledge/Forge and queue surfaces responsive |
| `prometheus learning status --json` | no pending, processing, retry, rejected, or dead-letter jobs/receipts |
| `cowork doctor` | pass with one optional missing-router warning |
| `cowork toolchain check` | all required tools present |
| `codex doctor --json` | every substantive check passed; only `TERM=dumb` failed in the non-interactive certification shell |

The Prometheus warning is explicit: discovery-budget measurements are not yet
recorded for Claude, Codex, OpenCode, or Kimi. Inventory itself is deterministic
(147 discovered site entries, 145 loadable payloads). This is a measurement gap,
not a correctness or installation failure.

Cowork's status display incorrectly labeled Forge and Knowledge unreachable
because it uses unauthenticated/incorrect probe semantics; the direct MCP probes
returned 401-auth-required and MCP 200 respectively, and Cowork's required-tool
check passed. The same status command unexpectedly included a read-only probe of
the excluded installed Sovereign endpoint. It made no mutation, but its lack of
an exclusion flag is a recorded operational defect and the result is not used as
Sovereign certification evidence.

The services doctor initially omitted `/usr/local/sbin` from its search path and
reported `logrotate` missing even though the root doctor verified the executable
and loaded rotation definition. The path was corrected and the rerun resolved
`/usr/local/sbin/logrotate`.

## Warning dispositions

- `npm ci` emitted upstream deprecation warnings for `inflight@1.0.6` and
  `glob@8.1.0`; both audits are clean and replacement is dependency-upstream
  work, not a release waiver.
- Repository-wide Prettier still identifies 15 unrelated pre-existing files.
  Every release-touched format-scoped file is clean; the baseline is recorded
  rather than hidden as a false green.
- Codex Doctor inherits `TERM=dumb` and `NO_COLOR=1` from this tool execution
  environment even with a PTY. Auth, config, Git, installation, MCP, provider
  reachability, WebSocket, runtime, search, sandbox, state databases, rollout
  parity, system, title, and update checks all pass.
- The installed Liter LLM configuration lacks an exported master-key variable
  and two optional role-model entries. The adversarial judge gateway itself is
  reachable and passed the Prometheus doctor; user credentials were preserved.

## Evidence classification

Artifact, disposable-runtime, and installed-service evidence above are current
for this Mac. No external deployment certification is claimed. GitHub workflow
output is never certification evidence, and no aggregate readiness percentage
is derived from these unlike evidence classes.

## 2026-08-04 binary and documentation recovery addendum

This addendum preserves the earlier certification record and documents the
focused 1.7.0 recovery requested after stale installed binary metadata and the
Pages layout regression were found. It used root base
`e9f48ca39a68d355b2e9e7ed5259901fb117ab3a`, Knowledge commit
`4a62bef615b1c210a94f2f97e59757be21eead94`, and Memory commit
`ec238aae39bd0b60722baba980d63e133e7ce879`.

No Rust test suite or hosted validation was run. KBD and Sovereign Sync were not
invoked. The local scope was limited to release builds, exact version checks,
documentation contracts and packaging, Mermaid parsing, production site builds,
browser/accessibility inspection, ad-hoc signing, and installed-artifact checks.

All five installed commands now return the exact release contract:

```text
prometheus 1.7.0
pk 1.7.0
pk-cherry 1.7.0
prometheus-learning-worker 1.7.0
surreal-memory-server 1.7.0
```

Each installed Mach-O has a valid ad-hoc signature and the same UUID as its
release-build artifact. The prior executables are recoverable from
`~/.prometheus/repair/1.7.0-binary-docs-recovery-20260804T053000/`. The Memory
binary's `--version` and `-V` paths produced identical output, zero stderr, and
created no files in an empty isolated home. The Memory and Knowledge services
were restarted from the installed paths; the learning worker was also kicked
and remained managed by its LaunchAgent.

The focused documentation results were:

- `npm run docs:check`: exit 0, including 50 Mermaid diagrams across 41 files;
- `npm run build:deploy` with `site/docs-catalog` initially absent: exit 0 and
  deterministic regeneration of 145 skills across 17 categories;
- production Docusaurus build: exit 0 with broken links treated as errors; and
- local production inspection at 375, 768, 1024, 1280, and 1536 CSS pixels:
  no horizontal overflow, responsive 1/2/2/4/4 capability-card columns,
  desktop dropdown and mobile-menu operation, no sub-24px sampled interactive
  target, visible 3px keyboard focus, and no console errors.

The lifecycle page rendered one Mermaid SVG and contained no Mermaid error
text. The focused accessibility inspection found one `main`, one primary
navigation landmark, one footer, one `h1`, no heading-level skips, no unnamed
visible controls, no images without alt text, and no duplicate IDs.

Screenshots are archived for
[375 px](./screenshots/home-375.png),
[768 px](./screenshots/home-768.png),
[1024 px](./screenshots/home-1024.png),
[1536 px light](./screenshots/home-1536.png),
[1536 px dark](./screenshots/home-1536-dark.png), and the
[rendered lifecycle diagram](./screenshots/lifecycle-1536.png). Machine-readable
hashes and results are in
[`binary-docs-recovery.json`](./binary-docs-recovery.json).
