#!/usr/bin/env bash
# Regression: the managed region bootstrap writes must fit the ceiling verify.sh
# enforces, for every profile and with every stack, and operator prose outside
# the markers must not count against that ceiling.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
fail() { printf 'FAIL: %s\n' "$1" >&2; exit 1; }

for profile in mixed lean; do
  proj="$work/$profile"
  mkdir -p "$proj"
  # 1,200 words of operator prose: alone it would exceed the lean ceiling.
  {
    printf '# Project rules\n\n'
    for _ in $(seq 1 120); do printf 'one two three four five six seven eight nine ten\n'; done
  } > "$proj/AGENTS.md"

  bash "$here/scripts/bootstrap.sh" --path "$proj" --profile "$profile" \
    --stacks rust,typescript,flutter,go,python >/dev/null 2>&1 \
    || fail "bootstrap failed for profile $profile"

  out="$(bash "$here/scripts/verify.sh" --path "$proj" 2>&1 || true)"
  line="$(printf '%s\n' "$out" | grep 'AGENTS.md size' || true)"
  [[ "$line" == PASS* ]] || fail "profile $profile: $line"
  printf '%s' "$line" | grep -q 'project/carried' \
    || fail "profile $profile: operator prose not reported separately: $line"
done

printf 'PASS: resident budget holds for mixed and lean with all stacks\n'
