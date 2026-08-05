# Assessment — session-defect-closure

_Generated 2026-08-05. Scope: `prometheus-skill-pack` only. Every finding below was
reproduced with a command in this session; nothing is carried over on memory alone._

## Method

Eleven gates were run against the working tree: validation gates, generated-artifact
drift, git hygiene, the two installer defect classes fixed earlier this session,
doc/reality drift, live-port verification, dangling service artifacts, submodule
exposure, knowledge-base health, exec-phase state, and generation payload size.

Two candidate findings were **rejected after verification** and are recorded here so
they are not re-raised:

- `install-mcp-services.sh` uses `declare -A` (bash 5) and touches launchd. Not a
  defect: no plist invokes it — it *installs* plists — and `/bin/bash -n` parses it
  cleanly. It never runs under the bash 3.2 launchd path.
- Six `127.0.0.1:7892` references remain in the sync skills. Not a defect: they are
  the deliberate `--tcp` alternative documented in commit `e0c40ec`.

## Findings

Ranked by whether they block a "finished" declaration.

### F1 — `SKILLS.md` index is out of date (BLOCKING)

`npm run check:skills-index` fails: `SKILLS.md skills index is OUT OF DATE`.
This is an explicit item on the CLAUDE.md publishing checklist, so the repo cannot
be declared release-clean while it fails.

- **Verify:** `npm run check:skills-index`
- **Fix:** `npm run generate:skills-index`, commit the result.
- **Risk:** none; output is deterministic.

### F2 — Uncommitted generated hook siblings (BLOCKING)

`hooks/codex-hooks.json` and `shared/harnesses/generated/claude-hooks.json` carry the
bundle-hash update from commit `67d29a2`, which committed only `hooks/hooks.json`.
All four artifacts currently agree on bundle `b0144d09…` and both parity checks pass,
so this is uncommitted drift rather than inconsistency — but a fresh clone would
regenerate a diff on first run.

- **Verify:** `git status --porcelain` shows both modified; all four files report
  the same 64-char bundle id.
- **Fix:** commit both alongside F3.

### F3 — Two in-flight source fixes uncommitted (BLOCKING)

Built and verified in this session, never committed:

| File | Change | Verified |
|---|---|---|
| `substrate/sovereign-sync/src/main.rs` | `--mode daemon` help said "HTTP on :7892"; corrected to Unix socket, and `--port` now states it applies only with `--tcp` | `cargo build --release` clean; `--help` output confirmed |
| `scripts/generate-commands.js` | Emitted `{{file:<relative>}}`, which Claude Code does not expand — 149 of 237 slash commands silently did nothing. Now emits an absolute read-then-follow directive, matching the working Codex generator. Also collapses multiline `description:` (153 skills affected) that produced invalid YAML frontmatter | Regenerated 147 files; 147/147 have valid single-line frontmatter, 0 invalid |

Outstanding for F3: `cargo fmt` + `cargo clippy` on the Rust change (per the
fmt-after-clippy rule), then commit. This was the step interrupted at user request.

### F4 — Submodule local fixes exist only in the working tree (HIGH)

| Submodule | Uncommitted | Content |
|---|---|---|
| `tools/surreal-memory-server` | 2 files | Heartbeat-flooding guard in `operations.rs`; metal/cuda feature propagation in `Cargo.toml` |
| `tools/prometheus-knowledge` | 1 file | `pk-learning-worker/src/main.rs` |

Both are compiled into the running binaries, so the machine is fine — but the changes
exist nowhere else. `git submodule update` or a fresh clone loses them silently.

- **Fix:** push upstream in each submodule repo, then bump the pin here. Requires a
  decision on which branch/PR; not mechanical.

### F5 — `ai.prometheus.liter-llm-api` plist is dangling (MEDIUM)

Added in `8464db7`. Carries **7 unsubstituted `__PROMETHEUS_*__` placeholders** and is
referenced by **no installer script**, so it can never be rendered or loaded.

Currently inert: `openai-proxy` serves `:8181`, and `scripts/check-model-config.sh`
reports no findings. It is dead weight, not a fault.

- **Decide:** wire it into `install-mcp-services.sh` with placeholder substitution, or
  delete it. Leaving it is the one option that should be ruled out — a plist that
  cannot load is a trap for the next reader.

### F6 — `pk` knowledge base: 429 mechanical issues (MEDIUM)

`pk lint --mechanical-only` reports 429 (was 463 earlier in the session; the count
moves as the Karpathy loop writes new session records).

**`pk lint --fix --mechanical-only` cannot fix these.** Verified: it reports
`0 auto-fixed` and changes no files. Root cause is structural, in
`prometheus-knowledge`:

- `pk-cli/src/main.rs:401` only attempts a fix when `report.auto_fixable` is true
- the mechanical lint path hardcodes `auto_fixable: false` (`pk-librarian/src/librarian.rs:261`)
- the flag is set true only on the **LLM** path (`prompts.rs:55`)
- `auto_fix()` has exactly one deterministic repair — `okf_autofix_type`, for a
  missing `type` field, which none of these 429 issues have

Roughly 230 are missing `description`, 123 are orphan pages. A bulk edit here treats
the symptom: the ingest path never populates `description`, so the count regrows every
session. The durable fix is upstream (populate at write time), which is outside this
repo.

- **Decide:** file upstream against `prometheus-knowledge`, or accept the count as a
  known non-blocking condition and stop reporting it.

### F7 — Recurring stale `.git/index.lock` (MEDIUM, unresolved cause)

Occurred **three times** during this session, each time a 0-byte file blocking a
commit until manually removed, each time during installer activity. No live git
process held it on any occasion.

Not currently present. Cause not identified — likely a hook dying mid-operation.

- **Investigate:** instrument the hook chain, or add a lock-age guard that reports
  rather than silently blocking. Cannot be fixed without reproducing it.

### F8 — Exec engine is scaffolding only (INFORMATIONAL — not a defect)

`prometheus-exec-code-execution-engine` is 1/4 changes complete. `change-exec-001`
(contracts) landed; `002` Tier-P sidecar, `003` Tier-W, `004` remote MCP are pending.
No `prometheus-exec` binary, no launchd plist, `exec-service/src/lib.rs` is a 10-line
stub.

There is no code-executor service to run, and its absence is correct. Recorded so it
is not mistaken for a broken service.

## Verified healthy (no action)

- `npm run validate` — 145 skills, 0 errors, 0 warnings
- Harness parity — 30 hooks × 2 manifests, bundle `b0144d09…`
- Codex plugin artifacts — up to date and valid
- All 7 documented TCP ports live; `sovereign-sync` healthy on its Unix socket
- Generation payload 91M (down from 188M after `67d29a2`)
- No `cp`-without-`codesign` and no `cmp`-based binary verification remain in `scripts/`

## Suggested change order

1. `change-sdc-001` — regenerate `SKILLS.md` (F1)
2. `change-sdc-002` — fmt/clippy + commit the two in-flight fixes and hook siblings (F2, F3)
3. `change-sdc-003` — push submodule local fixes upstream, bump pins (F4)
4. `change-sdc-004` — decide `liter-llm-api` plist: wire or delete (F5)
5. `change-sdc-005` — decide `pk` disposition: upstream ticket or accept (F6)
6. `change-sdc-006` — instrument the index.lock recurrence (F7)

Changes 1–2 are mechanical and unblock a green-gate declaration. Changes 3–6 each need
one decision from the operator before execution.
