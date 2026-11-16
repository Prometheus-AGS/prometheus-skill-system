# Goals

Owner-adopted scope (R4 of the VLBV change register, 2026-11-16).

- CR-31: add protocol evals under `evals/` that run VLBV acceptance tests AT 01-22 and the section 18 fresh-session check as cases against each harness capability profile. Installed harnesses: claude-code, codex, opencode, kimi, qwen-code. zed, minimax-code and mimo-code are reported not-run unless exercisable.
- CR-31 exit: one eval report per harness with pass, fail and not-run counts. A `capabilities.json` value moves to `supported` only with exercised evidence (FR-H7); unknown capabilities are listed, never assumed.
- CR-33: `kbd-phase-close-eval.sh` lists runner output artifacts dated before boundary authorization in the closure packet, or states that none were found. Report only, no tool guard (CF-36).
- Adopt a v2 contract before any generation (the first live v2 adoption). Implement completely before the single B4 boundary run. Unit tests are not evidence.
