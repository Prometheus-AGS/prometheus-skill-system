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

conflict_file="$(mktemp "${TMPDIR:-/tmp}/rebase-generated-conflicts.XXXXXX")" || exit 2
trap 'rm -f "$conflict_file"' EXIT
git diff --name-only --diff-filter=U -z > "$conflict_file" || exit 2
if [ ! -s "$conflict_file" ]; then
  echo "rebase-regenerate: no conflicted paths; continue the $in_progress yourself."
  exit 0
fi

generated="$(node scripts/generated-paths.mjs)" || {
  echo "rebase-regenerate: could not derive the generated path set" >&2
  exit 2
}

# Use the production classifier also applied to observed materialized outputs.
# It reads NUL-delimited Git paths and index/HEAD gitlink modes. Empty or absent
# submodule directories never turn source gitlinks into generated content.
node scripts/generated-paths.mjs --classify-conflicts
classification=$?
[ "$classification" -eq 0 ] || exit "$classification"

# Every conflicted path is generated: its content is discarded either way, so
# take any side that exists and let the generators rewrite it.
while IFS= read -r -d '' p; do
  [ -n "$p" ] || continue
  git --literal-pathspecs checkout --theirs -- "$p" 2>/dev/null \
    || git --literal-pathspecs checkout --ours -- "$p" 2>/dev/null \
    || git --literal-pathspecs rm -q --cached -- "$p" 2>/dev/null \
    || true
done < "$conflict_file"

fail() { echo "rebase-regenerate: FAILED: $*" >&2; exit 1; }

echo "rebase-regenerate: regenerating"
node scripts/generate-harness-adapters.js || fail "generate-harness-adapters.js"
node scripts/generate-skill-system-distribution.js || fail "generate-skill-system-distribution.js"

echo "rebase-regenerate: validating"
# Production validators only. check:distribution also runs legacy isolated
# suites; rebase resolution is not a substitute for the final integration gate.
# The distribution checker covers both Claude and Codex materializations and
# still rejects missing/stale owned payload files and import closure failures.
node scripts/generate-skill-system-distribution.js --check || fail "distribution/Codex production validation"
node scripts/check-harness-adapters.js || fail "harness production validation"

while IFS= read -r g; do
  [ -n "$g" ] || continue
  git --literal-pathspecs add -A -- "$g" || fail "git add $g"
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
