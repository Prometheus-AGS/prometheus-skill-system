# Communication between two running KBD sessions on one machine

The release claim is that two people or harnesses working on one Mac can exchange signed work updates and continue their work even when Companion is stopped. Both sessions must already be listening when the first update is issued, and both remain alive until the reply is observed. The same exchange must work through one shared local Companion. Two independent headless Companion processes on the Mac exercise the separate replication path without requiring another machine.

| Priority | Change | Observable outcome |
|---|---|---|
| Primary | [Two live sessions](../../../../changes/change-kis-001-live-sessions/spec.md) | A reports “Review is blocking release”. B sees it and responds “Reviewed and ready”. A sees B’s resolution in the original reader. Watch resumes across interruption, access failures are visible, and ownership transfers once on the shared local team state. |
| Required | [Companion on this machine](../../../../changes/change-kis-002-local-companion/spec.md) | The installed writer and candidate reader agree on the co-operator history. A’s signed push reaches B’s independent audit, event stream and checkout. Retries do not duplicate work; rejected peers and incompatible events preserve history. |
| Required | [Previously failing local suites](../../../../changes/change-kis-003-local-health/spec.md) | Reconcile all nine doctor failures and the discovery warning across the actual installation profiles. Pass strict validation for Full’s entire recorded 158-item inventory, including 130 previously failing items, and preserve Companion’s passing strict check. |

The [contract proposal](adopted-contract.json) maps nine cases to nine registered tasks and eighteen local positive and negative gate commands. Execution must retain evidence that each condition was actually exercised. A successful health endpoint, a local read substituted for a remote observation, or the absence of a failing test after deleting its input does not certify this phase.

The proposed observation deadline is 30 seconds per live update on this host. The release claim covers structured KBD work updates and same-authority task handoff. Free-form private chat, file replication and physical LAN discovery are separate product capabilities. Each session uses disposable project data for certification. The operator’s production journal and launch agents remain available.

## Readiness for Plan

The specifications are authored. The release and its tests remain pending. The current adoption helper derives the contract path from the literal canonical ID containing `::`; this nested phase is physically stored under `children/intersession-communication`. Draft lint and formal adoption therefore have different outcomes. Before adopting, use a version of the helper that resolves the registered parent/child directory and verifies the original IDs and evidence. Keep the canonical ID and re-run the strict read-only check on the actual child file. A copy or symlink in the wrong directory is not adoption evidence. The native specifications are hash-bound through calibration; the OpenSpec-only digest helper has no delta for this native phase.
