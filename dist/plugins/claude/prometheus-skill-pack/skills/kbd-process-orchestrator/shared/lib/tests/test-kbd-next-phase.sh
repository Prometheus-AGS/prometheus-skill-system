#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"
SKILL_ROOT="$(cd ../../.. && pwd -P)"
SCRIPT="$SKILL_ROOT/skills/kbd-next-phase/scripts/kbd-next-phase.sh"
SANDBOX="$(mktemp -d)"
trap 'rm -rf "$SANDBOX"' EXIT

mkdir -p "$SANDBOX/project/.kbd-orchestrator/phases/old-child" "$SANDBOX/mock-root/shared/lib" "$SANDBOX/mock-bin"
cat > "$SANDBOX/project/.kbd-orchestrator/current-waypoint.json" <<'JSON'
{"generatedBy":"kbd-runtime","phase":"parent","activePhaseId":"old-child","path":["parent","old-child"],"status":"complete"}
JSON
cat > "$SANDBOX/project/.kbd-orchestrator/project.json" <<'JSON'
{"name":"fixture","status":"reflected"}
JSON
cat > "$SANDBOX/project/.kbd-orchestrator/phases/old-child/reflection.md" <<'EOF'
# Reflection

## Recommended Next Phase

Continue the recovered control plane.
EOF
cat > "$SANDBOX/mock-root/shared/lib/hooks.sh" <<'EOF'
kbd_hooks_fire() { printf 'hook %s\n' "$*" >> "$HOOK_LOG"; }
EOF
cat > "$SANDBOX/mock-bin/prometheus" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "$PROMETHEUS_LOG"
case " $* " in
  *" --help "*) printf '  guard\n' ;;
  *" status --json "*) printf '%s\n' '{"activePath":{"phaseId":"old-child"},"phases":{"old-child":{"status":"complete"}}}' ;;
  *" guard evaluate "*) printf '%s\n' '{"exactSignal":"phase boundary","position":"parent > old-child","authoritativeRevision":7}' ;;
esac
EOF
chmod +x "$SANDBOX/mock-bin/prometheus"

export PROMETHEUS_LOG="$SANDBOX/prometheus.log"
export HOOK_LOG="$SANDBOX/hooks.log"
export KBD_ORCHESTRATOR_ROOT="$SANDBOX/mock-root"
export PATH="$SANDBOX/mock-bin:$PATH"
cd "$SANDBOX/project"
"$SCRIPT" successor >/dev/null

if grep -q 'phase transition .*--id old-child --status complete' "$PROMETHEUS_LOG"; then
  printf 'FAIL: next-phase repeated the reflected phase completion transition\n' >&2
  exit 1
fi
if [[ -f "$HOOK_LOG" ]] && grep -q 'hook phase after old-child' "$HOOK_LOG"; then
  printf 'FAIL: next-phase emitted a duplicate old-child completion record\n' >&2
  exit 1
fi
grep -q 'phase create .*--id successor' "$PROMETHEUS_LOG"
grep -q 'phase activate .*--id successor' "$PROMETHEUS_LOG"
grep -q 'phase transition .*--id successor --status in-progress' "$PROMETHEUS_LOG"
printf 'pass: reflected phase is not completed or recorded twice before successor activation\n'
