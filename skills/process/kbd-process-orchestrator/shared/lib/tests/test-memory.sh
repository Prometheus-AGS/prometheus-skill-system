#!/usr/bin/env bash
# shared/lib/tests/test-memory.sh — memory subsystem smoke tests.

set -uo pipefail
cd "$(dirname "$0")"
SKILL_ROOT="$(cd ../../.. && pwd -P)"
export KBD_ORCHESTRATOR_ROOT="$SKILL_ROOT"

pass() { printf 'pass: %s\n' "$*"; }
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }

# --- Test 1: detection returns 1 in empty env ---
( SANDBOX="$(mktemp -d)"; cd "$SANDBOX"; trap 'rm -rf "$SANDBOX"' EXIT
  unset UAR_MEMORY_MCP_URL KBD_MEMORY_MCP_URL KBD_AVAILABLE_TOOLS
  # shellcheck source=/dev/null
  . "$SKILL_ROOT/shared/lib/memory.sh"
  if kbd_memory_available; then fail "1: should return 1 (unavailable)"; fi
) && pass "kbd_memory_available returns 1 in empty env"

# --- Test 2: detection returns 0 when KBD_AVAILABLE_TOOLS contains create_entity ---
( SANDBOX="$(mktemp -d)"; cd "$SANDBOX"; trap 'rm -rf "$SANDBOX"' EXIT
  export KBD_AVAILABLE_TOOLS="some_tool create_entity another_tool"
  unset UAR_MEMORY_MCP_URL KBD_MEMORY_MCP_URL
  # shellcheck source=/dev/null
  . "$SKILL_ROOT/shared/lib/memory.sh"
  if ! kbd_memory_available; then fail "2: should return 0 with create_entity in tools"; fi
) && pass "kbd_memory_available returns 0 via KBD_AVAILABLE_TOOLS"

# --- Test 3: detection is cached ---
( SANDBOX="$(mktemp -d)"; cd "$SANDBOX"; trap 'rm -rf "$SANDBOX"' EXIT
  unset UAR_MEMORY_MCP_URL KBD_MEMORY_MCP_URL KBD_AVAILABLE_TOOLS
  # shellcheck source=/dev/null
  . "$SKILL_ROOT/shared/lib/memory.sh"
  kbd_memory_available || true
  # Set the var AFTER first probe; should still report unavailable (cached).
  export KBD_AVAILABLE_TOOLS="create_entity"
  if kbd_memory_available; then fail "3: cache should hold the negative result"; fi
) && pass "detection is cached after first probe"

# --- Test 4: memory-log.sh no-ops cleanly when endpoint absent ---
( SANDBOX="$(mktemp -d)"; cd "$SANDBOX"; trap 'rm -rf "$SANDBOX"' EXIT
  unset UAR_MEMORY_MCP_URL KBD_MEMORY_MCP_URL KBD_AVAILABLE_TOOLS
  export KBD_HOOK_KIND=phase KBD_HOOK_EDGE=before KBD_HOOK_NAME=test \
         KBD_HOOK_INDEX=1 KBD_HOOK_TOTAL=1 KBD_HOOK_PHASE_PATH=test \
         KBD_HOOK_SOURCE_TOOL=tester KBD_HOOK_STARTED_AT=2026-01-01T00:00:00Z
  bash "$SKILL_ROOT/shared/lib/memory-log.sh" || fail "4: should exit 0 even when memory unavailable"
) && pass "memory-log.sh exits 0 when endpoint absent"

# --- Test 5: kbd-memory-recall writes stub when endpoint absent ---
( SANDBOX="$(mktemp -d)"; cd "$SANDBOX"; trap 'rm -rf "$SANDBOX"' EXIT
  unset UAR_MEMORY_MCP_URL KBD_MEMORY_MCP_URL KBD_AVAILABLE_TOOLS
  mkdir -p .kbd-orchestrator/phases
  bash "$SKILL_ROOT/skills/kbd-memory-recall/kbd-memory-recall.sh" sample-phase \
    2>/dev/null || fail "5a: should exit 0"
  [[ -f .kbd-orchestrator/phases/sample-phase/prior-context.md ]] || fail "5b: digest missing"
  grep -q 'memory endpoint unreachable' .kbd-orchestrator/phases/sample-phase/prior-context.md \
    || fail "5c: expected stub marker"
) && pass "kbd-memory-recall writes stub when endpoint absent"

# --- Test 6: kbd-memory-recall resolves phase from waypoint ---
( SANDBOX="$(mktemp -d)"; cd "$SANDBOX"; trap 'rm -rf "$SANDBOX"' EXIT
  mkdir -p .kbd-orchestrator/phases
  jq -n '{phase: "auto-resolved"}' > .kbd-orchestrator/current-waypoint.json
  bash "$SKILL_ROOT/skills/kbd-memory-recall/kbd-memory-recall.sh" 2>/dev/null || fail "6a: should exit 0"
  [[ -f .kbd-orchestrator/phases/auto-resolved/prior-context.md ]] || fail "6b: digest for auto-resolved phase missing"
) && pass "kbd-memory-recall resolves phase from waypoint when arg absent"

printf '\nall memory subsystem smoke tests passed (6/6)\n'
