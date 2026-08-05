# prometheus-exec

`prometheus-exec` is the evidence-producing execution service for Prometheus. Its defining output is a signed execution receipt: portable evidence of what ran, under which capabilities and limits, and which content-addressed outputs were produced.

The execution service is an optional evidence path. It does not inspect or restrict an agent's ordinary Bash, Python, Edit, or Write tool use.

The local sidecar exposes a mode-`0600` Unix-domain socket, persists accepted runs and
ordered events before execution, and returns signed receipts whose streams and declared
artifacts are stored in the content-addressed store. The current runtime-certified Tier P
worker uses macOS Seatbelt; unsupported hosts stay health-live but fail readiness rather
than claiming an execution backend.

```bash
prometheus-exec init --identity ./device-key.json
prometheus-exec daemon \
  --socket ./runtime/exec.sock \
  --state-dir ./state \
  --identity ./device-key.json

prometheus-exec run \
  --socket ./runtime/exec.sock \
  --state-dir ./state \
  --identity ./device-key.json \
  --runtime python3 \
  --code ./job.py \
  --input records=./records.json \
  --format json

prometheus-exec status \
  --socket ./runtime/exec.sock \
  --run-id '<run-uuid>' \
  --format json

prometheus-exec doctor \
  --socket ./runtime/exec.sock \
  --state-dir ./state \
  --identity ./device-key.json \
  --format json
```

`run` signs the request with the configured device identity, places code and named inputs
in the shared CAS, submits over the Unix socket, and waits for the durable terminal
receipt. Reusing the same request ID and canonical payload replays the original run;
conflicting payloads are rejected. `status` retrieves durable state without resubmitting.
`doctor` is diagnostic only: it verifies the running binary and same-UID socket, bounded
health/readiness, identity consistency, reconciled run records, sandbox availability, and
every CAS blob without creating or repairing state.

Portable contracts and offline verification remain available independently of the daemon:

```bash
prometheus-exec verify \
  --receipt ./receipt.json \
  --public-key '<unpadded-base64url-public-key>' \
  --artifacts ./run-root \
  --format json
prometheus-exec contracts --output-dir ../../docs/reference/api
```

`init` atomically creates a mode-`0600` Ed25519 identity and refuses to replace an existing file. Its stdout contains only the public key, algorithm, and key ID. `verify` reads only caller-supplied files; it does not initialize a daemon, bind a socket, or use the network.

Revision-1 signatures use RFC 8785 canonical JSON with the top-level `signature` field omitted. Ed25519 uses raw 32-byte public keys and raw 64-byte signatures. P-256 uses compressed SEC1 public keys and fixed-width IEEE P1363 signatures. Binary values are unpadded base64url; key IDs are `<algorithm>:<sha256-of-public-key>`.

Exit status `0` means every selected cryptographic, semantic, request, and artifact check passed. Status `1` means a structured verification failure. Status `2` means the command or input could not be processed.

The MCP stdio mode exposes `exec-run`, `exec-status`, `exec-events`, `exec-receipt`, `exec-artifact`, and `exec-verify` through the same durable facade. For response-loss reconciliation, clients send the same `requestId`, `issuedAt`, and canonical arguments again; the tool returns the original run and sets `replayed` to `true`. Reusing an ID with a different canonical payload is rejected. Oversized `exec-artifact` results include structured Unix-domain HTTP retrieval guidance (`method`, `socketPath`, and content-addressed `path`) instead of truncating bytes. `verify-bundle` validates a portable evidence index without daemon or network state.

Canonical documentation: [Execution overview](/docs/execution/overview-and-use-cases), [local API/CLI/MCP](/docs/execution/local-api-cli-and-mcp), and [operations](/docs/execution/installation-doctor-and-recovery).
