# Reflection — phase-demo-delta (honest test fixture)

## Goal Achievement

- G1: MET — authentication system deployed, 47 integration tests passing
- G2: NOT MET — API rate limiting was not implemented; the endpoint exists but has no
  per-client throttling
- G3: PARTIAL — unit test coverage is 74%, below the 80% target; auth module has 91%
  but the payment module has 52%
- G4: MET — OpenAPI spec generated and validated against live endpoints
- G5: MET — staging deployment successful; production deployment blocked pending G2

## Delta (Planned vs. Delivered)

**G2 — rate limiting** was planned for week 2 but deprioritized when the auth module
took 3 days longer than estimated due to a JWT library incompatibility with the target
Node version. The decision to defer rate limiting was made explicitly on day 9.

**G3 — coverage gap** in the payment module traces to 4 untested error paths in the
refund flow. These paths require mocking a Stripe webhook that is not yet set up in
the test environment.

## Root Cause

G2 slipped because the auth estimate did not account for library compatibility
research time. Estimate was 2 days for implementation; actual was 5 days including
the library evaluation.

G3 gap in payment module is a test infrastructure gap, not a code gap — the refund
paths exist and are correct, but the Stripe test webhook is not configured.

## Corrective Actions

1. Add rate limiting to change-001 of the next phase. Scope: per-client sliding
   window (60 req/min), Redis-backed, with bypass for internal service tokens.
2. Set up Stripe test webhooks in CI this week. Assign to the same person who owns
   the payment module. Target: 80% coverage before next phase execute.
3. Update estimation template to include library compatibility research as a
   separate line item (1 day minimum for any new dependency).

## Recommended Next Phase

phase-api-hardening — focus: rate limiting (G2 carry-forward), webhook CI setup,
and coverage sweep to reach 80%.
