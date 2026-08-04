## Context

The repository already contains hardened patterns for signed canonical payloads (`kbd-runtime` and `sovereign-sync`), immutable archive segments, explicit evidence states, and side-effect-free verification. It does not contain an execution contract crate or a `prometheus-exec` binary. This first change establishes the portable evidence boundary before any executor, service, transport, or estate integration exists.

The contracts must work in standalone, estate, embedded, mobile, and remote builds. Consequently, `exec-contracts` cannot import KBD, Sovereign Sync, a web framework, a sandbox backend, or an async runtime. The repository's validated Wasmtime line is 46; Wasmtime is intentionally absent from this change and enters only with Tier W.

## Goals / Non-Goals

**Goals:**

- Represent every public execution envelope with versioned Rust types and deterministic JSON serialization.
- Give requests, receipts, artifacts, and receipt-log segments stable SHA-256 identities over RFC 8785 canonical bytes.
- Verify Ed25519 receipts today while retaining an explicit P-256 algorithm discriminator for later hardware-backed signers.
- Make verification portable and offline, including artifact content verification and receipt-log chain verification.
- Establish `prometheus-exec init` and `verify` as small, side-effect-bounded commands.
- Generate deterministic JSON Schema and OpenAPI components from the same Rust types.

**Non-Goals:**

- Execute Wasm components or native processes.
- Start a daemon, bind a socket, expose REST/MCP/FFI, or contact a remote peer.
- Anchor receipts into KBD or use KBD as an execution dependency.
- Implement mobile hardware P-256 key storage; the schema and verifier remain algorithm-agile.
- Restrict an agent's use of Bash, Python, Edit, Write, or any other tool.

## Decisions

### Transport-free contract crate

`substrate/exec-contracts` owns all wire types, canonicalization, hashing, signature verification, schema generation, and immutable receipt-segment verification. It has no internal repository dependencies. Later service and backend crates depend inward on it.

Alternative considered: place schemas in `exec-service`. Rejected because embedded/mobile consumers and the standalone verifier would then inherit transport and runtime dependencies.

### Canonical signed envelopes

Requests and receipts serialize with camel-case field names. A signature is computed over RFC 8785 canonical JSON with the top-level `signature` member omitted. Hashes are lowercase `sha256:<hex>` values. Unknown schema versions fail closed. The canonical request hash includes signer identity and algorithm but excludes only the signature value.

Alternative considered: sign normal `serde_json` output. Rejected because map ordering and number rendering are not a portable signature contract.

### Algorithm-tagged keys and signatures

`sigAlg` is mandatory and supports `ed25519` and `p256`. Key IDs include the algorithm and SHA-256 fingerprint of the encoded public key. Ed25519 signing and both algorithm verification are implemented with software fixtures; platform key custody is added by later forms. The verifier rejects algorithm/key mismatches before cryptographic verification.

Alternative considered: an Ed25519-only schema migration followed by P-256 later. Rejected because it would make early receipts structurally incompatible with mobile hardware-backed receipts.

### Explicit receipt invariants

Verification is layered: schema/version validation, canonical request-hash comparison when a request is supplied, receipt signature verification, terminal-state validation, evidence-class/tier compatibility, time ordering, artifact path safety, and optional artifact byte hashing. A successful signature alone is never reported as a successful receipt verification.

### Immutable receipt segments

A segment header records its sequence, previous segment hash, creation time, and receipt count. Its identity hashes the canonical unsigned segment body. Entries preserve append order and each stores the receipt hash plus receipt. Verification checks segment identity, previous-link expectation, entry hashes, signatures via a key resolver, and count consistency. Segments are immutable after sealing; single-writer enforcement belongs to `exec-core` in the next change.

Alternative considered: reuse KBD journal types directly. Rejected because receipts must remain independently verifiable and standalone/mobile builds must not gain a KBD dependency.

### Side-effect-bounded CLI

`prometheus-exec init` atomically creates an Ed25519 identity file with mode `0600`, refuses overwrite unless explicitly requested, and prints only public identity metadata. `verify` reads caller-selected inputs and writes diagnostics; it never initializes storage, logging services, sockets, or network clients. The binary begins at version `1.7.0` to align with the certified release family.

### Deterministic generated contracts

The crate exposes schema/OpenAPI generation functions, and checked-in artifacts are updated by an explicit local command. Generation uses sorted maps and a stable pretty-printer; local contract checks fail on diff. GitHub Actions do not validate product behavior.

## Risks / Trade-offs

- **[Canonicalization/library mismatch]** Different implementations may disagree on edge-case JSON numbers. → Golden vectors cover Unicode, ordering, optional fields, and numeric boundaries; the public contract names RFC 8785 rather than a Rust serializer.
- **[P-256 encoding ambiguity]** SEC1 and DER encodings can be confused. → Public keys use SEC1 compressed bytes and signatures use fixed-width IEEE P1363 bytes, both base64url without padding, with the encoding stated in schema descriptions.
- **[Artifact path traversal]** A signed receipt could reference unsafe paths. → Receipt verification rejects absolute paths, parent traversal, and paths outside `outputs/` before opening files.
- **[Verifier key lookup complexity]** Remote receipts may use different device keys. → The verifier accepts an explicit key resolver keyed by `executingDevice.keyId`; no implicit trust-store mutation occurs.
- **[Large receipt logs]** Loading complete segments may consume memory. → Revision 1 caps segment and receipt sizes and verifies segments independently; streaming archive verification can be added without changing the format.
- **[Version coupling]** Product release version and contract schema version have different lifecycles. → Binary/crate version is `1.7.0`; envelope `schemaVersion` is independently fixed at `1`.

## Migration Plan

This is additive. Land the contract crate and CLI without activating a service. Later changes consume these types through path dependencies. Rollback removes the new crates and generated files; no existing runtime state is migrated or deleted.

## Open Questions

- Hardware-backed P-256 key custody and attestation extensions remain deferred; the revision-1 verifier establishes only portable signature verification.
- Transparency-log publication remains outside revision 1; sealed receipt segments are exportable but not publicly witnessed.
