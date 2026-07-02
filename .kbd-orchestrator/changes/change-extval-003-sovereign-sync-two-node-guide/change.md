---
id: change-extval-003-sovereign-sync-two-node-guide
title: Two-node sovereign-sync setup guide
phase: phase-external-validation
priority: P1 (enables G3)
agent: claude-code
status: done
scope:
  - docs/SOVEREIGN_SYNC_TESTING.md
---

# change-extval-003-sovereign-sync-two-node-guide — Two-node sovereign-sync setup guide

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

- Document binary prerequisites (sovereign-sync binary must be built first)
- Write Docker Compose two-node setup (two service definitions, overlay network)
- Write manual two-host setup (SSH, copy binary, environment variables)
- Document step-by-step sync verification sequence (start, push, share ticket, import, verify)
- Document expected output at each step
- Write troubleshooting section (firewall, QUIC UDP port, iroh NodeAddr format)
- Smoke-test the Docker Compose setup locally before finalizing

## Tasks

- [x] 1. Document binary prerequisites (sovereign-sync binary must be built first)
- [x] 2. Write Docker Compose two-node setup (two service definitions, overlay network)
- [x] 3. Write manual two-host setup (SSH, copy binary, environment variables)
- [x] 4. Document step-by-step sync verification sequence (start, push, share ticket, import, verify)
- [x] 5. Document expected output at each step
- [x] 6. Write troubleshooting section (firewall, QUIC UDP port, iroh NodeAddr format)
- [x] 7. Smoke-test the Docker Compose setup locally before finalizing
