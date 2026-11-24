# Decisions for Spec · November 24, 2026

- Keep one signed authority per registered project on each device. Use the existing Full handoff flow when both sessions address one authoritative local team-state file. Cross-device journal delivery cannot change ownership there. A session can continue locally while Companion is absent.
- Propose the Full v1.17.0 revision as Companion’s compatibility target. Preserve the two enrolled operators in the scenario that exposed the old reader. This analysis does not authorize a release or change that pin.
- Keep addressed peer sync explicit. Use the receiver's independent audit and frontier as the application evidence. Preserve separate results for local commit, local fallback, transport receipt and applied history.
- Measure the encoded history sent for a representative project. Keep bounded delivery and test restart and retry. Distinguish one-host process verification from physical LAN evidence.

[Decision rationale, evidence and outstanding checks](analysis.md)
