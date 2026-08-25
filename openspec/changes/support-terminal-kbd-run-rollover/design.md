## Context

The canonical runtime stores signed events per replica and folds them into one project document. `RunInitialized` is currently restricted to revision zero, lifecycle terminals have no outgoing transition, and the CLI documents cancellation as requiring a new run without offering that operation.

## Goals / Non-Goals

**Goals:**

- Add a successor boundary without replacing the project manifest or deleting prior events.
- Keep local CLI execution, remote control-plane execution, CRDT folding, and compatibility projections aligned.
- Make `/kbd-new-phase` safe immediately after cancellation.

**Non-Goals:**

- Resuming or rewriting a terminal run.
- Changing project identity, replica adoption, or historical event signatures.
- Introducing a second journal format or storage migration.

## Decisions

### Reuse `RunInitialized` as the run boundary

Add optional `previous_run_id` and `reason` fields and allow this event after revision zero only when the current lifecycle is terminal. This keeps initialization semantics in one event kind and lets existing schema-v2 readers ignore the additive payload fields. A new unrelated event kind was rejected because it expands the wire vocabulary without improving the state model.

### Add a dedicated `RunStart` command

The command envelope targets the current run and supplies the successor ID, reason, and optional exact next work. Preparation emits `RunInitialized` with the successor as the event's top-level run ID. This preserves optimistic concurrency and makes retries idempotent through the existing command ID ledger.

### Reset run-scoped fields in the reducer

On successor initialization, replace lifecycle, plan revision, checkpoint, exact-next work, active path, phases, completion, decisions, blockers, and claims with defaults. Preserve causal, device, signing, pin, replica, and command authority fields. Starting a successor requires no unresolved lifecycle conflict.

### Fold ordered run IDs instead of counting distinct IDs

Remove the `run_ids.len() > 1` merge shortcut. Causally ordered initialization boundaries update the current run normally; concurrent lifecycle-slot candidates continue through deterministic conflict selection.

### Commit before releasing PAUSE

The CLI submits the signed command, writes compatibility projections, and only then calls the existing pause-valve release operation. Failed durability or projection leaves the valve untouched.

## Risks / Trade-offs

- **Older running control-plane binaries reject the new command semantics** → Build and restart the CLI and Sovereign Sync consumer before emitting a successor event.
- **A reducer reset could erase project authority accidentally** → Unit tests assert the exact preserved and reset field sets.
- **Concurrent rollover can expose weak conflict scoping** → Add a two-replica test that proves only concurrent successors conflict.

## Migration Plan

1. Validate and build the updated runtime, CLI, and Sovereign Sync consumer locally.
2. Replace the installed binaries and restart only Sovereign Sync.
3. Start the successor run through the CLI and verify audit, status, projections, and PAUSE handling.
4. Roll back binaries if needed; do not delete or rewrite any committed rollover event.
