# change-extval-003-sovereign-sync-two-node-guide

**Phase:** phase-external-validation  
**Type:** documentation  
**Status:** proposed  
**Priority:** P1 (enables G3)

## Summary

Write `docs/SOVEREIGN_SYNC_TESTING.md` — a guide for validating sovereign-sync P2P
CRDT sync across two distinct network namespaces (Docker Compose or two physical/
virtual hosts).

## Motivation

G3 requires two-node sync across distinct machines. The same-host `sync_roundtrip`
test confirms internal correctness but not real P2P transport. This guide enables
any external tester with Docker or a second machine to reproduce the scenario.

## Deliverables

- `docs/SOVEREIGN_SYNC_TESTING.md`

## Tasks

- [ ] Document binary prerequisites (sovereign-sync binary must be built first)
- [ ] Write Docker Compose two-node setup (two service definitions, overlay network)
- [ ] Write manual two-host setup (SSH, copy binary, environment variables)
- [ ] Document step-by-step sync verification sequence (start, push, share ticket, import, verify)
- [ ] Document expected output at each step
- [ ] Write troubleshooting section (firewall, QUIC UDP port, iroh NodeAddr format)
- [ ] Smoke-test the Docker Compose setup locally before finalizing
