#!/usr/bin/env bats
# Regression test: os_list and os_mark_done must agree on task ordinal
# numbering when tasks.md contains nested/indented sub-bullets under a
# parent task. Prior to the fix, os_mark_done's awk counted every checkbox
# line regardless of indentation, while os_list defers to the openspec CLI's
# `instructions apply --json`, which only counts top-level (non-indented)
# checkboxes. That mismatch caused mark-done to flip the wrong line whenever
# a task had nested sub-bullets (see kbd-apply.sh os_mark_done).

setup() {
  export KBD_APPLY_LIB_ONLY=1
  SCRIPT="$(cd "$(dirname "$BATS_TEST_FILENAME")/.." && pwd)/kbd-apply.sh"
  TMPDIR_ROOT="$(mktemp -d)"
  cd "$TMPDIR_ROOT"
  mkdir -p "openspec/changes/test-nested"
  cat > "openspec/changes/test-nested/proposal.md" <<'EOF'
# Proposal: test-nested

## Why
Fixture.

## What Changes
- Add a validator with sub-rules
EOF
  cat > "openspec/changes/test-nested/tasks.md" <<'EOF'
## 1. Implementation

- [ ] 1.1 Set up project scaffolding
- [ ] 1.2 Wire up config loader
- [ ] 1.3 Create validator.rs
  - [ ] Add email format sub-rule
  - [ ] Add phone format sub-rule
  - [ ] Add postal code sub-rule
  - [ ] Add currency sub-rule
  - [ ] Add date format sub-rule
- [ ] 1.4 Wire validator into pipeline
- [ ] 1.5 Write unit tests
- [ ] 1.6 Write integration tests
- [ ] 1.7 Update docs/sync-rules-reference.md
- [ ] 1.8 Run cargo clippy
EOF
  # shellcheck source=/dev/null
  . "$SCRIPT"
}

teardown() {
  rm -rf "$TMPDIR_ROOT"
}

@test "os_mark_done ordinal 8 flips the 8th top-level task, not a nested sub-bullet" {
  run os_mark_done "test-nested" "8"
  [ "$status" -eq 0 ]
  grep -qE '^\- \[x\] 1\.8 Run cargo clippy$' "openspec/changes/test-nested/tasks.md"
  ! grep -qE '^\s+- \[x\]' "openspec/changes/test-nested/tasks.md"
}

@test "os_mark_done ordinal 3 flips the parent task line, not any of its 5 nested sub-bullets" {
  run os_mark_done "test-nested" "3"
  [ "$status" -eq 0 ]
  grep -qE '^\- \[x\] 1\.3 Create validator\.rs$' "openspec/changes/test-nested/tasks.md"
  ! grep -qE '^\s+- \[x\]' "openspec/changes/test-nested/tasks.md"
}

@test "nested sub-bullets never count toward top-level ordinals 4-8" {
  os_mark_done "test-nested" "4"
  os_mark_done "test-nested" "5"
  os_mark_done "test-nested" "6"
  os_mark_done "test-nested" "7"
  os_mark_done "test-nested" "8"
  grep -qE '^\- \[x\] 1\.4 Wire validator into pipeline$' "openspec/changes/test-nested/tasks.md"
  grep -qE '^\- \[x\] 1\.5 Write unit tests$' "openspec/changes/test-nested/tasks.md"
  grep -qE '^\- \[x\] 1\.6 Write integration tests$' "openspec/changes/test-nested/tasks.md"
  grep -qE '^\- \[x\] 1\.7 Update docs/sync-rules-reference\.md$' "openspec/changes/test-nested/tasks.md"
  grep -qE '^\- \[x\] 1\.8 Run cargo clippy$' "openspec/changes/test-nested/tasks.md"
  # None of the 5 nested sub-bullets should ever have flipped to [x].
  local nested_done
  nested_done="$(grep -cE '^\s+- \[x\]' "openspec/changes/test-nested/tasks.md" || true)"
  [ "$nested_done" -eq 0 ]
}

@test "os_list count (8 top-level tasks) matches the number of ordinals os_mark_done accepts" {
  run os_list "test-nested"
  [ "$status" -eq 0 ]
  local list_count
  list_count="$(printf '%s\n' "$output" | grep -c .)"
  [ "$list_count" -eq 8 ]

  # Every ordinal 1..8 from os_list must land on a distinct top-level line
  # when passed through os_mark_done — i.e. no collisions on nested lines.
  local i
  for i in 1 2 3 4 5 6 7 8; do
    os_mark_done "test-nested" "$i"
  done
  local top_done nested_done
  top_done="$(grep -cE '^\- \[x\]' "openspec/changes/test-nested/tasks.md" || true)"
  nested_done="$(grep -cE '^\s+- \[x\]' "openspec/changes/test-nested/tasks.md" || true)"
  [ "$top_done" -eq 8 ]
  [ "$nested_done" -eq 0 ]
}
