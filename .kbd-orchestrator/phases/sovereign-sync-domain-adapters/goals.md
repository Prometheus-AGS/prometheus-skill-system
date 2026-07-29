# Goals

- Wire skill-index domain adapter (Public CRDT/index rebuild) into sovereign-sync P2P push/pull
- Wire learner-model:<learner-id> domain adapter (Trusted CRDT merge) into sovereign-sync P2P push/pull
- Wire kbd-presence:<project-id> domain adapter (Trusted ephemeral CRDT) for non-authoritative KBD presence
- Define and enforce domain envelope validation (project/learner identity, privacy class) before any peer accepts a delta
- Add end-to-end replication proof per data-scope.md: source version vector, bytes transmitted, destination import/commit result, content-level assertion
- Explicitly exclude surreal-memory, secrets, raw transcripts, and KBD authoritative Raft state from any CRDT sync domain
