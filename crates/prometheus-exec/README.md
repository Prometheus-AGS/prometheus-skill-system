# prometheus-exec

`prometheus-exec` is the evidence-producing execution service for Prometheus. Its defining output is a signed execution receipt: portable evidence of what ran, under which capabilities and limits, and which content-addressed outputs were produced.

The execution service is an optional evidence path. It does not inspect or restrict an agent's ordinary Bash, Python, Edit, or Write tool use.

This first implementation slice provides portable contracts and offline verification:

```bash
prometheus-exec init --identity ./device-key.json
prometheus-exec verify \
  --receipt ./receipt.json \
  --public-key '<unpadded-base64url-public-key>' \
  --artifacts ./run-root \
  --format json
prometheus-exec contracts --output-dir ../../docs/reference/api
```

`init` atomically creates a mode-`0600` Ed25519 identity and refuses to replace an existing file. Its stdout contains only the public key, algorithm, and key ID. `verify` reads only caller-supplied files; it does not initialize a daemon, bind a socket, contact KBD or Sovereign Sync, or use the network.

Revision-1 signatures use RFC 8785 canonical JSON with the top-level `signature` field omitted. Ed25519 uses raw 32-byte public keys and raw 64-byte signatures. P-256 uses compressed SEC1 public keys and fixed-width IEEE P1363 signatures. Binary values are unpadded base64url; key IDs are `<algorithm>:<sha256-of-public-key>`.

Exit status `0` means every selected cryptographic, semantic, request, and artifact check passed. Status `1` means a structured verification failure. Status `2` means the command or input could not be processed.
