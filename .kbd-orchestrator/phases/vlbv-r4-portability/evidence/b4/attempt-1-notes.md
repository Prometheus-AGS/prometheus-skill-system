# B4 attempt 1 — stopped (2026-11-16)

Defects found during the run; the owner stopped the packet ("Stop and repair").

1. Claude Code in the scratch home reports "Not logged in": relocating CLAUDE_CONFIG_DIR loses its keychain login.
2. run.mjs scored a deterministic `present` check as FAIL when the harness never ran (claude-code AT19); an infrastructure failure must be BLOCKED.
3. Stopping run.mjs left three scratch roots (codex AT12 s1-s3) holding copied ~/.codex/auth.json in the system temp dir; no signal handler cleans active scratch roots. They were deleted by hand; the real auth.json is unchanged.

Finding to keep (not a defect): codex AT04 FAIL — no test ran in any sample (deterministic checks clean), but the grader judged one of three samples FAIL, and AT04 needs 3 of 3.

## B5 attempts (repair authorized 08:58:02Z)

- 2a (sequential): stopped after 3 Claude Code cases at the owner's request for speed; token auth confirmed working. Logs: evidence/b5-attempt-2a-sequential/.
- 2b (parallel harnesses, 4 cases at a time): stopped after two runner defects surfaced: (1) Kimi refuses `--prompt` with `--yolo`/`--auto`, so every Kimi session exited before starting; (2) the isolation check counted ~/.claude.json, which the interactive Claude Code session driving the run rewrites continuously — a false alarm. Logs: evidence/b5-attempt-2b-parallel/.
- 2c: Kimi runs plain `-p`; ~/.claude.json and Claude keychain items are excluded from the isolation manifest (eval sessions use a scratch config and the token, never those).
