#!/usr/bin/env bash
# vlbv-policy-check.sh — sourced by verify.sh and verify-v4.sh (BS-04).
# A generated governing file must state VLBV in the marked policy block, carry no
# retired compiler exception or opt-in VLBV wording, and emit no /opsx:* command.
# Requires the caller's ok/no helpers.
vlbv_policy_check() { # <label> <file>
  local label="$1" file="$2" text
  [[ -f "$file" ]] || return 0
  text="$(cat "$file")"
  if printf '%s' "$text" | grep -q '<!-- kbd-policy:begin -->' && printf '%s' "$text" | grep -qE '^- verification: VLBV$'; then
    ok "$label policy block" "kbd-policy: verification VLBV"
  else
    no "$label policy block" "missing <!-- kbd-policy:begin --> block with 'verification: VLBV' — re-run the bootstrap"
  fi
  if printf '%s' "$text" | grep -qiE 'narrow(ly targeted)? compiler (check|invocation)|compiler checks needed to unblock|For selected VLBV|When VLBV is selected'; then
    no "$label VLBV wording" "retired compiler exception or opt-in VLBV wording present — re-run the bootstrap"
  else
    ok "$label VLBV wording" "no retired exception or opt-in wording"
  fi
  if printf '%s' "$text" | grep -qE '(^|[^a-z])/opsx:'; then
    no "$label OpenSpec 2.0" "emits /opsx:* commands, removed in OpenSpec 2.0"
  else
    ok "$label OpenSpec 2.0" "no /opsx:* commands"
  fi
}
