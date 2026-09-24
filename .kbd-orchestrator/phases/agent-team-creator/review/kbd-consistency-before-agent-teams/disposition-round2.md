# Review round2 dispositions

Full: the mini driver source was absent from the full-only packet, not absent from implementation. Include the paired repository source diff and integrations in round3; scripts/kbd-apply.mjs removes all synthetic fallback counts and lib/kbd/spec-backend.mjs validates actual progress.

Mini: move local goals creation after successful canonical phase creation/activation/transition in both helpers. A real runtime duplicate registration scenario covers rejection without a local goals artifact. Canonical commands remain separate events; a later canonical failure does not roll back earlier successful canonical events or claim atomic multi-command transactions.
