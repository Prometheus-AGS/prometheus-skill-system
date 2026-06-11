# Assessment — memory-and-karpathy

Phase 4 of the approved framework-evolution plan. Closes the "memory writes are
manual" gap and the karpathy/CI carry-forwards.

## Ground truth (verified)

| Fact | Location | Implication |
|------|----------|-------------|
| HTTP JSON-RPC to surreal-memory is a proven pattern | mem0-compress.sh:36-50 | memory-bridge.sh reuses it (curl POST, python parse, non-fatal) |
| Orchestrator has a builtin event bus with augment/on_failure:ignore | KBD/hooks/hooks.json | memory write-back wires as builtin entries on execute:before / reflect:end — NOT Claude Code hooks |
| kbd-memory-log already mirrors hook fires to memory (best-effort) | KBD/hooks builtin "kbd-memory-log" | precedent for default-on, graceful-degrade memory writes |
| Stop hook already runs forge reflect → pk ingest | forge-reflect-on-stop.sh | reflect:end pk ingest is additive; keep Stop ingest |
| memory.sh detects availability (tools / env / config / probe) | KBD/shared/lib/memory.sh | memory-bridge reuses the detection contract |
| CI has a rust-cli job (dtolnay/rust-toolchain@stable, cargo check/clippy) | validate.yml:61-80 | sycophancy binary build fits this job pattern |
| sycophancy crate is a real workspace (sycophancy-core + -mcp) | skills/imported/sycophancy-correction/ | `cargo build --release` produces the gate binary |
| karpathy-tokenizer is a reference/education skill (no enforcement surface) | skills/rust/karpathy-tokenizer/SKILL.md | explicit reference-only decision; document, don't wire |

## Gaps this phase closes

| ID | Gap | From plan / carry-forward |
|----|-----|---------------------------|
| M1 | Memory WRITES are manual prose; nothing persists phase learnings to surreal-memory. No outbox for when the endpoint is down (it timed out during the original assessment). | Phase 4.1-4.3 |
| M2 | Cross-project scoping (global vs project) is documented but unenforced. | Phase 4.2 |
| M3 | pk-lint.sh + mem0-compress.sh orphaned; pk ingest only at Stop, not reflect. | Phase 4.4 |
| M4 | Sycophancy binary never built in CI; artifact gate's real path untested. | CF CA-4 |
| M5 | karpathy-tokenizer's role undocumented — re-litigated each session. | Phase 4.4 / CF |

## Constraints

- memory-bridge writes MUST never block: failure → append to
  `.kbd-orchestrator/memory-outbox.jsonl`, return 0 (the endpoint was down
  during this very project's assessment — outbox is mandatory, not optional).
- Wire write-back into the orchestrator builtin bus (on_failure:ignore), not
  Claude Code hooks, to compose with the existing kbd-memory-log precedent.
- `[GLOBAL]`-prefixed corrective-action lines → user_id="global"; else project.
- A SessionStart Claude Code hook drains the outbox when the endpoint returns.
- karpathy-tokenizer: leave as reference-only; record the decision in its
  SKILL.md and memory.

## Verdict

GO. Every piece reuses an existing pattern (HTTP bridge, builtin event bus, CI
rust job). No new infrastructure; the outbox is the one genuinely new safety
mechanism and it is a single append-only file.
