# Assessment — phase-codex-plugin-verify-and-publish

_Assessed 2026-07-12. Method: **actual verification** (gh CLI on live CI, repo inspection) — not assumption. Seeded from the prior phase's reflection caveats._

## Headline — the CI is RED, and it hides my new gate

Checking (not assuming) surfaced the real state: the **`Validate Skills` workflow has been failing on every recent push to `main`** (last 5 runs all red, including 2026-07-11 pre-dating this work — so **pre-existing**). Two independent failures, neither caused by the Codex work:

1. **`validate:signals`** (job "Validate AgentSkills.io Compliance") — `skills/process/cowork-management/SKILL.md: missing "## Progress Signals" section`. The job halts here.
2. **`Check Formatting`** — Prettier reports **32 files** with style issues (mostly under `tools/disk-space-guardian/`).

**Consequence for G-01:** my `validate:codex` step sits *after* `validate:signals` in the same job, so it shows `-` (never reached). The gate I shipped last phase has **never actually executed in CI**. G-01 cannot be "verified green" until the two pre-existing failures are fixed.

## Gap analysis (per goal)

| Goal | Verified current state | Gap | Effort |
|---|---|---|---|
| **G-01** validate:codex green in a real Actions run | CI **RED**; `validate:signals` + `Check Formatting` fail *before* my step runs | Fix (a) `cowork-management/SKILL.md` missing `## Progress Signals`, (b) 32 prettier files (`npm run format`), then confirm the run goes green and `validate:codex` executes | **M** |
| **G-02** MCP env round-trip | helper present; **keys not in the CI/shell env** (`TAVILY_API_KEY`/`FORGE_MCP_TOKEN` unset here — they live in `~/.bash_profile`) | run `codex-provision-mcp-env.sh` with keys sourced, install plugin, `codex doctor` — confirm ⚠ clears; scriptable | **S** |
| **G-03** real hooks run under Codex | hooks portable (`${CLAUDE_PLUGIN_ROOT:-$PLUGIN_ROOT}`, 39 refs) but only a **probe** was run — real hooks not yet executed under Codex | headless real-hook test via `codex exec --dangerously-bypass-hook-trust`; confirm the pack's actual SessionStart hooks run without empty-path errors | **S (automatable)** |
| **G-04** git-subdir resolves remotely | committed marketplace uses `source: local`; repo pushed (`ca8ff18` = origin/main); no git-subdir marketplace published | generate + publish a `git-subdir` marketplace, then `codex plugin marketplace add <git-url>` and confirm resolution; **needs a real publish decision** | **M** |

## Key observations

- **G-01 is the anchor and its scope grew** — the "verify CI" goal is really "**make CI green**," which means fixing two pre-existing breakages the Codex work merely revealed. Both are quick and independently valuable (a red `main` is bad regardless).
- The `cowork-management` skill was added without a `## Progress Signals` section — the validate:signals lint (correctly) rejects it. Fixing = add the mandated section (see CLAUDE.md → Progress Signaling).
- G-02 needs the keys **sourced** (they're in `~/.bash_profile`, not the automation env). The user may need to run it, or I run it with the keys exported.
- G-03 is automatable headlessly (proven mechanism last phase).
- G-04 genuinely needs an external publish — confirm intent with the user before publishing.

## Suggested change decomposition (input to /kbd-plan)

~5 changes: (1) add `## Progress Signals` to `cowork-management/SKILL.md` [unblocks validate:signals]; (2) `npm run format` the 32 prettier files [unblocks Check Formatting]; (3) push + confirm CI green incl. `validate:codex` [G-01]; (4) headless real-hook run under Codex [G-03]; (5) env round-trip check [G-02]. Plus (6) git-subdir publish + resolve [G-04] — gated on user go-ahead.

## Overall

Higher-value than the original "just watch a CI run": the phase now **fixes a red `main`** (two pre-existing failures) as the path to G-01, then closes the smaller verification items. G-04 (external publish) is the only one needing a user decision.
