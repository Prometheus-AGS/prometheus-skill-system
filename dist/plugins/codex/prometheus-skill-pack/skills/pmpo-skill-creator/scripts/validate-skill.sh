#!/usr/bin/env bash
set -euo pipefail

# Validate a generated skill against agentskills.io spec and quality standards
# Usage: validate-skill.sh <skill_directory>

SKILL_DIR="${1:?Usage: validate-skill.sh <skill_directory>}"

if [[ ! -d "$SKILL_DIR" ]]; then
  echo "ERROR: Directory '${SKILL_DIR}' does not exist" >&2
  exit 1
fi

PASS=0
FAIL=0
WARN=0

# NOTE: `((PASS++))` must not be the last command in these functions.
# `((x++))` evaluates to the PRE-increment value, so when the counter is 0 the
# arithmetic command exits 1. As the final command it sets the function's return
# status, and under `set -e` that aborted the entire script at the very first
# passing check — the validator never reached checks 2..N. Use `$((...))`
# assignment, which always exits 0, so the counters are safe at any value.
check_pass() { echo "  ✅ $1"; PASS=$((PASS + 1)); }
check_fail() { echo "  ❌ $1"; FAIL=$((FAIL + 1)); }
check_warn() { echo "  ⚠️  $1"; WARN=$((WARN + 1)); }

echo "=== Skill Validation: ${SKILL_DIR} ==="
echo ""

# 1. SKILL.md exists
echo "--- SKILL.md ---"
SKILL_MD="${SKILL_DIR}/SKILL.md"
if [[ -f "$SKILL_MD" ]]; then
  check_pass "SKILL.md exists"
else
  check_fail "SKILL.md missing"
  echo ""
  echo "RESULT: FAIL (${FAIL} failures, ${WARN} warnings, ${PASS} passes)"
  exit 1
fi

# 2. Frontmatter validation
if head -1 "$SKILL_MD" | grep -q "^---"; then
  check_pass "Frontmatter delimiters present"
else
  check_fail "Missing frontmatter (no --- delimiter)"
fi

# Extract frontmatter.
# `head -n -1` (drop last line) is a GNU coreutils extension that BSD/macOS head
# rejects with "illegal line count -- -1". On macOS that made FRONTMATTER empty,
# so every field check below failed no matter what the file actually contained.
# `sed '$d'` is the portable equivalent.
FRONTMATTER=$(sed -n '2,/^---$/p' "$SKILL_MD" | sed '$d')

# Check name field
if echo "$FRONTMATTER" | grep -q "^name:"; then
  NAME=$(echo "$FRONTMATTER" | grep "^name:" | head -1 | sed 's/^name: *//')
  if [[ ${#NAME} -le 64 ]]; then
    check_pass "name field valid (${NAME})"
  else
    check_fail "name field too long (${#NAME} > 64)"
  fi
else
  check_fail "name field missing"
fi

# Check description field
if echo "$FRONTMATTER" | grep -q "^description:"; then
  check_pass "description field present"
else
  check_fail "description field missing"
fi

# Line count
LINES=$(wc -l < "$SKILL_MD")
if [[ $LINES -le 500 ]]; then
  check_pass "SKILL.md line count OK (${LINES} ≤ 500)"
else
  check_warn "SKILL.md exceeds 500 lines (${LINES})"
fi

echo ""

# 3. JSON schema validation
echo "--- JSON Schemas ---"
SCHEMA_COUNT=0
SCHEMA_VALID=0
# Skip *.template.json and anything under a templates/ directory. A template is
# JSON-shaped but deliberately NOT valid JSON — it carries {{placeholders}} that
# are substituted at generation time, so `required: [{{required_fields}}]` is
# correct source and parsing it will always fail. Flagging templates would make
# every skill that ships one permanently invalid.
# (Previously unreachable: the `set -e` counter bug aborted this script before
# it ever reached this group, so the false positive was invisible.)
for f in $(find "$SKILL_DIR" -name "*.schema.json" -o -name "*.json" | grep -v node_modules); do
  case "$f" in
    *.template.json|*/templates/*) continue ;;
  esac
  SCHEMA_COUNT=$((SCHEMA_COUNT + 1))
  if python3 -c "import json; json.load(open('$f'))" 2>/dev/null; then
    check_pass "$(basename $f)"
    SCHEMA_VALID=$((SCHEMA_VALID + 1))
  else
    check_fail "$(basename $f) — invalid JSON"
  fi
done
if [[ $SCHEMA_COUNT -eq 0 ]]; then
  echo "  (no JSON files found)"
fi

echo ""

# 4. Script validation
echo "--- Scripts ---"
for f in $(find "$SKILL_DIR/scripts" -name "*.sh" 2>/dev/null); do
  SCRIPT_NAME=$(basename "$f")
  
  # Check executable
  if [[ -x "$f" ]]; then
    check_pass "${SCRIPT_NAME} executable"
  else
    check_fail "${SCRIPT_NAME} not executable"
  fi
  
  # Check shebang
  if head -1 "$f" | grep -q "^#!/"; then
    check_pass "${SCRIPT_NAME} has shebang"
  else
    check_fail "${SCRIPT_NAME} missing shebang"
  fi
  
  # Check syntax
  if bash -n "$f" 2>/dev/null; then
    check_pass "${SCRIPT_NAME} syntax OK"
  else
    check_fail "${SCRIPT_NAME} syntax errors"
  fi
done
if [[ ! -d "$SKILL_DIR/scripts" ]]; then
  echo "  (no scripts directory)"
fi

echo ""

# 5. Cross-reference integrity
echo "--- Cross-References ---"
REF_TOTAL=0
REF_VALID=0
# Match `references/...` only when it is NOT preceded by another path segment.
# A bare `grep -oh 'references/...'` also matches the tail of a fully-qualified
# cross-skill path such as `skills/process/liter-llm-bridge/references/x.md`,
# strips the owning-skill prefix, and then reports the remainder as a broken
# local link. Those citations are correct and resolve elsewhere in the pack, so
# flagging them made the validator reject valid skills. `[^/[:alnum:]_.-]` as the
# preceding character keeps genuinely-local references and drops qualified ones.
for ref in $(grep -rohE '(^|[^/[:alnum:]_.-])references/[a-zA-Z0-9/_.-]*' \
               "$SKILL_DIR/prompts/" "$SKILL_DIR/SKILL.md" 2>/dev/null \
             | grep -oE 'references/[a-zA-Z0-9/_.-]*' | sed 's/[.,)]*$//' | sort -u); do
  REF_TOTAL=$((REF_TOTAL + 1))
  if [[ -e "$SKILL_DIR/$ref" ]]; then
    check_pass "$ref"
    REF_VALID=$((REF_VALID + 1))
  else
    check_fail "$ref — not found"
  fi
done
if [[ $REF_TOTAL -eq 0 ]]; then
  echo "  (no cross-references found)"
fi

echo ""

# 6. Plugin manifest (if present)
echo "--- Plugin ---"
PLUGIN_JSON="$SKILL_DIR/.claude-plugin/plugin.json"
if [[ -f "$PLUGIN_JSON" ]]; then
  if python3 -c "import json; d=json.load(open('$PLUGIN_JSON')); assert 'name' in d; assert 'description' in d" 2>/dev/null; then
    check_pass "plugin.json valid"
  else
    check_fail "plugin.json missing required fields"
  fi
else
  echo "  (no plugin manifest)"
fi

echo ""

# 7. Sub-skills
echo "--- Sub-Skills ---"
for skill_dir in $(find "$SKILL_DIR/skills" -name "SKILL.md" 2>/dev/null); do
  SUBSKILL=$(dirname "$skill_dir" | xargs basename)
  if head -1 "$skill_dir" | grep -q "^---"; then
    check_pass "skills/${SUBSKILL}/SKILL.md"
  else
    check_fail "skills/${SUBSKILL}/SKILL.md — missing frontmatter"
  fi
done
if [[ ! -d "$SKILL_DIR/skills" ]]; then
  echo "  (no sub-skills directory)"
fi

echo ""

# 8. Adversarial-review sycophancy screen
#
# This is the SINGLE ENFORCED GATE for the sycophancy screen (ratified by
# change-arc-001, goal 6). It shells out to the existing helper rather than
# reimplementing the check here: a second copy of the screen — including its
# rejection cap — would drift from the one adversarial review uses, and the two
# would silently disagree about whether a report is acceptable.
#
# Creators invoke THIS script. They must not call check-findings-sycophancy.sh
# directly, or the enforcement point moves and this group becomes decorative.
echo "--- Adversarial Review Screen ---"
SYCO_FEEDBACK=""
FINDINGS_JSON=""
for cand in \
  "$SKILL_DIR/.review/findings.json" \
  "$SKILL_DIR/review/findings.json"; do
  [[ -f "$cand" ]] && { FINDINGS_JSON="$cand"; break; }
done

if [[ -z "$FINDINGS_JSON" ]]; then
  # No review has run yet. Not a failure: validation legitimately runs before
  # the review in the Reflect phase, and the review consumes this output.
  echo "  (no findings.json — adversarial review has not run yet)"
else
  SYCO_HELPER=""
  for root in "${CLAUDE_PLUGIN_ROOT:-}" "${PLUGIN_ROOT:-}" \
              "$(cd "$(dirname "$0")" && pwd)/../../../.."; do
    [[ -n "$root" ]] || continue
    cand="$root/skills/process/adversarial-review/scripts/check-findings-sycophancy.sh"
    [[ -f "$cand" ]] && { SYCO_HELPER="$cand"; break; }
  done

  if [[ -z "$SYCO_HELPER" ]]; then
    # Absent tooling must not silently pass a gate. Warn rather than fail, so a
    # partial install degrades visibly instead of reporting a clean screen.
    check_warn "check-findings-sycophancy.sh not found — screen not run"
  else
    # Capture stdout (the helper's actionable feedback) and its exit separately.
    SYCO_FEEDBACK="$(bash "$SYCO_HELPER" --findings "$FINDINGS_JSON" \
                      --counter-key "skill-validate" 2>&1)" && SYCO_RC=0 || SYCO_RC=$?
    if [[ ${SYCO_RC:-0} -eq 0 ]]; then
      check_pass "adversarial findings pass the sycophancy screen"
      SYCO_FEEDBACK=""
    else
      # Propagate the helper's rejection into THIS script's FAIL counter, which
      # is what makes the screen a gate rather than advice.
      check_fail "adversarial findings rejected by the sycophancy screen"
    fi
  fi
fi

echo ""

# Summary
TOTAL=$((PASS + FAIL + WARN))
echo "=== RESULT ==="
echo "  Passes:   ${PASS}"
echo "  Failures: ${FAIL}"
echo "  Warnings: ${WARN}"
echo "  Total:    ${TOTAL}"

# Surface the sycophancy screen's feedback here, inside the block a caller
# actually reads. A rejection whose reason is not shown is indistinguishable
# from an unexplained failure, and the creator cannot act on it — the feedback
# is the entire point of the screen, not a side effect of it.
if [[ -n "${SYCO_FEEDBACK:-}" ]]; then
  echo ""
  echo "  --- Adversarial review screen feedback ---"
  printf '%s\n' "$SYCO_FEEDBACK" | sed 's/^/  /'
fi
echo ""

if [[ $FAIL -eq 0 ]]; then
  echo "  ✅ SKILL VALID"
  exit 0
else
  echo "  ❌ SKILL INVALID (${FAIL} failures)"
  exit 1
fi
