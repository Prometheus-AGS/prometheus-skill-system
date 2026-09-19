#!/usr/bin/env bash
# shared/scripts/lib/slug.sh — slug and package-id helpers shared by the learn and
# research skills. Source it; do not execute it.
#
#   subject_to_slug "<text>"       lowercase, non-alphanumerics to "-", trimmed (unbounded length;
#                                  the learn-goal id convention)
#   slug_from_text "<text>" [N]    at most N words (default 5), filler words dropped, same
#                                  normalisation; never empty ("research" when nothing survives)
#   package_id_new "<slug>"        "<slug>-<yyyymmdd>-<4hex>", the research package directory name
#   package_id_is_valid "<id>"     exit 0 when <id> matches ^[a-z0-9]+(-[a-z0-9]+){0,4}-[0-9]{8}-[0-9a-f]{4}$
#
# bash 3.2 compatible (constraint C-05): no mapfile, no declare -A, no ${var,,}.

subject_to_slug() {
  printf '%s\n' "$1" \
    | tr '[:upper:]' '[:lower:]' \
    | sed 's/[^a-z0-9]\{1,\}/-/g' \
    | sed 's/^-//; s/-$//'
}

# Words that carry no topic information and are dropped from a bounded slug.
# Kept small and literal on purpose; a slug is a directory name, not a summary.
SLUG_FILLER_WORDS=" the a an of for and or in on to with what is are how does do current state latest recent about vs versus "

slug_from_text() {
  local text="$1" max="${2:-5}" out="" count=0 w
  local normalised
  normalised="$(printf '%s' "$text" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]\{1,\}/ /g')"
  for w in $normalised; do
    case "$SLUG_FILLER_WORDS" in
      *" $w "*) continue ;;
    esac
    if [ -n "$out" ]; then out="$out-$w"; else out="$w"; fi
    count=$((count + 1))
    [ "$count" -ge "$max" ] && break
  done
  if [ -z "$out" ]; then
    # Every word was filler; fall back to the first real token or a fixed word.
    for w in $normalised; do out="$w"; break; done
    [ -n "$out" ] || out="research"
  fi
  printf '%s\n' "$out"
}

package_id_new() {
  local slug="$1" hex
  hex="$(od -An -N2 -tx1 /dev/urandom 2>/dev/null | tr -d ' \n')"
  [ -n "$hex" ] || hex="$(printf '%04x' $(( $(date +%s) % 65536 )))"
  printf '%s-%s-%s\n' "$slug" "$(date -u +%Y%m%d)" "$hex"
}

package_id_is_valid() {
  printf '%s' "$1" | grep -Eq '^[a-z0-9]+(-[a-z0-9]+){0,4}-[0-9]{8}-[0-9a-f]{4}$'
}
