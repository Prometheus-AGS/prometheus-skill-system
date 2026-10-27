---
type: SessionRecord
id: karpathy-session-c7308864949dce59
title: Karpathy session c7308864949d
tags:
- karpathy
- session-learning
sources:
- resource: session:cfc5bf77-8f81-45e6-975a-0283553c3962
generated:
  by: pk/1.11.0
  at: 2026-10-09T12:09:58.401533+00:00
created_at: 2026-10-09T12:09:58.401533+00:00
updated_at: 2026-10-09T12:09:58.401533+00:00
revision: 0
---

## Delta

Four of the six warnings have a known cause. The Codex memory warning is only partly explained, and I found a regression I caused. I didn't change anything in this investigation.

**A regression I caused: `surreal-memory-server` is back on v1.8.0.**
- Before the pull, the submodule was checked out at `0af8ae1` (v1.10.0). The superproject still pins `4161fc4`, which is v1.8.0 plus 22 commits.
- Running `git submodule update`, and then `install-binaries.sh`, built and installed that older version. The running server now reports 1.8.0.
- Upstream already has v1.10.0 and v1.10.1. Restoring v1.10.x means checking out the tag in the submodule, rebuilding (one cargo build) and re-signing the binaries as before. The pin in the superproject is stale and needs its own upstream PR.
- The Metal errors below started at 08:15Z today, before I touched anything, so the downgrade is not their cause. I don't know whether v1.10.x fixes them.

**The warnings**
1. **Learning backlog:** 93 jobs are in flight, 24 of them stale (the oldest for 90 hours), with nothing failed or rejected. The embedding step fails repeatedly with `Metal error: Could not lock resource: poisoned lock` (about 21,000 log lines), which pauses memory operations. The doctor's own "unreachable on 23001" note looks stale, because its separate surreal-memory check passes (ready, ingestion and search both true). A restart won't clear the backlog; the doctor says to use the quarantine/release or per-operation retry commands. The Metal error also needs its own fix.
2. **KBD runtime authority:** the control endpoint answers `unknown KBD project` (404) for this repo. I didn't look further. It may mean this project isn't registered, or that the repo root isn't being passed.
3. **KBD rollout gate:** this is expected, not a fault. Promotion is blocked until 7 successful days, 100 real mutations and 10,000 synthetic ones; you have 1 day and 2 mutations.
4. **Discovery budgets:** none of the four harnesses has been measured. Running the measurement should clear it. I haven't run it.
5. **Codex memory:** the config is `generate_memories=false` and the other two keys unset. The doctor says to set all three to false and archive the memory summaries, but those summary files are absent anyway. It also claims the installed generation or memory helper can't be verified, which contradicts the other checks. I didn't find why.
6. **Marketplace source:** your uncommitted wiki files. This one clears when you commit or discard them.

**Decision for you:** should I restore the server to v1.10.1 (or v1.10.0, which you had before), rebuild and reinstall it? I'd start there because it's the one thing I broke. After that I could take the Metal error, then the Codex memory check.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: cfc5bf77-8f81-45e6-975a-0283553c3962
- Captured: 2026-10-09T12:09:56.543542Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-system

## Changed Paths

- crates/prometheus-exec/.prometheus/
- history.txt
