# Integrated runtime source overrides

Operator-approved release reconciliation on 2026-10-10. Operator-owned `versions.toml` remains unchanged. These exact release selections supersede its earlier source baseline for this delivery only; no database or framework upgrade is authorized.

- `tools/liter-llm`: `a6047386cc9fae4258b4a8577f6a016022095586` — preserve three approved release repair commits ahead of origin/main.
- `skills/imported/prometheus-entity-management`: `34a211563fe7f06118996a40cc398c4c3af953ef` — latest fork default branch.
- `tools/openai-proxy`: `b68861e0a6672f5ece900c58fd44b9e23a6b5691` — latest fork default branch.

Previously published binaries retain their original provenance until replaced by source-matched artifacts. Source selection is not runtime qualification.
