#!/usr/bin/env bash
# content-grounding-kb.sh — Privacy-safe KB corpus builder for learn-* skills.
#
# Routes knowledge-base queries to one of three adapters WITHOUT ever forwarding
# content to external APIs. KB content stays local.
#
# Usage:
#   content-grounding-kb.sh \
#     --kb <kb-id-or-type> \
#     --subject "linear algebra" \
#     --level "practitioner" \
#     --budget-sources 5 \
#     --output /path/to/kb-corpus.json \
#     [--include-misconceptions]
#
# --kb flag forms:
#   dify:<kb-name>          Query a Dify knowledge base by name
#   palace:<palace-id>      Query a surreal-memory palace by ID
#   local:<directory-path>  Ingest local .md/.txt/.json files
#
# Exit codes:
#   0 — success (full or partial)
#   1 — fatal error (bad args, missing credentials, unwritable output)

set -euo pipefail

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

log_info()  { echo "[content-grounding-kb] INFO:  $*" >&2; }
log_warn()  { echo "[content-grounding-kb] WARN:  $*" >&2; }
log_error() { echo "[content-grounding-kb] ERROR: $*" >&2; }

subject_to_slug() {
  local input="$1"
  echo "$input" \
    | tr '[:upper:]' '[:lower:]' \
    | sed 's/[^a-z0-9]\+/-/g' \
    | sed 's/^-//; s/-$//'
}

iso_now() {
  date -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || date +"%Y-%m-%dT%H:%M:%SZ"
}

# Append a source entry (JSON object) to the NDJSON accumulator file.
# Args: sources_file source_ref source_type confidence is_misconception content_summary
append_source() {
  local sources_file="$1"
  local source_ref="$2"
  local source_type="$3"
  local confidence="$4"
  local is_misconception="$5"
  local content_summary="$6"

  local escaped_ref; escaped_ref=$(printf '%s' "$source_ref"      | sed 's/"/\\"/g')
  local escaped_sum; escaped_sum=$(printf '%s' "$content_summary" | sed 's/"/\\"/g')

  cat >> "$sources_file" <<EOF
{"source_ref":"${escaped_ref}","source_type":"${source_type}","confidence":${confidence},"is_misconception":${is_misconception},"content_summary":"${escaped_sum}"}
EOF
}

source_count() {
  local sources_file="$1"
  if [[ -f "$sources_file" ]]; then
    wc -l < "$sources_file" | tr -d ' '
  else
    echo 0
  fi
}

budget_reached() {
  [[ "$(source_count "$SOURCES_TMP")" -ge "$BUDGET_SOURCES" ]]
}

# ---------------------------------------------------------------------------
# Privacy guard — warn loudly if external API env vars are set
# ---------------------------------------------------------------------------
warn_external_api_vars() {
  local found_vars=()
  for var in FIRECRAWL_API_KEY OPENAI_API_KEY ANTHROPIC_API_KEY TAVILY_API_KEY \
             SERPER_API_KEY BRAVE_SEARCH_API_KEY GOOGLE_API_KEY; do
    if [[ -n "${!var:-}" ]]; then
      found_vars+=("$var")
    fi
  done

  if [[ ${#found_vars[@]} -gt 0 ]]; then
    log_warn "PRIVACY NOTICE: The following external API env vars are set in this shell:"
    for v in "${found_vars[@]}"; do
      log_warn "  $v"
    done
    log_warn "content-grounding-kb.sh will NOT use these vars. KB content stays local."
    log_warn "If you want public web grounding, run content-grounding.sh instead."
  fi
}

# ---------------------------------------------------------------------------
# Argument parsing
# ---------------------------------------------------------------------------

KB_ID=""
SUBJECT=""
LEVEL="practitioner"
BUDGET_SOURCES=5
OUTPUT=""
INCLUDE_MISCONCEPTIONS=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --kb)                        KB_ID="$2";           shift 2 ;;
    --subject)                   SUBJECT="$2";         shift 2 ;;
    --level)                     LEVEL="$2";           shift 2 ;;
    --budget-sources)            BUDGET_SOURCES="$2";  shift 2 ;;
    --output)                    OUTPUT="$2";          shift 2 ;;
    --include-misconceptions)    INCLUDE_MISCONCEPTIONS=true; shift ;;
    --)                          shift; break ;;
    *)  log_error "Unknown argument: $1"
        echo "{\"status\":\"error\",\"message\":\"Unknown argument: $1\"}"
        exit 1 ;;
  esac
done

# ---------------------------------------------------------------------------
# Validate required arguments
# ---------------------------------------------------------------------------

if [[ -z "$KB_ID" ]]; then
  log_error "--kb is required (e.g. --kb dify:my-kb or --kb palace:my-palace-id)"
  echo '{"status":"error","message":"--kb is required"}'
  exit 1
fi

if [[ -z "$SUBJECT" ]]; then
  log_error "--subject is required"
  echo '{"status":"error","message":"--subject is required"}'
  exit 1
fi

if [[ -z "$OUTPUT" ]]; then
  log_error "--output is required"
  echo '{"status":"error","message":"--output is required"}'
  exit 1
fi

if ! [[ "$BUDGET_SOURCES" =~ ^[0-9]+$ ]] || [[ "$BUDGET_SOURCES" -lt 1 ]]; then
  log_error "--budget-sources must be a positive integer"
  echo '{"status":"error","message":"--budget-sources must be a positive integer"}'
  exit 1
fi

# Ensure output directory exists
OUTPUT_DIR="$(dirname "$OUTPUT")"
if [[ ! -d "$OUTPUT_DIR" ]]; then
  if ! mkdir -p "$OUTPUT_DIR" 2>/dev/null; then
    log_error "Cannot create output directory: $OUTPUT_DIR"
    echo "{\"status\":\"error\",\"message\":\"Cannot create output directory: ${OUTPUT_DIR}\"}"
    exit 1
  fi
fi

# Parse --kb prefix
KB_TYPE="${KB_ID%%:*}"
KB_VALUE="${KB_ID#*:}"

if [[ "$KB_TYPE" == "$KB_VALUE" ]]; then
  # No colon found — treat as unknown
  log_error "--kb must be prefixed with dify:, palace:, or local: (got: ${KB_ID})"
  echo "{\"status\":\"error\",\"message\":\"--kb must be prefixed with dify:, palace:, or local:\"}"
  exit 1
fi

case "$KB_TYPE" in
  dify|palace|local) ;;
  *)
    log_error "Unknown --kb type '${KB_TYPE}' — must be one of: dify, palace, local"
    echo "{\"status\":\"error\",\"message\":\"Unknown --kb type: ${KB_TYPE}\"}"
    exit 1
    ;;
esac

SUBJECT_SLUG="$(subject_to_slug "$SUBJECT")"
KB_SLUG="$(subject_to_slug "$KB_ID")"
CORPUS_ID="${KB_SLUG}-${SUBJECT_SLUG}"
BUILD_AT="$(iso_now)"

# Temp accumulator
SOURCES_TMP="$(mktemp /tmp/content-grounding-kb-sources-XXXXXX.ndjson)"
trap 'rm -f "$SOURCES_TMP"' EXIT

# ---------------------------------------------------------------------------
# Privacy guard — run before any adapter logic
# ---------------------------------------------------------------------------
warn_external_api_vars

log_info "KB corpus build: kb='${KB_ID}' subject='${SUBJECT}' level='${LEVEL}' budget=${BUDGET_SOURCES}"
log_info "Corpus ID: ${CORPUS_ID}"
log_info "privacy_mode=true — no content forwarded to external APIs"

# ---------------------------------------------------------------------------
# Adapter: dify:<kb-name>
# ---------------------------------------------------------------------------
run_dify_adapter() {
  local kb_name="$1"

  if [[ -z "${DIFY_API_KEY:-}" ]]; then
    log_error "DIFY_API_KEY is required for the dify adapter"
    echo '{"status":"error","message":"DIFY_API_KEY required for dify adapter"}'
    exit 1
  fi

  local dify_base="${DIFY_BASE_URL:-http://localhost/v1}"
  log_info "Dify adapter: querying knowledge base '${kb_name}' at ${dify_base} ..."

  local response
  response="$(
    curl --silent --max-time 20 \
      -X POST "${dify_base}/knowledge-bases/search" \
      -H "Authorization: Bearer ${DIFY_API_KEY}" \
      -H "Content-Type: application/json" \
      -d "{\"query\":\"${SUBJECT}\",\"knowledge_base_name\":\"${kb_name}\",\"top_k\":${BUDGET_SOURCES}}" \
      2>/dev/null
  )" || true

  if [[ -z "$response" ]]; then
    log_warn "Dify returned empty response for kb='${kb_name}'"
    return
  fi

  if ! command -v jq >/dev/null 2>&1; then
    log_warn "jq not available — cannot parse Dify response"
    return
  fi

  while IFS= read -r item; do
    budget_reached && break
    local ref score summary
    ref="$(echo "$item"    | jq -r '.document_name // .id // "dify-doc"')"
    score="$(echo "$item"  | jq -r '.score // 0.85')"
    summary="$(echo "$item" | jq -r '.content // "" | .[0:500]')"
    [[ -z "$summary" ]] && summary="Dify KB document on ${SUBJECT} from ${kb_name}"
    append_source "$SOURCES_TMP" "dify:${kb_name}/${ref}" "dify_kb" "${score}" "false" "${summary}"
    log_info "  + dify_kb: ${ref} (score=${score})"
  done < <(echo "$response" | jq -c '.data[]? // empty' 2>/dev/null)

  if $INCLUDE_MISCONCEPTIONS && ! budget_reached; then
    for query in "common misconceptions about ${SUBJECT}" "wrong beliefs about ${SUBJECT}"; do
      budget_reached && break
      local miscon_resp
      miscon_resp="$(
        curl --silent --max-time 20 \
          -X POST "${dify_base}/knowledge-bases/search" \
          -H "Authorization: Bearer ${DIFY_API_KEY}" \
          -H "Content-Type: application/json" \
          -d "{\"query\":\"${query}\",\"knowledge_base_name\":\"${kb_name}\",\"top_k\":3}" \
          2>/dev/null
      )" || true

      if [[ -n "$miscon_resp" ]]; then
        while IFS= read -r item; do
          budget_reached && break
          local mref mscore msum
          mref="$(echo "$item"   | jq -r '.document_name // .id // "dify-misconception"')"
          mscore="$(echo "$item" | jq -r '.score // 0.75')"
          msum="$(echo "$item"   | jq -r '.content // "" | .[0:500]')"
          [[ -z "$msum" ]] && msum="Common misconception about ${SUBJECT} from Dify KB '${kb_name}'"
          append_source "$SOURCES_TMP" "dify:${kb_name}/${mref}" "known_misconception" "${mscore}" "true" "${msum}"
          log_info "  + misconception (dify): ${mref}"
        done < <(echo "$miscon_resp" | jq -c '.data[]? // empty' 2>/dev/null)
      fi
    done
  fi
}

# ---------------------------------------------------------------------------
# Adapter: palace:<palace-id>
# ---------------------------------------------------------------------------
run_palace_adapter() {
  local palace_id="$1"

  if [[ -z "${SURREAL_MEMORY_URL:-}" ]]; then
    log_error "SURREAL_MEMORY_URL is required for the palace adapter"
    echo '{"status":"error","message":"SURREAL_MEMORY_URL required for palace adapter"}'
    exit 1
  fi

  log_info "Palace adapter: querying palace '${palace_id}' at ${SURREAL_MEMORY_URL} ..."

  local response
  response="$(
    curl --silent --max-time 20 \
      -X POST "${SURREAL_MEMORY_URL}/api/v1/palace/recall" \
      -H "Content-Type: application/json" \
      -d "{\"palace_id\":\"${palace_id}\",\"query\":\"${SUBJECT}\",\"top_k\":${BUDGET_SOURCES}}" \
      2>/dev/null
  )" || true

  if [[ -z "$response" ]]; then
    log_warn "surreal-memory palace returned empty response for palace_id='${palace_id}'"
    return
  fi

  if ! command -v jq >/dev/null 2>&1; then
    log_warn "jq not available — cannot parse palace response"
    return
  fi

  while IFS= read -r item; do
    budget_reached && break
    local ref score summary
    ref="$(echo "$item"     | jq -r '.id // .source_ref // "palace-result"')"
    score="$(echo "$item"   | jq -r '.score // 0.80')"
    summary="$(echo "$item" | jq -r '.content // .summary // "" | .[0:500]')"
    [[ -z "$summary" ]] && summary="Palace RAG result on ${SUBJECT} from palace '${palace_id}'"
    append_source "$SOURCES_TMP" "palace:${palace_id}/${ref}" "palace_rag" "${score}" "false" "${summary}"
    log_info "  + palace_rag: ${ref} (score=${score})"
  done < <(echo "$response" | jq -c '.results[]? // empty' 2>/dev/null)

  if $INCLUDE_MISCONCEPTIONS && ! budget_reached; then
    for query in "common misconceptions about ${SUBJECT}" "wrong beliefs about ${SUBJECT}"; do
      budget_reached && break
      local miscon_resp
      miscon_resp="$(
        curl --silent --max-time 20 \
          -X POST "${SURREAL_MEMORY_URL}/api/v1/palace/recall" \
          -H "Content-Type: application/json" \
          -d "{\"palace_id\":\"${palace_id}\",\"query\":\"${query}\",\"top_k\":3}" \
          2>/dev/null
      )" || true

      if [[ -n "$miscon_resp" ]]; then
        while IFS= read -r item; do
          budget_reached && break
          local mref mscore msum
          mref="$(echo "$item"   | jq -r '.id // .source_ref // "palace-misconception"')"
          mscore="$(echo "$item" | jq -r '.score // 0.70')"
          msum="$(echo "$item"   | jq -r '.content // .summary // "" | .[0:500]')"
          [[ -z "$msum" ]] && msum="Known misconception about ${SUBJECT} from palace '${palace_id}'"
          append_source "$SOURCES_TMP" "palace:${palace_id}/${mref}" "known_misconception" "${mscore}" "true" "${msum}"
          log_info "  + misconception (palace): ${mref}"
        done < <(echo "$miscon_resp" | jq -c '.results[]? // empty' 2>/dev/null)
      fi
    done
  fi
}

# ---------------------------------------------------------------------------
# Adapter: local:<directory-path>
# ---------------------------------------------------------------------------
run_local_adapter() {
  local dir_path="$1"

  log_info "Local adapter: scanning '${dir_path}' for .md/.txt/.json files ..."

  if [[ ! -d "$dir_path" ]]; then
    log_warn "Local directory not found: ${dir_path} — emitting empty partial"
    return
  fi

  local file_count=0
  while IFS= read -r filepath; do
    budget_reached && break
    [[ -f "$filepath" ]] || continue

    local filename ext summary source_type confidence
    filename="$(basename "$filepath")"
    ext="${filename##*.}"

    case "$ext" in
      json)
        # If the file matches the grounding-corpus schema (has a sources array),
        # unpack individual source entries; otherwise treat the file itself as a source.
        if command -v jq >/dev/null 2>&1; then
          local has_sources
          has_sources="$(jq -r 'if (.sources | type) == "array" then "yes" else "no" end' \
                         "$filepath" 2>/dev/null || echo "no")"

          if [[ "$has_sources" == "yes" ]]; then
            log_info "  ~ ${filename}: grounding-corpus schema detected — extracting inner sources"
            while IFS= read -r inner_item; do
              budget_reached && break
              local iref itype iconf imisco isum
              iref="$(echo "$inner_item"   | jq -r '.source_ref  // "local-source"')"
              itype="$(echo "$inner_item"  | jq -r '.source_type // "mcp_filesystem"')"
              iconf="$(echo "$inner_item"  | jq -r '.confidence  // 0.75')"
              imisco="$(echo "$inner_item" | jq -r '.is_misconception // false')"
              isum="$(echo "$inner_item"   | jq -r '.content_summary // "" | .[0:500]')"
              [[ -z "$isum" ]] && isum="Corpus entry from ${filename}"
              append_source "$SOURCES_TMP" "${iref}" "${itype}" "${iconf}" "${imisco}" "${isum}"
              log_info "  + corpus-entry from ${filename}: ${iref}"
            done < <(jq -c '.sources[]? // empty' "$filepath" 2>/dev/null)
            file_count=$((file_count + 1))
            continue
          else
            summary="$(jq -r '.content_summary // .content // .summary // "" | .[0:500]' \
                        "$filepath" 2>/dev/null || echo "")"
            [[ -z "$summary" ]] && summary="$(head -c 500 "$filepath" 2>/dev/null || echo "")"
          fi
        else
          summary="$(head -c 500 "$filepath" 2>/dev/null || echo "")"
        fi
        source_type="mcp_filesystem"
        confidence="0.75"
        ;;
      md|txt)
        summary="$(head -c 500 "$filepath" 2>/dev/null || echo "")"
        source_type="mcp_filesystem"
        confidence="0.75"
        ;;
      *)
        log_info "  ~ skipping unsupported file type: ${filename}"
        continue
        ;;
    esac

    [[ -z "$summary" ]] && summary="Local file: ${filename}"
    append_source "$SOURCES_TMP" "file://${filepath}" "${source_type}" "${confidence}" "false" "${summary}"
    log_info "  + mcp_filesystem: ${filename}"
    file_count=$((file_count + 1))
  done < <(find "$dir_path" -maxdepth 2 \( -name '*.md' -o -name '*.txt' -o -name '*.json' \) | sort)

  if [[ "$file_count" -eq 0 ]]; then
    log_warn "Local directory is empty or contains no supported files: ${dir_path}"
  fi
}

# ---------------------------------------------------------------------------
# Dispatch to adapter
# ---------------------------------------------------------------------------
case "$KB_TYPE" in
  dify)
    run_dify_adapter "$KB_VALUE"
    ;;
  palace)
    run_palace_adapter "$KB_VALUE"
    ;;
  local)
    run_local_adapter "$KB_VALUE"
    ;;
esac

# ---------------------------------------------------------------------------
# Assemble corpus JSON
# ---------------------------------------------------------------------------
FINAL_COUNT="$(source_count "$SOURCES_TMP")"
log_info "Collected ${FINAL_COUNT} sources (budget: ${BUDGET_SOURCES})"

SOURCES_JSON="["
FIRST=true
if [[ -f "$SOURCES_TMP" ]] && [[ -s "$SOURCES_TMP" ]]; then
  while IFS= read -r line; do
    [[ -z "$line" ]] && continue
    if $FIRST; then
      SOURCES_JSON+="$line"
      FIRST=false
    else
      SOURCES_JSON+=",$line"
    fi
  done < "$SOURCES_TMP"
fi
SOURCES_JSON+="]"

cat > "$OUTPUT" <<EOF
{
  "corpus_id": "${CORPUS_ID}",
  "subject": "${SUBJECT}",
  "target_level": "${LEVEL}",
  "schema_version": "1.0.0",
  "built_at": "${BUILD_AT}",
  "kb_source": "${KB_ID}",
  "privacy_mode": true,
  "sources": ${SOURCES_JSON}
}
EOF

log_info "KB corpus written to: ${OUTPUT}"

# ---------------------------------------------------------------------------
# Exit with appropriate status
# ---------------------------------------------------------------------------
if [[ "$FINAL_COUNT" -eq 0 ]]; then
  log_warn "No sources collected — emitting partial (KB may be empty or unreachable)"
  echo "{\"status\":\"partial\",\"source_count\":0,\"corpus_path\":\"${OUTPUT}\"}"
  exit 0
elif [[ "$FINAL_COUNT" -lt "$BUDGET_SOURCES" ]]; then
  log_warn "Partial corpus: found ${FINAL_COUNT} of ${BUDGET_SOURCES} requested sources"
  echo "{\"status\":\"partial\",\"source_count\":${FINAL_COUNT},\"corpus_path\":\"${OUTPUT}\"}"
  exit 0
else
  echo "{\"status\":\"ok\",\"corpus_path\":\"${OUTPUT}\",\"source_count\":${FINAL_COUNT}}"
  exit 0
fi
