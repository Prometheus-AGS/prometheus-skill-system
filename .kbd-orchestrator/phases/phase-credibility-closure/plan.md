# Plan: phase-credibility-closure

**Total changes:** 16  
**Wave structure:** P0 (serial, blocking) → P1 (serial, high-value) → P2/P3 (parallel) → P4 (serial, final gate)  
**Change backend:** OpenSpec (`openspec/changes/change-credibility-NNN-*/`)

---

## Wave 1 — P0 Security (serial, must complete before Wave 2)

| # | Change ID | Title | Agent | Effort |
|---|-----------|-------|-------|--------|
| 1 | `change-credibility-001-remove-hardcoded-api-key` | Remove hardcoded Tavily API key from configure-mcp-all-tools.sh | claude | S |
| 2 | `change-credibility-002-bind-loopback` | Bind forge-mcp to 127.0.0.1 instead of 0.0.0.0 | claude | XS |
| 3 | `change-credibility-003-bearer-auth` | Add bearer token auth to forge-mcp /mcp endpoint | claude | M |
| 4 | `change-credibility-004-path-confinement` | Canonicalize and confine task_path in forge_enrich handler | claude | S |

**BLOCKING PREREQUISITE for C01:** User must rotate the Tavily API key at tavily.com BEFORE pushing C01. The key `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` in `scripts/configure-mcp-all-tools.sh:25` must be invalidated externally before this commit is merged.

---

## Wave 2 — P1 Capability (serial, after Wave 1)

| # | Change ID | Title | Agent | Effort |
|---|-----------|-------|-------|--------|
| 5 | `change-credibility-005-real-validate` | Wire forge validate to call ConstitutionChecker (not a stub) | claude | M |
| 6 | `change-credibility-006-drift-readback` | Wire drift data read-back in Enricher::enrich() Phase A | claude | M |
| 7 | `change-credibility-007-forge-status` | Add forge status command; label stubs as [EXPERIMENTAL] | claude | S |

---

## Wave 3 — P2/P3 Quality (all parallel, after Wave 2)

| # | Change ID | Title | Agent | Effort |
|---|-----------|-------|-------|--------|
| 8 | `change-credibility-008-unit-tests` | Fix Unicode doctest + add ≥15 unit tests to forge-rs | claude | M |
| 9 | `change-credibility-009-rust-ci` | Add forge-rs-test CI job to validate.yml | claude | S |
| 10 | `change-credibility-010-bdd-tests` | Add BDD feature files + cucumber step defs (forge validate + enrich) | claude | M |
| 11 | `change-credibility-011-submodule-https` | Change artifact-refiner submodule URL from SSH to HTTPS | claude | XS |
| 12 | `change-credibility-012-machine-state-gitignore` | Remove machine state from tracking; add to .gitignore | claude | S |
| 13 | `change-credibility-013-pin-submodule` | Pin sycophancy-correction submodule to stable commit SHA | claude | XS |
| 14 | `change-credibility-014-package-lock` | Commit package-lock.json; switch CI to npm ci | claude | XS |
| 15 | `change-credibility-015-contributing-docs` | Add CONTRIBUTING.md, GitHub issue templates, deployment-modes doc | claude | M |

C08–C15 are fully independent. Apply them in any order or in parallel.

---

## Wave 4 — P3 Final Gate (serial, after all prior waves)

| # | Change ID | Title | Agent | Effort |
|---|-----------|-------|-------|--------|
| 16 | `change-credibility-016-sycophancy-claim-audit` | Run detect_sycophancy on production readiness claim; score < 0.15 required | claude | S |

---

## Ordering Rationale

1. **Security first (Wave 1):** P0 findings block all other work. A hardcoded key and network-exposed service are not acceptable to have in the repo while capability work is ongoing.
2. **Capability before tests (Wave 2):** Unit tests and BDD tests for `forge validate` (C08, C10) cannot be meaningful until the real validator is wired (C05). Tests written against a stub would test the stub.
3. **Wave 3 parallelism:** C08-C15 touch completely disjoint files. None modify the same `Cargo.toml`, `.gitmodules`, or workflow file simultaneously. They can be executed in a single parallel batch.
4. **Sycophancy gate last (Wave 4):** C16 evaluates the claim after evidence is assembled. Running it before the fixes land would produce a failing score that is not informative.

---

## Open Questions Before Execution

- **OQ-01 (BLOCKING for C01):** Has the Tavily API key `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` been rotated at tavily.com? This is a USER action required before C01 can be pushed.
- **OQ-02:** Does `forge-enricher::check_constitution` currently exist as a function or is it embedded in `Enricher::enrich()`? Affects C05 scope.
- **OQ-03:** Does `.gitmodules` currently use SSH or HTTPS for `skills/imported/artifact-refiner`? Read the file at execution time to confirm.

---

## Library/Tool Decisions

| Library | Verdict | Reason |
|---------|---------|--------|
| `tower-http::ValidateRequestHeaderLayer` | ADOPT (already in workspace) | Simple bearer token; zero new deps |
| `@cucumber/cucumber@^11` | ADOPT (npm, MIT) | Standard BDD runner; v11 is current |
| `dtolnay/rust-toolchain@stable` | ADOPT (GitHub Action) | De facto Rust CI action |
| `gitleaks/gitleaks-action@v2` | ADOPT (GitHub Action) | MIT secret scanner |
| Path confinement | BUILD with stdlib only | `std::path::canonicalize` + `starts_with` — zero new crates |

---

## How to Apply

```bash
# Wave 1 (serial — C01 requires Tavily key rotation first)
/kbd-apply change-credibility-001-remove-hardcoded-api-key
/kbd-apply change-credibility-002-bind-loopback
/kbd-apply change-credibility-003-bearer-auth
/kbd-apply change-credibility-004-path-confinement

# Wave 2 (serial)
/kbd-apply change-credibility-005-real-validate
/kbd-apply change-credibility-006-drift-readback
/kbd-apply change-credibility-007-forge-status

# Wave 3 (parallel — any order)
/kbd-apply change-credibility-008-unit-tests
/kbd-apply change-credibility-009-rust-ci
/kbd-apply change-credibility-010-bdd-tests
/kbd-apply change-credibility-011-submodule-https
/kbd-apply change-credibility-012-machine-state-gitignore
/kbd-apply change-credibility-013-pin-submodule
/kbd-apply change-credibility-014-package-lock
/kbd-apply change-credibility-015-contributing-docs

# Wave 4 (serial — final gate)
/kbd-apply change-credibility-016-sycophancy-claim-audit
```
