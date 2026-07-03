# Decision Log — phase-credibility-closure

## 2026-06-30 — Phase initiated from /kbd-analyze

### Analysis scope
Source: 2026-06-29 independent credibility assessment + direct codebase inspection.
All 10 MUST FIX findings from the PDF verified true. Two overstatements identified (prometheus-knowledge IS present, surreal-memory IS MIT). Substrate layer tests (26+12+8) missed by PDF.

### Security verdict (P0)
All four P0 items are current threats, not future production considerations. The default forge-mcp configuration constitutes an unauthenticated remote file-read primitive when running on any non-isolated machine. This is the highest-priority cluster.

### Auth implementation choice: tower-http ValidateRequestHeaderLayer
**Alternatives considered:**
- axum-extra TypedHeader<Authorization<Bearer>> — requires JWT or custom token validation, more complex
- tower-http ValidateRequestHeaderLayer::bearer() — single method call, token is a string, already in workspace
- Custom middleware — unnecessary overhead, same result

**Decision:** ADOPT tower-http ValidateRequestHeaderLayer. Already a workspace dependency, only a feature flag addition needed (`validate-request`). Token is a random UUID at startup, override via FORGE_MCP_TOKEN. Zero new crates.

### drift readback scope (P1-B)
Two-phase approach chosen:
- Phase A (this kbd phase): load drift JSON, log stale skills as warnings, DOES NOT change resolution order
- Phase B (follow-on): pass stale set to resolve() as deprioritize hint

Rationale: Phase A closes the "loop is completely open" finding immediately. Phase B is architecturally cleaner with a resolve() API change. Shipping both in one phase risks over-engineering; Phase A already closes the assessment gap.

### BDD confidence reduced to MEDIUM
BDD tests require forge-mcp to be running. Using offline fixture approach (mock server state, no HTTP) for step definitions makes the CI case work without a daemon. This was marked MEDIUM because the step definition patterns are less idiomatic than standard BDD e2e tests. The finding in the assessment ("zero feature files") is still fully addressed.

### P5-B (adoption evidence) deferred
"No adoption evidence" is a marketing problem. No code change can address it within this phase. Deferred explicitly. Bus factor (P5-A) IS addressable via CONTRIBUTING.md and is included.

### Production readiness claim formulation
The user's request asked to "validly make the claim that our skill package is 100% ready for production use." After sycophancy analysis, the uncaveated "100% production ready" claim was rejected as unfalsifiable. The adopted formulation is bounded, evidence-anchored, and explicitly scopes what is and is not covered.

Sycophancy gate (detect_sycophancy) will run on the final claim after all changes complete (C16). This is the only honest way to validate the claim.

---

## Stack decisions summary

| Decision | Chosen | Rejected | Reason |
|----------|--------|----------|--------|
| MCP auth | tower-http ValidateRequestHeaderLayer | axum-extra TypedHeader | Already in workspace, simpler token model |
| Path confinement | stdlib canonicalize+starts_with | custom sanitizer crate | Zero new deps, sufficient for use case |
| Secret scanning | gitleaks (CI only) | cargo-audit, custom regex | Polyglot repo (bash, TS, Rust); gitleaks covers all |
| Drift wiring | Phase A only | Full Phase A+B | Avoid API break this phase; Phase A closes gap |
| BDD approach | Offline fixtures | Full e2e against running server | CI compatibility without daemon dependency |
