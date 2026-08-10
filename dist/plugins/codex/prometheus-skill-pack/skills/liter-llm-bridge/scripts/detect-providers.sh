#!/usr/bin/env bash
# detect-providers.sh — scan environment for known LLM provider keys and
# emit a JSON report mapping providers → presence, plus class coverage.
#
# Output schema:
# {
#   "providers": {
#     "<id>": { "key_var": "<ENV_VAR>", "present": bool, "classes": [<class>...] },
#     ...
#   },
#   "coverage": {
#     "small":    [<provider id>...],
#     "medium":   [<provider id>...],
#     "frontier": [<provider id>...]
#   }
# }
#
# See references/provider-env-vars.md for the canonical class assignments.

set -euo pipefail

# Provider table: id|env_var|classes (comma-separated)
PROVIDERS=(
  "anthropic|ANTHROPIC_API_KEY|frontier,medium"
  "openai|OPENAI_API_KEY|frontier,medium,small"
  "google|GOOGLE_API_KEY|frontier,medium"
  "gemini|GEMINI_API_KEY|frontier,medium"
  "groq|GROQ_API_KEY|small,medium"
  "together|TOGETHER_API_KEY|small,medium"
  "mistral|MISTRAL_API_KEY|small,medium,frontier"
  "cohere|COHERE_API_KEY|small,medium"
  "fireworks|FIREWORKS_API_KEY|small,medium"
  "openrouter|OPENROUTER_API_KEY|small,medium,frontier"
  "ollama|OLLAMA_HOST|small"
  "vllm|VLLM_BASE_URL|small,medium"
  "lmstudio|LMSTUDIO_BASE_URL|small"
  "llamacpp|LLAMA_CPP_SERVER|small"
)

# Build providers JSON
providers_json=""
declare -a small_cov=()
declare -a medium_cov=()
declare -a frontier_cov=()

for entry in "${PROVIDERS[@]}"; do
  IFS='|' read -r id var classes <<< "$entry"
  present="false"
  if [[ -n "${!var:-}" ]]; then
    present="true"
    IFS=',' read -ra cls_arr <<< "$classes"
    for c in "${cls_arr[@]}"; do
      case "$c" in
        small)    small_cov+=("$id") ;;
        medium)   medium_cov+=("$id") ;;
        frontier) frontier_cov+=("$id") ;;
      esac
    done
  fi

  classes_json=$(printf '"%s",' $(echo "$classes" | tr ',' ' ') | sed 's/,$//')
  providers_json+="\"$id\":{\"key_var\":\"$var\",\"present\":$present,\"classes\":[$classes_json]},"
done
providers_json="${providers_json%,}"

# Build coverage arrays (handle empty arrays under set -u)
to_json_array() {
  if [[ $# -eq 0 ]]; then echo "[]"; return; fi
  local out="["
  for i in "$@"; do out+="\"$i\","; done
  echo "${out%,}]"
}

small_cov_json=$(to_json_array ${small_cov[@]+"${small_cov[@]}"})
medium_cov_json=$(to_json_array ${medium_cov[@]+"${medium_cov[@]}"})
frontier_cov_json=$(to_json_array ${frontier_cov[@]+"${frontier_cov[@]}"})

cat <<EOF
{
  "providers": { $providers_json },
  "coverage": {
    "small": $small_cov_json,
    "medium": $medium_cov_json,
    "frontier": $frontier_cov_json
  }
}
EOF
