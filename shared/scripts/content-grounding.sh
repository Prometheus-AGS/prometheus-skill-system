#!/usr/bin/env bash
# content-grounding.sh — Assembles a teaching corpus for a given subject
# by walking a prioritized source chain and outputting a JSON corpus file.
#
# Usage:
#   content-grounding.sh \
#     --subject "linear algebra" \
#     --level "practitioner" \
#     --budget-sources 8 \
#     --budget-minutes 10 \
#     --output /path/to/corpus.json \
#     [--include-misconceptions]
#
# Exit codes:
#   0 — success (full or partial)
#   1 — fatal error (bad args, unwritable output, etc.)

set -euo pipefail

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

log_info()  { echo "[content-grounding] INFO:  $*" >&2; }
log_warn()  { echo "[content-grounding] WARN:  $*" >&2; }
log_error() { echo "[content-grounding] ERROR: $*" >&2; }

subject_to_slug() {
  local input="$1"
  # Lowercase, replace non-alphanumeric runs with hyphens, trim leading/trailing hyphens
  echo "$input" \
    | tr '[:upper:]' '[:lower:]' \
    | sed 's/[^a-z0-9]\+/-/g' \
    | sed 's/^-//; s/-$//'
}

iso_now() {
  date -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || date +"%Y-%m-%dT%H:%M:%SZ"
}

# Append a source entry (JSON object) to the sources array file.
# Args: sources_file source_ref source_type confidence is_misconception content_summary
append_source() {
  local sources_file="$1"
  local source_ref="$2"
  local source_type="$3"
  local confidence="$4"
  local is_misconception="$5"
  local content_summary="$6"

  # Escape double-quotes in string fields
  local escaped_ref; escaped_ref=$(printf '%s' "$source_ref"     | sed 's/"/\\"/g')
  local escaped_sum; escaped_sum=$(printf '%s' "$content_summary" | sed 's/"/\\"/g')

  cat >> "$sources_file" <<EOF
{"source_ref":"${escaped_ref}","source_type":"${source_type}","confidence":${confidence},"is_misconception":${is_misconception},"content_summary":"${escaped_sum}"}
EOF
}

# Return the current count of lines (sources) in the sources file
source_count() {
  local sources_file="$1"
  if [[ -f "$sources_file" ]]; then
    wc -l < "$sources_file" | tr -d ' '
  else
    echo 0
  fi
}

# ---------------------------------------------------------------------------
# Argument parsing (long options via manual loop)
# ---------------------------------------------------------------------------

SUBJECT=""
LEVEL="practitioner"
BUDGET_SOURCES=8
BUDGET_MINUTES=10
OUTPUT=""
INCLUDE_MISCONCEPTIONS=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --subject)          SUBJECT="$2";           shift 2 ;;
    --level)            LEVEL="$2";             shift 2 ;;
    --budget-sources)   BUDGET_SOURCES="$2";    shift 2 ;;
    --budget-minutes)   BUDGET_MINUTES="$2";    shift 2 ;;
    --output)           OUTPUT="$2";            shift 2 ;;
    --include-misconceptions) INCLUDE_MISCONCEPTIONS=true; shift ;;
    --) shift; break ;;
    *)  log_error "Unknown argument: $1"; echo '{"status":"error","message":"Unknown argument: '"$1"'"}'; exit 1 ;;
  esac
done

# ---------------------------------------------------------------------------
# Validate required arguments
# ---------------------------------------------------------------------------

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
    echo '{"status":"error","message":"Cannot create output directory: '"$OUTPUT_DIR"'"}'
    exit 1
  fi
fi

SUBJECT_SLUG="$(subject_to_slug "$SUBJECT")"
CORPUS_ID="${SUBJECT_SLUG}-${LEVEL}"
BUILD_AT="$(iso_now)"

# Temp file to accumulate NDJSON source lines
SOURCES_TMP="$(mktemp /tmp/content-grounding-sources-XXXXXX.ndjson)"
trap 'rm -f "$SOURCES_TMP"' EXIT

log_info "Starting corpus build: subject='$SUBJECT' level='$LEVEL' budget=$BUDGET_SOURCES sources"
log_info "Corpus ID: $CORPUS_ID"
log_info "Include misconceptions: $INCLUDE_MISCONCEPTIONS"

# ---------------------------------------------------------------------------
# Helper: budget check
# ---------------------------------------------------------------------------
budget_reached() {
  [[ "$(source_count "$SOURCES_TMP")" -ge "$BUDGET_SOURCES" ]]
}

# ---------------------------------------------------------------------------
# Tier 1: Dify Knowledge Base
# ---------------------------------------------------------------------------
if ! budget_reached; then
  if [[ -z "${DIFY_API_KEY:-}" ]]; then
    log_warn "DIFY_API_KEY not set — skipping Dify KB tier"
  else
    DIFY_BASE="${DIFY_BASE_URL:-http://localhost/v1}"
    log_info "Querying Dify KB at ${DIFY_BASE} ..."

    DIFY_RESPONSE="$(
      curl --silent --max-time 15 \
        -X POST "${DIFY_BASE}/knowledge-bases/search" \
        -H "Authorization: Bearer ${DIFY_API_KEY}" \
        -H "Content-Type: application/json" \
        -d "{\"query\":\"${SUBJECT}\",\"top_k\":5}" 2>/dev/null
    )" || true

    if [[ -n "$DIFY_RESPONSE" ]]; then
      # Extract results — best-effort; tolerate missing jq gracefully
      if command -v jq >/dev/null 2>&1; then
        while IFS= read -r item; do
          budget_reached && break
          ref="$(echo "$item" | jq -r '.document_name // .id // "dify-result"')"
          summary="$(echo "$item" | jq -r '.content // "" | .[0:200]')"
          [[ -z "$summary" ]] && summary="Dify KB document on ${SUBJECT}"
          append_source "$SOURCES_TMP" "$ref" "dify_kb" "0.85" "false" "$summary"
          log_info "  + dify_kb source: $ref"
        done < <(echo "$DIFY_RESPONSE" | jq -c '.data[]? // empty' 2>/dev/null)
      else
        log_warn "jq not available — skipping Dify result parsing"
      fi
    else
      log_warn "Dify KB returned empty response"
    fi

    # Misconceptions from Dify
    if $INCLUDE_MISCONCEPTIONS && ! budget_reached && [[ -n "${DIFY_API_KEY:-}" ]]; then
      for query in "common misconceptions about ${SUBJECT}" "wrong beliefs about ${SUBJECT}"; do
        budget_reached && break
        MISCON_RESP="$(
          curl --silent --max-time 15 \
            -X POST "${DIFY_BASE}/knowledge-bases/search" \
            -H "Authorization: Bearer ${DIFY_API_KEY}" \
            -H "Content-Type: application/json" \
            -d "{\"query\":\"${query}\",\"top_k\":3}" 2>/dev/null
        )" || true

        if [[ -n "$MISCON_RESP" ]] && command -v jq >/dev/null 2>&1; then
          while IFS= read -r item; do
            budget_reached && break
            ref="$(echo "$item" | jq -r '.document_name // .id // "dify-misconception"')"
            summary="$(echo "$item" | jq -r '.content // "" | .[0:200]')"
            [[ -z "$summary" ]] && summary="Common misconception about ${SUBJECT} from Dify KB"
            append_source "$SOURCES_TMP" "$ref" "known_misconception" "0.75" "true" "$summary"
            log_info "  + misconception source (dify): $ref"
          done < <(echo "$MISCON_RESP" | jq -c '.data[]? // empty' 2>/dev/null)
        fi
      done
    fi
  fi
fi

# ---------------------------------------------------------------------------
# Tier 2: surreal-memory palace recall
# ---------------------------------------------------------------------------
if ! budget_reached; then
  if [[ -z "${SURREAL_MEMORY_URL:-}" ]]; then
    log_warn "SURREAL_MEMORY_URL not set — skipping surreal-memory palace tier"
  else
    log_info "Querying surreal-memory palace at ${SURREAL_MEMORY_URL} ..."

    PALACE_RESPONSE="$(
      curl --silent --max-time 15 \
        -X POST "${SURREAL_MEMORY_URL}/api/v1/palace/recall" \
        -H "Content-Type: application/json" \
        -d "{\"query\":\"${SUBJECT}\",\"level\":\"${LEVEL}\",\"top_k\":5}" 2>/dev/null
    )" || true

    if [[ -n "$PALACE_RESPONSE" ]] && command -v jq >/dev/null 2>&1; then
      while IFS= read -r item; do
        budget_reached && break
        ref="$(echo "$item" | jq -r '.id // .source_ref // "palace-result"')"
        summary="$(echo "$item" | jq -r '.content // .summary // "" | .[0:200]')"
        [[ -z "$summary" ]] && summary="Palace RAG result on ${SUBJECT}"
        append_source "$SOURCES_TMP" "$ref" "palace_rag" "0.80" "false" "$summary"
        log_info "  + palace_rag source: $ref"
      done < <(echo "$PALACE_RESPONSE" | jq -c '.results[]? // empty' 2>/dev/null)
    else
      log_warn "surreal-memory palace returned empty or unparseable response"
    fi

    # Misconceptions from palace
    if $INCLUDE_MISCONCEPTIONS && ! budget_reached && [[ -n "${SURREAL_MEMORY_URL:-}" ]]; then
      for query in "common misconceptions about ${SUBJECT}" "wrong beliefs about ${SUBJECT}"; do
        budget_reached && break
        PALACE_MISCON="$(
          curl --silent --max-time 15 \
            -X POST "${SURREAL_MEMORY_URL}/api/v1/palace/recall" \
            -H "Content-Type: application/json" \
            -d "{\"query\":\"${query}\",\"level\":\"${LEVEL}\",\"top_k\":3}" 2>/dev/null
        )" || true

        if [[ -n "$PALACE_MISCON" ]] && command -v jq >/dev/null 2>&1; then
          while IFS= read -r item; do
            budget_reached && break
            ref="$(echo "$item" | jq -r '.id // .source_ref // "palace-misconception"')"
            summary="$(echo "$item" | jq -r '.content // .summary // "" | .[0:200]')"
            [[ -z "$summary" ]] && summary="Known misconception about ${SUBJECT} from palace"
            append_source "$SOURCES_TMP" "$ref" "known_misconception" "0.70" "true" "$summary"
            log_info "  + misconception source (palace): $ref"
          done < <(echo "$PALACE_MISCON" | jq -c '.results[]? // empty' 2>/dev/null)
        fi
      done
    fi
  fi
fi

# ---------------------------------------------------------------------------
# Tier 3: MCP filesystem — ~/.prometheus/learn/corpus/<slug>/
# ---------------------------------------------------------------------------
if ! budget_reached; then
  CORPUS_DIR="${HOME}/.prometheus/learn/corpus/${SUBJECT_SLUG}"
  if [[ -d "$CORPUS_DIR" ]]; then
    log_info "Loading from MCP filesystem at ${CORPUS_DIR} ..."
    while IFS= read -r json_file; do
      budget_reached && break
      [[ -f "$json_file" ]] || continue
      filename="$(basename "$json_file")"

      if command -v jq >/dev/null 2>&1; then
        is_miscon="$(jq -r '.is_misconception // false' "$json_file" 2>/dev/null || echo "false")"
        summary="$(jq -r '.content_summary // .summary // .content // "" | .[0:200]' "$json_file" 2>/dev/null || echo "")"
        source_type_fs="$(jq -r '.source_type // "mcp_filesystem"' "$json_file" 2>/dev/null || echo "mcp_filesystem")"
        confidence_fs="$(jq -r '.confidence // 0.75' "$json_file" 2>/dev/null || echo "0.75")"
      else
        is_miscon="false"
        summary="Corpus file: ${filename}"
        source_type_fs="mcp_filesystem"
        confidence_fs="0.75"
      fi

      [[ -z "$summary" ]] && summary="Corpus file: ${filename}"

      # If include_misconceptions is off, skip misconception files
      if ! $INCLUDE_MISCONCEPTIONS && [[ "$is_miscon" == "true" ]]; then
        log_info "  ~ skipping misconception file (flag not set): $filename"
        continue
      fi

      append_source "$SOURCES_TMP" "file://${json_file}" "$source_type_fs" "$confidence_fs" "$is_miscon" "$summary"
      log_info "  + mcp_filesystem source: $filename"
    done < <(find "$CORPUS_DIR" -maxdepth 1 -name '*.json' | sort)
  else
    log_warn "MCP filesystem corpus directory not found: ${CORPUS_DIR} — skipping"
  fi
fi

# ---------------------------------------------------------------------------
# Tier 4: Web search via Firecrawl
# ---------------------------------------------------------------------------
if ! budget_reached; then
  if [[ -z "${FIRECRAWL_API_KEY:-}" ]]; then
    log_warn "FIRECRAWL_API_KEY not set — skipping web search tier"
  else
    FIRECRAWL_URL="${FIRECRAWL_BASE_URL:-https://api.firecrawl.dev}/v1/search"
    log_info "Querying Firecrawl web search at ${FIRECRAWL_URL} ..."

    WEB_RESPONSE="$(
      curl --silent --max-time 30 \
        -X POST "${FIRECRAWL_URL}" \
        -H "Authorization: Bearer ${FIRECRAWL_API_KEY}" \
        -H "Content-Type: application/json" \
        -d "{\"query\":\"${SUBJECT} ${LEVEL} tutorial reference\",\"limit\":5}" 2>/dev/null
    )" || true

    if [[ -n "$WEB_RESPONSE" ]] && command -v jq >/dev/null 2>&1; then
      while IFS= read -r item; do
        budget_reached && break
        ref="$(echo "$item" | jq -r '.url // .link // "web-result"')"
        summary="$(echo "$item" | jq -r '.description // .snippet // .markdown // "" | .[0:200]')"
        [[ -z "$summary" ]] && summary="Web search result on ${SUBJECT}"
        append_source "$SOURCES_TMP" "$ref" "web_search" "0.65" "false" "$summary"
        log_info "  + web_search source: $ref"
      done < <(echo "$WEB_RESPONSE" | jq -c '.data[]? // .results[]? // empty' 2>/dev/null)
    else
      log_warn "Firecrawl returned empty or unparseable response"
    fi

    # Misconceptions via web search
    if $INCLUDE_MISCONCEPTIONS && ! budget_reached; then
      for query in "common misconceptions about ${SUBJECT}" "wrong beliefs about ${SUBJECT}"; do
        budget_reached && break
        MISCON_WEB="$(
          curl --silent --max-time 30 \
            -X POST "${FIRECRAWL_URL}" \
            -H "Authorization: Bearer ${FIRECRAWL_API_KEY}" \
            -H "Content-Type: application/json" \
            -d "{\"query\":\"${query}\",\"limit\":3}" 2>/dev/null
        )" || true

        if [[ -n "$MISCON_WEB" ]] && command -v jq >/dev/null 2>&1; then
          while IFS= read -r item; do
            budget_reached && break
            ref="$(echo "$item" | jq -r '.url // .link // "web-misconception"')"
            summary="$(echo "$item" | jq -r '.description // .snippet // "" | .[0:200]')"
            [[ -z "$summary" ]] && summary="Common misconception about ${SUBJECT} from web"
            append_source "$SOURCES_TMP" "$ref" "known_misconception" "0.60" "true" "$summary"
            log_info "  + misconception source (web): $ref"
          done < <(echo "$MISCON_WEB" | jq -c '.data[]? // .results[]? // empty' 2>/dev/null)
        fi
      done
    fi
  fi
fi

# ---------------------------------------------------------------------------
# Assemble corpus JSON
# ---------------------------------------------------------------------------
FINAL_COUNT="$(source_count "$SOURCES_TMP")"
log_info "Collected ${FINAL_COUNT} sources (budget: ${BUDGET_SOURCES})"

# Build sources JSON array from NDJSON lines
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

# Write corpus JSON
cat > "$OUTPUT" <<EOF
{
  "corpus_id": "${CORPUS_ID}",
  "subject": "${SUBJECT}",
  "target_level": "${LEVEL}",
  "schema_version": "1.0.0",
  "built_at": "${BUILD_AT}",
  "sources": ${SOURCES_JSON}
}
EOF

log_info "Corpus written to: $OUTPUT"

# ---------------------------------------------------------------------------
# Exit with appropriate status
# ---------------------------------------------------------------------------
if [[ "$FINAL_COUNT" -eq 0 ]]; then
  log_error "No sources found — corpus is empty"
  echo "{\"status\":\"error\",\"message\":\"No sources found for subject: ${SUBJECT}\"}"
  exit 1
elif [[ "$FINAL_COUNT" -lt "$BUDGET_SOURCES" ]]; then
  log_warn "Partial corpus: found ${FINAL_COUNT} of ${BUDGET_SOURCES} requested sources"
  echo "{\"status\":\"partial\",\"source_count\":${FINAL_COUNT},\"corpus_path\":\"${OUTPUT}\"}"
  exit 0
else
  echo "{\"status\":\"ok\",\"corpus_path\":\"${OUTPUT}\",\"source_count\":${FINAL_COUNT}}"
  exit 0
fi
