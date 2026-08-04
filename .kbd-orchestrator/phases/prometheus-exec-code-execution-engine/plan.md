PLAN: prometheus-exec-code-execution-engine
Project: prometheus-skill-system
Date: 2026-08-04
OpenSpec available: YES
Changes to implement: 4

CHANGE LIST (ordered)
1. change-exec-001-contracts-verification: Portable execution contracts, signed receipts, offline verification, OpenAPI, and receipt-log format
   - Scope: substrate/exec-contracts | crates/prometheus-exec verify/init | OpenAPI | docs sync
   - Depends on: NONE
   - Recommended agent: Codex
   - Est. complexity: L
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Details: Define the transport-free request, capability, receipt, event, artifact, grant, and error schemas with RFC 8785 canonical hashing and Ed25519 verification plus reserved P-256 agility. Implement offline verify/init commands and a KBD-independent hash-linked receipt segment verifier before any runtime can produce evidence.

2. change-exec-002-tier-p-sidecar: Tier P execution kernel, artifact CAS, policy/grants, idempotent service, UDS sidecar, and local API
   - Scope: exec-core | exec-tier-p | exec-service | prometheus-exec daemon/run/status/doctor | REST/SSE | installer/doctor
   - Depends on: change-exec-001-contracts-verification
   - Recommended agent: Codex
   - Est. complexity: L
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Details: Execute Python, Node, and Bash through process-scoped macOS Seatbelt and Linux bwrap/Landlock adapters, never unsandboxed while claiming attestation. Add content-addressed outputs, signed receipts, Cedar auto-approval, SSH grant manifests, restart-safe idempotency, health-first UDS startup, and receipt/event retrieval.

3. change-exec-003-tier-w-mobile: Tier W Wasmtime 46 component host, supply-chain verification, and embedded/mobile interfaces
   - Scope: exec-tier-w | prometheus:component host bindings | signed generation/hash pins | skill-ffi | backend/version matrix
   - Depends on: change-exec-001-contracts-verification, change-exec-002-tier-p-sidecar
   - Recommended agent: Codex
   - Est. complexity: L
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Details: Execute the existing reference component with fuel, epoch, memory, output, and capability limits; select Cranelift or Pulley without changing verified semantics. Load only trusted-generation or hash-pinned components and expose the shared service through the existing FRB boundary without making Tier W depend on Tier P.

4. change-exec-004-remote-mcp-docs: MCP parity, enrolled-peer remote dispatch, certification integration, canonical documentation, and all-tool release
   - Scope: exec-remote | sovereign-sync envelope adapter | rmcp | certification | docs/OpenAPI | installers/doctors | plugin generation
   - Depends on: change-exec-002-tier-p-sidecar, change-exec-003-tier-w-mobile
   - Recommended agent: Codex
   - Est. complexity: L
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Details: Wire exec-run/status/artifacts/verify to the same service layer as REST, add store-and-forward R-class routing over existing enrollment roots, and integrate pending_evidence certification semantics. Complete Docusaurus, ADRs, local doctors, signed binary installation, and the 14-target plugin generation.

EXECUTION ROUND ORDER
Round 1: change-exec-001-contracts-verification
Round 2: change-exec-002-tier-p-sidecar
Round 3: change-exec-003-tier-w-mobile
Round 4: change-exec-004-remote-mcp-docs

SCOPE BOUNDARIES AND GATES
- No command-string policing, shell/Python restrictions, or mandatory interpreter rule.
- Wasmtime 46 is the repository fabric line; v41 from the draft is superseded.
- Sessions, Windows Tier P, hardware P-256 signing, and transparency logs remain deferred decisions D-01/D-02/D-03/D-06.
- This Mac can certify macOS Tier P and desktop Tier W. Linux, Windows, iOS, Android, physical-device, and remote multi-peer claims require their named environments; missing external evidence must remain pending rather than false-green.
- GitHub Actions remain documentation synchronization and Pages deployment only. All product checks run locally.

COMMANDS TO RUN
/opsx:continue change-exec-001-contracts-verification
/opsx:continue change-exec-002-tier-p-sidecar
/opsx:continue change-exec-003-tier-w-mobile
/opsx:continue change-exec-004-remote-mcp-docs

SYCOPHANCY REVIEW
- Detection score: 0.0. No classified patterns; no correction required.

PLAN COMPLETE
