# Tasks — change-drt-007-install-surface-repair

- [x] 1 Add EnvironmentVariables with a __PROMETHEUS_PATH__ placeholder to the research plist template, matching the shared/launchagents convention, and extend the sed in install-binaries.sh to substitute it
- [x] 2 Confirm by command that the research plist is NOT a services-manifest input (generate-service-manifest.mjs reads only shared/launchagents and shared/systemd), record the correction to analysis D-11b in the spec, and raise the migration question rather than silently moving the file
- [x] 3 Add a daemon self-check reporting the resolved harness path and driver path with sizes, so a stale install is visible before a job is started
- [!] 4 Republish the plugin generation so the installed driver matches the repo; prove the generator idempotent and pass the drift validators (C-01)
- [x] 5 Write tests/installed-service-smoke.sh: render and install the plist, bootstrap under launchd, start a fixture job, assert a non-null harness_pid; AND install a marker-less stub driver fixture and assert the daemon health output reports it stale with both paths and sizes; then record the repair against D-A and D-B

`[!]` = BLOCKED with a recorded reason (see tasks.json).
