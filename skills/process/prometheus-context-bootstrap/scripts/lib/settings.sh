#!/usr/bin/env bash
# scripts/lib/settings.sh — .claude/settings.json: create from the template, repair the legacy
# .kbd-orchestrator deny rule, merge hook wiring and the skill budget into an existing file.
#
# Sourced by bootstrap.sh; one responsibility, split out to keep every script under 500 lines.
# Uses the caller's globals: project_path REF dry_run force no_hooks, and its record / warn / copy_managed.
# Sets: settings_unwired (1 when hooks are installed but could not be wired).

wire_settings() {
  settings_unwired=0
  settings="$project_path/.claude/settings.json"
  if [[ -f "$settings" && "$force" != "1" ]]; then
    # Merge rather than skip. An earlier version skipped, which left four hooks
    # installed and zero referenced — the prose had been removed and nothing
    # replaced it. Merge adds only absent keys; existing keys are never touched.
    if ! command -v jq >/dev/null 2>&1; then
      record "SKIP" ".claude/settings.json" "exists and jq absent — hooks NOT wired"
      warn "jq absent: cannot merge hook wiring. Hooks are installed but inert."
      settings_unwired=1
    else
      # Releases up to 1.10.0 shipped `deny: Edit(.kbd-orchestrator/**)`. KBD stage
      # artifacts (assessment, analysis, plan, reflection, ledgers) and kbd-init's
      # own outputs are agent-written Markdown, so that rule blocked the lifecycle
      # it was meant to protect: bootstrap told the operator to run /kbd-init next,
      # and /kbd-init could no longer write. Remove exactly that entry and nothing
      # else. Narrower, hand-written rules are the operator's and stay untouched.
      if jq -e '(.permissions.deny // []) | index("Edit(.kbd-orchestrator/**)")' "$settings" >/dev/null 2>&1; then
        if [[ "$dry_run" == "1" ]]; then
          record "REPAIR" ".claude/settings.json" "would remove legacy deny Edit(.kbd-orchestrator/**)"
        else
          repaired="$settings.tmp.$$"
          if jq '.permissions.deny |= map(select(. != "Edit(.kbd-orchestrator/**)"))' "$settings" > "$repaired" 2>/dev/null && [[ -s "$repaired" ]]; then
            cp "$settings" "$settings.bak.$(date -u +%Y%m%dT%H%M%SZ)"
            mv -f "$repaired" "$settings"
            record "REPAIR" ".claude/settings.json" "removed legacy deny Edit(.kbd-orchestrator/**); .bak kept"
          else
            rm -f "$repaired"
            warn "could not remove legacy deny Edit(.kbd-orchestrator/**) — KBD stages will be blocked until it is removed by hand"
          fi
        fi
      fi

      need_hooks=0; need_budget=0
      jq -e '.hooks.PreToolUse' "$settings" >/dev/null 2>&1 || need_hooks=1
      jq -e '.skillListingBudgetFraction' "$settings" >/dev/null 2>&1 || need_budget=1
      [[ "$no_hooks" == "1" ]] && need_hooks=0

      if [[ "$need_hooks" == "0" && "$need_budget" == "0" ]]; then
        record "SKIP" ".claude/settings.json" "already wired"
      elif [[ "$dry_run" == "1" ]]; then
        record "MERGE" ".claude/settings.json" "would add: $( [[ $need_hooks == 1 ]] && printf 'hooks '; [[ $need_budget == 1 ]] && printf 'skill-budget')"
      else
        merged="$settings.tmp.$$"
        if jq --slurpfile t "$REF/settings.template.json" \
              --argjson wh "$need_hooks" --argjson wb "$need_budget" '
              . as $cur
              | (if $wh == 1 then .hooks = ($t[0].hooks // {}) else . end)
              | (if $wb == 1 then .skillListingBudgetFraction = ($t[0].skillListingBudgetFraction // 0.02) else . end)
              | (if ($cur.permissions // null) == null then .permissions = ($t[0].permissions // {}) else . end)
            ' "$settings" > "$merged" 2>/dev/null && [[ -s "$merged" ]]; then
          cp "$settings" "$settings.bak.$(date -u +%Y%m%dT%H%M%SZ)"
          mv -f "$merged" "$settings"
          record "MERGE" ".claude/settings.json" "added: $( [[ $need_hooks == 1 ]] && printf 'hooks '; [[ $need_budget == 1 ]] && printf 'skill-budget'); .bak kept"
        else
          rm -f "$merged"
          record "SKIP" ".claude/settings.json" "merge failed — hooks NOT wired"
          warn "settings.json merge failed; hooks are installed but inert"
          settings_unwired=1
        fi
      fi
    fi
  else
    if [[ "$no_hooks" == "1" ]] && command -v jq >/dev/null 2>&1; then
      [[ "$dry_run" != "1" ]] && { mkdir -p "$(dirname "$settings")"; jq 'del(.hooks)' "$REF/settings.template.json" > "$settings"; }
      record "CREATE" ".claude/settings.json" "permissions + skill budget (hooks omitted)"
    else
      copy_managed "$REF/settings.template.json" "$settings" "permissions, skill budget, hook wiring"
    fi
  fi
}
