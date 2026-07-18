# Cross-Tool Skill and Learning-System Validation

Date: 2026-07-18

Repository: `Prometheus-AGS/prometheus-skill-system`

Baseline commit: `7c83ad929e23ec99e9595633610dcb431a8d4768`

## Result

The complete Prometheus skill payload is installed and verified across every AI
tool detected on this workstation. The Karpathy hooks, PK LLM wiki, Feynman
learning loop, learner-model FSRS runtime, Tier-2 UI bridge, and sovereign-sync
runtime all passed behavioral tests. No KBD phase transition was executed and no
repository `.kbd-orchestrator` state was changed.

## Installed payload proof

`node scripts/verify-installed-skills.js --json` discovered 139 skills and
reported 14 of 14 platforms healthy. Every source file was compared byte-for-byte
and every executable bit was compared with its installed copy.

| Platform | Verified payloads | Command fallbacks | Preserved collisions | Result |
|---|---:|---:|---:|---|
| Claude Code / Desktop | 139 | 0 | 1 | PASS |
| OpenCode | 139 | 0 | 1 | PASS |
| Kimi Code | 139 | 0 | 0 | PASS |
| MiniMax | 139 | 0 | 0 | PASS |
| Cursor | 139 | 0 | 0 | PASS |
| Codex | 103 | 36 | 0 | PASS |
| Gemini | 139 | 0 | 0 | PASS |
| Roo | 139 | 0 | 0 | PASS |
| Windsurf | 139 | 0 | 0 | PASS |
| Windsurf legacy | 139 | 0 | 1 | PASS |
| Amp | 139 | 0 | 1 | PASS |
| Zed | 139 | 0 | 0 | PASS |
| Antigravity | 139 | 0 | 0 | PASS |
| Cline | 139 | 0 | 0 | PASS |

Codex intentionally uses a curated catalog: 103 payloads are installed as real
directories and 36 skills remain available through registered prompt fallbacks.
User-owned name collisions were preserved; matching Prometheus payloads were
installed under deterministic `prometheus-<skill>` names instead.

MiniMax now receives complete real-directory copies with `_meta.json`, including
scripts, references, templates, assets, and executable modes. The same copier is
used by both canonical installers. Copy and uninstall behavior is covered by an
isolated 139-skill packaging regression test.

## Claude Desktop acceptance proof

The local directory-marketplace plugin was cleanly reinstalled and enabled:

```text
prometheus-skill-pack@prometheus-skill-pack
Version: 1.6.0
Scope: user
Status: enabled
```

The installed Claude plugin cache contains the executable helper at:

```text
skills/process/kbd-process-orchestrator/skills/kbd-next-phase/scripts/kbd-next-phase.sh
```

The helper, Tier-2 render script, learning corpus, content-grounding script, and
`kbd-doctor` instructions were compared with repository source and matched.

## Behavioral proof

### Karpathy loop and LLM wiki

- PK health hook fixture suite: 5 passed, 0 failed.
- Prompt hook: missing-PK degradation and focused-context injection passed.
- Stop hook: meaningful session ingestion and empty-session rejection passed.
- Real isolated PK round trip passed:
  source ingest → OKF wiki page → `index.md`/`log.md` → list/search/focus → lint.
- The live PK MCP endpoint completed a real JSON-RPC `initialize` request with
  HTTP 200 and a valid MCP result.

### Feynman and learner loops

- Goal → survey → Feynman artifact → grade: PASS.
- Retain → practice → certification: PASS.
- KB adapter: 12 passed, 0 failed, 0 skipped.
- Meta/harness parity: 25 passed, 0 failed, 0 skipped across all 12 learn skills.
- Learner-model: 25 unit tests and 1 doctest passed.
- Real learner JSON-RPC seed/get/observe/review/load passed.
- Retention review persisted an observation and advanced the FSRS card.
- Missing concept updates now fail explicitly instead of silently succeeding.

### Tier-2 UI and synchronization

- Tier-2 progress intent render: PASS.
- Tier-2 question render → submit response → collect response: PASS.
- Surface bridge unit target, formatting, and warning-denied clippy: PASS.
- Sovereign-sync: 12 unit tests and 8 integration tests passed.
- Sovereign-sync warning-denied clippy: PASS.
- Legacy `com.prometheusags.surface-bridge` and
  `com.prometheusags.sovereign-sync` services were booted out and their plist
  files archived with timestamps.
- Canonical `ai.prometheus.surface-bridge` and
  `ai.prometheus.sovereign-sync` services own ports 7890 and 7892.

## Live service evidence

| Service | Manager state | Protocol status |
|---|---|---|
| SurrealDB | running | HTTP 200 |
| Surreal Memory | running | HTTP 200 |
| PK / prometheus-knowledge | running | MCP initialize 200 |
| Forge | running | authentication required (HTTP 401, expected) |
| Surface bridge | running | HTTP 200 |
| Sovereign sync | running | HTTP 200 |
| Prometheus nudge | scheduled, currently idle | launchd registered |

Stdio MCP tools remain client-managed and are reported separately from HTTP
daemons.

## Repository validation evidence

| Command / suite | Result |
|---|---|
| `npm run check-format` | PASS |
| `npm run validate` | 139 skills valid, no errors or warnings |
| `npm run validate:strict` | 139 skills valid, no errors or warnings |
| `npm run validate:signals` | 49 checked, PASS |
| `npm run validate:codex` | generated artifacts current |
| `npm run skill-matrix:ci` | no description collisions above threshold |
| `npm test` | 12 of 12 deterministic suites passed |
| KBD next-phase packaging regression | PASS |
| Cross-tool complete-payload regression | PASS |
| Learner-model fmt/test/clippy | PASS |
| Surface-bridge fmt/test/clippy | PASS |
| Sovereign-sync fmt/test/clippy | PASS |
| `git diff --check` | PASS |

The formerly broken `npm test` entrypoint now runs the deterministic repository
suite instead of referencing a nonexistent file. CI also runs the cross-tool
payload regression and learner-model runtime contract.

## GitHub Actions parity and runner status

Every command represented by `.github/workflows/validate.yml` and
`.github/workflows/sovereign-sync.yml` was replayed locally. Rust jobs used the
stable toolchain (`rustc 1.97.0`), matching the workflows. This parity pass found
and corrected one mechanical `rustfmt` drift in
`substrate/storage-provider/src/loro_adapter.rs`; the complete rerun then passed.

| Workflow surface | Local result |
|---|---|
| AgentSkills validation, signals, Codex artifacts, packaging regressions | PASS |
| Formatting and hooks symlink integrity | PASS |
| Prometheus CLI check and warning-denied clippy | PASS |
| Sycophancy release build and real artifact-gate E2E | 3 passed, 0 failed |
| forge-rs fmt, warning-denied clippy, and all tests | PASS |
| BDD smoke/strict validation and Cucumber scenarios | 7 scenarios, 42 steps passed |
| Gitleaks full-history scan | 419 commits, no leaks |
| Skill collision and learn-grade regression guards | PASS |
| Learner-model runtime contract | PASS |
| cowork-skills and disk-space-guardian cargo checks | PASS |
| storage-provider stable fmt/clippy/test | 28 tests passed |
| sovereign-sync stable fmt/clippy/test | 12 unit + 8 integration tests passed |
| sovereign-client stable fmt/clippy/test | 3 tests passed |

GitHub-hosted jobs remained queued without runner steps because the
`Prometheus-AGS` organization Actions budget is configured as `$0` with
`prevent_further_usage=true`, and the organization has no self-hosted runners.
Repository Actions permissions are enabled. This is an external runner-capacity
constraint rather than an untested source path; changing the paid-usage budget
requires organization-owner authorization.

## Installation notes

- OpenCode's goal plugin is installed through the supported command
  `opencode plugin -g @prevalentware/opencode-goal-plugin` and is present in both
  server and TUI configs.
- MCP configuration is complete for Claude Code, OpenCode, Codex, Kimi, MiniMax,
  and Cursor. The Windsurf legacy MCP config path is absent, so it is correctly
  skipped; both Windsurf skill directories are nevertheless fully verified.
- The canonical binary installer now builds and installs learner-model,
  surface-bridge, and sovereign-sync.
