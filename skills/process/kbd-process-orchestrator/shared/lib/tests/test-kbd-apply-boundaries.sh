#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"
SKILL_ROOT="$(cd ../../.. && pwd -P)"
APPLY="$SKILL_ROOT/skills/kbd-apply/kbd-apply.sh"
SANDBOX="$(mktemp -d)"
trap 'rm -rf "$SANDBOX"' EXIT

mkdir -p "$SANDBOX/project/.kbd-orchestrator/changes/change-a" "$SANDBOX/mock-root/shared/lib" "$SANDBOX/mock-bin"
cat > "$SANDBOX/project/.kbd-orchestrator/current-waypoint.json" <<'JSON'
{"generatedBy":"kbd-runtime","activePhaseId":"phase-a","phase":"phase-a"}
JSON
cat > "$SANDBOX/project/.kbd-orchestrator/changes/change-a/spec.md" <<'EOF'
# change-a
EOF
cat > "$SANDBOX/project/.kbd-orchestrator/changes/change-a/tasks.json" <<'JSON'
{"changeId":"change-a","schemaVersion":"1","tasks":[{"id":"1","title":"only task","done":false,"doneAt":null,"doneBy":null}]}
JSON

cat > "$SANDBOX/mock-root/shared/lib/hooks.sh" <<'EOF'
kbd_hooks_fire() { printf 'hook %s\n' "$*" >> "$BOUNDARY_LOG"; }
EOF
cat > "$SANDBOX/mock-root/shared/lib/position.sh" <<'EOF'
kbd_position_sync() { :; }
EOF
cat > "$SANDBOX/mock-root/shared/lib/runtime-authority.sh" <<'EOF'
kbd_runtime_authoritative() { return 0; }
kbd_runtime_status_json() {
  printf '%s\n' '{"activePath":{"phaseId":"phase-a"},"phases":{"phase-a":{"changes":{}}}}'
}
EOF
cat > "$SANDBOX/mock-root/shared/lib/bottleneck-guard.sh" <<'EOF'
kbd_bottleneck_active() { return 0; }
kbd_bottleneck_evaluate() {
  printf 'guard %s\n' "$*" >> "$BOUNDARY_LOG"
  printf '{"exactSignal":"%s","position":"phase-a","authoritativeRevision":1}\n' "$1-$2-$3"
}
kbd_bottleneck_print_signal() { printf '%s\n' "$1" | jq -r '.exactSignal'; }
EOF
cat > "$SANDBOX/mock-bin/prometheus" <<'EOF'
#!/usr/bin/env bash
printf 'prometheus %s\n' "$*" >> "$BOUNDARY_LOG"
EOF
chmod +x "$SANDBOX/mock-bin/prometheus"

export BOUNDARY_LOG="$SANDBOX/boundaries.log"
export KBD_ORCHESTRATOR_ROOT="$SANDBOX/mock-root"
export PATH="$SANDBOX/mock-bin:$PATH"
cd "$SANDBOX/project"
"$APPLY" begin-task change-a 1 1 1 "only task" >/dev/null
"$APPLY" end-task change-a 1 1 1 "only task" >/dev/null

expected="$(cat <<'EOF'
prometheus kbd --path . change register --command-id apply:change-register:phase-a:change-a --phase phase-a --id change-a --title change-a
prometheus kbd --path . task register --command-id apply:task-register:phase-a:change-a:1 --phase phase-a --change change-a --id 1 --title only task --sequence 1
guard change before change-a 1
guard task before change-a/1 1
prometheus kbd --path . change register --command-id apply:change-register:phase-a:change-a --phase phase-a --id change-a --title change-a
prometheus kbd --path . task register --command-id apply:task-register:phase-a:change-a:1 --phase phase-a --change change-a --id 1 --title only task --sequence 1
prometheus kbd --path . task transition --command-id apply:task-in-progress:phase-a:change-a:1 --phase phase-a --change change-a --id 1 --status in-progress --summary only task
guard change before change-a 0
guard task before change-a/1 0
hook change before change-a 1 1
hook task before change-a:1 1 1
guard task after change-a/1 1
guard change after change-a 1
prometheus kbd --path . change register --command-id apply:change-register:phase-a:change-a --phase phase-a --id change-a --title change-a
prometheus kbd --path . task register --command-id apply:task-register:phase-a:change-a:1 --phase phase-a --change change-a --id 1 --title only task --sequence 1
prometheus kbd --path . task transition --command-id apply:task-complete:phase-a:change-a:1 --phase phase-a --change change-a --id 1 --status complete --summary only task
guard task after change-a/1 0
guard change after change-a 0
hook task after change-a:1 1 1
hook change after change-a 1 1
EOF
)"
actual="$(cat "$BOUNDARY_LOG")"
[[ "$actual" == "$expected" ]] || {
  printf 'FAIL: boundary order mismatch\nexpected:\n%s\nactual:\n%s\n' "$expected" "$actual" >&2
  exit 1
}
printf 'pass: final task commits task and change receipts before completion hooks\n'
