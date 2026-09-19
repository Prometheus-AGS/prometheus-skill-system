#!/usr/bin/env bash
# learn-coherence.sh — integration gate for change-rah-008.
#
# Exercises the real production scripts across their file boundaries:
#   1. shared/scripts/content-grounding-kb.sh against the fixture KB: every
#      source carries key_points[] and misconceptions[]; authored lists are
#      kept; misconception entries carry their text; quotes and backslashes
#      survive; the corpus is valid JSON.
#   2. Both skill wrappers (learn-goal, learn-kb) produce output byte-identical
#      to the shared script for the same arguments.
#   3. feynman-loop/scripts/write-artifact.sh writes an artifact, and the path
#      learn-retain documents (artifacts/<concept-id>/*.json) and the directory
#      learn-certify documents (artifacts/<concept-id>/) both resolve it; a
#      second artifact for the same concept lands beside it; an artifact
#      without concept_id is refused.
#   4. The three SKILL.md files and write-artifact.sh name the same path.
#
# bash 3.2 compatible (no mapfile, no declare -A). Exit 0 only when every
# assertion holds.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
ROOT="$(cd "$HERE/../../../.." && pwd -P)"
FIXTURE_KB="$HERE/fixtures/kb-local"
SHARED="$ROOT/shared/scripts/content-grounding-kb.sh"
WRAP_GOAL="$ROOT/skills/learn/learn-goal/scripts/content-grounding-kb.sh"
WRAP_KB="$ROOT/skills/learn/learn-kb/scripts/content-grounding-kb.sh"
WRITE_ARTIFACT="$ROOT/skills/learn/feynman-loop/scripts/write-artifact.sh"

PASS=0
FAIL=0
ok()   { PASS=$((PASS + 1)); echo "  ok   - $1"; }
fail() { FAIL=$((FAIL + 1)); echo "  FAIL - $1" >&2; }
check() { # check <description> <command...>
  local desc="$1"; shift
  if "$@" >/dev/null 2>&1; then ok "$desc"; else fail "$desc"; fi
}

TMP="$(mktemp -d "${TMPDIR:-/tmp}/learn-coherence-XXXXXX")"
trap 'rm -rf "$TMP"' EXIT

command -v jq >/dev/null 2>&1 || { echo "learn-coherence: jq is required" >&2; exit 2; }

echo "== 1. corpus shape from the shared grounding script"
export CONTENT_GROUNDING_BUILD_AT="2026-09-07T00:00:00Z"
ARGS=(--kb "local:$FIXTURE_KB" --subject "linear algebra" --level practitioner --budget-sources 10 --include-misconceptions)
STATUS="$(bash "$SHARED" "${ARGS[@]}" --output "$TMP/shared.json" 2>"$TMP/shared.err")" || { fail "shared script exits 0"; cat "$TMP/shared.err" >&2; }
check "shared script reports ok|partial with 5 sources" \
  jq -e '(.status == "ok" or .status == "partial") and .source_count == 5' <<<"$STATUS"
check "corpus is valid JSON with schema_version 1.1.0" jq -e '.schema_version == "1.1.0"' "$TMP/shared.json"
check "every source has key_points[] and misconceptions[] arrays" \
  jq -e '[.sources[] | (.key_points|type) == "array" and (.misconceptions|type) == "array"] | all' "$TMP/shared.json"
check "every source has a non-empty key_points[]" \
  jq -e '[.sources[] | (.key_points|length) > 0] | all' "$TMP/shared.json"
check "key points are sentences of content_summary (md file: 3 sentences)" \
  jq -e '.sources[] | select(.source_ref | endswith("vectors.md")) | .key_points | length == 3 and (.[0] | startswith("Vectors add componentwise"))' "$TMP/shared.json"
check "sentence split honours ! and ? (txt file: 4 sentences)" \
  jq -e '.sources[] | select(.source_ref | endswith("matrices.txt")) | .key_points | length == 4' "$TMP/shared.json"
check "authored key_points[] are kept verbatim" \
  jq -e '.sources[] | select(.source_ref == "kb:eigen") | .key_points == ["eigenvectors keep direction","eigenvalue is the scale factor"]' "$TMP/shared.json"
check "misconception entry carries its text in misconceptions[]" \
  jq -e '.sources[] | select(.is_misconception) | .misconceptions == ["A vector with a larger norm is always the more important one."] and .source_type == "known_misconception" and .confidence == 0.6' "$TMP/shared.json"
check "non-misconception sources have misconceptions == []" \
  jq -e '[.sources[] | select(.is_misconception | not) | .misconceptions == []] | all' "$TMP/shared.json"
check "quotes and backslashes in summaries and refs survive" \
  jq -e '.sources[] | select(.source_ref == "kb:quoted \"summary\"") | .content_summary | contains("\"quotes\"") and contains("back\\slashes")' "$TMP/shared.json"
check "misconception flag is a boolean on every source" \
  jq -e '[.sources[] | (.is_misconception|type) == "boolean"] | all' "$TMP/shared.json"
check "learn-grade Step 3 input is non-empty: some source has a misconception" \
  jq -e '[.sources[].misconceptions[]] | length >= 1' "$TMP/shared.json"

echo "== 1b. --normalize brings an older corpus to the same shape"
EVAL_CORPUS="$ROOT/skills/learn/learn-grade/references/eval-dataset/corpora/cellular-respiration-corpus.json"
NSTATUS="$(bash "$SHARED" --normalize "$EVAL_CORPUS" --output "$TMP/normalized.json" 2>"$TMP/normalize.err")" || { fail "--normalize exits 0"; cat "$TMP/normalize.err" >&2; }
check "--normalize reports ok with every source (12, no budget truncation)" \
  jq -e '.status == "ok" and .source_count == 12' <<<"$NSTATUS"
check "normalized corpus has key_points[] and misconceptions[] on all 12 sources" \
  jq -e '(.sources|length) == 12 and ([.sources[] | (.key_points|type) == "array" and (.misconceptions|type) == "array" and (.key_points|length) > 0] | all)' "$TMP/normalized.json"
check "the 5 misconception sources carry their text; the other 7 carry []" \
  jq -e '([.sources[] | select(.is_misconception) | (.misconceptions|length) == 1] | length == 5 and all) and ([.sources[] | select(.is_misconception|not) | .misconceptions == []] | length == 7 and all)' "$TMP/normalized.json"
AUTHORED_N="$(jq '[.sources[] | select(has("key_points"))] | length' "$EVAL_CORPUS")"
check "the eval corpus has authored key_points to protect (count > 0)" test "$AUTHORED_N" -gt 0
check "authored key_points[] in the eval corpus are kept verbatim (joined on source_ref, $AUTHORED_N sources)" \
  bash -c 'diff <(jq -cS "[.sources[] | select(has(\"key_points\")) | {source_ref, key_points}] | sort_by(.source_ref)" "$1") <(jq -cS --slurpfile o "$1" "[.sources[] | select(.source_ref as \$r | \$o[0].sources[] | select(has(\"key_points\")) | .source_ref == \$r) | {source_ref, key_points}] | sort_by(.source_ref)" "$2")' _ "$EVAL_CORPUS" "$TMP/normalized.json"
check "corpus identity fields survive (subject, concept_id, maintainer), corpus_id is set, schema is 1.1.0" \
  bash -c 'jq -e --arg sub "$(jq -r .subject "$1")" --arg con "$(jq -r .concept_id "$1")" --arg m "$(jq -r .maintainer "$1")" "(.corpus_id|type) == \"string\" and (.corpus_id|length) > 0 and .subject == \$sub and .concept_id == \$con and .maintainer == \$m and .schema_version == \"1.1.0\"" "$2"' _ "$EVAL_CORPUS" "$TMP/normalized.json"
check "an input corpus_id is kept verbatim" \
  bash -c 'bash "$1" --normalize "$2" --output "$3" >/dev/null 2>&1 && jq -e ".corpus_id == \"fixture-authored\"" "$3"' _ "$SHARED" "$FIXTURE_KB/authored-corpus.json" "$TMP/normalized-fixture.json"
# Per-source fields the builder does not own must survive normalization
# (round-2 finding 3): a KB's own concept_id or tags are data, not noise.
cat > "$TMP/extras-corpus.json" <<'JSON'
{"corpus_id":"extras","subject":"extras","concept_id":"top-level","schema_version":"1.0.0","sources":[
 {"source_ref":"a","content_summary":"One. Two.","is_misconception":false,"concept_id":"per-source-c1","tags":["x","y"]},
 {"source_ref":"b","content_summary":"Bad idea.","is_misconception":true,"key_points":["authored"],"custom":{"n":1}}]}
JSON
bash "$SHARED" --normalize "$TMP/extras-corpus.json" --output "$TMP/extras.json" >/dev/null 2>&1 || fail "--normalize on the extras corpus exits 0"
check "per-source fields the builder does not own survive normalization" \
  jq -e '.sources[0].concept_id == "per-source-c1" and .sources[0].tags == ["x","y"] and .sources[1].custom.n == 1' "$TMP/extras.json"
check "rebuilt fields still win over preserved extras" \
  jq -e '.sources[1].key_points == ["authored"] and .sources[1].misconceptions == ["Bad idea."] and .sources[0].key_points == ["One.","Two."]' "$TMP/extras.json"
check "sources stay in input order after the extras fold" \
  jq -e '[.sources[].source_ref] == ["a","b"]' "$TMP/extras.json"
check "--normalize refuses a file without a sources array" \
  bash -c '! bash "$1" --normalize "$2" --output "$3" >/dev/null 2>&1' _ "$SHARED" "$FIXTURE_KB/vectors.md" "$TMP/bad.json"
check "the eval corpus file itself was not rewritten" jq -e '.schema_version == "1.0.0"' "$EVAL_CORPUS"

echo "== 2. wrapper byte-parity"
bash "$WRAP_GOAL" "${ARGS[@]}" --output "$TMP/goal.json" >/dev/null 2>&1 || fail "learn-goal wrapper exits 0"
bash "$WRAP_KB"   "${ARGS[@]}" --output "$TMP/kb.json"   >/dev/null 2>&1 || fail "learn-kb wrapper exits 0"
check "learn-goal wrapper output is byte-identical to the shared script" cmp "$TMP/shared.json" "$TMP/goal.json"
check "learn-kb wrapper output is byte-identical to the shared script"   cmp "$TMP/shared.json" "$TMP/kb.json"
check "wrappers carry no adapter logic (no curl, no append_source)" \
  bash -c '! grep -q "curl\|append_source" "$1" "$2"' _ "$WRAP_GOAL" "$WRAP_KB"
check "wrapper resolves under /bin/bash 3.2" /bin/bash "$WRAP_KB" "${ARGS[@]}" --output "$TMP/kb32.json"
check "bash 3.2 run is byte-identical too" cmp "$TMP/shared.json" "$TMP/kb32.json"
check "the shared script itself runs under /bin/bash 3.2" /bin/bash "$SHARED" "${ARGS[@]}" --output "$TMP/shared32.json"
check "shared script under /bin/bash 3.2 is byte-identical" cmp "$TMP/shared.json" "$TMP/shared32.json"
check "the wrapper hands the shared script the same interpreter it was started with" \
  grep -q 'exec "\${BASH:-bash}"' "$WRAP_KB"
# Installed generations ship the script without shared/scripts/lib/, so the inline
# slug fallback must produce the same corpus_id as lib/slug.sh.
mkdir -p "$TMP/nolib"
cp "$SHARED" "$TMP/nolib/content-grounding-kb.sh"
bash "$TMP/nolib/content-grounding-kb.sh" "${ARGS[@]}" --output "$TMP/nolib.json" >/dev/null 2>&1 || fail "shared script runs without lib/slug.sh beside it"
check "inline slug fallback (no lib/) is byte-identical to the lib/slug.sh path" cmp "$TMP/shared.json" "$TMP/nolib.json"
check "that run really had no lib beside it" test ! -e "$TMP/nolib/lib/slug.sh"
unset CONTENT_GROUNDING_BUILD_AT

echo "== 3. artifact round trip across the three skills' documented paths"
export PROMETHEUS_LEARN_HOME="$TMP/learn"
GOAL="goal-coherence"
artifact_json() { # artifact_json <artifact_id> <concept_id> <closed_at>
  jq -cn --arg a "$1" --arg c "$2" --arg t "$3" --arg g "$GOAL" '{
    artifact_id: $a, goal_id: $g, concept_id: $c, depth: 0, audience: "peer",
    explanation_text: "x", grade_id: "grade-1", overall_score: 0.8,
    transfer_scores: [0.85, 0.7],
    verification: [{label:"verified", evidence:"grade-1.json transfer_problems[0] against kb:eigen"},
                   {label:"inferred", evidence:"grade-1.json transfer_problems[1]"}],
    provenance: {grade_file: "/tmp/grade-1.json", corpus_path: "/tmp/corpus.json", grader: "learn-grade", graded_at: $t},
    retention_scheduled: true, child_loops: [], closed_at: $t }'
}
OUT1="$(bash "$WRITE_ARTIFACT" --goal-id "$GOAL" --artifact-json "$(artifact_json artifact-c1-peer-0-1000 c1 2026-09-01T00:00:00Z)")"
check "write-artifact.sh exits ok" jq -e '.ok == true and .concept_id == "c1"' <<<"$OUT1"
P1="$(jq -r .path <<<"$OUT1")"
check "written path is goals/<goal>/artifacts/<concept-id>/<artifact-id>.json" \
  test "$P1" = "$PROMETHEUS_LEARN_HOME/goals/$GOAL/artifacts/c1/artifact-c1-peer-0-1000.json"
check "the file exists" test -f "$P1"
# learn-retain: artifacts/<concept-id>/*.json
RETAIN_MATCHES=$(ls "$PROMETHEUS_LEARN_HOME/goals/$GOAL"/artifacts/c1/*.json 2>/dev/null | wc -l | tr -d ' ')
check "learn-retain glob artifacts/<concept-id>/*.json resolves the file" test "$RETAIN_MATCHES" = "1"
# learn-certify: artifacts/<concept-id>/ directory
check "learn-certify directory artifacts/<concept-id>/ contains the file" \
  test -f "$PROMETHEUS_LEARN_HOME/goals/$GOAL/artifacts/c1/$(basename "$P1")"
bash "$WRITE_ARTIFACT" --goal-id "$GOAL" --artifact-json "$(artifact_json artifact-c1-peer-1-2000 c1 2026-09-02T00:00:00Z)" >/dev/null
RETAIN_MATCHES=$(ls "$PROMETHEUS_LEARN_HOME/goals/$GOAL"/artifacts/c1/*.json 2>/dev/null | wc -l | tr -d ' ')
check "a second artifact for the concept lands beside the first" test "$RETAIN_MATCHES" = "2"
LATEST="$(jq -rs 'sort_by(.closed_at, .artifact_id) | last | .artifact_id' "$PROMETHEUS_LEARN_HOME/goals/$GOAL"/artifacts/c1/*.json)"
check "learn-retain 'most recent' rule (latest closed_at) picks the second" test "$LATEST" = "artifact-c1-peer-1-2000"
bash "$WRITE_ARTIFACT" --goal-id "$GOAL" --artifact-json "$(artifact_json artifact-c2-peer-0-1000 c2 2026-09-01T00:00:00Z)" >/dev/null
check "another concept gets its own directory" test -f "$PROMETHEUS_LEARN_HOME/goals/$GOAL/artifacts/c2/artifact-c2-peer-0-1000.json"
NOCONCEPT="$(artifact_json artifact-x-peer-0-1 c1 2026-09-01T00:00:00Z | jq -c 'del(.concept_id)')"
check "an artifact without concept_id is refused" \
  bash -c '! bash "$1" --goal-id "$2" --artifact-json "$3" >/dev/null 2>&1' _ "$WRITE_ARTIFACT" "$GOAL" "$NOCONCEPT"
check "a concept_id with a path separator is refused" \
  bash -c '! bash "$1" --goal-id "$2" --artifact-json "$3" >/dev/null 2>&1' _ "$WRITE_ARTIFACT" "$GOAL" "$(artifact_json artifact-y-peer-0-1 '../escape' 2026-09-01T00:00:00Z)"
check "a goal_id with a path separator is refused" \
  bash -c '! bash "$1" --goal-id "../../escape-goal" --artifact-json "$2" >/dev/null 2>&1' _ "$WRITE_ARTIFACT" "$(artifact_json artifact-z-peer-0-1 c1 2026-09-01T00:00:00Z)"
check "a dot-segment artifact_id is refused" \
  bash -c '! bash "$1" --goal-id "$2" --artifact-json "$3" >/dev/null 2>&1' _ "$WRITE_ARTIFACT" "$GOAL" "$(artifact_json .. c1 2026-09-01T00:00:00Z)"
check "nothing was written outside the goal's artifact store or the learn home" \
  bash -c '! test -e "$1/goals/escape" && ! test -e "$1/escape-goal" && ! test -e "$(dirname "$1")/escape-goal" && ! test -e "$1/goals/$2/artifacts/c1/...json"' _ "$PROMETHEUS_LEARN_HOME" "$GOAL"
unset PROMETHEUS_LEARN_HOME

echo "== 4. the documented paths agree"
for f in skills/learn/feynman-loop/SKILL.md skills/learn/learn-retain/SKILL.md skills/learn/learn-certify/SKILL.md skills/learn/feynman-loop/scripts/write-artifact.sh; do
  check "$f documents artifacts/<concept-id>/" grep -q 'artifacts/<concept-id>/' "$ROOT/$f"
done
check "positive control: the same scan flags see write-artifact.sh (a .sh file)" \
  bash -c 'grep -rln "artifacts/<concept-id>/" "$1/skills/learn" --include=SKILL.md --include="*.sh" --exclude-dir=tests | grep -q "feynman-loop/scripts/write-artifact.sh"' _ "$ROOT"
check "no skill still documents the old flat or hyphenated artifact paths" \
  bash -c '! grep -rn "artifacts/<artifact-id>.json\|artifacts/<concept-id>-\*" "$1/skills/learn" --include=SKILL.md --include="*.sh" --exclude-dir=tests' _ "$ROOT"
check "learn-grade Step 1 routes older corpora through --normalize, not a second derivation" \
  bash -c 'grep -q -- "--normalize" "$1" && ! grep -q "derive them the same way" "$1"' _ "$ROOT/skills/learn/learn-grade/SKILL.md"
check "learn-grade Step 1 documents key_points and misconceptions per source" \
  bash -c 'grep -q "\"key_points\": \[\"string\"\]" "$1" && grep -q "\"misconceptions\": \[\"string\"\]" "$1"' _ "$ROOT/skills/learn/learn-grade/SKILL.md"
check "HARNESS.md reads sources[].misconceptions[] (workaround removed)" \
  bash -c 'grep -q "sources\[\].misconceptions\[\]" "$1" && ! grep -q "content_summary where" "$1"' _ "$ROOT/skills/learn/learn-grade/references/eval-dataset/HARNESS.md"
check "HARNESS.md does not claim the eval corpora lack authored key_points" \
  bash -c '! grep -q "without those two arrays" "$1" && grep -q "only some carry an authored" "$1"' _ "$ROOT/skills/learn/learn-grade/references/eval-dataset/HARNESS.md"
check "learn-certify and learn-retain document the PROMETHEUS_LEARN_HOME override" \
  bash -c 'grep -q "PROMETHEUS_LEARN_HOME" "$1" && grep -q "PROMETHEUS_LEARN_HOME" "$2"' _ "$ROOT/skills/learn/learn-certify/SKILL.md" "$ROOT/skills/learn/learn-retain/SKILL.md"

echo
echo "learn-coherence: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
