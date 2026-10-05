#!/usr/bin/env bash
# Integration test for scripts/rebase-regenerate.sh. Builds a throwaway repo from
# the worktree's current files (committed and uncommitted, submodule contents
# included) and never touches the source worktree's own git state.
set -u

SRC="$(cd "$(dirname "$0")/../.." && pwd)"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/rebase-regenerate-test.XXXXXX")"
trap 'rm -rf "$TMP"' EXIT
REPO="$TMP/repo"
mkdir -p "$REPO"

failures=0
ok()   { echo "ok   - $1"; }
bad()  { echo "FAIL - $1" >&2; failures=$((failures + 1)); }
check() { if [ "$1" = 0 ]; then ok "$2"; else bad "$2"; fi; }

# Copy every file git would consider (tracked + untracked-not-ignored, plus the
# contents of submodule directories) without using the worktree's git state for
# anything but reading.
( cd "$SRC" && {
    git ls-files -z --cached --others --exclude-standard
  } | tr '\0' '\n' | while IFS= read -r f; do
      if [ -d "$f" ] && [ ! -L "$f" ]; then
        find "$f" -path '*/.git' -prune -o -path '*/node_modules' -prune -o -path '*/target' -prune -o \( -type f -o -type l \) -print
      elif [ -e "$f" ] || [ -L "$f" ]; then
        printf '%s\n' "$f"
      fi
    done | tar -cf - -T - ) | ( cd "$REPO" && tar -xf - )
[ -d "$SRC/node_modules" ] && ln -s "$SRC/node_modules" "$REPO/node_modules"

export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@example.invalid GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@example.invalid
cd "$REPO" || exit 1
git init -q -b main . >/dev/null
git config core.autocrlf false
git config core.filemode true
git add -A >/dev/null 2>&1
# Restore submodules as gitlinks (their files stay on disk), as in the source repo.
git -C "$SRC" ls-files -s | awk '$1 == "160000" { print $2, $4 }' | while read -r sha path; do
  git rm -r -q --cached -- "$path" >/dev/null 2>&1
  git update-index --add --cacheinfo "160000,$sha,$path"
done
git commit -q -m base >/dev/null 2>&1 || { echo "cannot build base commit" >&2; exit 1; }

GEN_A=hooks/codex-hooks.json
GEN_B=dist/plugins/claude/prometheus-skill-pack/skill-index.json
SRC_FILE=docs/CONTRIBUTING.md

# --- Scenario 3: outside a rebase/merge exits 2 ------------------------------
out="$(/bin/bash scripts/rebase-regenerate.sh 2>&1)"; rc=$?
[ "$rc" = 2 ] && [ -n "$out" ]; check $? "outside a rebase exits 2 with a message (rc=$rc)"

# --- Scenario 1: only generated paths conflict -------------------------------
git checkout -q -b side main
printf '\nside-edit\n' >> "$GEN_A"; printf '\nside-edit\n' >> "$GEN_B"
git commit -qam side
git checkout -q main
printf '\nmain-edit\n' >> "$GEN_A"; printf '\nmain-edit\n' >> "$GEN_B"
git commit -qam mainline
git checkout -q side
git rebase main >/dev/null 2>&1
[ -d .git/rebase-merge ] || [ -d .git/rebase-apply ]; check $? "setup: rebase stopped on conflicts"

/bin/bash scripts/rebase-regenerate.sh >"$TMP/s1.out" 2>&1; rc=$?
[ "$rc" = 0 ]; check $? "generated-only conflicts resolved, validators pass (rc=$rc)"
[ "$rc" = 0 ] || tail -20 "$TMP/s1.out" >&2
[ -z "$(git diff --name-only --diff-filter=U)" ]; check $? "no unmerged paths remain"
grep -q 'git rebase --continue' "$TMP/s1.out"; check $? "prints the git rebase --continue command"
[ -d .git/rebase-merge ] || [ -d .git/rebase-apply ]; check $? "helper did not continue the rebase itself"
generated_set="$(node scripts/generated-paths.mjs)"
stray=""
# Submodule gitlinks (mode 160000 on either side) are not content the helper
# stages; whether a submodule is initialised in the source checkout must not
# change the verdict.
for p in $(git diff --cached --raw | awk '$1 != ":160000" && $2 != "160000" { print $NF }'); do
  hit=0
  for g in $generated_set; do
    case "$g" in */) case "$p" in "$g"*) hit=1 ;; esac ;; *) [ "$p" = "$g" ] && hit=1 ;; esac
  done
  [ "$hit" = 1 ] || stray="$stray $p"
done
[ -z "$stray" ]; check $? "git diff --cached holds only generated paths${stray:+ (stray:$stray)}"
! grep -q 'main-edit\|side-edit' "$GEN_A" "$GEN_B"; check $? "conflicting edits were discarded in favour of regenerated content"
git rebase --abort >/dev/null 2>&1
git reset -q --hard >/dev/null 2>&1

# --- Scenario 2: a non-generated source file conflicts -----------------------
git checkout -q -f main >/dev/null 2>&1
git checkout -q -b base2
printf '\nbase2\n' >> "$SRC_FILE"; printf '\nbase2\n' >> "$GEN_A"
git commit -qam base2
git checkout -q -b topic main
printf '\ntopic\n' >> "$SRC_FILE"; printf '\ntopic\n' >> "$GEN_A"
git commit -qam topic
git rebase base2 >/dev/null 2>&1
[ -d .git/rebase-merge ] || [ -d .git/rebase-apply ]; check $? "setup: second rebase stopped on conflicts"

snapshot() { { git status --porcelain; git ls-files -s | git hash-object --stdin; git diff | git hash-object --stdin; git diff --cached | git hash-object --stdin; } 2>&1; }
before="$(snapshot)"
/bin/bash scripts/rebase-regenerate.sh >"$TMP/s2.out" 2>&1; rc=$?
[ "$rc" = 1 ]; check $? "source-file conflict exits 1 (rc=$rc)"
grep -q "$SRC_FILE" "$TMP/s2.out"; check $? "names the conflicted source file"
after="$(snapshot)"
[ "$before" = "$after" ]; check $? "changes nothing (status, index, worktree identical)"
git rebase --abort >/dev/null 2>&1

if [ "$failures" -ne 0 ]; then echo "$failures check(s) failed" >&2; exit 1; fi
echo "all checks passed"
