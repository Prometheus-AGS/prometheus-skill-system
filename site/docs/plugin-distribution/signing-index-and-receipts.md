---
title: Signing, indexes, and receipts
description: Manifest trust, shared skill selection, transactional activation, and target evidence.
---

# Signing, indexes, and receipts

Each plugin generation is a signed, immutable transaction containing the
payload and every search-index projection. Activation switches the store pointer after its staged verification. Actual target copies and separately configured clients need their own projection and loaded-artifact evidence. A running client can retain older state.

```mermaid
flowchart LR
  Source["Repository + pinned gitlinks"] --> Index["Canonical skill index"]
  Source --> Manifest["Canonical generation manifest"]
  Index --> Manifest
  Manifest --> Sign["Ed25519 signature"]
  Sign --> Verify["Independent trust-store verification"]
  Verify --> Receipts["14 signed target receipts"]
  Receipts --> Current["Atomic current pointer"]
  Current --> Host["Host search"]
  Current --> Agent["Generated agents"]
  Current --> Mobile["Mobile FFI"]
```

## Provenance and trust

The manifest includes the source commit, source-tree state, external source
gitlink commit SHAs, every file hash and mode, target payload modes, hook bundle
identity, and the canonical skill-index hash. It is signed with Ed25519. The
private key and `trust/allowed-signers.json` use private permissions and are not
part of the generation payload.

First installation can enroll the local signer only when no trust store exists.
Once a trust store exists, an unknown signer is rejected. Verification checks
the public-key fingerprint, signature envelope, canonical manifest bytes, file
inventory, and index receipts before activation.

## One index implementation

The generation carries byte-identical host, generated-agent, and mobile index
projections. The shared Rust `skill-index` crate verifies the index SHA-256 and
provides deterministic ranking. Local index consumers can use that selector. The mobile FFI accepts caller-provided `index_json`; it does not independently authenticate generation signatures. Connected-client host search belongs to its relocated owner. A parity receipt prevents a target-specific index from drifting
silently.

## Target receipts and collisions

All 14 supported targets receive a signed receipt containing the generation,
signer, payload mode, payload hash, and skill-index hash. Symlink and copy
targets are verified according to their projection mode. A missing, unsigned,
or mismatched receipt fails verification.

Existing non-owned paths and bundle identities are collisions, not files to
overwrite. A bundle name may be reused only when it resolves inside the
generation store and its verified bundle identity and dispatcher hash match.

## Activation and rollback

The installer verifies a complete staged generation, writes signed receipts,
updates `previous`, and atomically switches `current`. Stable dispatchers and
the stable index resolve through `current`. Rollback selects `previous`,
reprojects copy targets, verifies all receipts, and then swaps the pointers.

Use `node scripts/install-plugin-generation.js --verify` for a read-only check after phase production. A nonempty inherited `CODEX_HOME` is normalized and wins over `--home` for the selected Codex projection. Logical signed receipts remain in the plugin store; verification also inspects the actual selected root and ownership markers.
Tampering with payload, manifest, signature, trust store, index, receipt, or
pointer is a hard failure.
