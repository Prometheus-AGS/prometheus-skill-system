# Goals

- Add memory-bridge.sh with mandatory outbox fallback for surreal-memory writes
- Automatic write-back of accepted reflections to surreal-memory (global vs project scoping)
- Wire pk-health on SessionStart and pk ingest at reflect:end; register pk-lint/mem0-compress
- Build sycophancy-correction binary in CI + 1 real e2e gate test (carry-forward CA-4)
- Record explicit reference-only decision for karpathy-tokenizer
