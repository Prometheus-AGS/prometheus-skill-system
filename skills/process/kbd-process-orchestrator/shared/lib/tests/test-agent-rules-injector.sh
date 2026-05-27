#!/usr/bin/env bash
# shared/lib/tests/test-agent-rules-injector.sh

set -uo pipefail
cd "$(dirname "$0")"
SKILL_ROOT="$(cd ../../.. && pwd -P)"
SCRIPT="$SKILL_ROOT/skills/kbd-inject-agent-rules/kbd-inject-agent-rules.sh"
export KBD_ORCHESTRATOR_ROOT="$SKILL_ROOT"

pass() { printf 'pass: %s\n' "$*"; }
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }

# Test 1: first write into empty CLAUDE.md
( SANDBOX="$(mktemp -d)"; cd "$SANDBOX"; trap 'rm -rf "$SANDBOX"' EXIT
  "$SCRIPT" --target CLAUDE.md >/dev/null || fail "1a: should succeed"
  [[ -f CLAUDE.md ]] || fail "1b: CLAUDE.md not created"
  grep -q "agent-rules:start v1" CLAUDE.md || fail "1c: start marker missing"
  grep -q "agent-rules:end"      CLAUDE.md || fail "1d: end marker missing"
  grep -q "Think Before Coding"  CLAUDE.md || fail "1e: Karpathy rule missing"
  grep -q "Plan Mode First"      CLAUDE.md || fail "1f: Boris Cherny rule missing"
) && pass "first write into empty CLAUDE.md"

# Test 2: write into pre-existing content (append + preserve)
( SANDBOX="$(mktemp -d)"; cd "$SANDBOX"; trap 'rm -rf "$SANDBOX"' EXIT
  printf '# Project\n\nExisting content here.\n' > CLAUDE.md
  "$SCRIPT" --target CLAUDE.md >/dev/null || fail "2a"
  grep -q "Existing content here" CLAUDE.md || fail "2b: existing content not preserved"
  grep -q "agent-rules:start v1" CLAUDE.md || fail "2c"
) && pass "first write appends without disturbing existing content"

# Test 3: idempotent — second run leaves bit-identical file
( SANDBOX="$(mktemp -d)"; cd "$SANDBOX"; trap 'rm -rf "$SANDBOX"' EXIT
  printf '# Project\n' > CLAUDE.md
  "$SCRIPT" --target CLAUDE.md >/dev/null
  cp CLAUDE.md CLAUDE.md.snapshot
  "$SCRIPT" --target CLAUDE.md >/dev/null
  cmp -s CLAUDE.md CLAUDE.md.snapshot || fail "3: second run produced a diff"
) && pass "idempotent — second run is bit-identical"

# Test 4: replace-in-place preserves surrounding content
( SANDBOX="$(mktemp -d)"; cd "$SANDBOX"; trap 'rm -rf "$SANDBOX"' EXIT
  cat > CLAUDE.md <<EOF
# Project

Top text.

<!-- agent-rules:start v1 -->
old content that will be replaced
<!-- agent-rules:end -->

Bottom text.
EOF
  "$SCRIPT" --target CLAUDE.md >/dev/null || fail "4a"
  grep -q "Top text" CLAUDE.md    || fail "4b: top text lost"
  grep -q "Bottom text" CLAUDE.md || fail "4c: bottom text lost"
  grep -q "Think Before Coding" CLAUDE.md || fail "4d: new content missing"
  grep -qF "old content that will be replaced" CLAUDE.md && fail "4e: old content NOT replaced"
  exit 0
) && pass "replace-in-place preserves surrounding content"

# Test 5: refuse multi-start
( SANDBOX="$(mktemp -d)"; cd "$SANDBOX"; trap 'rm -rf "$SANDBOX"' EXIT
  cat > CLAUDE.md <<EOF
<!-- agent-rules:start v1 -->
a
<!-- agent-rules:end -->

<!-- agent-rules:start v1 -->
b
<!-- agent-rules:end -->
EOF
  out="$("$SCRIPT" --target CLAUDE.md 2>&1)" && fail "5a: should fail"
  echo "$out" | grep -qi "start markers" || fail "5b: expected start-markers error"
) && pass "refuse multi-start markers"

# Test 6: refuse missing end
( SANDBOX="$(mktemp -d)"; cd "$SANDBOX"; trap 'rm -rf "$SANDBOX"' EXIT
  printf '%s\nbody\n' '<!-- agent-rules:start v1 -->' > CLAUDE.md
  out="$("$SCRIPT" --target CLAUDE.md 2>&1)" && fail "6a"
  echo "$out" | grep -qiF "without an end marker" || fail "6b: expected missing-end error: got: $out"
) && pass "refuse start-without-end"

# Test 7: --dry-run modifies nothing
( SANDBOX="$(mktemp -d)"; cd "$SANDBOX"; trap 'rm -rf "$SANDBOX"' EXIT
  printf '# Project\n' > CLAUDE.md
  before="$(cat CLAUDE.md)"
  "$SCRIPT" --target CLAUDE.md --dry-run >/dev/null || fail "7a"
  after="$(cat CLAUDE.md)"
  [[ "$before" == "$after" ]] || fail "7b: --dry-run modified the file"
) && pass "--dry-run modifies nothing"

# Test 8: --target both writes identical content into CLAUDE.md and AGENTS.md
( SANDBOX="$(mktemp -d)"; cd "$SANDBOX"; trap 'rm -rf "$SANDBOX"' EXIT
  printf '# C\n' > CLAUDE.md
  printf '# A\n' > AGENTS.md
  "$SCRIPT" --target both >/dev/null || fail "8a"
  c_block="$(awk '/<!-- agent-rules:start/,/<!-- agent-rules:end -->/' CLAUDE.md)"
  a_block="$(awk '/<!-- agent-rules:start/,/<!-- agent-rules:end -->/' AGENTS.md)"
  [[ "$c_block" == "$a_block" ]] || fail "8b: blocks differ between CLAUDE.md and AGENTS.md"
) && pass "--target both writes byte-identical blocks"

# Test 9: invalid --target rejected
( SANDBOX="$(mktemp -d)"; cd "$SANDBOX"; trap 'rm -rf "$SANDBOX"' EXIT
  out="$("$SCRIPT" --target invalid.md 2>&1)" && fail "9a"
  echo "$out" | grep -qiF -- "--target must be" || fail "9b: expected usage error: got: $out"
) && pass "invalid --target rejected"

# Test 10: explicit --pack agent-rules == default
( SANDBOX="$(mktemp -d)"; cd "$SANDBOX"; trap 'rm -rf "$SANDBOX"' EXIT
  "$SCRIPT" --pack agent-rules --target CLAUDE.md >/dev/null || fail "10a"
  grep -q "agent-rules:start v1" CLAUDE.md || fail "10b"
  grep -q "Think Before Coding" CLAUDE.md || fail "10c"
) && pass "--pack agent-rules == default behavior"

# Test 11: --pack uiux-routing writes uiux markers
( SANDBOX="$(mktemp -d)"; cd "$SANDBOX"; trap 'rm -rf "$SANDBOX"' EXIT
  "$SCRIPT" --pack uiux-routing --target CLAUDE.md >/dev/null || fail "11a"
  grep -q "uiux-routing:start v1" CLAUDE.md || fail "11b: uiux start marker missing"
  grep -q "uiux-routing:end" CLAUDE.md || fail "11c: uiux end marker missing"
  grep -q "UI/UX work routing" CLAUDE.md || fail "11d: heading missing"
  grep -q "agent-rules:start" CLAUDE.md && fail "11e: agent-rules markers should not appear"
  exit 0
) && pass "--pack uiux-routing writes uiux markers only"

# Test 12: both packs co-exist; managing one doesn't touch the other
( SANDBOX="$(mktemp -d)"; cd "$SANDBOX"; trap 'rm -rf "$SANDBOX"' EXIT
  "$SCRIPT" --pack agent-rules --target CLAUDE.md >/dev/null
  "$SCRIPT" --pack uiux-routing --target CLAUDE.md >/dev/null
  grep -q "agent-rules:start v1" CLAUDE.md || fail "12a: agent-rules region lost"
  grep -q "uiux-routing:start v1" CLAUDE.md || fail "12b: uiux-routing region missing"
  # Re-running agent-rules must not affect uiux-routing region
  snap="$(awk '/uiux-routing:start/,/uiux-routing:end/' CLAUDE.md)"
  "$SCRIPT" --pack agent-rules --target CLAUDE.md >/dev/null
  snap2="$(awk '/uiux-routing:start/,/uiux-routing:end/' CLAUDE.md)"
  [[ "$snap" == "$snap2" ]] || fail "12c: re-running agent-rules disturbed uiux-routing"
  # And vice versa
  snap="$(awk '/agent-rules:start/,/agent-rules:end -->/' CLAUDE.md)"
  "$SCRIPT" --pack uiux-routing --target CLAUDE.md >/dev/null
  snap2="$(awk '/agent-rules:start/,/agent-rules:end -->/' CLAUDE.md)"
  [[ "$snap" == "$snap2" ]] || fail "12d: re-running uiux-routing disturbed agent-rules"
) && pass "both packs co-exist; managing one preserves the other"

# Test 13: invalid pack rejected
( SANDBOX="$(mktemp -d)"; cd "$SANDBOX"; trap 'rm -rf "$SANDBOX"' EXIT
  out="$("$SCRIPT" --pack bogus --target CLAUDE.md 2>&1)" && fail "13a"
  echo "$out" | grep -qiF -- "--pack must be" || fail "13b: expected pack error: $out"
) && pass "invalid --pack value rejected"

printf '\nall agent-rules-injector smoke tests passed (13/13)\n'
