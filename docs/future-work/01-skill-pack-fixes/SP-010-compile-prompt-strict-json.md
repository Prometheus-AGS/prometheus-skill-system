---
id: SP-010
title: compile_user_prompt strict JSON parser
status: ready
priority: P1
estimated_effort: 1d
agent_role: rust-codegraph
depends_on: []
unblocks: []
related: []
created_from_conversation_turn: 3-4
---

# SP-010 — compile_user_prompt strict JSON parser

## Problem

The `compile_user_prompt` function (in pk-librarian or equivalent) expects an LLM to emit a JSON object describing the user's intent. The current parser strips ` ```json ` fences, but does not handle:

- Preamble text before the JSON ("Here's the JSON: ...").
- Trailing commentary after the JSON ("...let me know if you need anything else.").
- Unescaped quotes in string values.
- Trailing commas (some models emit them).

When parsing fails, the librarian falls back to no-knowledge-injection, silently degrading.

## Evidence

Read the parser. It will be a small function with a regex strip of the ` ``` ` fences and a `serde_json::from_str` call. No preamble handling.

Run a session where the LLM happens to add a preamble. Observe that the librarian fails to inject context.

## Why it matters

The librarian's value is its retrieval. When parsing fails silently, retrievals don't happen, and the failure mode is invisible (no warning, no log entry without SP-006). The user just gets less-relevant agent output and doesn't know why.

## Proposed fix

Replace the lenient parser with a strict mode plus a forgiving fallback:

**Strict mode (preferred).** The librarian's prompt to the LLM uses structured-output / JSON-mode where supported (most local Qwen variants and Claude do). Strict mode requires the LLM to emit JSON-only with no preamble or trailing text. If the model supports it, this is the path.

**Forgiving fallback.** When strict mode is not available (older models, custom endpoints), use a recovery parser:

1. Strip ` ```json ` and ` ``` ` fences.
2. Find the first `{` and the last `}`. Treat everything between as the candidate JSON.
3. Run `serde_json::from_str` on the candidate.
4. On failure, attempt repair: trailing-comma removal, single-quote-to-double-quote (only outside strings).
5. On final failure, log the raw model output to `~/.prometheus/parse-failures/<timestamp>.txt` and return an error to the caller (which gates on SP-006 for visibility).

## Trade-offs and risks

- **Risk: aggressive repair masks model misbehaviour.** A model that consistently emits malformed JSON should be replaced, not patched. Log the failures per above so the issue is visible.
- **Risk: structured-output mode unsupported on user's chosen model.** Fall back gracefully.
- **Performance.** Negligible — JSON parsing is microsecond-scale.

## Acceptance criteria

- [ ] Strict mode is used when the configured model supports it.
- [ ] Forgiving fallback handles preamble, trailing commentary, fenced JSON, trailing commas.
- [ ] Repair attempts are logged so persistent malformation is visible.
- [ ] On terminal failure, the raw output is preserved at `~/.prometheus/parse-failures/<ts>.txt` and an error propagates.
- [ ] Unit tests cover: clean JSON, fenced JSON, preamble + JSON, JSON + trailing commentary, trailing-comma JSON, malformed-beyond-repair JSON.

## Implementation steps

1. Identify the parser function in pk-librarian (or wherever).
2. Add a `parse_user_prompt_strict` and `parse_user_prompt_forgiving` pair.
3. Switch the call site to try strict, then forgiving.
4. Write the unit tests.
5. Add the failure-log directory with a periodic cleanup (out of scope here; track if it grows).

## Dependencies

None.

## Open questions

- Which models in the prometheus stack support JSON mode? Verify per provider in liter-llm; document in the librarian's config.
- Should the parser also return a confidence score (e.g. "this is parsed JSON" vs "this is repaired JSON")? Useful for downstream logic; default to including it as a parse-quality enum.
