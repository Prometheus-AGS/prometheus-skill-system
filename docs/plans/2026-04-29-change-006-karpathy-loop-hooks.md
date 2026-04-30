# change-006-karpathy-loop-hooks Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Close the Karpathy learning loop by wiring `prometheus-knowledge` (`pk`) into Claude Code hooks, and sweep all owned SKILL.md files to add the `license: MIT` frontmatter field.

**Architecture:** Three mutually independent work streams executed in order: (1) the `pk-focus-on-prompt.sh` hook script + `hooks.json` `UserPromptSubmit` entry; (2) the `Stop` hook addition for `forge reflect`; (3) a sed-based sweep of all 64 owned SKILL.md files plus a one-line validator warning. Gaps C1, C2, and A4 from the assessment.

**Tech Stack:** Bash, JSON (hooks.json), Node.js/AJV (validate-skills.js), YAML frontmatter, `pk` CLI (graceful no-op when absent), `forge` CLI (graceful no-op when absent).

---

## Context: what you need to know before touching anything

- **`hooks/hooks.json`** at the repo root is the Claude Code plugin hooks manifest. It is consumed by Claude Code when the plugin is installed. The hook runner passes data to `UserPromptSubmit` hooks via **stdin as JSON** with shape `{"session_id":"...","prompt":"..."}`. The hook's stdout is **appended to the user prompt context** — so the script must print nothing on degraded-path and print focused content on the happy path.
- **`Stop` hooks** fire when a Claude Code session ends. They receive no stdin data.
- **`CLAUDE_PLUGIN_ROOT`** is an env var injected by Claude Code at hook execution time; it points to the installed plugin root (i.e. this repo). Scripts live under `shared/scripts/`.
- **`pk`** is the `prometheus-knowledge` CLI. When not on `$PATH`, every hook must silently exit 0. The hook contract says the exit code must be 0 on any failure — a non-zero exit blocks the user prompt.
- **`forge`** is the forge-rs CLI. Same graceful-degradation rule applies.
- **64 SKILL.md files** (non-imported, non-worktree) are missing `license: MIT`. The `skills/imported/` subtree is a git submodule we must not touch. The `.claude/worktrees/` path is a working copy artifact and must also be excluded.
- **`scripts/validate-skills.js`** already has `license: { type: 'string' }` in the AJV schema and a `this.addWarning()` helper. It just needs a check inserted after the frontmatter validation block to warn when `license` is absent.

---

## Task 1: Write the pk-focus-on-prompt.sh hook script

**Files:**
- Create: `shared/scripts/pk-focus-on-prompt.sh`

**Step 1: Create the file**

```bash
cat > shared/scripts/pk-focus-on-prompt.sh << 'SCRIPT'
#!/usr/bin/env bash
# pk-focus-on-prompt.sh — UserPromptSubmit hook: injects pk-focus context for the current prompt.
# Contract: reads JSON from stdin, prints focus output to stdout (or nothing on degraded path).
# Must always exit 0.
set -uo pipefail

# --- Graceful degradation: pk must be on PATH ---
if ! command -v pk &>/dev/null; then
  exit 0
fi

# --- Read prompt text from stdin JSON ---
PROMPT_JSON="$(cat)"
PROMPT_TEXT="$(printf '%s' "$PROMPT_JSON" | python3 -c \
  "import sys, json; d=json.load(sys.stdin); print(d.get('prompt',''))" 2>/dev/null || true)"

if [ -z "$PROMPT_TEXT" ]; then
  exit 0
fi

# --- Extract top-5 keywords (naive: strip punctuation, take longest unique words) ---
KEYWORDS="$(printf '%s' "$PROMPT_TEXT" \
  | tr '[:upper:]' '[:lower:]' \
  | tr -cs 'a-z0-9 ' ' ' \
  | tr ' ' '\n' \
  | awk 'length>4' \
  | sort -u \
  | sort -rn \
  | head -5 \
  | tr '\n' ' ' \
  | sed 's/ $//')"

if [ -z "$KEYWORDS" ]; then
  exit 0
fi

# --- Run pk focus with a hard timeout ---
FOCUS_OUTPUT="$(timeout 2.5 pk focus "$KEYWORDS" --max-articles 3 2>/dev/null || true)"

if [ -n "$FOCUS_OUTPUT" ]; then
  printf '\n\n--- prometheus-knowledge context ---\n%s\n--- end pk context ---\n' "$FOCUS_OUTPUT"
fi

exit 0
SCRIPT
chmod +x shared/scripts/pk-focus-on-prompt.sh
```

**Step 2: Verify the script is syntactically valid**

```bash
bash -n shared/scripts/pk-focus-on-prompt.sh && echo "syntax OK"
```
Expected output: `syntax OK`

**Step 3: Test the degraded path (pk not on PATH)**

```bash
PATH="" bash shared/scripts/pk-focus-on-prompt.sh < /dev/null
echo "exit: $?"
```
Expected output:
```
exit: 0
```
(No output before the exit line — the script printed nothing.)

**Step 4: Test with a sample prompt JSON**

```bash
echo '{"session_id":"test","prompt":"implement Karpathy tokenizer training loop in Rust"}' \
  | timeout 3 bash shared/scripts/pk-focus-on-prompt.sh 2>/dev/null || true
echo "exit: $?"
```
Expected: exits 0 in under 3 seconds. If `pk` is not installed, prints nothing and exits 0. If `pk` is installed, may print focus output or nothing.

**Step 5: Commit**

```bash
git add shared/scripts/pk-focus-on-prompt.sh
git commit -m "feat(hooks): add pk-focus-on-prompt.sh for UserPromptSubmit hook"
```

---

## Task 2: Add UserPromptSubmit entry to hooks.json

**Files:**
- Modify: `hooks/hooks.json`

The file currently has `SessionStart`, `PreToolUse`, `PostToolUse`, `SubagentStop`, and `Stop` sections. We need to insert a new top-level `UserPromptSubmit` key.

**Step 1: Add the UserPromptSubmit section**

In `hooks/hooks.json`, add a new `UserPromptSubmit` key alongside the existing keys. Place it after `SessionStart` and before `PreToolUse` for logical reading order:

```json
"UserPromptSubmit": [
  {
    "hooks": [
      {
        "type": "command",
        "command": "bash ${CLAUDE_PLUGIN_ROOT}/shared/scripts/pk-focus-on-prompt.sh 2>/dev/null",
        "timeout": 3000
      }
    ]
  }
],
```

**Step 2: Validate the JSON is well-formed**

```bash
node -e "JSON.parse(require('fs').readFileSync('hooks/hooks.json','utf8')); console.log('JSON valid')"
```
Expected: `JSON valid`

**Step 3: Commit**

```bash
git add hooks/hooks.json
git commit -m "feat(hooks): add UserPromptSubmit pk-focus hook (gap C1)"
```

---

## Task 3: Add forge-reflect Stop hook step

**Files:**
- Modify: `hooks/hooks.json`

The existing `Stop` block has three hooks under `skills/process/iterative-evolver/scripts/`. We need to add a fourth hook that runs `forge reflect` when `.forge/iterations/` exists.

**Step 1: Create the forge-reflect helper script**

```bash
cat > shared/scripts/forge-reflect-on-stop.sh << 'SCRIPT'
#!/usr/bin/env bash
# forge-reflect-on-stop.sh — Stop hook: runs forge reflect if forge and an iterations dir exist.
# Must always exit 0.
set -uo pipefail

if ! command -v forge &>/dev/null; then
  exit 0
fi

if [ ! -d ".forge/iterations" ]; then
  exit 0
fi

# Run forge reflect; errors are non-fatal
forge reflect 2>&1 || true

# If pk is available, ingest the reflection
if command -v pk &>/dev/null; then
  pk ingest 2>&1 || true
fi

exit 0
SCRIPT
chmod +x shared/scripts/forge-reflect-on-stop.sh
```

**Step 2: Verify syntax**

```bash
bash -n shared/scripts/forge-reflect-on-stop.sh && echo "syntax OK"
```
Expected: `syntax OK`

**Step 3: Add to the Stop hook block in hooks.json**

In the `Stop` section's `hooks` array, append a new entry after the last existing `finalize-session.sh` command:

```json
{
  "type": "command",
  "command": "bash ${CLAUDE_PLUGIN_ROOT}/shared/scripts/forge-reflect-on-stop.sh 2>&1 || true"
}
```

**Step 4: Validate JSON**

```bash
node -e "JSON.parse(require('fs').readFileSync('hooks/hooks.json','utf8')); console.log('JSON valid')"
```
Expected: `JSON valid`

**Step 5: Commit**

```bash
git add shared/scripts/forge-reflect-on-stop.sh hooks/hooks.json
git commit -m "feat(hooks): add forge-reflect Stop hook step (gap C2)"
```

---

## Task 4: Add SubagentStop fallback matcher

**Files:**
- Modify: `hooks/hooks.json`

The change spec calls for a `SubagentStop` fallback entry (matcher `*`) that emits a generic checkpoint for unrecognized sub-agent names. This prevents silent drops when a new sub-agent type is added.

**Step 1: Add the fallback script**

```bash
cat > shared/scripts/subagent-checkpoint-fallback.sh << 'SCRIPT'
#!/usr/bin/env bash
# subagent-checkpoint-fallback.sh — SubagentStop fallback: generic checkpoint for unknown agents.
# Must always exit 0.
set -uo pipefail
AGENT_NAME="${SUBAGENT_NAME:-unknown}"
TIMESTAMP="$(date -u +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || echo 'unknown')"
echo "SubagentStop checkpoint: agent=${AGENT_NAME} at=${TIMESTAMP}" >&2
exit 0
SCRIPT
chmod +x shared/scripts/subagent-checkpoint-fallback.sh
```

**Step 2: Verify syntax**

```bash
bash -n shared/scripts/subagent-checkpoint-fallback.sh && echo "syntax OK"
```

**Step 3: Add fallback SubagentStop entry to hooks.json**

In the `SubagentStop` array, add a new object at the end with no `matcher` (catches all unmatched agents):

```json
{
  "hooks": [
    {
      "type": "command",
      "command": "bash ${CLAUDE_PLUGIN_ROOT}/shared/scripts/subagent-checkpoint-fallback.sh 2>&1 || true"
    }
  ]
}
```

Note: An object without a `matcher` field is the Claude Code way to express "match anything not already matched."

**Step 4: Validate JSON**

```bash
node -e "JSON.parse(require('fs').readFileSync('hooks/hooks.json','utf8')); console.log('JSON valid')"
```

**Step 5: Commit**

```bash
git add shared/scripts/subagent-checkpoint-fallback.sh hooks/hooks.json
git commit -m "feat(hooks): add SubagentStop fallback matcher for unknown sub-agents"
```

---

## Task 5: Add license warning to the validator (gap A4 — schema side)

**Files:**
- Modify: `scripts/validate-skills.js`

The schema already accepts `license` as an optional string. We just need to emit a warning when it's absent after frontmatter validation passes.

**Step 1: Find the insertion point**

In `scripts/validate-skills.js`, look for the block that checks `frontmatter.name !== skillName` (around line 119). After the name-check block and before the `body` empty-check, insert:

```js
      // Warn when license is absent (forward-compat with future strict validation)
      if (!frontmatter.license) {
        this.addWarning(skillName, 'Missing recommended frontmatter field: license');
      }
```

**Step 2: Run the validator to confirm the warning fires**

```bash
node scripts/validate-skills.js 2>&1 | grep -c "Missing recommended" || true
```
Expected: a positive number (≥1), confirming at least one file triggers the warning.

**Step 3: Confirm the validator still exits 0 (warnings are not errors)**

```bash
node scripts/validate-skills.js; echo "exit: $?"
```
Expected: `exit: 0`

**Step 4: Commit**

```bash
git add scripts/validate-skills.js
git commit -m "feat(validator): warn on missing license frontmatter field (gap A4)"
```

---

## Task 6: Add `license: MIT` to all owned SKILL.md files

**Files:**
- Modify: 64 SKILL.md files under `skills/` (excluding `skills/imported/` submodule and `.claude/worktrees/`)

This is a mechanical sweep. The frontmatter block in every target file starts with `---` on line 1. We insert `license: MIT` on the line immediately after the opening `---`.

**Step 1: Dry-run — count how many files will be modified**

```bash
find skills/ -name "SKILL.md" \
  | grep -v "skills/imported/" \
  | xargs grep -L "license:" \
  | wc -l
```
Expected: 64 (approximately — the exact count may vary if new skills were added).

**Step 2: Run the sweep using perl in-place edit (preserves encoding, no temp files)**

```bash
find skills/ -name "SKILL.md" \
  | grep -v "skills/imported/" \
  | xargs grep -L "license:" \
  | xargs perl -i -0pe 's/^(---\n)/\1license: MIT\n/'
```

This regex matches the very first `---\n` in the file (the YAML opening fence) and inserts `license: MIT\n` immediately after it.

**Step 3: Verify no file is missing the field now**

```bash
find skills/ -name "SKILL.md" \
  | grep -v "skills/imported/" \
  | xargs grep -L "license:" \
  | wc -l
```
Expected: `0`

**Step 4: Spot-check two files manually**

```bash
head -5 skills/rust/async-patterns/SKILL.md
head -5 skills/process/kbd-process-orchestrator/SKILL.md
```
Expected: both show `license: MIT` on line 2 or 3 of the frontmatter.

**Step 5: Run the validator and confirm no license warnings remain for owned skills**

```bash
node scripts/validate-skills.js 2>&1 | grep "Missing recommended" | grep -v "imported" | wc -l
```
Expected: `0`

**Step 6: Confirm overall validator exit code**

```bash
node scripts/validate-skills.js; echo "exit: $?"
```
Expected: `exit: 0`

**Step 7: Commit**

```bash
git add skills/
git commit -m "chore(skills): add license: MIT to all 64 owned SKILL.md files (gap A4)"
```

---

## Task 7: Final validation sweep and progress.json update

**Step 1: Run full npm validate**

```bash
npm run validate
```
Expected: green (exit 0), no errors. Warnings from `skills/imported/` are acceptable since we don't own that submodule.

**Step 2: Confirm hook scripts are executable**

```bash
ls -la shared/scripts/pk-focus-on-prompt.sh \
        shared/scripts/forge-reflect-on-stop.sh \
        shared/scripts/subagent-checkpoint-fallback.sh
```
Expected: all show `-rwxr-xr-x` permissions.

**Step 3: Confirm hooks.json has all four new entries**

```bash
node -e "
const h = JSON.parse(require('fs').readFileSync('hooks/hooks.json','utf8')).hooks;
console.log('UserPromptSubmit:', !!h.UserPromptSubmit);
console.log('Stop hooks count:', h.Stop[0].hooks.length);
console.log('SubagentStop entries:', h.SubagentStop.length);
"
```
Expected:
```
UserPromptSubmit: true
Stop hooks count: 4
SubagentStop entries: 6
```

**Step 4: Update progress.json**

Edit `.kbd-orchestrator/phases/phase-compliance-and-power-multiplier/progress.json`:
- Add a new entry to `changes_log` for `change-006-karpathy-loop-hooks` with `status: "DONE"`
- Increment `changes_completed` from 5 to 6
- Set `next_change` to `change-007-opencode-real-plugin`

**Step 5: Update current-waypoint.json**

Edit `.kbd-orchestrator/current-waypoint.json`:
- Set `last_completed` to `"change-006-karpathy-loop-hooks"`
- Set `next_action` to `"/kbd-execute change-007-opencode-real-plugin"`
- Update `changes_completed` to 6

**Step 6: Final commit**

```bash
git add .kbd-orchestrator/
git commit -m "chore(kbd): mark change-006 DONE, advance waypoint to change-007"
```

---

## Acceptance Verification

After all tasks complete, run these checks:

```bash
# 1. Hook script graceful degradation
PATH="" bash shared/scripts/pk-focus-on-prompt.sh < /dev/null && echo "PASS: pk-focus degrades cleanly"

# 2. forge-reflect script graceful degradation
PATH="" bash shared/scripts/forge-reflect-on-stop.sh && echo "PASS: forge-reflect degrades cleanly"

# 3. JSON well-formed
node -e "JSON.parse(require('fs').readFileSync('hooks/hooks.json','utf8')); console.log('PASS: hooks.json valid JSON')"

# 4. No license gaps (owned skills)
COUNT=$(find skills/ -name "SKILL.md" | grep -v "skills/imported/" | xargs grep -L "license:" | wc -l)
[ "$COUNT" -eq 0 ] && echo "PASS: all owned SKILL.md have license field" || echo "FAIL: $COUNT files missing license"

# 5. Full validator passes
npm run validate && echo "PASS: npm run validate green"
```

All five checks must print `PASS`.
