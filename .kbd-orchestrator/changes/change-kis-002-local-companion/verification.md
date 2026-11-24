# Same-machine Companion verification contract

Run at the final local integration boundary after the coherent production changes. The proposed Full pack entry point `scripts/verify-kbd-intersession.sh` must invoke Companion's real `scripts/verify-kbd-peers.py` and production binaries. The existing Python verifier has no per-case interface yet; extending its real-process cases belongs to the implementation. The pack wrapper must pass the selected case and propagate its actual result. No test was authored or executed during Spec.

## Fixture and receipts for every case

From the Full pack's registered checkout, set `COMPANION_ROOT` to the Companion checkout. Record the actual repository revisions and dirty-source digests when applicable, Cargo.lock SHA-256, installed pack CLI path/version/hash, both rebuilt binary hashes, supported/historical writer identities, per-case fixture input SHA-256, and baseline event hashes and frontier. The runner generates temporary roots with restrictive permissions and bounded socket names, starts actual processes, and proves both sessions were alive before the originating action. Record process IDs, distinct socket paths and data-root identities, CLI/HTTP exit or status, event IDs, hashes, ordered observations, retry results, and clean shutdown. Use bounded waits and preserve stage-specific startup errors. Remove temporary keys, pairing tickets, homes, registries and sockets on exit. Persist only redacted evidence and public identifiers. The fixture uses local discovery and loopback/same-host sockets with distinct identities; it needs no external host or production launch agent.

At the boundary, compile both binaries from the resolved Companion checkout with the locked workspace. Run the installed Full writer against disposable trusted project state. A missing supported release, historical fixture, usable local peer transport, or required production executable returns exit 2 (blocked). An unmet assertion returns a failing exit. Exit 0 requires every specified observable and deliberate rejection in that invocation to be checked. Preserve each original failing receipt alongside the result of the authorized repair. `--negative-only` actively exercises the stated failure controls and succeeds only if their rejection and invariant assertions hold.

## PEER-001 · Reader compatibility · PEER-01

- **Entry and boundary.** Use `prometheus kbd device enroll --command-id <id> --device-id <device> --public-key <pubkey> --operator` from the project, then start the real `prometheus-companion-headless` processes. Read `/ready`, project status and `/api/v1/kbd/projects/<id>/audit` directly over each node's own socket. Record that both readers replay the same signed grant, preserve all earlier audit hashes, and advertise the same enrolled active operators and nonempty frontier. Separately open the authentic older compatible signed history in a scratch store.
- **Negative.** Start the exact candidate reader binary (match the positive run’s SHA-256) on a disposable copy of a valid signed history containing a future event kind that this candidate does not support. Verify signature, signer enrollment, predecessor hash and sequence independently on the fixture before startup; hash both the history and the unsupported record. Require a diagnostic identifying the unreadable event kind or version, failed readiness for the affected project, and matching before/after signed event count, chain and original history bytes. An HTTP process liveness response may still succeed. The prior reader’s failure on the supported grant is baseline evidence only and cannot satisfy this negative control. A malformed record or invalid signature makes the fixture invalid. Missing valid fixture returns exit 2.
- **Final commands.**
```sh
bash scripts/verify-kbd-intersession.sh --case PEER-001
bash scripts/verify-kbd-intersession.sh --case PEER-001 --negative-only
```

## PEER-002 · Receiver-observed work · PEER-02

- **Entry and boundary.** Before submission record the two live session IDs/PIDs, their distinct checkout and authority roots, Companion process IDs, real socket addresses and A/B public key IDs. Session B establishes its own active subscription to B’s `/api/v1/kbd/projects/<id>/events/stream`; record the subscriber PID and start acknowledgment at B’s seed frontier. In session A, A’s pack CLI signs the unique command using A’s enrolled key with `PROMETHEUS_KBD_CONTROL_PLANE=1` and A’s control socket. Independently capture the actual POST to A’s `/api/v1/kbd/projects/<id>/commands` and its accepted response at the socket boundary (for example a transparent forwarding socket connected only to A). Correlate the command ID, originating A session and CLI PID, endpoint, request signer and A’s audit event signer. A successful exit or an event in A’s local store alone is insufficient. Read B’s own socket to establish its seed frontier and absence of the event before push; both original sessions must still be active. Use A’s `sovereign-sync --mode sign-kbd-push --project-id <id> --peer <B>` and POST the unchanged bytes to A’s `/api/v2/sync/pushes`. The sender receipt identifies B. The original B subscriber and B’s own audit yield the matching command ID, event ID, signature and integrity hash; equal frontiers and waypoint revision establish application. Record B’s receiver PID and its observation time. Repeat the POST, compare the full receipt, count exactly one matching audit entry, restart B and check again.
- **Negative.** Hold back the signed push after A records the original command while B's stream stays connected. B's actual audit has no matching event and its frontier remains at seed. A sender-side receipt or a local fallback cannot pass the oracle. After a successful push, try different, correctly signed content with the same request ID and require conflict, original immutable receipt, unchanged receiver event count and frontier.
- **Final commands.**
```sh
bash scripts/verify-kbd-intersession.sh --case PEER-002
bash scripts/verify-kbd-intersession.sh --case PEER-002 --negative-only
```

## PEER-003 · Rejection, uncertainty and reconnect · PEER-03

- **Entry and boundary.** Stop or disconnect Companion, use the normal pack CLI to commit locally, then restore the two real peers. Check B remains stale until explicit sync. Separately interrupt the real socket reply after A durably records a fresh command; preserve the stable command ID, verify the ambiguous result, and use A and B's direct audits to reconcile. On reconnect, verify both signed histories converge with exactly one effect. Replay of an already failed push must return the original failure receipt; a new signed push request can retry outstanding delivery of the original command.
- **Negative.** Give the unpaired third node a valid address; assert refusal. Keep transport paired for each other denial: use a project-unenrolled signer, enroll then revoke a signer before submission, and address the wrong project with valid keys for another project. Reset the baseline after enrollment or revocation and before the attempted push. Inspect the receiving authority directly after each attempt for an unchanged frontier and missing command; capture the sender/receiver denial or failure evidence. A sender-side transport acknowledgment cannot establish application success. Never weaken the original trust gate to make setup succeed.
- **Final commands.**
```sh
bash scripts/verify-kbd-intersession.sh --case PEER-003
bash scripts/verify-kbd-intersession.sh --case PEER-003 --negative-only
```

## PEER-004 · Bounded backlog and projection recovery · PEER-04

- **Entry and boundary.** Produce a representative backlog of valid signed commands via the real CLI/runtime in a private project; measure transmitted encoded sizes, not just `project.loro` on disk. Record accepted bounds, number of frames, bytes and interruption point. Interrupt after real progress, reconnect, finish the transfer, and compare the complete expected signed event set and final project frontier at B. No accepted frame may exceed the configured 16 MiB maximum. Where bounded incremental transfer is needed, verify the common baseline and resume only from a verified receiver state.
- **Negative.** Challenge the bound with a single oversized frame and require rejection before successful application; B's baseline stays unchanged. In a separate run, make only B's waypoint destination unwritable using a real temporary filesystem condition, leaving its journal writable. Verify the import is durable, the waypoint is stale and the condition is detectable; restore writability and exercise the supported operator recovery path. Require regenerated waypoint `sourceRevision` and `frontier` to match B's direct status, exactly one matching signed event and no new logical command from repair. A successful duplicate transport receipt is insufficient.
- **Final commands.**
```sh
bash scripts/verify-kbd-intersession.sh --case PEER-004
bash scripts/verify-kbd-intersession.sh --case PEER-004 --negative-only
```
