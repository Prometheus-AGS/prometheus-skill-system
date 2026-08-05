# Remediation completed

The corrected cumulative review found three critical defects. Commit `5bc2411` addresses each one:

1. `ExecRunParams` now accepts the paired stable `requestId` and `issuedAt`, uses them in the canonical signed request, and has focused same-ID/same-hash replay plus same-ID/different-hash conflict tests.
2. `verify_dispatch` now requires the target endpoint in the bound enrollment snapshot before queue insertion, with a queue-level regression proving no record is written for an unknown target.
3. `docs/codex-plugin.md` and the CLAUDE.md Codex integration section now document immutable hook bundle-ID refresh, paired manifest generation, 14-receipt verification, and unchanged Bash/Python policy.

Focused tests, warnings-denied clippy, deterministic docs sync, OpenAPI/docs contracts, Mermaid parsing, and the production Docusaurus build pass locally. Re-evaluate the entire corrected cumulative packet and do not suppress any remaining blocker.
