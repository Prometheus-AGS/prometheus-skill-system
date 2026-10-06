# Production integration acceptance

Use this contract for every behavior under specification, in native-kbd
`verification.md` and OpenSpec scenario/verification artifacts. Fill each field
with project-specific requirements; placeholders are not acceptance evidence.
Keep backend task IDs unchanged and link each behavior to its owning task and
the parent phase's final integration gate.

## Behavior: <concrete user operation>

| Field | Required declaration |
|---|---|
| Canonical identity | <phase / change / stable backend task ID> |
| Source identity | <repository, revision or candidate tree, generated package identity> |
| Production entry point | <shipped CLI/installer/server/UI path and exact invocation or user operation> |
| Real collaborators | <actual services, subprocesses, stores and installed artifacts used by that entry point> |
| Boundary exercised | <process, filesystem, network, database, protocol or UI boundary; name endpoints/paths> |
| Observable result | <externally observable state/output, including persistence or restart when relevant> |
| Negative control | <real invalid/absent/denied/conflicting input and expected externally observable rejection; no replacement mock> |
| Isolation | <scratch HOME, CODEX_HOME, CORTEX_DATA_DIR, plugin/queue/database roots, service ports and cleanup owner> |
| Prerequisites | <real tools/services, protected approvals and required source/packaging completion> |
| Final local gate | <exact command, argv, cwd and environment references; gate runs only at the completed-production boundary> |
| Evidence | <local receipt/log path, source identity, exit code and actual observed result; pending until run> |
| Limitations | <unsupported platforms/configurations and any behavior this gate cannot establish> |

### Execution boundary

Complete the entire parent phase's coherent production implementation before
authoring, modifying or executing tests. Specify acceptance requirements now;
write executable scenarios and run the smallest full integration batch only at
the final boundary. Fix failures in coherent batches and rerun the applicable
confirming final gate before commit/push/release. One machine-wide Cargo/rustc
build at a time; use existing repository build rules and isolated targets.

Exercise the installed/generated deliverable when packaging or installation is
the behavior under specification. Calling an internal helper directly does not
establish installer dispatch, executable modes or installed-generation behavior.
Use actual collaborating processes and stores; unit tests, structural assertions,
mock-only probes and snapshots are supplemental and cannot satisfy acceptance.

Do not touch real harness/memory roots or live service ports during tests. Declare
every test resource and cleanup boundary. Missing services, tools, approvals or
usable isolation are BLOCKED, not skipped success. Exit 2 is always BLOCKED;
exit 0 counts only when the declared observable and negative control were actually
exercised. Record source implementation, integration acceptance, publication and
installed operation separately. Never fabricate receipts or substitute hosted CI.
