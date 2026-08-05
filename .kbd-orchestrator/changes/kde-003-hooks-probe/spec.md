# kde-003 — probe whether the daimon honours `hooks` and `systemPrompt`

**Phase:** kimi-desktop-extensibility
**Scope:** a throwaway probe package + the E4/E5 status lines in `assessment.md`; NO change to the pack's generator or shipping package
**Backend:** native-kbd

## Problem

`hooks` and `systemPrompt` / `systemPromptPath` are documented for Kimi Code CLI
plugins, and would be the route to parity with the pack's 30-hook bundle. But
**neither appears in any of the 12 vendor packages installed by Moonshot itself**,
so their support on the *desktop daimon* path is unproven.

## Why this is a probe and not an implementation

This repo has already shipped a documented-but-inert extension path: the Codex
`config.toml [hooks]` snake_case form parsed cleanly (`config.toml parse ok`) and
**never fired**. It was reverted. The same failure mode recurred twice more this
session — `{{file:}}` slash commands and dangling skill symlinks — each time
because presence was mistaken for function.

The only output of this change is a **verdict**. Do not wire the pack's hook
bundle in the same change.

## Approach

Build a minimal package `prometheus-hook-probe` in `plugin-packages/` declaring:

- a `hooks` rule on the earliest available event, whose command appends a line
  with a timestamp to a file under the probe package root;
- a `systemPrompt` containing a distinctive, unlikely sentinel string.

Restart Kimi Desktop. Then:

- **hooks verdict** — did the file get written?
- **systemPrompt verdict** — ask the model to repeat the sentinel; does it know it?

## Acceptance criteria

1. Probe package installs and does not break Kimi Desktop startup (it must not
   take the app down the way a bad OpenCode reference does).
2. A written verdict for `hooks`: fires / does not fire, with the evidence.
3. A written verdict for `systemPrompt`: honoured / ignored, with the evidence.
4. Verdicts recorded in the phase directory, and reflected back into
   `assessment.md` E4/E5.
5. **The probe package is removed afterwards.** It must not linger in
   app-managed state.

## Out of scope

- Wiring the real hook bundle (that is kde-004, conditional on this verdict)
- Any change to `install-kimi-desktop-plugin.sh`

## What a negative result may and may not conclude

t1 concedes that a wrong event name produces a false negative. A single silent
probe therefore does NOT close the question — it is indistinguishable from
"the hook fired on an event we did not subscribe to."

A negative verdict is only conclusive when at least one of these holds:

- the probe subscribed to **every** event name found in the daimon bundle, or
- a positive control fired (some other declared field in the same probe package
  demonstrably took effect, proving the package was loaded at all).

Absent either, record the result as **inconclusive — probe did not establish
loading**, not as "not supported". An inconclusive result keeps OQ-1 open; only
a controlled negative closes it.

## Scope note — assessment update is deliberate

t4 edits `assessment.md`, which is outside the probe package. That is intended:
the whole product of this change is a recorded verdict, and a verdict that is
not written back leaves E4/E5 permanently marked "unproven". Declared scope is
therefore: the throwaway probe package **plus** the E4/E5 status lines in
`assessment.md`. No source file and no shipping package is touched.

## Follow-up reference

`kde-004` is named as the conditional follow-up but is deliberately NOT created
in this stage: its content depends entirely on this change's verdict, and
specifying it now would presume the answer. It is a forward reference, not a
dependency.

## Safety

The probe's hook command must be inert — append a line to a file, nothing more.
No network, no mutation outside the probe directory.
