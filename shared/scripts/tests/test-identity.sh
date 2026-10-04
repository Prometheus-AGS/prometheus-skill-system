#!/bin/bash
# test-identity.sh — integration test for the project-id and agent-identity
# resolvers (design §1). Runs real git repositories with two worktrees and real
# hook payload shapes for Claude Code and Codex. bash 3.2 compatible.
#
# Isolation: scratch HOME (so global git config and runtime discovery are the
# fixture's), runtime lookup disabled unless a level explicitly tests it.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIB="$HERE/../lib"
PROJECT_ID="$LIB/project_id.py"
IDENTITY="$LIB/agent_identity.py"

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT
export HOME="$SCRATCH/home"
mkdir -p "$HOME"
git config --global user.email "fixture@example.invalid"
git config --global user.name "Fixture"
git config --global init.defaultBranch main
unset PROMETHEUS_PROJECT_ID PROMETHEUS_USER_ID PROMETHEUS_HARNESS || true
export PROMETHEUS_PROJECT_ID_SKIP_RUNTIME=1

pass=0
fail() { echo "FAIL: $*" >&2; exit 1; }
ok() { pass=$((pass + 1)); echo "ok - $*"; }
field() { python3 -c 'import json,sys; print(json.loads(sys.stdin.read())[sys.argv[1]])' "$1"; }

REPO="$SCRATCH/repo"
mkdir -p "$REPO"
git -C "$REPO" init -q
echo seed > "$REPO/README"
git -C "$REPO" add README
git -C "$REPO" commit -q -m seed
git -C "$REPO" worktree add -q "$SCRATCH/wt2" -b second
REPO_REAL="$(cd "$REPO" && pwd -P)"

# --- level 4: git common dir, shared by all worktrees ------------------------
a="$(python3 "$PROJECT_ID" --cwd "$REPO")"
b="$(python3 "$PROJECT_ID" --cwd "$SCRATCH/wt2")"
expected="project:$(printf '%s' "$REPO_REAL/.git" | shasum -a 256 | awk '{print $1}')"
[ "$a" = "$expected" ] || fail "git-level id $a != $expected"
[ "$a" = "$b" ] || fail "worktrees resolved different ids: $a vs $b"
ok "level 4: two worktrees share project:<sha256(git common dir)>"

# --- level 3: runtime project UUID (fake prometheus binary on PATH) -----------
mkdir -p "$SCRATCH/bin"
cat > "$SCRATCH/bin/prometheus" <<'EOF'
#!/bin/bash
[ "$1 $2 $3" = "kbd status --json" ] && echo '{"projectId":"11111111-2222-3333-4444-555555555555"}'
EOF
chmod +x "$SCRATCH/bin/prometheus"
c="$(PATH="$SCRATCH/bin:$PATH" PROMETHEUS_PROJECT_ID_SKIP_RUNTIME=0 python3 "$PROJECT_ID" --cwd "$REPO" --json | field projectId)"
[ "$c" = "11111111-2222-3333-4444-555555555555" ] || fail "runtime level returned $c"
ok "level 3: runtime-registered project UUID wins over git"

# --- level 2: .prometheus/project.json (searched upward) ----------------------
mkdir -p "$REPO/.prometheus" "$REPO/sub/dir"
printf '{"projectId":"from-project-json"}\n' > "$REPO/.prometheus/project.json"
d="$(PATH="$SCRATCH/bin:$PATH" PROMETHEUS_PROJECT_ID_SKIP_RUNTIME=0 python3 "$PROJECT_ID" --cwd "$REPO/sub/dir" --json)"
[ "$(printf '%s' "$d" | field projectId)" = "from-project-json" ] || fail "project.json level: $d"
[ "$(printf '%s' "$d" | field source)" = "project.json" ] || fail "project.json source: $d"
ok "level 2: .prometheus/project.json wins over runtime, found from a subdirectory"

# --- level 1: environment ------------------------------------------------------
e="$(PROMETHEUS_PROJECT_ID=from-env python3 "$PROJECT_ID" --cwd "$REPO/sub/dir")"
[ "$e" = "from-env" ] || fail "env level returned $e"
ok "level 1: PROMETHEUS_PROJECT_ID wins over everything"

# --- user scope ------------------------------------------------------------------
u="$(python3 "$PROJECT_ID" --cwd "$REPO" --json | field userScope)"
expected_user="user:$(printf '%s' fixture@example.invalid | shasum -a 256 | cut -c1-16)"
[ "$u" = "$expected_user" ] || fail "user scope $u != $expected_user"
ok "user scope is user:<sha256(global git email)[:16]>"

# --- bash wrapper (bash 3.2) ------------------------------------------------------
w="$(cd "$REPO" && PROMETHEUS_PROJECT_ID=wrapped /bin/bash "$LIB/project-id.sh")"
[ "$w" = "wrapped" ] || fail "project-id.sh returned $w"
ok "project-id.sh wrapper runs under /bin/bash"

# --- agent identity -----------------------------------------------------------------
mkdir -p "$REPO/.agent-team/tlm-fixture"
cat > "$REPO/.agent-team/tlm-fixture/team.json" <<'EOF'
{"schemaVersion":1,"id":"tlm-fixture","outcome":"fixture","scope":"project","harness":"claude",
 "roles":[
  {"id":"backend-dev","description":"api","prompt":"p","skills":[],"owns":["src/api/**"],"inputs":[],"outputs":[],"dependsOn":[]},
  {"id":"ui-dev","description":"ui","prompt":"p","skills":[],"owns":["src/ui/**","src/ui/forms/**"],"inputs":[],"outputs":[],"dependsOn":[]}
 ]}
EOF
identity() { printf '%s' "$1" | python3 "$IDENTITY" --cwd "$REPO" "${@:2}"; }

r="$(identity '{"agent_type":"prometheus-skill-pack:backend-dev","agent_id":"a1","session_id":"s1"}')"
[ "$(printf '%s' "$r" | field roleId)" = "backend-dev" ] || fail "plugin-prefixed role: $r"
[ "$(printf '%s' "$r" | field teamId)" = "tlm-fixture" ] || fail "sole team not selected: $r"
[ "$(printf '%s' "$r" | field projectId)" = "from-project-json" ] || fail "identity project id: $r"
ok "Claude plugin agent_type <plugin>:backend-dev -> role backend-dev in the sole team"

r="$(identity '{"agent_type":"backend_dev","agent_id":"01a1","turn_id":"t1"}')"
[ "$(printf '%s' "$r" | field roleId)" = "backend-dev" ] || fail "codex underscore role: $r"
[ "$(printf '%s' "$r" | field harness)" = "codex" ] || fail "codex harness detection: $r"
ok "Codex agent_type backend_dev -> role backend-dev (underscore mapped back)"

r="$(identity '{"agent_type":"general-purpose","tool_input":{"file_path":"'"$REPO"'/src/ui/forms/login.tsx"}}')"
[ "$(printf '%s' "$r" | field roleId)" = "ui-dev" ] || fail "owns-glob role: $r"
[ "$(printf '%s' "$r" | field roleSource)" = "owns" ] || fail "owns source: $r"
ok "unknown agent_type resolves by longest owns glob over the touched path"

r="$(identity '{"agent_type":"general-purpose"}')"
[ "$(printf '%s' "$r" | field roleId)" = "unresolved" ] || fail "unresolved: $r"
ok "no name match and no touched path -> unresolved"

identity 'not json' >/dev/null || fail "malformed payload must exit 0"
ok "malformed payload exits 0"

# --is-team-role (used by team-role-guard.sh)
python3 "$IDENTITY" --cwd "$REPO" --is-team-role backend_dev || fail "--is-team-role backend_dev should succeed"
if python3 "$IDENTITY" --cwd "$REPO" --is-team-role planner; then fail "--is-team-role planner should fail"; fi
ok "--is-team-role matches team roles only"

# active team selection via project-routing.json with two teams
mkdir -p "$REPO/.agent-team/other"
sed 's/"tlm-fixture"/"other"/; s/backend-dev/planner/' "$REPO/.agent-team/tlm-fixture/team.json" > "$REPO/.agent-team/other/team.json"
r="$(identity '{"agent_type":"backend-dev"}')"
[ "$(printf '%s' "$r" | field teamId)" = "@solo" ] || fail "two teams without routing must be @solo: $r"
printf '{"schemaVersion":1,"activeTeam":"other"}\n' > "$REPO/.agent-team/project-routing.json"
r="$(identity '{"agent_type":"planner"}')"
[ "$(printf '%s' "$r" | field teamId)" = "other" ] || fail "routing activeTeam: $r"
[ "$(printf '%s' "$r" | field roleId)" = "planner" ] || fail "routing role: $r"
ok "project-routing.json activeTeam selects the team; ambiguity without it is @solo"

# no team at all
NOTEAM="$SCRATCH/noteam"; mkdir -p "$NOTEAM"
r="$(printf '{"agent_type":"x"}' | python3 "$IDENTITY" --cwd "$NOTEAM")"
[ "$(printf '%s' "$r" | field teamId)" = "@solo" ] || fail "no team: $r"
ok "no .agent-team -> @solo"

echo "test-identity: $pass passed"
