## Context

See proposal and phase analysis. `ChangeRegistered` already rejects duplicate map keys. Legacy collection can overwrite duplicate embedded IDs; boundary resolution uses qualified map keys before ID/title fallback. These observations do not reproduce the reported 1.11.3 incident.

## Goals / Non-Goals

Provide safe diagnosis and repair in the existing signed runtime. Do not repair the owner's live incident without its journal and explicit mapping, change closure semantics, or rewrite prior events. Recalled lesson: “issue #194 incident root cause remains unconfirmed without affected journal/version and map-key versus embedded-ID inspection.”

## Decisions

1. Keep phase-scoped identity and existing duplicate-key rejection. Inspect keys and embedded IDs before conversion. Refuse ambiguous import with original bytes preserved; no last-row-wins deduplication.
2. Add proposed CLI `prometheus kbd --path <project> change repair-identity --command-id <id> --phase <phase> --expected-revision <revision> --resolution-file <mapping.json> --reason <reason>`. This is a new interface to implement, not an existing command. The mapping binds source record keys/content digests, survivor key, explicit task mappings and every conflicting status/receipt disposition. Reject unknown fields and unresolved collisions.
3. Append a typed signed repair event through current command authority and optimistic revision checks. Store retained source snapshots and provenance inside the event/owned receipt. Effective identity resolves retired keys without erasing their history. Repeated command ID/content returns its prior result; changed content under the same ID refuses. Consumed budgets and real task evidence never reset.
4. A repair mapping cannot manufacture PASS, task completion, evidence or a start receipt. If outcome or receipt reconciliation requires a separate authorization, fail with that named prerequisite. Projection repair regenerates only confirmed runtime-owned views.
5. Refine CLI/reconcile diagnostics before repair attempts. Include phase, key, embedded ID, competing key/title and receipt identity as applicable. Continue using the existing task mapper; distinguish zero matches from multiple matches.

## Risks / Trade-offs

- Replay of a historically malformed signed import may need a narrowly scoped compatibility reader to expose the records for repair. Preserve cryptographic integrity and original bytes; do not weaken ordinary registration or silently normalize conflicts.
- The original incident fixture is unavailable. Qualify specified damaged-state families locally at phase end; report the actual incident as NOT_EXERCISED until its evidence is available.

## Migration Plan

No automatic migration or live repair. Ship additive diagnosis/repair and documented invocation. Refused operations preserve state; successful repair is append-only and its reversal requires another explicit mapping, never deletion of the repair event.

## Ownership and ordering

This change owns runtime identity/import/repair, CLI identity diagnostics and apply/reconcile handling. It does not edit bootstrap/team/BDD surfaces. Final generated-output owner: kbd-194-196-delivery-qualification. No tests or implementation review before all parent production work completes.

## Analyze reuse evidence

- library: cand-001 — Existing signed KBD runtime and apply/reconcile path (adapt). Add targeted identity inspection and an explicit typed, signed repair path only for confirmed canonical damage; preserve existing guards and receipts. Readiness: scenarioMatrix, evidenceInvalidation and boundaryPacket; current runtime does not expose the needed repair command.
  - Source: https://github.com/Prometheus-AGS/prometheus-skill-system/blob/bfe572ff8a97a21c7610c44ea6b39d5f97a857b7/substrate/kbd-runtime/src/lib.rs#L2073 — New ChangeRegistered events already reject an existing phase-map key.
  - Source: https://github.com/Prometheus-AGS/prometheus-skill-system/blob/bfe572ff8a97a21c7610c44ea6b39d5f97a857b7/substrate/kbd-runtime/src/lib.rs#L6897 — Legacy conversion collects rows into a BTreeMap by embedded ID; duplicate rows can overwrite earlier values.
  - Source: https://github.com/Prometheus-AGS/prometheus-skill-system/blob/bfe572ff8a97a21c7610c44ea6b39d5f97a857b7/tools/prometheus-cli/crates/prometheus-cli/src/commands/kbd.rs#L805 — Boundary resolution is active-phase scoped; qualified task lookup uses map keys before an ID/title fallback.
  - Source: https://github.com/Prometheus-AGS/prometheus-skill-system/blob/bfe572ff8a97a21c7610c44ea6b39d5f97a857b7/skills/process/kbd-process-orchestrator/skills/kbd-apply/reconcile.mjs#L54 — Duplicate progress IDs currently cause a generic scan error.
- library: cand-002 — Existing adopted contract, decisions, readiness and typed phase closure (adopt). No parallel ledger or generic closure replacement. Readiness: identity, adoptionSource, scenarioMatrix, budgets, workerRoutes and boundaryPacket; existing machinery can support the integration, but #194 repair still needs production code.
  - Source: https://github.com/Prometheus-AGS/prometheus-skill-system/blob/bfe572ff8a97a21c7610c44ea6b39d5f97a857b7/skills/process/kbd-process-orchestrator/references/schemas/adopted-contract.schema.json — Contract has stable scenario IDs, oracle, negativeControl, budgets, evidence invalidation, workerRoutes and a single post-implementation boundary packet.
  - Source: https://github.com/Prometheus-AGS/prometheus-skill-system/blob/bfe572ff8a97a21c7610c44ea6b39d5f97a857b7/substrate/kbd-runtime/src/phase_closure.rs#L775 — Typed closure already handles pass/refusal and refuses unsupported cancelled/deferred dispositions.
