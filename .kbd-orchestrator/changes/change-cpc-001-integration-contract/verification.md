# Verification — change-cpc-001-integration-contract

Repository: `prometheus-skill-pack`  
Depends on: none

## Acceptance criteria

- `node scripts/generate-service-manifest.mjs` twice produces byte-identical `shared/services.manifest.json`; `--check` exits 0 clean and non-zero after touching a template.
- `--test contract` passes through the compiled CLI: absent endpoint is silent with exit 0; discovered endpoint is named; a declaration with too-new contract version fails with both versions.
- `npm run validate:codex` and `npm run check:skills-index` pass (no plugin surface change in this change).
- Rule: no `cargo build` or workspace-wide check during implementation; the listed verify commands are the gate.

## Verify commands

Every acceptance criterion above maps to a command here; run from the repository named above, locally, after the edit batch.

```verify
node scripts/generate-service-manifest.mjs && shasum -a 256 shared/services.manifest.json > /tmp/m1 && node scripts/generate-service-manifest.mjs && shasum -a 256 shared/services.manifest.json | diff - /tmp/m1
cp shared/launchagents/ai.prometheus.pk-cherry.plist /tmp/p.bak && printf '\n' >> shared/launchagents/ai.prometheus.pk-cherry.plist && ( node scripts/generate-service-manifest.mjs --check; test $? -ne 0 ); mv /tmp/p.bak shared/launchagents/ai.prometheus.pk-cherry.plist
node scripts/generate-service-manifest.mjs --check
cargo test --manifest-path tools/prometheus-cli/Cargo.toml -p prometheus-cli --test contract
npm run validate:codex && npm run check:skills-index
```

## Evidence

Executed 2026-09-02 locally. No hosted CI cited.

### Passing gates

- `node scripts/generate-service-manifest.mjs` — wrote `shared/services.manifest.json` (12 services).
- Idempotence: two consecutive runs produced byte-identical output (`shasum -a 256` compared, identical).
- `node scripts/generate-service-manifest.mjs --check` on a clean tree — `up to date`, exit 0.
- Drift detection: a verified real edit to `shared/launchagents/ai.prometheus.pk-cherry.plist` (`ThrottleInterval` 10 → 99) produced exit 1 with `stale: ai.prometheus.pk-cherry`; the template was restored and `--check` returned to exit 0.
- `npm run check:services-manifest` — `up to date`, exit 0.
- `cargo test --manifest-path tools/prometheus-cli/Cargo.toml -p prometheus-cli --test contract` — **6 passed, 0 failed**, through the compiled binary:
  - `contract_show_is_silent_and_successful_when_no_endpoint_exists` (exit 0, **stderr empty**, `endpoint: null`, `endpoint_source: absent`)
  - `contract_show_names_an_explicit_endpoint_override` (trailing slash trimmed)
  - `contract_show_reports_the_service_manifest_when_present`
  - `contract_validate_accepts_a_conforming_declaration`
  - `contract_validate_refuses_a_declaration_requiring_a_newer_contract` (names both 2.0.0 and 1.0.0)
  - `contract_validate_refuses_a_bare_hook_bundle_name`
- `node -e "require('./site/sidebars.js')"` — parses; `kbd/integration-contract` added to the Control Plane category.
- `package.json` parses; `generate:services-manifest` and `check:services-manifest` wired.

### Pre-existing failures, not caused by this change

- `npm run validate:codex` — fails with `generated output is stale: dist/plugins/claude/prometheus-skill-pack`.
- `npm run check:skills-index` — reports `SKILLS.md skills index is OUT OF DATE` (163 committed vs 164 present).

Both were verified as **pre-existing and unrelated**: the skills-index generator reads only `skills/` and `SKILLS.md`, and this change modifies neither; `dist/` is tracked (2,350 files) and carries no modification from this change. Regenerating either surface would fold an unrelated 164th skill and a distribution refresh into this change, so both were deliberately left alone and `SKILLS.md` was restored byte-identical to `HEAD` (sha256 `de9aab20…4288b`) after a diagnostic regeneration. **They are owned by a separate change**; `change-cpc-012` and `change-cpc-009` both run `validate:codex` and will surface them again with the plugin surfaces they actually touch.

### Notes

- A `git stash` baseline attempt failed with `could not write index`; its output was discarded as invalid rather than cited, and a clean-worktree baseline also could not run (submodule contents absent). The conclusion above rests on generator-input analysis instead.
- One Cargo build ran at a time; no workspace-wide build or `cargo build` was invoked.
