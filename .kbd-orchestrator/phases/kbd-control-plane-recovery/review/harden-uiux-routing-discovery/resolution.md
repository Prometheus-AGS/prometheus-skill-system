# Adversarial review resolution — round 1

Original verdict: BLOCK (2 critical, 5 warning, 1 suggestion)

Sycophancy screen: PASS, score `0.01785714365541935`; no regeneration.

## Dispositions

1. **Stale waypoint — resolved with authoritative context.** Projection replay
   repaired all writable derived surfaces. The same `/kbd-apply` command is the
   deliberate resumable command while a completed implementation is awaiting
   mandatory review, verification, and archival. The detector retained two
   review-time missing-start-receipt blockers, and typed clear commands at
   revisions 201 and 202 documented why no historical receipt was invented.
   There are zero unresolved blockers.
2. **Live service dependency — fixed.** Scenario 6 is now opt-in via
   `KBD_MEMORY_LIVE_PROBE=1`. Default integration is hermetic (8/8), and the
   explicit installed-service integration also passes locally (9/9).
3. **Possibly unset `title` — rejected.** The `end-task` branch assigns
   `change`, `id`, `i`, `n`, shifts four arguments, then assigns `title="$*"`
   before either guard call.
4. **Title uniqueness — fixed documentation; runtime remains fail-closed.** The
   inaccurate uniqueness comment was removed. Duplicate titles are rejected as
   ambiguous. Supporting explicit composite subjects requires a separate guard
   contract change and is not claimed here.
5. **Cumulative memory diff — accepted packet-scope warning.** The earlier
   memory change was independently specified, refined, verified, and archived;
   the repository intentionally defers commit until the whole parent phase.
6. **Generated distributions — accepted and deferred.** The planned change
   `reconcile-kbd-control-plane-projections` owns deterministic double
   generation, source/install parity, and installed-surface certification.
7. **Empty memory URL — rejected.** `memory-log.sh` calls
   `kbd_memory_available`, resolves the URL, and exits on an empty value before
   curl.
8. **Bracketed IPv6 no-proxy token — fixed.** All three affected curl calls now
   use `127.0.0.1,localhost,::1`.
9. **HTTP route errors labeled unreachable — fixed after round two.** Recall
   now captures transport success and HTTP status separately. A reachable 404
   produces a distinct atomic HTTP-error stub and has a real local integration
   scenario.

## Revalidation

- Bash syntax: PASS.
- Hermetic memory full integration: PASS 10/10.
- Explicit live memory full integration: PASS 11/11.
- Agent-rules injector full integration: PASS 14/14.
- Strict skill validation: PASS, 23 skills, zero errors.
- `git diff --check`: PASS.

## Round 2 dispositions

Round two remained BLOCK because untracked archive/execution evidence was not
visible to the Git-diff packet and because entity-search HTTP failures still
shared the transport-unreachable stub.

- The canonical waypoint was advanced through signed plan revision 5 to
  `/kbd-apply add-kbd-registry-prune`; generated waypoint and reminder
  projections now agree with the first pending change.
- The archived memory change has all six tasks checked, its execution evidence
  is present, and its capability delta is merged at
  `openspec/specs/kbd-memory-integration/spec.md`. These intended artifacts are
  now staged so the final diff packet can inspect them.
- The current UIUX execution evidence, artifact-refiner log, and review
  resolution are staged for the same reason.
- Reachable HTTP errors now have a distinct stub and integration scenario.
- The unused `kbd_memory_rest_base` alias was removed.
- The repeated empty-URL warning remains disproved by the explicit
  `[[ -n "$url" ]] || exit 0` immediately before curl.
- Generated distribution work remains assigned to the explicit pending
  reconciliation change; no intermediate distribution is claimed certified.

## Final-round policy correction

The next review correctly identified that C-01 did not yet express the approved
same-parent-phase batching policy. C-01 now permits only a named reconciliation
change within the same parent phase, forbids distribution-certification claims
by intermediate changes, and still blocks phase completion, commit, or push
until double generation, tracked-hash identity, drift validation, and installed
parity pass. This phase names `reconcile-kbd-control-plane-projections` as that
owner. Direct edits to generator/plugin surfaces still require same-change
validation. The round-two and final sycophancy receipts are populated with their
actual PASS results rather than zero-byte files.

The post-policy review exposed a recursive packet defect: earlier review
receipts were included in the candidate diff and the judge then blocked because
no later PASS existed. The source packet builder now excludes only the current
target's review-receipt directory while retaining all implementation,
constraints, OpenSpec, and execution evidence. An MCP-only integration scenario
also proves the existing empty-URL guard exits silently without HTTP traffic.

## Passing review

The non-recursive packet received `PASS` from judge model `k3` with producer
`gpt-5.6-sol`, `cross_model_check: verified-distinct`, and a sycophancy score of
zero. It reported no critical findings, three warnings, and no suggestions. The
two actionable hardening warnings were applied: C-01 now scopes its prohibition
to parent-phase certification/final commit/push, and diff-mode packet building
fails closed on empty phase/target values or any leaked review-receipt diff.
