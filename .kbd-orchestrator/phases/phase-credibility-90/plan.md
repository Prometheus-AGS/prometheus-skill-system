# Plan — phase-credibility-90

**Total changes:** 2 (serial)
**Change backend:** Native KBD (no OpenSpec needed for 2-change phase)

---

## Change 1 — Add BDD test CI job

**ID:** `change-90-001-bdd-ci`
**Title:** Add `bdd-test` CI job to `.github/workflows/validate.yml`
**Effort:** S

The job must:
1. Check out with `submodules: true`
2. Set up Node.js 22
3. Set up Rust stable (reuse cache from `forge-rs-test` if possible, or add a separate cache)
4. Build the forge binary: `cargo build --manifest-path tools/forge-rs/Cargo.toml`
5. Install Node deps: `npm ci --ignore-scripts`
6. Run BDD: `npm run cucumber`

The `FORGE_BIN` env var in `tests/steps/forge-steps.ts` defaults to `tools/forge-rs/target/debug/forge` — this path is satisfied by step 4.

---

## Change 2 — Commit and push

**ID:** `change-90-002-commit-push`
**Title:** Stage all phase-credibility-closure and phase-credibility-90 changes; commit and push

Files to stage (explicitly — do NOT use `git add -A`):
- `.github/ISSUE_TEMPLATE/bug_report.md`
- `.github/ISSUE_TEMPLATE/feature_request.md`
- `.github/ISSUE_TEMPLATE/skill_proposal.md`
- `.github/workflows/validate.yml`
- `.gitleaks.toml`
- `.gitignore`
- `.gitmodules`
- `CONTRIBUTING.md`
- `docs/deployment-modes.md`
- `package.json`
- `package-lock.json` (generate if not present)
- `scripts/configure-mcp-all-tools.sh`
- `tests/features/forge-validate.feature`
- `tests/features/forge-enrich.feature`
- `tests/steps/forge-steps.ts`
- `tools/forge-rs/Cargo.lock`
- `tools/forge-rs/Cargo.toml`
- `tools/forge-rs/crates/forge-cli/src/main.rs`
- `tools/forge-rs/crates/forge-core/src/lib.rs`
- `tools/forge-rs/crates/forge-enricher/src/lib.rs`
- `tools/forge-rs/crates/forge-mcp/src/lib.rs`

Do NOT stage:
- `.prometheus/` (gitignored machine state)
- `.kbd-orchestrator/` (runtime state — not committed by convention)
- `memory/` (project memory directory)
- `skills/imported/sycophancy-correction` (submodule — handle separately if needed)

Commit message:
```
feat(credibility): close all P0-P3 assessment findings; reach ≥90% readiness

- Remove hardcoded Tavily key from configure-mcp-all-tools.sh (OQ-01 resolved)
- Bind forge-mcp to 127.0.0.1; add bearer token auth via tower-http
- Canonicalize and confine task_path in forge_enrich (path traversal)
- Wire forge validate to ConstitutionChecker; exit 1 on Error severity
- Add drift readback: load_stale_skills() warns on acceptance_rate < 0.5
- Add forge status command showing environment health
- Fix Unicode doctest; add 20 unit tests (forge-core, forge-enricher)
- Add forge-rs-test CI job (fmt + clippy + test)
- Add BDD feature files + TypeScript step definitions for forge validate/enrich
- Add bdd-test CI job (build forge + cucumber)
- Convert artifact-refiner submodule from SSH to HTTPS
- Gitignore machine-local state (.prometheus/, *.local.env)
- Pin sycophancy-correction submodule to SHA 6687d6f9
- Track package-lock.json; use npm ci in all CI jobs
- Add CONTRIBUTING.md, GitHub issue templates, docs/deployment-modes.md
- Add gitleaks CI secret scan + .gitleaks.toml allowlist for planning docs
```

Push: `git push origin main` (or open PR if branch protection is enabled)

---

## Ordering rationale

Change 1 must complete before Change 2 so the BDD CI job is included in the commit.
