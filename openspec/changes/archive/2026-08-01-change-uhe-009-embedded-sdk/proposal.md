# Expose an embedded SDK for skills

**Change:** `change-uhe-009-embedded-sdk`
**Phase:** uar-host-execution
**Goal:** R4

## Why

See `.kbd-orchestrator/phases/uar-host-execution/plan.md` for full rationale,
acceptance criteria, and the two-round adversarial review record.

## Outcome: R4 facade delivered, one task corrected rather than executed

```
test result: ok. 4 passed; 0 failed
```

### What shipped

`src/skills_api.rs` — `SkillsApi`, the embedder-facing surface: `list`,
`list_enabled`, `get`, `install`, `toggle`, `query`. Reached via
`EmbeddedRuntime::skills()` and re-exported at the crate root.

It owns `Arc<SkillService>` **privately**. Re-exporting the service would name an
internal type in the public API and recreate exactly the coupling this change
removes.

### Task 2 was corrected, not executed

The plan said *"Keep `uar::runtime::skills` internals private."* Measured, they
were already fully public — `uar` → `runtime` → `skills` is `pub` at every level.

| Consumer | Count |
|---|---|
| Files under `src/` importing those types | 16 |
| Integration tests importing them | 6 |

Making them private would fail the build immediately and is a breaking change to
a surface external code may already depend on — a deliberate deprecation with its
own migration, not a task inside an SDK change.

**What shipped instead is the seam that makes that narrowing possible later.**
Recorded in `skills/mod.rs` rather than silently skipped.

The plan's premise was also half-wrong: `EmbeddedRuntime` already existed with
`skill_service()`. It just handed back an internal type. The facade sits
alongside; `skill_service()` is kept and documented as advanced use.

### An R4 finding worth knowing before shipping an SDK

`EmbeddedRuntime::build()` **requires an LLM driver**
(`E_EMBEDDED_LOCAL_DRIVER_REQUIRED`, asserted by an in-crate test). A host that
wants *only* the skill catalogue — a mobile app listing capabilities, an
installer registering pack skills — must still supply a driver it has no use for.

Deliberate, so not a bug; but a real ergonomic edge in the embedding story.
Recorded rather than worked around.

### Tested against a REAL driver, not only a stub

`MockLlmDriver` proves the builder *accepts* a driver. It proves nothing about
whether an embedded host can be built around one that actually reaches a model.

So a second test uses the **local Ollama install** through `LiterLlmDriver` — the
same OpenAI-compatible path a production embedder takes. No new driver was
needed: Ollama serves `/v1` (verified 200), and `registry.rs:297` already
documents custom providers for "local Ollama instances".

Runtime tells the story: **13.77s** with the real driver vs **0.21s** for the
mock-only run. It genuinely reached the model rather than skipping.

Harness at `tests/common/ollama.rs`, with `OLLAMA_BASE_URL` / `OLLAMA_TEST_MODEL`
overridable for CI. Absent Ollama, it **skips loudly** — a silent skip lets
coverage rot unnoticed.
