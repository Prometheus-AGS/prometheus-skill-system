#!/usr/bin/env bash
# Integration test for scripts/run-research.sh against references/stage-contracts.md.
#
#   bash tests/driver-contract.sh                  # all scenarios
#   bash tests/driver-contract.sh --scenario NAME  # one of: full-run shallow-depth resume-from-04 failing-05
#                                                  #   invalid-05-blocks-06 direct-scale hooks-fired
#                                                  #   hook-failure checkpoint-mode review-clean-verified
#                                                  #   review-critical review-warning review-unavailable
#                                                  #   review-without-verify review-real-dispatch
#                                                  #   frontmatter-body
#
# Every scenario runs the real driver with the fixture runner in
# tests/fixtures/stage-runner.sh and the fixture judge in tests/fixtures/judge.sh
# (RESEARCH_JUDGE_CMD, so no scenario needs a gateway) and asserts on files left
# on disk (checkpoint, sidecar, manifest, hook log, runner call log, judge call
# log), never on driver stdout alone. Runs under bash 3.2 (constraint C-05).
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
SKILL_DIR="$(cd "$HERE/.." && pwd)"
DRIVER="$SKILL_DIR/scripts/run-research.sh"
RUNNER="bash $HERE/fixtures/stage-runner.sh"
JUDGE="bash $HERE/fixtures/judge.sh"
ONLY=""
[ "${1:-}" = "--scenario" ] && ONLY="${2:-}"

PASS=0; FAIL=0
ok()   { echo "  PASS $*"; PASS=$((PASS+1)); }
fail() { echo "  FAIL $*" >&2; FAIL=$((FAIL+1)); }
assert() { # assert "<desc>" <command...>
  local d="$1"; shift
  if "$@" >/dev/null 2>&1; then ok "$d"; else fail "$d"; fi
}
fresh() { # fresh → prints a new root; sets ROOT LOG CALLS JLOG
  ROOT="$(mktemp -d)"; LOG="$ROOT/hooks.log"; CALLS="$ROOT/calls.txt"; JLOG="$ROOT/judge.log"; : > "$LOG"; : > "$CALLS"; : > "$JLOG"
}
run_driver() { # run_driver <extra env...> -- <driver args...>; sets RC
  local envs=()
  while [ $# -gt 0 ] && [ "$1" != "--" ]; do envs+=("$1"); shift; done; shift
  set +e
  env RESEARCH_STAGE_RUNNER="$RUNNER" RESEARCH_JUDGE_CMD="$JUDGE" RESEARCH_OUTPUT_DIR="$ROOT" RESEARCH_HOOK_LOG="$LOG" FIXTURE_CALLS="$CALLS" FIXTURE_JUDGE_LOG="$JLOG" KBD_PRODUCER_MODEL="fixture/producer-under-test" ${envs[@]+"${envs[@]}"} bash "$DRIVER" "$@" >"$ROOT/stdout.txt" 2>"$ROOT/stderr.txt"
  RC=$?
  set -e
}
judge_calls() { wc -l < "$JLOG" | tr -d ' '; }
report_status() { grep -o '^verification_status: [a-z]*' "$(pkg_dir)/report.md" | sed 's/verification_status: //'; }
pkg_dir() { ls -d "$ROOT"/*/ 2>/dev/null | head -1 | sed 's:/$::'; }
calls() { tr '\n' ' ' < "$CALLS" | sed 's/ $//'; }
hooks() { cut -d' ' -f2- "$LOG" | sed 's/ exit=[0-9]*//' | tr '\n' ',' | sed 's/,$//'; }
sidecar() { ls "$(pkg_dir)"/*.provenance.md 2>/dev/null | head -1; }
verdict() { grep -o 'Verification:\*\* [A-Z ]*' "$(sidecar)" | sed 's/Verification:\*\* //'; }

scenario() { [ -z "$ONLY" ] || [ "$ONLY" = "$1" ]; }

if scenario full-run; then
  echo "[full-run] four sub-questions, deep: all ten stages, validated package"
  fresh; run_driver FIXTURE_SUBQ=4 -- --query "Current state of vector databases for production RAG systems" --depth deep
  assert "driver exits 0" test "$RC" -eq 0
  assert "runner called for 01..09 in order" test "$(calls)" = "01 02 03 04 05 06 07 08 09"
  assert "checkpoint status complete with ten stages" jq -e '.status=="complete" and (.stages_completed|length)==10 and .scale=="full"' "$(pkg_dir)/checkpoint.json"
  assert "manifest validates (check-research-package --package)" bash "$SKILL_DIR/scripts/check-research-package.sh" --package "$(pkg_dir)"
  assert "manifest lists ten stages and scale full" jq -e '(.stages_completed|length)==10 and .scale=="full"' "$(pkg_dir)/manifest.json"
  # The report was judged (fixture judge, PASS) after 09 and before export, so the
  # sidecar records the review and the verdict is a clean PASS.
  assert "sidecar verdict is exactly PASS (review recorded)" test "$(verdict)" = "PASS"
  assert "sidecar records the review verdict" grep -q 'Adversarial review:\*\* PASS (0 CRITICAL, 0 WARNING)' "$(sidecar)"
  assert "judge called exactly once, after stage 09 completed" bash -c "test \$(wc -l < '$JLOG') -eq 1 && grep -q '09 | PASS' '$JLOG'"
  assert "review packet and findings live in the package" bash -c "test -s '$(pkg_dir)/review/packet.json' && test -s '$(pkg_dir)/review/findings.json'"
  assert "checkpoint records the review and adversarial_review_used" jq -e '.review.verdict=="PASS" and .integrations.adversarial_review_used==true and .blocked_review==null' "$(pkg_dir)/checkpoint.json"
  assert "no ledger line was left pending" test ! -f "$(pkg_dir)/.ledger-pending.md"
  assert "pre-research hook result reached the plan ledger" grep -q 'hook pre-research exit 0' "$(pkg_dir)/plan.md"
fi

if scenario shallow-depth; then
  echo "[shallow-depth] shallow runs 01 02 03 04 05 09 10 and still completes with a package"
  fresh; run_driver FIXTURE_SUBQ=4 -- --query "shallow depth fixture query" --depth shallow
  assert "driver exits 0" test "$RC" -eq 0
  assert "runner called for 01 02 03 04 05 09" test "$(calls)" = "01 02 03 04 05 09"
  assert "checkpoint complete with seven stages" jq -e '.status=="complete" and .stages_completed==["01","02","03","04","05","09","10"]' "$(pkg_dir)/checkpoint.json"
  assert "manifest validates" bash "$SKILL_DIR/scripts/check-research-package.sh" --package "$(pkg_dir)"
  assert "plan.md has the three ledger sections" bash -c "grep -q '^## Task ledger' '$(pkg_dir)/plan.md' && grep -q '^## Verification log' '$(pkg_dir)/plan.md' && grep -q '^## Decision log' '$(pkg_dir)/plan.md'"
  assert "decision log records the scale decision" grep -q 'scale full: planner emitted 4' "$(pkg_dir)/plan.md"
fi

if scenario resume-from-04; then
  echo "[resume-from-04] runner fails at 05, then --resume re-runs from 05 only"
  fresh; run_driver FIXTURE_EXIT_STAGE=05 -- --query "resume fixture query" --depth deep
  assert "first run blocked (exit 1)" test "$RC" -eq 1
  assert "stages 01..04 recorded complete" jq -e '.stages_completed==["01","02","03","04"] and .status=="blocked" and .blocked.stage=="05"' "$(pkg_dir)/checkpoint.json"
  assert "sidecar BLOCKED names stage 05" bash -c "grep -q 'Verification:\*\* BLOCKED' '$(sidecar)' && grep -q 'Blocked:\*\* stage 05' '$(sidecar)'"
  : > "$CALLS"
  P="$(pkg_dir)"; run_driver -- --resume "$P"
  assert "resume exits 0" test "$RC" -eq 0
  assert "resume ran 05..09 only" test "$(calls)" = "05 06 07 08 09"
  assert "no pre-research on resume" bash -c "! grep -q 'pre-research' '$LOG' || test \$(grep -c pre-research '$LOG') -eq 1"
  assert "final checkpoint complete" jq -e '.status=="complete" and (.stages_completed|length)==10' "$P/checkpoint.json"
  assert "stale block cleared after recovery" jq -e '.blocked==null' "$P/checkpoint.json"
  assert "final sidecar has Blocked: none next to its verdict" bash -c "grep -q 'Blocked:\*\* none' '$(sidecar)'"
fi

if scenario failing-05; then
  echo "[failing-05] stage 05 writes an invalid artifact: BLOCKED sidecar, non-zero exit, 06 never runs"
  fresh; run_driver FIXTURE_FAIL_STAGE=05 -- --query "failing verify fixture" --depth deep
  assert "driver exits non-zero" test "$RC" -ne 0
  assert "runner stopped at 05" test "$(calls)" = "01 02 03 04 05"
  assert "checkpoint blocked at 05 with the contract reason" jq -e '.status=="blocked" and .blocked.stage=="05" and (.blocked.reason|test("artifact contract"))' "$(pkg_dir)/checkpoint.json"
  assert "sidecar verdict BLOCKED" test "$(verdict)" = "BLOCKED"
  assert "no manifest was exported" test ! -f "$(pkg_dir)/manifest.json"
fi

if scenario invalid-05-blocks-06; then
  echo "[invalid-05-blocks-06] 05 recorded complete but its artifact is corrupt: 06 is refused"
  fresh; run_driver FIXTURE_EXIT_STAGE=06 -- --query "gate fixture query" --depth deep
  P="$(pkg_dir)"; echo '{"scores":"corrupt"}' > "$P/sources/credibility.json"
  : > "$CALLS"
  # The runner cannot repair 05 either (FAIL_STAGE=05), so the driver must block before 06.
  run_driver FIXTURE_FAIL_STAGE=05 -- --resume "$P"
  assert "driver exits non-zero" test "$RC" -ne 0
  assert "06 was never invoked" bash -c "! grep -qx 06 '$CALLS'"
  assert "block names stage 05" jq -e '.status=="blocked" and .blocked.stage=="05"' "$P/checkpoint.json"
  assert "sidecar verdict BLOCKED" test "$(verdict)" = "BLOCKED"
fi

if scenario direct-scale; then
  echo "[direct-scale] two sub-questions: stages 01 02 03 05 09 10, no subagent stages"
  fresh; run_driver FIXTURE_SUBQ=2 -- --query "what is the feynman technique" --depth deep
  assert "driver exits 0" test "$RC" -eq 0
  assert "runner called for 01 02 03 05 09 only" test "$(calls)" = "01 02 03 05 09"
  assert "checkpoint scale direct" jq -e '.scale=="direct" and .stages_completed==["01","02","03","05","09","10"]' "$(pkg_dir)/checkpoint.json"
  assert "manifest scale direct" jq -e '.scale=="direct"' "$(pkg_dir)/manifest.json"
  assert "sidecar PASS WITH NOTES (direct)" test "$(verdict)" = "PASS WITH NOTES"
  assert "decision log records direct" grep -q 'scale direct: planner emitted 2' "$(pkg_dir)/plan.md"
fi

if scenario hooks-fired; then
  echo "[hooks-fired] all four hooks, in order, with one unresolved contradiction"
  fresh; run_driver FIXTURE_SUBQ=4 FIXTURE_UNRESOLVED=1 -- --query "hooks fixture query" --depth deep
  assert "driver exits 0" test "$RC" -eq 0
  EXPECT="pre-research,post-stage 01,post-stage 02,post-stage 03,post-stage 04,post-stage 05,post-stage 06,on-contradiction 06,post-stage 07,post-stage 08,post-stage 09,post-stage 10,post-export"
  assert "hook markers in order: $EXPECT" test "$(hooks)" = "$EXPECT"
  assert "checkpoint hook_log has 13 entries all exit 0" jq -e '(.hook_log|length)==13 and ([.hook_log[]|select(.exit!=0)]|length)==0' "$(pkg_dir)/checkpoint.json"
  assert "contradiction event log written in the package" test -s "$(pkg_dir)/contradiction-events.log"
  echo "[hooks-fired] negative control: no unresolved contradiction, no on-contradiction marker"
  fresh; run_driver FIXTURE_SUBQ=4 -- --query "hooks negative fixture" --depth deep
  assert "no on-contradiction marker" bash -c "! grep -q on-contradiction '$LOG'"
fi

if scenario hook-failure; then
  echo "[hook-failure] a post-stage hook exiting non-zero blocks the stage, names the hook, and resume recovers"
  # Stub hook dir: real hooks, except post-stage fails when FIXTURE_HOOK_FAIL_STAGE matches.
  STUB="$(mktemp -d)/hooks"; mkdir -p "$STUB"; cp "$SKILL_DIR"/hooks/*.sh "$STUB/"
  cat > "$STUB/post-stage.sh" <<'EOF'
#!/usr/bin/env bash
if [ "${FIXTURE_HOOK_FAIL_STAGE:-}" = "${RESEARCH_CURRENT_STAGE:-}" ]; then echo "[stub post-stage] simulated failure at $RESEARCH_CURRENT_STAGE" >&2; exit 5; fi
exec bash "$(dirname "$0")/post-stage.real.sh"
EOF
  cp "$SKILL_DIR/hooks/post-stage.sh" "$STUB/post-stage.real.sh"
  fresh; run_driver RESEARCH_HOOK_DIR="$STUB" FIXTURE_SUBQ=4 FIXTURE_HOOK_FAIL_STAGE=03 -- --query "hook failure fixture" --depth deep
  assert "driver exits non-zero" test "$RC" -ne 0
  assert "block names the post-stage hook at 03" jq -e '.status=="blocked" and .blocked.stage=="03" and (.blocked.reason|test("post-stage hook"))' "$(pkg_dir)/checkpoint.json"
  assert "hook_log records exit 5 for post-stage" jq -e '[.hook_log[]|select(.hook=="post-stage" and .exit==5)]|length==1' "$(pkg_dir)/checkpoint.json"
  assert "stage 03 not recorded complete" jq -e '.stages_completed==["01","02"]' "$(pkg_dir)/checkpoint.json"
  assert "sidecar BLOCKED names stage 03" bash -c "grep -q 'Blocked:\*\* stage 03' '$(sidecar)'"
  P="$(pkg_dir)"; run_driver RESEARCH_HOOK_DIR="$STUB" -- --resume "$P"
  assert "resume without the failure completes" test "$RC" -eq 0
  assert "checkpoint complete and blocked cleared" jq -e '.status=="complete" and .blocked==null and (.stages_completed|length)==10' "$P/checkpoint.json"
  assert "final sidecar shows Blocked: none with a PASS verdict" bash -c "grep -q 'Blocked:\*\* none' '$(sidecar)' && grep -q 'Verification:\*\* PASS' '$(sidecar)'"
  echo "[hook-failure] the same at stage 10 must not wedge the run"
  fresh; run_driver RESEARCH_HOOK_DIR="$STUB" FIXTURE_SUBQ=4 FIXTURE_HOOK_FAIL_STAGE=10 -- --query "stage ten hook failure fixture" --depth deep
  assert "driver exits non-zero at stage 10" test "$RC" -ne 0
  assert "stage 10 rolled back out of stages_completed" jq -e '.status=="blocked" and .blocked.stage=="10" and (.stages_completed|index("10"))==null' "$(pkg_dir)/checkpoint.json"
  P="$(pkg_dir)"; run_driver RESEARCH_HOOK_DIR="$STUB" -- --resume "$P"
  assert "resume re-runs export and completes" test "$RC" -eq 0
  assert "manifest and sidecar verdicts agree after recovery" bash -c "test \"\$(jq -r .verification_verdict '$P/manifest.json')\" = \"\$(grep -o 'Verification:\*\* [A-Z ]*' '$(sidecar)' | sed 's/Verification:\*\* //')\""
fi

if scenario checkpoint-mode; then
  echo "[checkpoint-mode] no runner: driver stops at stage 01 with a next_stage line and exit 3"
  fresh
  set +e; env RESEARCH_OUTPUT_DIR="$ROOT" RESEARCH_HOOK_LOG="$LOG" bash "$DRIVER" --query "checkpoint mode fixture" --depth deep >"$ROOT/stdout.txt" 2>"$ROOT/stderr.txt"; RC=$?; set -e
  assert "driver exits 3" test "$RC" -eq 3
  assert "stdout carries next_stage 01" bash -c "jq -e '.next_stage==\"01\" and .skill==\"stage-01-planner\"' '$ROOT/stdout.txt'"
  assert "checkpoint awaiting_stage" jq -e '.status=="awaiting_stage" and .current_stage=="01"' "$(pkg_dir)/checkpoint.json"
  assert "interim sidecar is not BLOCKED" bash -c "! grep -q 'Verification:\*\* BLOCKED' '$(sidecar)'"
  # Simulate the harness running stage 01, then resume: next stop is stage 02.
  P="$(pkg_dir)"; FIXTURE_SUBQ=4 bash "$HERE/fixtures/stage-runner.sh" 01 "$P"
  set +e; env RESEARCH_OUTPUT_DIR="$ROOT" RESEARCH_HOOK_LOG="$LOG" bash "$DRIVER" --resume "$P" >"$ROOT/stdout.txt" 2>"$ROOT/stderr.txt"; RC=$?; set -e
  assert "resume exits 3 at stage 02" bash -c "test $RC -eq 3 && jq -e '.next_stage==\"02\"' '$ROOT/stdout.txt'"
  assert "stage 01 recorded complete" jq -e '.stages_completed==["01"]' "$P/checkpoint.json"
fi

if scenario review-clean-verified; then
  echo "[review-clean-verified] passing gates and a PASS review: the run ends verified, not self-blocked"
  fresh; run_driver FIXTURE_SUBQ=4 FIXTURE_REPORT_GATE=pass -- --query "clean verified fixture query" --depth deep
  assert "driver exits 0" test "$RC" -eq 0
  assert "checkpoint complete, gate and review both used" jq -e '.status=="complete" and .integrations.feynman_gate_used==true and .integrations.adversarial_review_used==true and .review.verdict=="PASS"' "$(pkg_dir)/checkpoint.json"
  assert "report frontmatter verified" test "$(report_status)" = "verified"
  assert "manifest verification_status verified, verdict PASS" jq -e '.verification_status=="verified" and .verification_verdict=="PASS"' "$(pkg_dir)/manifest.json"
  assert "sidecar verdict PASS" test "$(verdict)" = "PASS"
  assert "exported package passes the drift check" bash "$SKILL_DIR/scripts/check-research-package.sh" --package "$(pkg_dir)"
  assert "derive agrees" test "$(bash "$SKILL_DIR/scripts/check-research-package.sh" --derive "$(pkg_dir)")" = "verified"
  assert "ledger records the derived status" grep -q 'report verification_status set to verified (derived after review)' "$(pkg_dir)/plan.md"
  # Honesty at packet time: the judge saw the pre-review value (partial, since the
  # review had not run) beside a sidecar saying the review was pending, not `verified`
  # beside `not run`.
  assert "judge saw the pre-review frontmatter (partial) and a pending review" grep -q 'saw=partial | pending' "$JLOG"
  assert "judge saw the producer identity" grep -q 'producer=fixture/producer-under-test' "$JLOG"
  assert "checkpoint records the producer model" jq -e '.producer_model=="fixture/producer-under-test"' "$(pkg_dir)/checkpoint.json"
  echo "[review-clean-verified] negative control: same gates, no review possible → partial, never verified"
  fresh; run_driver FIXTURE_SUBQ=4 FIXTURE_REPORT_GATE=pass FIXTURE_JUDGE=unavailable -- --query "clean but unjudged fixture query" --depth deep
  assert "driver exits 0" test "$RC" -eq 0
  assert "report lowered from verified to partial" test "$(report_status)" = "partial"
  assert "manifest partial" jq -e '.verification_status=="partial"' "$(pkg_dir)/manifest.json"
fi

if scenario review-critical; then
  echo "[review-critical] fixture judge returns one CRITICAL: sidecar BLOCKED, report lowered from verified to partial, package still exported"
  fresh; run_driver FIXTURE_SUBQ=4 FIXTURE_REPORT_GATE=pass FIXTURE_JUDGE=critical -- --query "critical review fixture query" --depth deep
  assert "driver exits 0 (the run completed; the verdict is what is blocked)" test "$RC" -eq 0
  assert "checkpoint complete with the review recorded as BLOCK, 1 CRITICAL, 1 WARNING" jq -e '.status=="complete" and .review.verdict=="BLOCK" and .review.critical==1 and .review.warning==1 and .integrations.adversarial_review_used==true' "$(pkg_dir)/checkpoint.json"
  assert "sidecar verdict BLOCKED" test "$(verdict)" = "BLOCKED"
  assert "sidecar Blocked line names the review" grep -q 'Blocked:\*\* adversarial review: 1 CRITICAL' "$(sidecar)"
  assert "sidecar lists the WARNING under Review warnings" bash -c "grep -q 'Review warnings:' '$(sidecar)' && grep -q 'sub-question Q3 is not answered' '$(sidecar)'"
  assert "report frontmatter lowered from verified to partial" test "$(report_status)" = "partial"
  assert "manifest exported with verification_status partial and verdict BLOCKED" jq -e '.verification_status=="partial" and .verification_verdict=="BLOCKED"' "$(pkg_dir)/manifest.json"
  assert "exported package passes the drift check" bash "$SKILL_DIR/scripts/check-research-package.sh" --package "$(pkg_dir)"
  assert "ledger records the BLOCK" grep -q 'report review BLOCK: 1 CRITICAL' "$(pkg_dir)/plan.md"
fi

if scenario review-warning; then
  echo "[review-warning] a WARNING is appended to the sidecar and does not change the verdict"
  fresh; run_driver FIXTURE_SUBQ=4 FIXTURE_JUDGE=warning -- --query "warning review fixture query" --depth deep
  assert "driver exits 0" test "$RC" -eq 0
  assert "sidecar verdict PASS" test "$(verdict)" = "PASS"
  assert "sidecar carries the warning text" grep -q 'inferred claim appears in the executive summary' "$(sidecar)"
  assert "report frontmatter unchanged" test "$(report_status)" = "partial"
  assert "manifest verdict PASS" jq -e '.verification_verdict=="PASS"' "$(pkg_dir)/manifest.json"
fi

if scenario review-unavailable; then
  echo "[review-unavailable] no judge reachable: recorded as blocked, never skipped silently; package exports partial"
  fresh; run_driver FIXTURE_SUBQ=4 FIXTURE_REPORT_GATE=pass FIXTURE_JUDGE=unavailable -- --query "unavailable judge fixture query" --depth deep
  assert "driver exits 0 (run completed)" test "$RC" -eq 0
  assert "checkpoint records blocked_review judge unavailable and no review" jq -e '.status=="complete" and (.blocked_review|test("^blocked: judge unavailable")) and .review==null and .integrations.adversarial_review_used==false' "$(pkg_dir)/checkpoint.json"
  assert "sidecar contains the blocked line" grep -q 'Adversarial review:\*\* blocked: judge unavailable' "$(sidecar)"
  assert "sidecar verdict PASS WITH NOTES (a note, not a silent skip)" test "$(verdict)" = "PASS WITH NOTES"
  assert "report frontmatter lowered from verified to partial" test "$(report_status)" = "partial"
  assert "manifest verification_status partial" jq -e '.verification_status=="partial"' "$(pkg_dir)/manifest.json"
  assert "exported package passes the drift check" bash "$SKILL_DIR/scripts/check-research-package.sh" --package "$(pkg_dir)"
  assert "no judge call was recorded" test "$(judge_calls)" -eq 0
  echo "[review-unavailable] resume with the judge back retries the review once and ends verified"
  P="$(pkg_dir)"; run_driver FIXTURE_REPORT_GATE=pass -- --resume "$P"
  assert "resume exits 0" test "$RC" -eq 0
  assert "judge called exactly once on resume" test "$(judge_calls)" -eq 1
  assert "blocked_review cleared, review recorded PASS" jq -e '.blocked_review==null and .review.verdict=="PASS"' "$P/checkpoint.json"
  assert "report and manifest now verified" bash -c "test \"\$(grep -o '^verification_status: [a-z]*' '$P/report.md')\" = 'verification_status: verified' && jq -e '.verification_status==\"verified\"' '$P/manifest.json' >/dev/null"
fi

if scenario review-without-verify; then
  echo "[review-without-verify] stage 05 artifact gone by review time: judge never dispatched, refusal recorded, export partial"
  fresh; run_driver FIXTURE_SUBQ=4 FIXTURE_REPORT_GATE=pass FIXTURE_DROP_05_AT_09=1 -- --query "review without verification fixture" --depth deep
  assert "driver exits 0 (run completed)" test "$RC" -eq 0
  assert "judge was never called" test "$(judge_calls)" -eq 0
  assert "checkpoint records the refusal" jq -e '.blocked_review=="blocked: review refused, stage 05 verification missing or invalid" and .review==null' "$(pkg_dir)/checkpoint.json"
  assert "sidecar carries the refusal line" grep -q 'blocked: review refused, stage 05 verification missing or invalid' "$(sidecar)"
  assert "report frontmatter lowered from verified to partial" test "$(report_status)" = "partial"
  assert "manifest verification_status partial" jq -e '.verification_status=="partial"' "$(pkg_dir)/manifest.json"
  assert "ledger records the refusal" grep -q 'adversarial review blocked: review refused' "$(pkg_dir)/plan.md"
  echo "[review-without-verify] resume re-runs the stale stage 05, then retries the review once (verifier before reviewer, in order)"
  P="$(pkg_dir)"; run_driver FIXTURE_REPORT_GATE=pass -- --resume "$P"
  assert "resume exits 0" test "$RC" -eq 0
  assert "stage 05 re-ran before the judge was called" bash -c "grep -qx 05 '$CALLS' && test \$(wc -l < '$JLOG') -eq 1 && grep -q '05' '$JLOG'"
  assert "refusal cleared, review recorded" jq -e '.blocked_review==null and .review.verdict=="PASS"' "$P/checkpoint.json"
  echo "[review-without-verify] a resume whose stage 05 still cannot validate refuses again"
  fresh; run_driver FIXTURE_SUBQ=4 FIXTURE_DROP_05_AT_09=1 -- --query "still unverifiable fixture" --depth deep
  P="$(pkg_dir)"; run_driver FIXTURE_FAIL_STAGE=05 -- --resume "$P"
  assert "resume blocks at stage 05 and the judge is never called" bash -c "test $RC -ne 0 && test \$(wc -l < '$JLOG') -eq 0 && jq -e '.blocked.stage==\"05\"' '$P/checkpoint.json' >/dev/null"
fi

if scenario review-real-dispatch; then
  echo "[review-real-dispatch] no RESEARCH_JUDGE_CMD: the driver calls the real packet builder and dispatch-judge.sh's CLI (stubbed at the gateway boundary)"
  # A copy of the adversarial-review skill whose dispatch-judge.sh is a stub that
  # checks the CLI contract the driver must honour (--mode artifact --packet X
  # --out Y), requires a known producer in the packet, and answers per
  # FIXTURE_DISPATCH: pass (writes PASS findings), gateway (exit 3), garbage (exit 2).
  ADV_STUB="$(mktemp -d)/adversarial-review"; mkdir -p "$ADV_STUB/scripts" "$ADV_STUB/assets"
  cp "$SKILL_DIR/../../process/adversarial-review/scripts/build-review-packet.sh" "$ADV_STUB/scripts/"
  cp "$SKILL_DIR/../../process/adversarial-review/assets/reviewer-mandate-artifact.md" "$ADV_STUB/assets/"
  cat > "$ADV_STUB/scripts/dispatch-judge.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
MODE=""; PACKET=""; OUT=""
while [ $# -gt 0 ]; do case "$1" in --mode) MODE="$2"; shift 2;; --packet) PACKET="$2"; shift 2;; --out) OUT="$2"; shift 2;; *) echo "[stub judge] unexpected argument: $1" >&2; exit 4;; esac; done
[ "$MODE" = "artifact" ] || { echo "[stub judge] --mode must be artifact for a research packet (got '$MODE')" >&2; exit 4; }
[ -s "$PACKET" ] || { echo "[stub judge] --packet missing" >&2; exit 4; }
[ -n "$OUT" ] || { echo "[stub judge] --out missing" >&2; exit 4; }
[ "$(jq -r '.target' "$PACKET")" = "research" ] || { echo "[stub judge] packet target is not research" >&2; exit 4; }
PRODUCER="$(jq -r '.producer_model // "unknown"' "$PACKET")"
[ "$PRODUCER" != "unknown" ] || { echo "[stub judge] PRODUCER_UNKNOWN: the driver passed no producer identity" >&2; exit 4; }
case "${FIXTURE_DISPATCH:-pass}" in
  gateway) echo "[stub judge] WARN: no OpenAI-compatible endpoint reachable" >&2; exit 3 ;;
  garbage) echo "[stub judge] ERROR: unusable judge output" >&2; exit 2 ;;
  pass) jq -n --arg p "$PRODUCER" '{mode:"artifact",verdict:"PASS",judge_model:"stub/judge",isolation_mode:"rest-gateway:stub",producer_model:$p,cross_model_check:"verified-distinct",findings:[],checked_classes:["stub"]}' > "$OUT"; echo "[stub judge] wrote $OUT" >&2 ;;
esac
EOF
  real_dispatch() { # real_dispatch <FIXTURE_DISPATCH> <extra env...>
    local mode="$1"; shift
    fresh
    set +e
    env RESEARCH_STAGE_RUNNER="$RUNNER" RESEARCH_ADV_DIR="$ADV_STUB" RESEARCH_OUTPUT_DIR="$ROOT" RESEARCH_HOOK_LOG="$LOG" FIXTURE_CALLS="$CALLS" KBD_PRODUCER_MODEL="fixture/producer-under-test" FIXTURE_SUBQ=4 FIXTURE_REPORT_GATE=pass FIXTURE_DISPATCH="$mode" "$@" bash "$DRIVER" --query "real dispatch $mode fixture query" --depth deep >"$ROOT/stdout.txt" 2>"$ROOT/stderr.txt"
    RC=$?
    set -e
  }
  real_dispatch pass
  assert "pass: driver exits 0 through the real dispatch branch" test "$RC" -eq 0
  assert "pass: stub judge accepted the CLI shape and the producer, verdict recorded" jq -e '.review.verdict=="PASS" and .producer_model=="fixture/producer-under-test"' "$(pkg_dir)/checkpoint.json"
  assert "pass: packet carried the producer identity" jq -e '.producer_model=="fixture/producer-under-test"' "$(pkg_dir)/review/packet.json"
  assert "pass: report ends verified" test "$(report_status)" = "verified"
  real_dispatch gateway
  assert "gateway: exit 3 is recorded as judge unavailable" jq -e '.blocked_review | test("^blocked: judge unavailable \\(dispatch exited 3")' "$(pkg_dir)/checkpoint.json"
  assert "gateway: report partial" test "$(report_status)" = "partial"
  real_dispatch garbage
  assert "garbage: exit 2 is recorded as judge refused, distinct from unavailable" jq -e '.blocked_review | test("^blocked: judge refused \\(dispatch exited 2")' "$(pkg_dir)/checkpoint.json"
  assert "garbage: sidecar names the refusal" grep -q 'Adversarial review:\*\* blocked: judge refused' "$(sidecar)"
  echo "[review-real-dispatch] negative control: no producer identity is a refusal the driver cannot hide"
  fresh
  set +e; env RESEARCH_STAGE_RUNNER="$RUNNER" RESEARCH_ADV_DIR="$ADV_STUB" RESEARCH_OUTPUT_DIR="$ROOT" RESEARCH_HOOK_LOG="$LOG" FIXTURE_CALLS="$CALLS" FIXTURE_SUBQ=4 FIXTURE_REPORT_GATE=pass bash "$DRIVER" --query "no producer fixture query" --depth deep >"$ROOT/stdout.txt" 2>"$ROOT/stderr.txt"; RC=$?; set -e
  assert "no producer: recorded as judge failed (exit 4), report partial" bash -c "jq -e '.blocked_review | test(\"^blocked: judge failed \\\\(dispatch exited 4\")' '$(pkg_dir)/checkpoint.json' >/dev/null && test \"\$(grep -o '^verification_status: [a-z]*' '$(pkg_dir)/report.md')\" = 'verification_status: partial'"
fi

if scenario frontmatter-body; then
  echo "[frontmatter-body] stage 09 contract reads only the frontmatter block, not keys quoted in the body"
  # Stop before 09, plant a report whose frontmatter lacks the keys but whose body
  # quotes them, then resume with a runner that refuses to run 09. If the contract
  # accepted the planted report the driver would skip the runner and carry on; it
  # must instead reject it and call the runner, which exits 9.
  fresh; run_driver FIXTURE_SUBQ=4 FIXTURE_EXIT_STAGE=09 -- --query "frontmatter body fixture query" --depth deep
  P="$(pkg_dir)"
  printf -- '---\ntitle: "no type here"\n---\n\n# Body\n\ntype: research-report\nverification_status: verified\n' > "$P/report.md"
  : > "$CALLS"; run_driver FIXTURE_EXIT_STAGE=09 -- --resume "$P"
  assert "planted report rejected: runner invoked for 09 and the run blocked there" bash -c "test $RC -ne 0 && grep -qx 09 '$CALLS' && jq -e '.blocked.stage==\"09\"' '$P/checkpoint.json' >/dev/null"
  assert "stage 09 not recorded complete" jq -e '(.stages_completed | index("09")) == null' "$P/checkpoint.json"
  echo "[frontmatter-body] control: a proper report validates without the runner"
  FIXTURE_SUBQ=4 bash "$HERE/fixtures/stage-runner.sh" 09 "$P"; : > "$CALLS"; run_driver FIXTURE_EXIT_STAGE=09 -- --resume "$P"
  assert "proper report accepted: runner not invoked for 09, run completes" bash -c "test $RC -eq 0 && ! grep -qx 09 '$CALLS'"
fi

echo
echo "driver-contract: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
