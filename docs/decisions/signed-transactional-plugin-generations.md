# Decision: sign and transactionally activate plugin payload plus skill index

**Status:** accepted · 2026-08-03 · release 1.7.0

## Context

Content hashes detect accidental drift but do not identify an approved signer.
Separately activated payload and search indexes can make hosts discover a skill
version different from the one they execute.

## Decision

Each canonical manifest is Ed25519-signed and verified against a separate plugin
trust store before activation. The immutable generation contains the payload,
canonical skill index, byte-identical generated-agent/mobile projections, mobile
parity metadata, root source commit, and external gitlink commit pins. One
`current` pointer activates payload and index; `previous` supports rollback.
Fourteen signed receipts bind target mode, skill digest, and index digest.

## Alternatives considered

- Hash-only manifests were rejected because origin was unauthenticated.
- Independent index deployment was rejected because rollback could mix versions.
- Silent collision fallback was rejected because it can omit a requested skill.

## Consequences

The first local install creates a mode-`0600` signing identity and trust store;
later unknown identities are rejected. Receipts and immutable generations use
more disk but make activation and rollback auditable.

## Verification

Fixtures cover signature and file tampering, untrusted keys, collisions, all
targets, external pins, host/agent/mobile index parity, rollback, and uninstall.
