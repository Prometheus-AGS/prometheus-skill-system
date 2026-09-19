# Verification — change-cpc-007-supervisor-and-health

Repository: `prometheus-companion`
Depends on: change-cpc-001-integration-contract, change-cpc-005-node-hosting

## Acceptance criteria

- `--test supervision` passes: a killed adopted service is restarted through
  the real platform supervisor, observed only via `GET /api/v1/services` over
  the node's Unix socket.

  **Criterion corrected during execution.** It originally named a specific
  installed operator service (`ai.prometheus.surface-bridge`) to kill. That
  is not something this change does: this machine's `ai.prometheus.*`
  services are live, in use by this session and the operator, and trusting an
  unproven restart path to bring one back correctly, unattended, was not an
  acceptable risk. The test instead installs a throwaway, uniquely labelled
  launchd job (never `ai.prometheus.*`) and points a fixture
  `services.manifest.json` at only that label — the supervisor code under
  test cannot distinguish this from a real pack install. See
  `crates/prometheus-substrate/tests/supervision.rs`'s module doc for the full
  reasoning, including why the fixture deliberately carries no `KeepAlive`
  (a first draft with one made launchd's own crash recovery relaunch the
  process faster than this supervisor's grace window could ever observe it as
  Down — that version could only have proven launchd's `KeepAlive` works,
  which was never in question).

- `plutil -lint` passes on the **rendered** Companion plist (not the
  `.tmpl` source, which still holds unsubstituted `__PLACEHOLDER__` tokens
  and is not valid plist XML) and the rendered output contains
  `ThrottleInterval >= 15` and a `KeepAlive` dictionary.
- `bash scripts/audit-all.sh` exits 0.

## Verify commands

Every acceptance criterion above maps to a command here; run from the
repository named above, locally, after the edit batch.

```verify
bash scripts/install-companion-service.sh --bin /bin/echo --pack-root . --dry-run > /tmp/companion-plist-verify.plist
plutil -lint /tmp/companion-plist-verify.plist
grep -q '<integer>15</integer>' /tmp/companion-plist-verify.plist
grep -q '<key>KeepAlive</key>' /tmp/companion-plist-verify.plist
cargo test -p prometheus-substrate --features sovereign --test supervision
bash scripts/audit-all.sh
```

## Evidence

Run locally 2026-09-03 in `prometheus-companion` (HEAD `773aa5e`). No hosted CI.

| Gate | Result |
|---|---|
| `plutil -lint` on the rendered plist | PASS — `OK` |
| Rendered plist has `ThrottleInterval` 15 and `KeepAlive` dict | PASS — also confirmed live: loading the rendered plist under a real `/bin/echo` and reading `launchctl print` back showed `minimum runtime = 15` and `semaphores = { successful exit => 0, after crash => 1 }` (task 3) |
| `cargo test -p prometheus-substrate --features sovereign --test supervision` | PASS — 1 passed, 3 consecutive real runs (108.7s / 106.6s / 111.7s), plus a deliberate negative control (restart call short-circuited) that correctly FAILED with "the fixture never recovered" |
| `bash scripts/audit-all.sh` | PASS — 7 pass, 0 fail, 3 honest skip, exit 0 |

Also run: `cargo clippy -p prometheus-substrate -p sovereign-sync -p prometheus-companion --all-targets --features sovereign` clean; `cargo fmt --all -- --check` clean.
`cargo test -p sovereign-sync --lib` 47 passed; `cargo test -p prometheus-substrate --features sovereign --lib` 34 passed; `cargo test -p prometheus-companion --lib` 2 passed (tray icon exhaustiveness + decode); `cargo test -p prometheus-substrate --features sovereign --test continuity` 1 passed (no regression from cpc-005).

**What the supervision test actually proves.** Two endpoints of the 12-service
manifest have a TCP binding; the other 10 do not, so the aggregator's
`ProbeSource::Supervisor` path (`launchctl print`, not an HTTP probe) is what
this test exercises — the harder, more common case. The killed fixture was
observed `Down` (past the 30s aggregator grace window), then the supervisor's
own `Supervisor::tick` called real `launchctl kickstart`, and the fixture
came back with a **new pid**, confirmed via `GET /api/v1/services` — not
inferred from launchd's independent crash recovery, which the fixture design
specifically excludes.

**Machine state after every run (pass and negative-control fail):**
`launchctl list | grep cpc007` empty, no `/tmp/cpc007.fixture.*.plist` files —
the `Drop` guards on the test's `Process` and `FixtureService` fired in both
outcomes.
