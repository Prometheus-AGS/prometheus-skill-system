# Authorisation — surreal-memory-server, 2026-07-31

**GRANTED, narrowly.** The user selected *"Yes — authorise this one-line fix"*.

## Scope

- **Granted:** add `rev = "a64be231..."` to the `rmcp` git dependency at
  `tools/surreal-memory-server/Cargo.toml:42`, and prove it with
  `rm Cargo.lock && cargo check --lib`.
- **NOT granted:** anything else in that repository. In particular, its
  `rmcp::model::Content` usage is **not** ported — that is upstream source
  needing its own decision.
- The user declined the broader "authorise the whole repo" option, so a future
  change touching that repo must ask again.

## Why the fix is safe

`a64be231527f923e9f84d4dd7bf3c3bd695ee53e` is **what the committed lockfile
already resolves to**. Pinning it changes nothing today; it only stops
`cargo update` from floating past the lock to branch HEAD.
