# Verification — change-tlh-02-codex-memories-install-and-doctor

Repository: `prometheus-skill-system`
Depends on: none

## Acceptance criteria

- On a scratch `CODEX_HOME` whose config.toml has comments and other tables, the script adds `[memories] generate_memories = false`, keeps every other line byte-identical, and a second run changes nothing.
- A config.toml that already has `[memories] generate_memories = true` is changed to `false` in place, not duplicated.
- When the edited file would not parse, the original is restored and the script exits non-zero; the installer still completes with a warning.
- An existing `memories/memory_summary.md` is moved to `memories-archive/`; `MEMORY.md` and `raw_memories.md` are untouched.
- With codex on PATH, `codex debug prompt-input` under the scratch `CODEX_HOME` (no auth.json at all: no copy and no symlink, so the operator's credentials are unreachable) contains no text from a seeded `memories/MEMORY.md`. Without codex, or if prompt-input refuses to run unauthenticated, that step reports BLOCKED (exit 2), never pass.
- Test order: all deterministic cases run first; the prompt-input probe runs last as an optional sub-case that prints SKIP when codex is absent or refuses unauthenticated, and only `REQUIRE_CODEX_PROBE=1` turns that SKIP into exit 2. The doctor check is the standing guard for the inert-files conclusion.
- The install-system.js step is exported as `applyCodexMemories({home})` and exercised in isolation with `node -e` under a scratch HOME.
- With `CODEX_MEMORIES_SCRIPT` pointing at a script that exits 1, the guarded helper sourced from install-skills-flat.sh and the install-system.js Codex memories step (run under a scratch HOME) both return 0 and print a warning.
- `prometheus doctor --json` reports `codex.memories` Yellow with the repair command for an unset config, Green after the script runs, and the check never increments failed.

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
if pgrep -x cargo >/dev/null || pgrep -x rustc >/dev/null; then echo "BLOCKED: another cargo/rustc build is running (one build at a time)" >&2; exit 2; fi
/bin/bash shared/scripts/tests/test-codex-memories-config.sh
cargo test --manifest-path tools/prometheus-cli/Cargo.toml -p prometheus-cli --test doctor
npm run check:distribution
```
