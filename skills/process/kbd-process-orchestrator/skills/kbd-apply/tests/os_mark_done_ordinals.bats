#!/usr/bin/env bats
# Regression test: os_list and os_mark_done must agree on which line a task ID
# names. os_list defers to `openspec instructions apply --json`. That API's
# counting rule has changed across releases: older OpenSpec counted only
# column-0 checkboxes (an earlier fix here made os_mark_done match that), while
# OpenSpec 1.10.0 (utils/task-progress.js TASK_LINE_PATTERN) counts EVERY
# checkbox line, nested sub-tasks at any indent and `*` bullets included. With
# the top-level-only awk, `mark-done 4` flipped the 4th top-level task while
# os_list's task 4 was a sub-bullet: the wrong task was checked off.
# os_mark_done now resolves the ID through OpenSpec's own task list
# (description + occurrence), so the two cannot drift whatever rule the
# installed OpenSpec uses.

bats_require_minimum_version 1.5.0

setup() {
  export KBD_APPLY_LIB_ONLY=1
  SCRIPT="$(cd "$(dirname "$BATS_TEST_FILENAME")/.." && pwd)/kbd-apply.sh"
  TMPDIR_ROOT="$(mktemp -d)"
  cd "$TMPDIR_ROOT"
  mkdir -p "openspec/changes/test-nested"
  printf 'schema: spec-driven\n' > openspec/config.yaml
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
    - [ ] Add E.164 normalization
- [ ] 1.4 Wire validator into pipeline
* [ ] 1.5 Write unit tests
- [ ] Write tests
- [ ] Write tests
EOF
  # shellcheck source=/dev/null
  . "$SCRIPT"
}

teardown() {
  rm -rf "$TMPDIR_ROOT"
}

# The os_list ID of the Nth task whose title is exactly $1 (default: first).
id_for() { os_list "test-nested" | awk -F'\t' -v t="$1" -v n="${2:-1}" '$3 == t && ++seen == n { print $1 }'; }
checked_lines() { grep -E '\[[xX]\]' "openspec/changes/test-nested/tasks.md"; }
checked_count() { checked_lines | grep -c . || true; }

@test "os_list counts nested sub-tasks and * bullets (current OpenSpec rule)" {
  run os_list "test-nested"
  [ "$status" -eq 0 ]
  [ "$(printf '%s\n' "$output" | grep -c .)" -eq 10 ]
}

@test "marking a nested sub-task's ID flips that sub-task, not a top-level task" {
  local id; id="$(id_for "Add email format sub-rule")"
  [ -n "$id" ]
  os_mark_done "test-nested" "$id"
  [ "$(checked_count)" -eq 1 ]
  checked_lines | grep -qE '^  - \[x\] Add email format sub-rule$'
}

@test "the ID os_list gives '1.4 Wire validator into pipeline' flips exactly that line" {
  os_mark_done "test-nested" "$(id_for "1.4 Wire validator into pipeline")"
  [ "$(checked_count)" -eq 1 ]
  checked_lines | grep -qE '^- \[x\] 1\.4 Wire validator into pipeline$'
}

@test "a * bullet task is marked by its os_list ID" {
  os_mark_done "test-nested" "$(id_for "1.5 Write unit tests")"
  checked_lines | grep -qE '^\* \[x\] 1\.5 Write unit tests$'
}

@test "duplicate titles: the second occurrence's ID flips the second line only" {
  os_mark_done "test-nested" "$(id_for "Write tests" 2)"
  [ "$(checked_count)" -eq 1 ]
  [ "$(grep -nE '\[x\] Write tests$' openspec/changes/test-nested/tasks.md | cut -d: -f1)" = \
    "$(grep -nE 'Write tests$' openspec/changes/test-nested/tasks.md | tail -1 | cut -d: -f1)" ]
}

@test "every os_list ID flips a distinct line; progress then reports complete" {
  local id
  for id in $(os_list "test-nested" | cut -f1); do
    os_mark_done "test-nested" "$id"
  done
  [ "$(checked_count)" -eq 10 ]
  ! grep -qE '\[ \]' "openspec/changes/test-nested/tasks.md"
  run os_progress "test-nested"
  [ "$output" = "10 10 0" ]
}

@test "marking one ID at a time sets exactly that ID's done flag in os_list" {
  local id
  for id in $(os_list "test-nested" | cut -f1); do
    os_mark_done "test-nested" "$id"
    os_list "test-nested" | awk -F'\t' -v id="$id" '$1 == id { exit ($2 == "1" ? 0 : 1) }'
  done
}
