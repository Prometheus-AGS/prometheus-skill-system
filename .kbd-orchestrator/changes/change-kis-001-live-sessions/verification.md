# Verify two running sessions

**Run location.** Registered Full pack root. Set COMPANION_ROOT to the Companion checkout on the same machine. The change introduces scripts/verify-kbd-intersession.sh as a dispatcher for the entire child. Its default success path exercises each named scenario through production binaries; --negative-only executes its independent failure controls. These are specified commands for the final local boundary. No such result is claimed here.

**Fixture.** Create two independent long-lived session processes with separate stdin/stdout streams, session IDs and reader cursors. Give them the same project authority and appropriate enrolled disposable key material. Create a second registered UUID as a control. Allocate private data roots, socket paths and short-lived team-state storage; retain public fingerprints, binary/source hashes, fixture manifest digest and receipts, then remove private keys, sockets, workspaces and processes. Never touch the operator’s live journal, launch agents or login credentials. Missing prerequisites or absent assertion evidence block the gate; exit 2 or a skip cannot pass.

| Case / owner | Production path and real boundaries | Required positive evidence | Independent negative control |
|---|---|---|---|
| LOCAL-001 / LIVE-01 | Open two separate live sessions running `prometheus kbd --path "$PROJECT" watch`. Establish both subscriptions, then run `blocker record --command-id "$ASK" --id "$BLOCKER" --summary "Review is blocking release"` and `blocker clear --command-id "$ACK" --id "$BLOCKER" --resolution "Reviewed and ready"` via the KBD CLI in A and B. Exercise native KBD journal/process boundaries and, for the connected variant, the actual Unix control socket. | B observes ASK and A observes ACK while original readers live; capture exact payload, command and event IDs, PID overlap, independent audit, resume after a missed event and final revision. Both no-Companion and single-Companion modes pass. | A third project with the same blocker label receives neither command. Stop the targeted endpoint and require the receiver-only proof to fail. |
| LOCAL-002 / LIVE-02 | `prometheus kbd status --json`, `audit --json`, `watch`, typed commands and `claim` in independent live processes. Exercise socket and on-disk journal; introduce real connection loss before dispatch and after a durable commit. | Local source explicitly reported on permitted fallback; stdout contracts preserved; original watcher PID survives loss and reports return to Companion; its post-reconnect event must be traceable to a response from the restored socket or absent from the fallback store, with no omissions or duplicate effects at the cursor; journal shows one logical effect; both claim clients agree or report the same explicit conflict. | Populated local history cannot convert denial, wrong-project or incompatible remote data into remote success. A denied mutation produces no local commit. A watcher that remains local after reconnect fails the remote-source assertion. |
| LOCAL-003 / LIVE-03 | Full agent-team-creator's `scripts/cli.mjs handoff-create --input <create.json>` and `handoff-accept --input <accept.json>`, two separate sessions, single local team-state file and real artifact in Git checkout. | Independent read before/after, same accepted packet and revision on replay, reachable evidence and correct new owner. | Wrong recipient, stale revision, and acceptance against an isolated copy cannot change the original state. |

Run each command from the pack root after the complete production batch:

```verify
bash scripts/verify-kbd-intersession.sh --case LOCAL-001
bash scripts/verify-kbd-intersession.sh --case LOCAL-001 --negative-only
bash scripts/verify-kbd-intersession.sh --case LOCAL-002
bash scripts/verify-kbd-intersession.sh --case LOCAL-002 --negative-only
bash scripts/verify-kbd-intersession.sh --case LOCAL-003
bash scripts/verify-kbd-intersession.sh --case LOCAL-003 --negative-only
```
