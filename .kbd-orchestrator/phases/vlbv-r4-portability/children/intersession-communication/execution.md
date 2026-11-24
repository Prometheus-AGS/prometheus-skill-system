# Same-machine KBD execution

The operator continued the bound revision 17 plan on November 24. Native KBD remains the execution backend. The existing Full and Companion checkouts contain the production inputs. The lead retains the single task journal. [Plan and task assignments](plan.md) · [Dispatch](execute-dispatch.json) · [Source identities](evidence/execute-inputs.json)

The Full CLI worker owns live reads, retries and watch, the Companion runtime worker owns both headless authorities and the dependency alignment, and the OpenSpec worker owns the historical strict inventory. They use native Codex gpt-6.2-sol at high effort. The Full lead handles the shared runner, state handoff, nested closure and installation health. Read the native change specification, its verification contract and the matching scenario in the adopted contract for every task.

The installed OpenSpec refresh selected 2.1.1 at execution preflight. The previously recorded selection was 2.1.0. Preserve evidence for both versions at verification. The full native installation requires a separate macOS GUI domain. No usable domain was found in the preflight. All scenario results remain pending.

## Implementation checkpoint

The Full CLI and team handoff changes cover resuming a running watch from a signed event, identifying the active read source, and retrying a claim or handoff against the original signed record. The local runner starts two sessions and records which process and authority supplied each observation.

Companion now aligns its four KBD dependencies, starts headless peer transport, and returns an acknowledgement after the receiving project authority applies an addressed update. Its backlog and recovery paths split outgoing history into bounded transfers, retain a verified frontier after interruption, and expose stale checkout projections separately from durable acceptance.

The Full installer and doctor changes record and inspect the selected installation profile. Strict-validation repairs are mapped to the 158 original Full items in [the recovery ledger](evidence/strict-recovery-ledger.json). These are implementation records. Neither the doctor nor OpenSpec has produced a new passing result.

The signed exchange with and without Companion can use disposable project directories on the current Mac. Only the full installer and LaunchAgent lifecycle check needs an isolated macOS login. The operator does not need to supply a login for the communication work. Keep the lifecycle check pending while the required environment is unavailable.

The PEER-004 runner covers bounded normal replication and a stale-waypoint recovery control, including replay of the identical signed request. The negative check for an individual frame over the receiver limit still lacks an independent observation at the receiving process. Automatic review stopped work on its proposed low-level sender. The runner reports this gap explicitly. Keep PEER-04 open.

The operator authorized signing the production implementation record and running local checks. The verified signature covers only that implementation status. The nine scenarios and their acceptance criteria are unchanged. The first local build started under the [boundary authorization](boundary-authorization.json). The isolated installer lifecycle remains pending.

The Full CLI build stopped at `commands/kbd.rs:2165` with Rust error E0277. The generic response decoder in `decode_commit` needs the concrete JSON value type. The independent reviewer confirmed that diagnosis and the signed contract change. No session launched. The [boundary results](boundary-results.json) record the failed build and each unexecuted check.

## Local repair and rerun

The operator authorized one type annotation and a local rerun with the installer deferred. Both candidate builds succeeded after restoring missing pinned Git objects in Cargo's cache. The original [B4 result](evidence/boundary/51214a4501f8c35a3c9d/boundary-results.json) and the [B5 result](evidence/boundary/c75e6420ad8d071c208c/boundary-results.json) are preserved.

All six local-session checks passed with 381 assertions. The independent reviewer confirmed a signed blocker and reply observed by the original watchers with Companion absent and present. Both current Companion authorities also read the enrolled co-operator grant. The next check tried to boot a historical authority from two isolated events at revisions 1357 and 1358. Their prerequisite events were missing from the fixture. The ordered run stopped before peer delivery, and the full installer remains pending.

## Historical replay and addressed delivery

With operator approval, PEER-001 now reads the complete 16-event journal and recorded Prometheus 1.13.0 state from the pinned Full v1.17.0 release. The positive and negative replay checks passed. PEER-002 and its negative control also passed. Companion B received the signed update once, its watcher and audit agreed with A, and a restarted B retained the event.

The independent reviewer verified the receipts. The ordered run stopped at PEER-003 before the offline command. A exited successfully, but its Unix socket pathname remained. The installer remains deferred. The [latest results](evidence/boundary/bbdd5e7edcfd32515d01/boundary-results.json) include the six previously passing local-session checks.

## Offline recovery

PEER-003 now probes the stopped Companion's Unix socket. A exited cleanly, the socket pathname remained, and the connection was refused. A committed the signed update offline. B saw it once after the recovery push. Lost-response and failed-delivery retries preserved the single durable effect. An independent review confirmed the 91 positive assertions and 101 assertions across the denial controls.

PEER-004 exercised an interrupted 40-command backlog. After B restarted with partial history, the resumed push returned HTTP 503. The saved response ends before the per-peer error detail. The ordered run stopped there and the installer remains deferred. [Run results](evidence/boundary/924a9130d0749a492237/boundary-results.json)

## Backlog recovery after receiver restart

The complete per-peer receipt showed a timeout connecting to B. B reopened on different UDP ports after the forced restart. Companion now uses the latest local mDNS address announcement for each paired endpoint when opening a direct connection. The diagnostic records which step times out and preserves the complete push receipt and interruption state. [Diagnostic observations](evidence/boundary/3bf466e844d60dbab4a6/run-PEER-004-19188ibi/observations.json)

The resumed push returned HTTP 201 and inserted the remaining 32 events. Both authorities finished at revision 43 with the same ordered hashes. The largest accepted frame measured 11,771,939 bytes. Stale projection recovery and replay of the same request preserved the audit; independent receiver rejection of oversized input remains unverified. [Recovery observations](evidence/boundary/3aa1f1f1a319d6b60a9a/run-PEER-004-jmyxq7ze/observations.json)

Strict OpenSpec validation passed on all 158 Full items and the Companion item. The named apply integration ran once and passed with zero skips. GREEN-002 failed to recognize Node's default human-readable test report because its parser expects TAP. The installer remains deferred. [Boundary results](evidence/boundary/3aa1f1f1a319d6b60a9a/boundary-results.json)

## OpenSpec reporter and packaged output

With the explicit TAP reporter, GREEN-002 recognized the named apply integration's pass. The subsequent distribution comparison found stale generated Claude plugin output. Inspection found outdated copies of four shared KBD scripts in both the Claude and Codex packages. The ordered run stopped before the negative controls. [Run results](evidence/boundary/bd1f803bc43670837dab/boundary-results.json)

The distribution generator refreshed 218 skills for Claude and Codex. All 194 ignored files in the original packages were preserved byte-for-byte. GREEN-002 then passed with 839 assertions, including the named apply integration and distribution comparison. Its 834-assertion negative run detected a missing change delta, a placeholder purpose and an omitted historical identity. [Distribution and OpenSpec evidence](evidence/boundary/8691c1404c3b13c0eca3/boundary-results.json)

The operator deferred installer execution to clean machines. Source inspection confirmed that Full and Mini look up versioned GitHub release assets and validate their checksums, and that the Docker installation paths select registry images by digest. The native installer and LaunchAgent lifecycle have no clean-machine certification from this run.

## Final local verification with installer checks deferred

Eight of nine scenarios passed. All 16 commands for those scenarios have verified receipts. The remaining scenario, GREEN-001, contains the two installer commands the operator deferred to clean machines. The adopted nine-scenario contract therefore still has an outstanding certification item. [Consolidated results](boundary-results.json)

Companion now remembers its local UDP ports per identity. Two independent signed transfers using fresh pairs recovered all 40 queued commands after the receiving process restarted. Both runs retained 11 events, added 32 and ended with identical ordered histories of 43 unique events, including three baseline events. [First transfer](evidence/boundary/aaf127b0bab4fcd6e9b6/run-PEER-004-8rz20v9p/observations.json) · [Independent repeat](evidence/boundary/aaf127b0bab4fcd6e9b6/run-PEER-004-gnnmxa5t/observations.json)

A running Companion receiver exercised its production frame reader with local files. It accepted exactly 16 MiB and rejected one additional byte. Its signed audit, revision, frontier, persisted document and waypoint matched the pre-check observations. [Receiver evidence](evidence/boundary/aaf127b0bab4fcd6e9b6/negative-PEER-004-qrcgd90m/observations.json)

## Representative release download accepted

The operator accepted one live download as sufficient evidence for release availability in this phase. On November 24, 2026, Full’s production release fetcher downloaded the macOS ARM64 archive for pack v1.17.0 from GitHub into an empty temporary stage. It verified the archive, manifest and 23 packaged files. The staged CLI exited successfully with `prometheus 1.15.0`. GitHub also listed assets for macOS, Windows and Linux, each for ARM64 and x86-64, on both Full v1.17.0 and Mini v1.20.1.

The release-download acceptance item is closed. Clean-machine installation, service startup and Docker pulls remain deferred. The original GREEN-001 execution receipts retain their unexercised status. [Download receipt](evidence/release-download-sample.json)
