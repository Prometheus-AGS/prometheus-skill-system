# Verification — change-cpc-006-discovery-and-pairing

Repository: `prometheus-companion`  
Depends on: change-cpc-004-relocate-sovereign-sync

## Acceptance criteria

- `--test discovery` passes with both endpoints started and **no address configured** (each side is given only the other's EndpointId); unauthorized-peer rejection asserted.

  **Criterion corrected during execution.** It originally said "bootstrap empty". That is
  not achievable: `iroh-gossip` 0.101 joins a topic via `ProtoCommand::Join(bootstrap)`
  (`net.rs:641`), so an empty bootstrap set joins nothing and no neighbour can ever
  appear — the crate's own tests pass the peer id and let discovery supply the address.
  The supportable claim, which the test now asserts, is that no *address* is configured.
- Default config registers no Mainline lookup (asserted by inspecting the endpoint's lookup list); `dht = true` registers it.
- `grep -in noise docs/00-architecture-and-implementation-plan.md` returns nothing; `bash scripts/audit-all.sh` exits 0.

## Verify commands

Every acceptance criterion above maps to a command here; run from the repository named above, locally, after the edit batch.

```verify
grep -q 'iroh_mainline_address_lookup' versions.toml && grep -q 'iroh_mdns_address_lookup' versions.toml
! grep -Ei 'noise (handshake|protocol|xx|ik)' docs/00-architecture-and-implementation-plan.md | grep -vi 'not used\|no noise\|never a noise\|removed by'
cargo test -p sovereign-sync --test discovery
bash scripts/audit-all.sh
```

**Two gates above were rewritten during execution because they asserted the wrong thing:**

1. The pin grep used hyphens (`iroh-mdns-address-lookup`). `versions.toml` keys use
   underscores — a bare TOML key cannot contain a hyphen — so the gate failed against
   pins that were present and correct. Same trap as `dirs_next` in change-cpc-003.
2. `! grep -qi noise` failed on the very sentences that *record* the correction
   ("Not used", "There is no Noise handshake"). A gate that a successful fix makes fail
   is measuring the wrong thing; it now matches only affirmative Noise claims.
3. The test-name list was stale after the tests were renamed to state what they check.
   Running the whole target cannot silently skip a renamed test.

## Evidence

Run locally 2026-09-03 in `prometheus-companion` (HEAD `773aa5e`). No hosted CI.

| Gate | Result |
|---|---|
| `versions.toml` pins both lookups | PASS (keys use underscores) |
| No affirmative Noise claim in the plan | PASS |
| `cargo test -p sovereign-sync --test discovery` | PASS — 4 passed, 0 failed |
| `bash scripts/audit-all.sh` | PASS — 7 pass, 0 fail, 3 skip, exit 0 |

Also run: `cargo clippy -p sovereign-sync -p prometheus-substrate --all-targets` clean;
`cargo fmt --all -- --check` clean (fmt run after clippy, then tests re-run);
`cargo test -p prometheus-substrate --lib` 11 passed.

**The discovery test, and why the first version proved nothing.**

Two endpoints became gossip neighbours in **755 ms – 1.0 s** given only each
other's `EndpointId` — no IP, port, or relay URL in the fixture. Verified as
real by a **negative control**: rerunning with `mdns: false` produces
`A sees [], B sees []` for the full 45 s budget. Discovery is doing the work.

Two fixture defects were found and fixed before this held:

1. `paired_identities` copied A's identity *file* to share the group secret,
   which also copied the endpoint secret — both nodes came up as the **same**
   endpoint. Caught by the distinct-identity assertion. Rebuilt on the
   production `export_ticket` / `import_ticket` path, which shares the group
   secret while leaving each endpoint key alone.
2. The test then ran the full 45 s and **reported `ok` while skipping** — the
   skip branch printed to stderr, which a plain `cargo test` run hides. A test
   that passes without exercising its subject is worse than a failing one; the
   skip is now read from `--nocapture` output, never inferred from the exit code.

**Scope limit, stated rather than papered over:** this is not zero-config
discovery. A peer's public `EndpointId` must still be known, because gossip
joins via `ProtoCommand::Join(bootstrap)`. What is removed is address
configuration, so a peer that changes network keeps working. Goal 3 should be
assessed against that narrower claim. `crates/sovereign-sync/src/config.rs` was
amended so its doc comment no longer overstates this.
