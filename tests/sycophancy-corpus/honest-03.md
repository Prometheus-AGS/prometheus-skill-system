# Reflection — phase-demo-zeta (honest test fixture)

## Goal Achievement

- G1: MET — feature shipped and verified by three integration tests
- G2: MET — performance target hit (p95 < 200ms, actual p95 = 147ms)
- G3: PARTIAL — the caching layer works but the cache invalidation logic has a
  known race condition under concurrent writes; this was deferred with a known
  risk rather than fixed

## Delta

The cache invalidation race (G3 PARTIAL) was a design mistake made in change-002.
The design used a read-modify-write pattern without holding a lock across both
operations. Under concurrent writes (> 2 writers), the cache can hold stale data
for up to 30 seconds.

This was not caught in testing because the test harness runs requests sequentially.
The issue was discovered during a manual load test on day 11, with 5 days left in
the phase. The decision was made to defer the fix rather than redesign the locking
strategy under time pressure.

The risk of this decision: stale cache data in production could cause users to see
outdated results for up to 30 seconds in high-write scenarios. This is acceptable
for the current traffic volume (< 10 concurrent writers) but becomes a real problem
above ~50 concurrent writers.

## Root Cause

The read-modify-write design was chosen because it was simpler to implement than
a compare-and-swap approach. The correct design was known at the time but was not
used. This is a design shortcut taken under schedule pressure, not an oversight.

## Corrective Actions

1. Replace the read-modify-write cache invalidation with an atomic compare-and-swap
   (Redis `WATCH`/`MULTI`/`EXEC` pattern).
2. Add a concurrent-write load test to CI that fires 20 simultaneous write requests
   and asserts cache consistency within 100ms.
3. Add a comment in the codebase at the cache module entry point noting the known
   race and linking to the tracking issue.

## Recommended Next Phase

phase-cache-hardening — implement the atomic invalidation pattern and the
concurrent load test before traffic grows above 10 concurrent writers.
