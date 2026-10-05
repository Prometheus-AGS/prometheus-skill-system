#!/usr/bin/env bash
# Resolve generated-only rebase/merge conflicts by regenerating, then validate.
#
# Exit codes: 0 resolved + validated (changes staged), 1 conflict outside the
# generated set or a validator failed, 2 not in a rebase/merge (or unusable env).
# It never runs `git rebase --continue` / `git commit` itself.
set -u

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT" || exit 2

git_path() { git rev-parse --git-path "$1" 2>/dev/null; }

in_progress=""
if [ -d "$(git_path rebase-merge)" ] || [ -d "$(git_path rebase-apply)" ]; then
  in_progress=rebase
elif [ -f "$(git_path MERGE_HEAD)" ]; then
  in_progress=merge
fi
if [ -z "$in_progress" ]; then
  echo "rebase-regenerate: no rebase or merge is in progress; nothing to do." >&2
  exit 2
fi

conflicted="$(git diff --name-only --diff-filter=U)"
if [ -z "$conflicted" ]; then
  echo "rebase-regenerate: no conflicted paths; continue the $in_progress yourself."
  exit 0
fi

generated="$(node scripts/generated-paths.mjs)" || {
  echo "rebase-regenerate: could not derive the generated path set" >&2
  exit 2
}

is_generated() {
  local p="$1" g
  while IFS= read -r g; do
    [ -n "$g" ] || continue
    case "$g" in
      */) case "$p" in "$g"*) return 0 ;; esac ;;
      *) [ "$p" = "$g" ] && return 0 ;;
    esac
  done <<EOF
$generated
EOF
  return 1
}

others=""
while IFS= read -r p; do
  [ -n "$p" ] || continue
  is_generated "$p" || others="$others$p
"
done <<EOF
$conflicted
EOF
if [ -n "$others" ]; then
  echo "rebase-regenerate: conflicts outside the generated set; resolve these by hand (nothing was changed):" >&2
  printf '%s' "$others" | sed 's/^/  /' >&2
  exit 1
fi

# Every conflicted path is generated: its content is discarded either way, so
# take any side that exists and let the generators rewrite it.
while IFS= read -r p; do
  [ -n "$p" ] || continue
  git checkout --theirs -- "$p" 2>/dev/null \
    || git checkout --ours -- "$p" 2>/dev/null \
    || git rm -q --cached -- "$p" 2>/dev/null \
    || true
done <<EOF
$conflicted
EOF

fail() { echo "rebase-regenerate: FAILED: $*" >&2; exit 1; }

echo "rebase-regenerate: regenerating"
node scripts/generate-harness-adapters.js || fail "generate-harness-adapters.js"
node scripts/generate-skill-system-distribution.js || fail "generate-skill-system-distribution.js"

echo "rebase-regenerate: validating"
npm run --silent check:distribution || fail "npm run check:distribution"
npm run --silent validate:harness-adapters || fail "npm run validate:harness-adapters"
npm run --silent validate:codex || fail "npm run validate:codex"

while IFS= read -r g; do
  [ -n "$g" ] || continue
  git add -A -- "$g" || fail "git add $g"
done <<EOF
$generated
EOF

echo "rebase-regenerate: generated paths resolved and staged. Next:"
if [ "$in_progress" = rebase ]; then
  echo "  git rebase --continue"
else
  echo "  git commit"
fi
exit 0
