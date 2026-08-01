# Tasks

- [ ] Re-run cargo tree -p rmcp --depth 0 on all five crates; correct the record if it disagrees
- [ ] Re-run grep -n protocolVersion on stdio_client.rs and mcp_client_pool.rs
- [ ] Re-run grep -rn Mcp-Session-Id across the pack and UAR to confirm nothing uses it
- [ ] Write the decision via decision-log.sh with alternatives, a falsifier, outcome_status pending
- [ ] Pass --mode decision review with cross_model_check verified-distinct
- [ ] Write NO code in this change
