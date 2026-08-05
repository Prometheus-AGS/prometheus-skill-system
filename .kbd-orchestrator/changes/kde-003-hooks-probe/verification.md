# Verification — kde-003

## Success is a verdict, not a feature

This change succeeds when both questions are answered with evidence, including
if both answers are "not supported." A negative result closes OQ-1 permanently
and stops the pack from planning around a capability that does not exist.

## Gates

1. Kimi Desktop starts normally with the probe installed.
2. The shipping `prometheus-skill-pack` package is untouched (still 145 skills).
3. Both verdicts recorded with the command output or model response that proves them.
4. Probe package removed; `plugin-packages/` contains no `prometheus-hook-probe`.

## Anti-pattern to avoid

Do not conclude "supported" from the manifest being accepted. Acceptance is
parsing, not execution — the distinction that made the Codex `[hooks]` path,
`{{file:}}` commands, and 145 dangling symlinks all look fine while doing
nothing.
