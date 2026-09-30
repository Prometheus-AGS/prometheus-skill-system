#!/usr/bin/env bats
# Regression test: kbd-apply must reuse runtime tasks that /kbd-plan already
# registered for a change, instead of registering a duplicate under the
# backend's ordinal ID. Before the fix, runtime_task_transition registered
# task "1" beside a planned "<change>-t1"; the planned tasks stayed pending,
# the change could never complete, and position signals counted both sets
# ("task 9 out of 10" for a 5-task change). Observed in know-me-decision,
# 2026-09-30.

bats_require_minimum_version 1.5.0

setup() {
  export KBD_APPLY_LIB_ONLY=1
  SCRIPT="$(cd "$(dirname "$BATS_TEST_FILENAME")/.." && pwd)/kbd-apply.sh"
  # shellcheck source=/dev/null
  . "$SCRIPT"
  TASKS='{"c1-t1":{"sequence":1,"title":"Set up scaffolding"},"c1-t2":{"sequence":2,"title":"Wire up config loader"},"c1-t3":{"sequence":3,"title":"Write tests"}}'
}

@test "exact registered ID is used as-is" {
  run --separate-stderr resolve_runtime_task_id "$TASKS" "c1-t2" 2 "1.2 Wire up config loader"
  [ "$status" -eq 0 ]
  [ "$output" = "c1-t2" ]
}

@test "backend ordinal maps to the planned task with the same title" {
  run --separate-stderr resolve_runtime_task_id "$TASKS" "2" 2 "1.2 Wire up config loader"
  [ "$status" -eq 0 ]
  [ "$output" = "c1-t2" ]
  [[ "$stderr" == *"maps to registered runtime task c1-t2"* ]]
}

@test "title match wins over sequence when tasks were reordered" {
  run --separate-stderr resolve_runtime_task_id "$TASKS" "1" 1 "1.3 Write tests"
  [ "$status" -eq 0 ]
  [ "$output" = "c1-t3" ]
}

@test "unique sequence match is used when titles differ" {
  run --separate-stderr resolve_runtime_task_id "$TASKS" "3" 3 "1.3 Write the integration tests"
  [ "$status" -eq 0 ]
  [ "$output" = "c1-t3" ]
}

@test "no registered tasks: the backend ID is registered (new change)" {
  run --separate-stderr resolve_runtime_task_id '{}' "1" 1 "1.1 Set up scaffolding"
  [ "$status" -eq 0 ]
  [ "$output" = "1" ]
}

@test "registered tasks but no title or sequence match: refuse, never duplicate" {
  run --separate-stderr resolve_runtime_task_id "$TASKS" "9" 9 "1.9 Something unplanned"
  [ "$status" -ne 0 ]
  [[ "$output" == *"refusing to register a duplicate"* ]]
}
