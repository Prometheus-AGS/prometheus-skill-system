# Tasks

- [ ] Restart sovereign-sync daemon on the Mac Pro and confirm `/health` OK
- [ ] Restart sovereign-sync daemon on the laptop with the paired
      `operator_id` + bootstrap endpoint in `config.toml`
- [ ] Confirm `GET /api/v1/sync/status` on each shows the other as a peer
      (non-empty `peers` array) within a reasonable window
- [ ] `POST /api/v1/sync/push {"domain":"skill-index"}` on machine A
- [ ] Confirm machine B's skill search reflects machine A's content
- [ ] Record outcome (pass/fail + logs) in this change's proposal.md or a
      follow-up change if it fails
