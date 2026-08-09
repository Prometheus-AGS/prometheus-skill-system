# shellcheck shell=bash
# Canonical KBD completion semantics.
#
# `completion.implementation` is the only source for the implementation
# counter when present. Legacy `changes_completed` / `changes_total` remain
# supported as aliases for pre-v4 ledgers. Evidence, certification, and
# publication are independent dimensions and MUST NOT reduce implementation.

kbd_progress_implementation_completed() {
  jq -r '
    .completion.implementation.completed
    // .implementation_completed
    // .changes_completed
    // 0
  ' "$1"
}

kbd_progress_implementation_total() {
  jq -r '
    .completion.implementation.total
    // .implementation_total
    // .changes_total
    // 0
  ' "$1"
}

kbd_progress_dimension_status() {
  local file="$1" dimension="$2"
  jq -r --arg dimension "$dimension" '
    .completion[$dimension].status
    // (if $dimension == "implementation" then
          (if ((.completion.implementation.completed // .implementation_completed // .changes_completed // 0)
               >= (.completion.implementation.total // .implementation_total // .changes_total // 0))
           then "COMPLETE" else "IN_PROGRESS" end)
        else "NOT_TRACKED" end)
  ' "$file"
}

kbd_progress_validate() {
  local file="$1"
  command -v jq >/dev/null 2>&1 || {
    printf 'kbd-progress: jq is required\n' >&2
    return 1
  }
  jq empty "$file" >/dev/null 2>&1 || {
    printf 'kbd-progress: invalid JSON: %s\n' "$file" >&2
    return 1
  }

  # Schema v2 is strict and array-only. Legacy readers remain intentionally
  # tolerant until `prometheus kbd migrate --apply` upgrades old ledgers.
  jq -e '
    def changes_list:
      if (.changes | type) == "array" then .changes
      elif (.changes | type) == "object" then [.changes[]]
      else [] end;
    def object_rows:
      [changes_list[] | select(type == "object")];
    def impl_done:
      .completion.implementation.completed
      // .implementation_completed
      // .changes_completed
      // 0;
    def impl_total:
      .completion.implementation.total
      // .implementation_total
      // .changes_total
      // 0;
    def valid_dimension_status:
      . == "NOT_TRACKED" or . == "NOT_REQUIRED" or . == "PENDING" or
      . == "IN_PROGRESS" or . == "COMPLETE" or . == "BLOCKED";

    (impl_done | type == "number" and . >= 0) and
    (impl_total | type == "number" and . >= 0) and
    (impl_done <= impl_total) and
    (if .completion? then
       (.completion.primaryCounter == "implementation") and
       (.completion.implementation | type == "object") and
       (.completion.implementation.status | valid_dimension_status) and
       ([.completion.evidence?, .completion.certification?, .completion.publication?]
        | map(select(. != null) | (.status | valid_dimension_status))
        | all) and
       (if has("changes_completed") then .changes_completed == impl_done else true end) and
       (if has("changes_total") then .changes_total == impl_total else true end) and
       (if has("implementation_completed") then .implementation_completed == impl_done else true end) and
       (if has("implementation_total") then .implementation_total == impl_total else true end)
     else true end) and
    (if .schemaVersion? == "2" then
       (.phase | type == "string" and length > 0) and
       (.last_updated | type == "string" and length > 0) and
       (.last_updated_by | type == "string" and length > 0) and
       (.changes | type == "array") and
       ([.changes[] | type == "object" and
                      (.id | type == "string" and length > 0) and
                      has("status") and has("implementation_status")] | all) and
       ([.changes[].id] | length == (unique | length))
     else true end) and
    (if (object_rows | length) > 0 and ([object_rows[] | has("implementation_status")] | any) then
       ([object_rows[] | has("implementation_status")] | all) and
       ([object_rows[] | select(.implementation_status == "COMPLETE")] | length) == impl_done
     else true end)
  ' "$file" >/dev/null || {
    printf 'kbd-progress: completion invariant failed: %s\n' "$file" >&2
    return 1
  }
}

# Mark one change's code/integration contract complete and atomically derive
# the implementation counter. Evidence/certification/publication fields are
# deliberately untouched.
#
# `changes` may be an array of objects (canonical), an array of strings
# (string-list ledgers), or a map keyed by change id. For string-list ledgers
# there is no per-change `implementation_status` to update — the phase-level
# counter is the source of truth — so the per-change update is a no-op and
# the mark-completion reduces to deriving the counter from `changes_completed`
# (caller is expected to have already advanced it). For object arrays we look
# up by `.id`; for maps we look up by key.
kbd_progress_mark_implementation_complete() {
  local file="$1" change_id="$2" tmp
  if [ "$(jq -r '.generatedBy // empty' "$file" 2>/dev/null)" = "kbd-runtime" ]; then
    command -v prometheus >/dev/null 2>&1 || {
      printf 'kbd-progress: prometheus CLI is required in runtime-authority mode\n' >&2
      return 1
    }
    local root phase state revision
    case "$file" in
      */.kbd-orchestrator/*) root="${file%%/.kbd-orchestrator/*}" ;;
      .kbd-orchestrator/*) root="." ;;
      *) printf 'kbd-progress: cannot resolve project root from %s\n' "$file" >&2; return 1 ;;
    esac
    phase="$(jq -r '.phase // empty' "$file" 2>/dev/null)"
    [ -n "$phase" ] || {
      printf 'kbd-progress: active phase is required\n' >&2
      return 1
    }
    prometheus kbd --path "$root" change transition \
      --command-id "implementation-complete:${phase}:${change_id}" \
      --phase "$phase" \
      --id "$change_id" \
      --status complete >/dev/null
    return $?
  fi
  tmp="$(mktemp "${file}.XXXXXX")" || return 1
  if ! jq --arg id "$change_id" '
    def changes_list:
      if (.changes | type) == "array" then .changes
      elif (.changes | type) == "object" then [.changes[]]
      else [] end;
    def object_rows:
      [changes_list[] | select(type == "object")];
    def mark_change:
      if (.changes | type) == "array" then
        .changes |= map(
          if (type == "object") and (.id == $id) then
            .implementation_status = "COMPLETE"
          else
            .
          end
        )
      elif (.changes | type) == "object" and (.changes | has($id)) then
        .changes[$id].implementation_status = "COMPLETE"
      else
        .
      end;
    def known_change:
      if (.changes | type) == "array" then
        (.changes | map(select((type == "object") and (.id == $id))) | length) > 0
          or (.changes | map(select(type == "string" and . == $id)) | length) > 0
      elif (.changes | type) == "object" then
        (.changes | has($id))
      else
        false
      end;
    if (known_change | not) then
      error("unknown change id: " + $id)
    else
      mark_change |
      (changes_list | length) as $change_count |
      (object_rows | length) as $object_count |
      (if $object_count > 0 then
         ([object_rows[] |
           select((.implementation_status //
                   (if .status == "DONE" then "COMPLETE" else "PENDING" end)) == "COMPLETE")]
          | length)
       else
         (.changes_completed // .implementation_completed // 0)
       end) as $done |
      (.implementation_total // .changes_total // $change_count) as $total |
      .completion = (.completion // {}) |
      .completion.primaryCounter = "implementation" |
      .completion.implementation = {
        completed: $done,
        total: $total,
        status: (if $done >= $total then "COMPLETE" else "IN_PROGRESS" end)
      } |
      .completion.evidence = (.completion.evidence // {status:"NOT_TRACKED", summary:null, blockers:[]}) |
      .completion.certification = (.completion.certification // {status:"NOT_TRACKED", summary:null, blockers:[]}) |
      .completion.publication = (.completion.publication // {status:"NOT_TRACKED", summary:null, blockers:[]}) |
      .implementation_completed = $done |
      .implementation_total = $total |
      .changes_completed = $done |
      .changes_total = $total
    end
  ' "$file" > "$tmp"; then
    rm -f "$tmp"
    return 1
  fi
  if ! kbd_progress_validate "$tmp"; then
    rm -f "$tmp"
    return 1
  fi
  mv "$tmp" "$file"
}
