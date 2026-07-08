# Tasks — change-prb-006-launchd-plist

- [x] Create `substrate/prometheus-research/com.prometheus.research.plist` with `<HOME>` placeholder
- [x] Add `prometheus-research` build+install section to `scripts/install-binaries.sh`
- [x] Add launchd bootstrap logic to install section (with bootout guard)
- [x] Read `scripts/install-binaries.sh` first, then Edit to insert the new section
- [x] Smoke test: `bash scripts/install-binaries.sh` completes without error
- [x] Verify: `launchctl list | grep prometheus.research` shows the service entry
