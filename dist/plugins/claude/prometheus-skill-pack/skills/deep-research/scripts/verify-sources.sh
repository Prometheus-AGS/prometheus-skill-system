#!/usr/bin/env bash
set -euo pipefail

# Score source URLs for credibility using domain heuristics.
# Usage: echo -e "url1\nurl2" | bash verify-sources.sh
# Or:    bash verify-sources.sh < url-list.txt
# Output: JSON array of {url, credibility_score, flags[]}

HIGH_AUTHORITY_DOMAINS=(
  ".edu" ".gov" "arxiv.org" "nature.com" "science.org"
  "acm.org" "ieee.org" "nih.gov" "ncbi.nlm.nih.gov"
  "scholar.google.com" "pubmed.ncbi.nlm.nih.gov"
)

LOW_QUALITY_DOMAINS=(
  "reddit.com" "twitter.com" "x.com" "facebook.com"
  "ehow.com" "answers.com" "quora.com" "medium.com"
  "buzzfeed.com" "huffpost.com"
)

score_url() {
  local url="$1"
  local score=50
  local flags=()

  # Domain authority check
  for domain in "${HIGH_AUTHORITY_DOMAINS[@]}"; do
    if [[ "$url" == *"$domain"* ]]; then
      score=$((score + 25))
      flags+=("high_authority_domain")
      break
    fi
  done

  for domain in "${LOW_QUALITY_DOMAINS[@]}"; do
    if [[ "$url" == *"$domain"* ]]; then
      score=$((score - 25))
      flags+=("low_quality_domain")
      break
    fi
  done

  # HTTPS bonus
  if [[ "$url" == https://* ]]; then
    score=$((score + 5))
  else
    flags+=("no_https")
  fi

  # Cap at 0-100
  [[ $score -lt 0 ]] && score=0
  [[ $score -gt 100 ]] && score=100

  # Build flags JSON array
  local flags_json="["
  local first=true
  for f in "${flags[@]:-}"; do
    [[ -z "$f" ]] && continue
    $first || flags_json+=","
    flags_json+="\"$f\""
    first=false
  done
  flags_json+="]"

  printf '{"url":"%s","credibility_score":%d,"flags":%s}' \
    "$url" "$score" "$flags_json"
}

# Read URLs from stdin
RESULTS=()
while IFS= read -r url; do
  [[ -z "$url" ]] && continue
  RESULTS+=("$(score_url "$url")")
done

# Output JSON array
echo "["
first=true
for result in "${RESULTS[@]:-}"; do
  [[ -z "$result" ]] && continue
  $first || echo ","
  echo "  $result"
  first=false
done
echo "]"
