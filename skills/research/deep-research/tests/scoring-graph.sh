#!/usr/bin/env bash
# Integration test for score-sources.py / verify-sources.sh, build-graph.sh, and
# detect-contradictions.sh (change-rah-010), against the real scripts and the
# fixtures in tests/fixtures/. Five scenarios from the change spec plus the
# cross-script claim-id agreement and a drift check of the assembled package.
#
#   bash tests/scoring-graph.sh                 # all
#   bash tests/scoring-graph.sh --scenario NAME # sparse | sensitivity | duplicate |
#                                               #   contradicts | semantic-blocked |
#                                               #   semantic-inferred | package
# Runs under bash 3.2 (constraint C-05). No gateway is needed: the semantic
# path is exercised with LITER_LLM_BASE_URL pointed at a closed port (blocked)
# and with a fixture judge through RESEARCH_SEMANTIC_JUDGE_CMD (inferred).
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
SKILL_DIR="$(cd "$HERE/.." && pwd)"
S="$SKILL_DIR/scripts"; FX="$HERE/fixtures"
ONLY=""; [ "${1:-}" = "--scenario" ] && ONLY="${2:-}"
PASS=0; FAIL=0
ok()   { echo "  PASS $*"; PASS=$((PASS+1)); }
fail() { echo "  FAIL $*" >&2; FAIL=$((FAIL+1)); }
assert() { local d="$1"; shift; if "$@" >/dev/null 2>&1; then ok "$d"; else fail "$d"; fi; }
scenario() { [ -z "$ONLY" ] || [ "$ONLY" = "$1" ]; }
W="$(mktemp -d)"; trap 'rm -rf "$W"' EXIT
NOW="2026-09-06"

if scenario sparse; then
  echo "[sparse] a registry with missing metadata scores without zeros; applied weights sum to 1 over present dimensions"
  python3 "$S/score-sources.py" --registry "$FX/registry-sparse.json" --now "$NOW" --out "$W/cred.json" --sensitivity "$W/sens.json"
  assert "credibility.json written with the spec shape" jq -e '(.verified_sources|type)=="array" and (.credibility_scores|type)=="object"' "$W/cred.json"
  assert "the blog (no author, no date, no references) has a score above zero" jq -e '.credibility_scores["https://example.org/blog/vector-search"] > 0' "$W/cred.json"
  assert "its missing dimensions are excluded, not zeroed (applied_weights lacks author_expertise and publication_recency)" jq -e '(.verified_sources + .filtered_sources) | map(select(.url=="https://example.org/blog/vector-search"))[0] | .applied_weights | (has("author_expertise")|not) and (has("publication_recency")|not) and has("domain_authority")' "$W/cred.json"
  assert "every source's applied_weights sum to 1 (within 1e-3)" jq -e '[(.verified_sources + .filtered_sources)[] | (.applied_weights | add) ] | all(. > 0.999 and . < 1.001)' "$W/cred.json"
  assert "the full-metadata arxiv source ranks first" jq -e '.verified_sources[0].url=="https://arxiv.org/abs/2401.00001" and .verified_sources[0].rank==1' "$W/cred.json"
  assert "sources are sorted by score descending" jq -e '[.verified_sources[].credibility_score] as $s | $s == ($s | sort | reverse)' "$W/cred.json"
  assert "the missing-evidence flag names the dimensions" jq -e '(.verified_sources + .filtered_sources) | map(select(.url=="https://example.org/blog/vector-search"))[0].flags | any(startswith("partial_evidence:author_expertise"))' "$W/cred.json"
  assert "string claims become {text, label: unverified, evidence: null}" jq -e '.verified_sources[0].claims[0] | .label=="unverified" and .evidence==null and (.text|length>0)' "$W/cred.json"
  assert "a year-only publication date still yields a recency signal (not silently unavailable)" jq -e '(.verified_sources + .filtered_sources) | map(select(.url|test("standards.gov")))[0] | .signals.publication_recency.available==true and (.applied_weights|has("publication_recency")) and (.signals.publication_recency.explanation|test("2024-01-01"))' "$W/cred.json"
  assert "verify-sources.sh delegates for a registry path" bash -c "bash '$S/verify-sources.sh' '$FX/registry-sparse.json' --now $NOW | jq -e '.scorer | test(\"score-sources.py\")'"
  assert "verify-sources.sh legacy stdin mode still prints the old array" bash -c "printf 'https://arxiv.org/abs/1\nhttps://reddit.com/r/x\n' | bash '$S/verify-sources.sh' | jq -e 'type==\"array\" and .[0].credibility_score > .[1].credibility_score and (.[1].flags|index(\"low_quality_domain\"))'"
fi

if scenario sensitivity; then
  echo "[sensitivity] sensitivity.json classifies each rank; the sparse blog is not stable and the arxiv source is"
  python3 "$S/score-sources.py" --registry "$FX/registry-sparse.json" --now "$NOW" --out "$W/cred2.json" --sensitivity "$W/sens2.json"
  assert "four alternate profiles shipped" jq -e '(.profiles|length)==4 and ([.profiles[].id]|sort)==["authority_heavy","balanced","methodology_heavy","recency_heavy"]' "$W/sens2.json"
  assert "every source has a stability in stable|sensitive|volatile and a rank per profile" jq -e '[.sources[] | (.stability|IN("stable","sensitive","volatile")) and (.profile_ranks|length==4)] | all' "$W/sens2.json"
  assert "the medium source (recent, low authority, no author) is not stable: recency-heavy lifts it" jq -e '.sources[] | select(.url|test("medium.com")) | .stability != "stable" and .rank_range >= 1' "$W/sens2.json"
  assert "the arxiv source is stable at rank 1" jq -e '.sources[] | select(.url|test("arxiv")) | .stability=="stable" and .base_rank==1' "$W/sens2.json"
  assert "summary counts add up and name the top source" jq -e '(.summary.stable + .summary.sensitive + .summary.volatile) == (.sources|length) and (.summary.top_source|test("arxiv")) and .summary.top_source_stable==true' "$W/sens2.json"
  assert "drivers explain a moved rank by its missing evidence" jq -e '.sources[] | select(.url|test("medium.com")) | .drivers | any(test("missing evidence"))' "$W/sens2.json"
fi

if scenario duplicate; then
  echo "[duplicate] the same sentence in two artifacts collapses to one content-addressed claim with the higher label"
  bash "$S/build-graph.sh" --registry "$FX/registry-sparse.json" --credibility "$FX/claims-duplicate.json" --package-id "fixture-pkg-20260906-0001" --out "$W/graph.json"
  assert "graph.json has the spec shape" jq -e 'has("topics") and has("claims") and has("relations")' "$W/graph.json"
  assert "exactly one claim for the shared sentence (case, trailing period, whitespace normalised)" jq -e '[.claims[] | select(.text | test("Hybrid search improves recall"; "i"))] | length == 1' "$W/graph.json"
  assert "its label is verified (the higher of verified and unverified) with the verified evidence" jq -e '.claims[] | select(.text | test("Hybrid search improves recall"; "i")) | .label=="verified" and (.evidence|test("recall@10"))' "$W/graph.json"
  echo "[duplicate] a verified label without evidence is downgraded, not laundered"
  jq '(.verified_sources[] | .claims[] | select(.text|test("400 ms")) | .evidence) = null' "$FX/claims-duplicate.json" > "$W/claims-noev.json"
  bash "$S/build-graph.sh" --registry "$FX/registry-sparse.json" --credibility "$W/claims-noev.json" --package-id "fixture-pkg-20260906-0001" --out "$W/graph-noev.json"
  assert "evidence-free verified claim becomes unverified with the reason recorded" jq -e '.claims[] | select(.text | test("400 ms")) | .label=="unverified" and (.evidence|test("downgraded from verified")) and .confidence <= 0.5' "$W/graph-noev.json"
  assert "its sources are the union of both artifacts" jq -e '.claims[] | select(.text | test("Hybrid search improves recall"; "i")) | (.sources|sort)==["src-arxiv","src-blog"]' "$W/graph.json"
  assert "claim ids are claim-<16 hex>" jq -e '[.claims[].id | test("^claim-[0-9a-f]{16}$")] | all' "$W/graph.json"
  assert "the id is content-addressed: sha256(scope:normalised text)[:16]" bash -c "expected=\"claim-\$(printf '%s' 'fixture-pkg-20260906-0001:hybrid search improves recall by 12% on the benchmark' | shasum -a 256 | cut -c1-16)\"; jq -e --arg e \"\$expected\" '.claims[] | select(.text | test(\"Hybrid search improves recall\"; \"i\")) | .id == \$e' '$W/graph.json'"
  assert "a cites relation exists from the merged claim to each of its sources" jq -e '(.claims[] | select(.text | test("Hybrid search improves recall"; "i")) | .id) as $c | [.relations[] | select(.from==$c and .type=="cites") | .to] | sort == ["src-arxiv","src-blog"]' "$W/graph.json"
  assert "graph.json validates against research-graph.schema.json (python jsonschema or structural)" python3 - "$W/graph.json" "$SKILL_DIR/references/schemas/research-graph.schema.json" <<'PY'
import json, sys
g = json.load(open(sys.argv[1])); s = json.load(open(sys.argv[2]))
try:
    import jsonschema
    jsonschema.Draft202012Validator(s).validate(g)
except ImportError:
    for c in g["claims"]:
        assert set(("id","text","label","critical","confidence","sources","contradicts")) <= set(c), c
PY
fi

if scenario contradicts; then
  echo "[contradicts] a contradicting numeric pair yields one contradicts relation between content-addressed ids"
  bash "$S/detect-contradictions.sh" --credibility "$FX/claims-duplicate.json" --package-id "fixture-pkg-20260906-0001" --out "$W/contra.json"
  assert "contradictions.json has the spec shape with one numeric entry" jq -e '(.contradictions|length)==1 and .contradictions[0].strategy_tried=="numeric" and .contradictions[0].label=="inferred" and .contradictions[0].resolved==false' "$W/contra.json"
  assert "the pair is the 40 ms vs 400 ms build-time claims from different sources" jq -e '.contradictions[0] | (.claim_a.text|test("40 ms")) and (.claim_b.text|test("400 ms")) and .claim_a.source != .claim_b.source and .topic=="latency_ms"' "$W/contra.json"
  bash "$S/build-graph.sh" --registry "$FX/registry-sparse.json" --credibility "$FX/claims-duplicate.json" --contradictions "$W/contra.json" --package-id "fixture-pkg-20260906-0001" --out "$W/graph2.json"
  assert "graph.json contains exactly one contradicts relation" jq -e '[.relations[] | select(.type=="contradicts")] | length == 1' "$W/graph2.json"
  assert "the relation joins the ids the detector assigned (same hash function in both scripts)" bash -c "a=\$(jq -r '.contradictions[0].claim_a.id' '$W/contra.json'); b=\$(jq -r '.contradictions[0].claim_b.id' '$W/contra.json'); jq -e --arg a \"\$a\" --arg b \"\$b\" '.relations[] | select(.type==\"contradicts\") | .from==\$a and .to==\$b' '$W/graph2.json'"
  assert "both claims list each other in contradicts[]" bash -c "a=\$(jq -r '.contradictions[0].claim_a.id' '$W/contra.json'); b=\$(jq -r '.contradictions[0].claim_b.id' '$W/contra.json'); jq -e --arg a \"\$a\" --arg b \"\$b\" '(.claims[] | select(.id==\$a) | .contradicts | index(\$b)) and (.claims[] | select(.id==\$b) | .contradicts | index(\$a))' '$W/graph2.json'"
  assert "the contradiction topic became a topic grouping both claims" jq -e '.topics[] | select(.name=="latency_ms") | (.claims|length)==2' "$W/graph2.json"
  echo "[contradicts] a contradictions file with legacy or foreign claim ids is readdressed by content, not crashed on"
  jq '.contradictions[0].claim_a.id = "claim-001" | .contradictions[0].claim_b.id = "claim-deadbeefdeadbeef"' "$W/contra.json" > "$W/contra-legacy.json"
  bash "$S/build-graph.sh" --registry "$FX/registry-sparse.json" --credibility "$FX/claims-duplicate.json" --contradictions "$W/contra-legacy.json" --package-id "fixture-pkg-20260906-0001" --out "$W/graph3.json"
  assert "legacy ids: one contradicts relation between the content-addressed ids, no synthetic duplicate claim" bash -c "a=\$(jq -r '.contradictions[0].claim_a.id' '$W/contra.json'); b=\$(jq -r '.contradictions[0].claim_b.id' '$W/contra.json'); jq -e --arg a \"\$a\" --arg b \"\$b\" '([.relations[] | select(.type==\"contradicts\")] | length == 1) and (.relations[] | select(.type==\"contradicts\") | .from==\$a and .to==\$b) and ([.claims[] | select(.id==\"claim-001\" or .id==\"claim-deadbeefdeadbeef\")] | length == 0)' '$W/graph3.json'"
fi

if scenario semantic-blocked; then
  echo "[semantic-blocked] the semantic path with no reachable gateway records every candidate pair as blocked, never silently absent"
  LITER_LLM_BASE_URL="http://127.0.0.1:1/v1" KBD_COMPLETE_TIMEOUT=3 bash "$S/detect-contradictions.sh" --credibility "$FX/claims-duplicate.json" --package-id "fixture-pkg-20260906-0001" --semantic --out "$W/contra-blocked.json"
  assert "semantic detection status is blocked with a reason" jq -e '.detection.semantic.attempted==true and .detection.semantic.status=="blocked" and (.detection.semantic.reason|test("gateway"))' "$W/contra-blocked.json"
  assert "at least one candidate pair was checked and recorded blocked and unresolved" jq -e '[.contradictions[] | select(.strategy_tried=="semantic")] | length >= 1 and all(.label=="blocked" and .resolved==false and (.audit_trail|test("not run")))' "$W/contra-blocked.json"
  assert "the numeric entry is still present and inferred" jq -e '[.contradictions[] | select(.strategy_tried=="numeric")] | length==1 and .[0].label=="inferred"' "$W/contra-blocked.json"
fi

if scenario semantic-inferred; then
  echo "[semantic-inferred] with a judge reachable, a pair the judge calls contradictory is recorded inferred"
  cat > "$W/judge.sh" <<'EOF'
#!/usr/bin/env bash
# Fixture judge: contradicts when both statements mention build time.
if grep -qi "build" "$1" && [ "$(grep -ci 'per 1,000 vectors' "$1")" -ge 2 ]; then
  echo '{"contradicts": true, "topic": "index build time", "confidence": 0.9, "reason": "40 ms and 400 ms per 1,000 vectors cannot both hold"}'
else
  echo '{"contradicts": false, "topic": "n/a", "confidence": 0.8, "reason": "different subjects"}'
fi
EOF
  # Remove the numeric pair so the semantic judge is what finds the build-time contradiction.
  jq '(.verified_sources[] | .claims[] | select(.text|test("40 ms")) | .text) |= "The index build takes forty milliseconds per 1,000 vectors"' "$FX/claims-duplicate.json" > "$W/claims-words.json"
  RESEARCH_SEMANTIC_JUDGE_CMD="bash $W/judge.sh" bash "$S/detect-contradictions.sh" --credibility "$W/claims-words.json" --package-id "fixture-pkg-20260906-0001" --semantic --out "$W/contra-inferred.json"
  assert "semantic status inferred, pairs were checked" jq -e '.detection.semantic.status=="inferred" and .detection.semantic.pairs_checked >= 1' "$W/contra-inferred.json"
  assert "the judged pair is recorded inferred with the judge's reason and confidence" jq -e '[.contradictions[] | select(.strategy_tried=="semantic")] | length==1 and .[0].label=="inferred" and .[0].confidence==0.9 and (.[0].audit_trail|test("critic model")) and .[0].topic=="index build time"' "$W/contra-inferred.json"
  assert "no numeric entry (the numbers were spelled out)" jq -e '[.contradictions[] | select(.strategy_tried=="numeric")] | length==0' "$W/contra-inferred.json"
  echo "[semantic-inferred] a judge that answers without a verdict leaves the pair blocked, never silently dropped"
  printf '#!/usr/bin/env bash\necho "I am not sure, it depends."\n' > "$W/judge-garbage.sh"
  RESEARCH_SEMANTIC_JUDGE_CMD="bash $W/judge-garbage.sh" bash "$S/detect-contradictions.sh" --credibility "$W/claims-words.json" --package-id "fixture-pkg-20260906-0001" --semantic --out "$W/contra-garbage.json"
  assert "unparseable replies: every checked pair is blocked with the parse failure named and status blocked" jq -e '.detection.semantic.status=="blocked" and ([.contradictions[] | select(.strategy_tried=="semantic")] | length >= 1 and all(.label=="blocked" and (.audit_trail|test("could not be parsed"))))' "$W/contra-garbage.json"
fi

if scenario package; then
  echo "[package] the labelled fixture package, with sensitivity.json, passes the drift check"
  assert "package-labelled carries sensitivity.json and its manifest lists it" bash -c "test -f '$FX/package-labelled/sensitivity.json' && jq -e '.files.sensitivity==\"sensitivity.json\"' '$FX/package-labelled/manifest.json'"
  assert "check-research-package.sh --package passes" bash "$S/check-research-package.sh" --package "$FX/package-labelled"
  echo "[package] a package assembled from the new scripts validates too"
  P="$W/assembled-20260906-00aa"; mkdir -p "$P/sources"; cp "$FX/registry-sparse.json" "$P/sources/registry.json"
  printf '{"package_id":"assembled-20260906-00aa","slug":"assembled","job_id":"job-0-assembled","query":"assembled package fixture query","depth":"deep","scale":"full","citation_style":"APA","kb_ids":[],"model_routing":{},"stages_planned":["01","02","03","04","05","06","07","08","09","10"],"stages_completed":["01","02","03","04","05","06","07","08","09","10"],"current_stage":null,"status":"complete","blocked":null,"created_at":"2026-09-06T00:00:00Z","last_updated_at":"2026-09-06T00:10:00Z","completed_at":"2026-09-06T00:10:00Z","integrations":{"surreal_memory_used":false,"sycophancy_correction_used":false,"feynman_gate_used":false,"adversarial_review_used":false},"hook_log":[],"notes":[]}\n' > "$P/checkpoint.json"
  bash "$S/verify-sources.sh" "$P" --now "$NOW"
  bash "$S/detect-contradictions.sh" "$P" --out "$P/contradictions.json"
  bash "$S/build-graph.sh" "$P" --out "$P/graph.json"
  printf '{"style":"APA","citations":[]}\n' > "$P/citations.json"
  printf -- '# Research Plan: assembled\n\n## Sub-questions\n\n- Q1\n- Q2\n- Q3\n\n## Task ledger\n\n## Verification log\n\n## Decision log\n' > "$P/plan.md"
  printf -- '---\ntype: research-report\ntitle: "Assembled"\nquery: "assembled package fixture query"\ndate: "2026-09-06"\nconfidence: 0.6\nverification_status: partial\nfeynman_grade: null\nmisconceptions_absent: null\nsources_count: 5\ncontradictions_resolved: 0\nokf_version: "0.1"\n---\n\n# Assembled\n' > "$P/report.md"
  bash "$S/write-provenance.sh" "$P" >/dev/null
  RESEARCH_POST_EXPORT_BY_CALLER=1 bash "$S/export-package.sh" "$P" >/dev/null 2>"$W/export.log" || { cat "$W/export.log" >&2; }
  assert "manifest lists sensitivity.json" jq -e '.files.sensitivity=="sensitivity.json"' "$P/manifest.json"
  assert "assembled package passes check-research-package.sh --package" bash "$S/check-research-package.sh" --package "$P"
  assert "derivation is partial: stage 05 labels are unverified until the verifier reads the sources" test "$(bash "$S/check-research-package.sh" --derive "$P")" = "partial"
fi

echo
echo "scoring-graph: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
