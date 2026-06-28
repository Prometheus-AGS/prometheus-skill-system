#!/usr/bin/env bash
# integration-meta.sh — learn-about-system + learn-harness integration test (change-learn-024)
#
# Tests routing and corpus structure for the meta-skills without running a full
# Feynman loop. No live MCP servers or network access required.
#
# Usage:
#   bash tests/learn/integration-meta.sh
#
# Exit codes:
#   0 — all tests passed
#   1 — one or more tests failed

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"

PASS=0
FAIL=0
SKIP=0

log()  { echo "$1"; }
pass() { PASS=$((PASS + 1)); log "  [PASS] $1"; }
fail() { FAIL=$((FAIL + 1)); log "  [FAIL] $1: $2"; }
skip() { SKIP=$((SKIP + 1)); log "  [SKIP] $1"; }

log ""
log "=== Meta-Skills Integration Test (change-learn-024) ==="
log ""

# ── Preflight: jq ─────────────────────────────────────────────────────────────
if ! command -v jq &>/dev/null; then
  echo "[ERROR] jq is required but not found. Install jq and re-run." >&2
  exit 1
fi

# ── Test 1: KBD lifecycle corpus structure ─────────────────────────────────────
log "Test 1: KBD lifecycle corpus structure"

KBD_CORPUS="$REPO_ROOT/docs/learn/meta-corpus/kbd-lifecycle-corpus.json"

if [[ ! -f "$KBD_CORPUS" ]]; then
  fail "kbd-corpus" "file not found: $KBD_CORPUS"
else
  # corpus_id check
  KBD_CORPUS_ID="$(jq -r '.corpus_id // empty' "$KBD_CORPUS")"
  if [[ "$KBD_CORPUS_ID" == "kbd-lifecycle" ]]; then
    pass "kbd-corpus: corpus_id == 'kbd-lifecycle'"
  else
    fail "kbd-corpus" "corpus_id expected 'kbd-lifecycle', got '${KBD_CORPUS_ID}'"
  fi

  # sources count
  KBD_SOURCES_LEN="$(jq '.sources | length' "$KBD_CORPUS")"
  if [[ "$KBD_SOURCES_LEN" -gt 10 ]]; then
    pass "kbd-corpus: sources count > 10 (${KBD_SOURCES_LEN} entries)"
  else
    fail "kbd-corpus" "sources count is ${KBD_SOURCES_LEN}, expected > 10"
  fi

  # at least 1 misconception
  KBD_MISCONCEPTION_COUNT="$(jq '[.sources[] | select(.is_misconception == true)] | length' "$KBD_CORPUS")"
  if [[ "$KBD_MISCONCEPTION_COUNT" -gt 0 ]]; then
    pass "kbd-corpus: ${KBD_MISCONCEPTION_COUNT} misconception(s) present"
  else
    fail "kbd-corpus" "no misconception entries found (expected ≥ 1)"
  fi

  # progress signaling covered
  PROGRESS_HIT="$(jq '[.sources[] | select(.content_summary | ascii_downcase | contains("progress"))] | length' "$KBD_CORPUS")"
  if [[ "$PROGRESS_HIT" -gt 0 ]]; then
    pass "kbd-corpus: progress signaling covered in at least one source"
  else
    fail "kbd-corpus" "no source with 'progress' in content_summary (progress signaling not covered)"
  fi

  # kbd-plan misconception covered
  PLAN_MISC_HIT="$(jq '[.sources[] | select(.is_misconception == true and (.content_summary | ascii_downcase | contains("plan")))] | length' "$KBD_CORPUS")"
  if [[ "$PLAN_MISC_HIT" -gt 0 ]]; then
    pass "kbd-corpus: kbd-plan misconception entry present"
  else
    fail "kbd-corpus" "no misconception entry with 'plan' in content_summary"
  fi
fi

# ── Test 2: skill pack corpus structure ───────────────────────────────────────
log "Test 2: skill pack corpus structure"

PACK_CORPUS="$REPO_ROOT/docs/learn/meta-corpus/skill-pack-corpus.json"

if [[ ! -f "$PACK_CORPUS" ]]; then
  fail "pack-corpus" "file not found: $PACK_CORPUS"
else
  # corpus_id check
  PACK_CORPUS_ID="$(jq -r '.corpus_id // empty' "$PACK_CORPUS")"
  if [[ "$PACK_CORPUS_ID" == "skill-pack" ]]; then
    pass "pack-corpus: corpus_id == 'skill-pack'"
  else
    fail "pack-corpus" "corpus_id expected 'skill-pack', got '${PACK_CORPUS_ID}'"
  fi

  # sources count
  PACK_SOURCES_LEN="$(jq '.sources | length' "$PACK_CORPUS")"
  if [[ "$PACK_SOURCES_LEN" -gt 10 ]]; then
    pass "pack-corpus: sources count > 10 (${PACK_SOURCES_LEN} entries)"
  else
    fail "pack-corpus" "sources count is ${PACK_SOURCES_LEN}, expected > 10"
  fi

  # at least 1 misconception
  PACK_MISCONCEPTION_COUNT="$(jq '[.sources[] | select(.is_misconception == true)] | length' "$PACK_CORPUS")"
  if [[ "$PACK_MISCONCEPTION_COUNT" -gt 0 ]]; then
    pass "pack-corpus: ${PACK_MISCONCEPTION_COUNT} misconception(s) present"
  else
    fail "pack-corpus" "no misconception entries found (expected ≥ 1)"
  fi

  # agentskills.io cross-platform fact covered
  AGENTSKILLS_HIT="$(jq '[.sources[] | select(.content_summary | ascii_downcase | contains("agentskills.io"))] | length' "$PACK_CORPUS")"
  if [[ "$AGENTSKILLS_HIT" -gt 0 ]]; then
    pass "pack-corpus: agentskills.io mentioned in at least one source"
  else
    fail "pack-corpus" "no source with 'agentskills.io' in content_summary"
  fi

  # Claude Code platform misconception covered
  CLAUDECODE_MISC_HIT="$(jq '[.sources[] | select(.is_misconception == true and (.content_summary | ascii_downcase | contains("claude code")))] | length' "$PACK_CORPUS")"
  if [[ "$CLAUDECODE_MISC_HIT" -gt 0 ]]; then
    pass "pack-corpus: Claude Code platform misconception entry present"
  else
    fail "pack-corpus" "no misconception entry with 'claude code' in content_summary"
  fi
fi

# ── Test 3: detect-surface-tier smoke test ────────────────────────────────────
log "Test 3: detect-surface-tier smoke test"

SURFACE_SCRIPT="$REPO_ROOT/shared/scripts/detect-surface-tier.sh"

if [[ ! -f "$SURFACE_SCRIPT" ]]; then
  fail "surface-tier" "script not found: $SURFACE_SCRIPT"
else
  SURFACE_JSON="$(bash "$SURFACE_SCRIPT" --json 2>/dev/null)" || {
    fail "surface-tier" "detect-surface-tier.sh exited non-zero"
    SURFACE_JSON=""
  }

  if [[ -n "$SURFACE_JSON" ]]; then
    # Validate JSON
    if echo "$SURFACE_JSON" | jq '.' &>/dev/null; then
      pass "surface-tier: output is valid JSON"
    else
      fail "surface-tier" "output is not valid JSON: $SURFACE_JSON"
    fi

    TIER_VAL="$(echo "$SURFACE_JSON" | jq -r '.tier // empty')"
    HARNESS_VAL="$(echo "$SURFACE_JSON" | jq -r '.harness // empty')"

    if [[ -n "$TIER_VAL" ]]; then
      pass "surface-tier: tier field present ('${TIER_VAL}')"
    else
      fail "surface-tier" "tier field is missing or empty"
    fi

    if [[ -n "$HARNESS_VAL" ]]; then
      pass "surface-tier: harness field present ('${HARNESS_VAL}')"
    else
      fail "surface-tier" "harness field is missing or empty"
    fi

    log "  [INFO] Detected surface: tier=${TIER_VAL} harness=${HARNESS_VAL}"
  fi
fi

# ── Test 4: learn-about-system skill file structure ───────────────────────────
log "Test 4: learn-about-system skill file structure"

LAS_SKILL="$REPO_ROOT/skills/learn/learn-about-system/SKILL.md"

if [[ ! -f "$LAS_SKILL" ]]; then
  fail "learn-about-system" "SKILL.md not found: $LAS_SKILL"
else
  pass "learn-about-system: SKILL.md exists"

  # Validate via npm (lenient mode — tolerate minor warnings)
  VALIDATE_OUT="$(cd "$REPO_ROOT" && npm run validate:skill skills/learn/learn-about-system 2>&1 || true)"
  if echo "$VALIDATE_OUT" | grep -q "All skills valid"; then
    pass "learn-about-system: validate:skill passes"
  elif echo "$VALIDATE_OUT" | grep -qi "0 error"; then
    pass "learn-about-system: validate:skill passes (0 errors)"
  else
    fail "learn-about-system" "npm run validate:skill did not confirm success"
  fi

  # Routing keyword checks
  for keyword in kbd skills harness; do
    if grep -qi "$keyword" "$LAS_SKILL"; then
      pass "learn-about-system: routing keyword '${keyword}' present in SKILL.md"
    else
      fail "learn-about-system" "routing keyword '${keyword}' not found in SKILL.md"
    fi
  done
fi

# ── Test 5: learn-harness skill file structure ────────────────────────────────
log "Test 5: learn-harness skill file structure"

LH_SKILL="$REPO_ROOT/skills/learn/learn-harness/SKILL.md"
LH_PARITY="$REPO_ROOT/skills/learn/learn-harness/references/harness-parity.md"

if [[ ! -f "$LH_SKILL" ]]; then
  fail "learn-harness" "SKILL.md not found: $LH_SKILL"
else
  pass "learn-harness: SKILL.md exists"

  VALIDATE_OUT="$(cd "$REPO_ROOT" && npm run validate:skill skills/learn/learn-harness 2>&1 || true)"
  if echo "$VALIDATE_OUT" | grep -q "All skills valid"; then
    pass "learn-harness: validate:skill passes"
  elif echo "$VALIDATE_OUT" | grep -qi "0 error"; then
    pass "learn-harness: validate:skill passes (0 errors)"
  else
    fail "learn-harness" "npm run validate:skill did not confirm success"
  fi
fi

if [[ ! -f "$LH_PARITY" ]]; then
  fail "learn-harness" "harness-parity.md not found: $LH_PARITY"
else
  pass "learn-harness: references/harness-parity.md exists"

  # Note: parity file uses "Claude Code" (title case) not "claude-code"
  for platform in "claude code" opencode codex; do
    if grep -qi "$platform" "$LH_PARITY"; then
      pass "learn-harness: parity file mentions '${platform}'"
    else
      fail "learn-harness" "parity file does not mention '${platform}'"
    fi
  done
fi

# ── Test 6: all 12 learn skills present ───────────────────────────────────────
log "Test 6: all 12 learn skills exist"

LEARN_SKILLS=(
  ui-surface
  learn-goal
  learn-survey
  learn-plan
  feynman-loop
  learn-grade
  learn-retain
  learn-practice
  learn-certify
  learn-kb
  learn-about-system
  learn-harness
)

FOUND=0
for skill in "${LEARN_SKILLS[@]}"; do
  SKILL_FILE="$REPO_ROOT/skills/learn/${skill}/SKILL.md"
  if [[ -f "$SKILL_FILE" ]]; then
    FOUND=$((FOUND + 1))
    # Run validation; don't abort on minor warnings
    VAL_OUT="$(cd "$REPO_ROOT" && npm run validate:skill "skills/learn/${skill}" 2>&1 || true)"
    if echo "$VAL_OUT" | grep -q "All skills valid"; then
      log "  [OK]   ${skill}: valid"
    elif echo "$VAL_OUT" | grep -qi "0 error"; then
      log "  [OK]   ${skill}: valid (0 errors)"
    else
      log "  [WARN] ${skill}: validation result unclear"
    fi
  else
    log "  [MISSING] ${skill}: SKILL.md not found"
  fi
done

if [[ "$FOUND" -eq "${#LEARN_SKILLS[@]}" ]]; then
  pass "learn skills: ${FOUND}/${#LEARN_SKILLS[@]} found"
else
  fail "learn skills" "${FOUND}/${#LEARN_SKILLS[@]} found — $(( ${#LEARN_SKILLS[@]} - FOUND )) missing"
fi

# ── Summary ───────────────────────────────────────────────────────────────────
log ""
log "=== Results: ${PASS} passed, ${FAIL} failed, ${SKIP} skipped ==="
log ""

if [[ "$FAIL" -gt 0 ]]; then
  log "[FAIL] meta-skills integration test — ${FAIL} test(s) failed"
  exit 1
fi

log "[PASS] meta-skills integration test"
exit 0
