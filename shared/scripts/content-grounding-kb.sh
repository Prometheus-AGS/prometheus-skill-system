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
#   content-grounding-kb.sh --normalize <corpus.json> --output <out.json>
#     Re-emit an existing corpus (any schema) through the same source builder,
#     so every source gains key_points[] / misconceptions[]; corpus identity
#     fields (corpus_id, subject, concept_id, ...) are kept. Used by the
#     learn-grade eval harness on its schema-1.0.0 corpora.
#
# --kb flag forms:
#   dify:<kb-name>          Query a Dify knowledge base by name
#   palace:<palace-id>      Query a surreal-memory palace by ID
#   local:<directory-path>  Ingest local .md/.txt/.json files
#
# Every emitted source carries (change-rah-008; the shape learn-grade reads):
#   source_ref, source_type, confidence, is_misconception, content_summary,
#   key_points[]      — the sentences of content_summary, unless the KB entry
#                       already carries an authored key_points[] (kept as is)
#   misconceptions[]  — [content_summary] when is_misconception is true, else []
#                       (or the entry's authored misconceptions[] when present)
#
# This is the one grounding script. skills/learn/learn-goal/scripts/ and
# skills/learn/learn-kb/scripts/ hold thin wrappers that exec this file.
#
# Environment:
#   CONTENT_GROUNDING_BUILD_AT   override built_at (tests; keeps output byte-stable)
#
# Exit codes:
#   0 — success (full or partial)
#   1 — fatal error (bad args, missing credentials, unwritable output, no jq)

set -euo pipefail

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

log_info()  { echo "[content-grounding-kb] INFO:  $*" >&2; }
log_warn()  { echo "[content-grounding-kb] WARN:  $*" >&2; }
log_error() { echo "[content-grounding-kb] ERROR: $*" >&2; }

# jq builds every source object (key_points derivation, escaping) and parses
# every adapter response; without it no corpus can be built.
if ! command -v jq >/dev/null 2>&1; then
  log_error "jq is required"
  echo '{"status":"error","message":"jq is required by content-grounding-kb.sh"}'
  exit 1
fi

# Slug helpers live in shared/scripts/lib/slug.sh (change-rah-003). The shared
# library is sourced when it sits next to this script; the inline definition
# covers an installed layout that carries the script without lib/.
_slug_lib="$(cd "$(dirname "${BASH_SOURCE[0]}")" 2>/dev/null && pwd)/lib/slug.sh"
if [ -f "$_slug_lib" ]; then
  # shellcheck source=shared/scripts/lib/slug.sh
  . "$_slug_lib"
else
  subject_to_slug() {
    printf '%s\n' "$1" \
      | tr '[:upper:]' '[:lower:]' \
      | sed 's/[^a-z0-9]\{1,\}/-/g' \
      | sed 's/^-//; s/-$//'
  }
fi

iso_now() {
  if [[ -n "${CONTENT_GROUNDING_BUILD_AT:-}" ]]; then
    printf '%s\n' "$CONTENT_GROUNDING_BUILD_AT"
    return
  fi
  date -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || date +"%Y-%m-%dT%H:%M:%SZ"
}

# Append a source entry (JSON object) to the NDJSON accumulator file.
# Args: sources_file source_ref source_type confidence is_misconception content_summary
#       [key_points_json] [misconceptions_json]
# The two optional arguments are JSON arrays carried over from a KB entry that
# already authored them; when absent or not arrays they are derived:
# key_points = sentences of content_summary; misconceptions = [content_summary]
# for a misconception entry, [] otherwise. jq does the escaping, so a summary
# containing quotes, backslashes, or newlines is still valid JSON.
append_source() {
  local sources_file="$1"
  local source_ref="$2"
  local source_type="$3"
  local confidence="$4"
  local is_misconception="$5"
  local content_summary="$6"
  local key_points_json="${7:-}"
  local misconceptions_json="${8:-}"

  case "$confidence" in
    ''|*[!0-9.]*) confidence="0.75" ;;
  esac
  case "$is_misconception" in
    true|false) ;;
    *) is_misconception="false" ;;
  esac
  [[ -n "$key_points_json" ]] || key_points_json='null'
  [[ -n "$misconceptions_json" ]] || misconceptions_json='null'

  jq -cn \
    --arg ref "$source_ref" \
    --arg type "$source_type" \
    --argjson conf "$confidence" \
    --argjson mis "$is_misconception" \
    --arg summary "$content_summary" \
    --argjson kp "$key_points_json" \
    --argjson mc "$misconceptions_json" '
    def sentences:
      [ splits("(?<=[.!?])\\s+") | gsub("\\s+"; " ") | gsub("^ | $"; "") | select(length > 0) ];
    def authored($v): if ($v | type) == "array"
      then [ $v[] | select(type == "string") | select(length > 0) ] else null end;
    {
      source_ref: $ref,
      source_type: $type,
      confidence: $conf,
      is_misconception: $mis,
      content_summary: $summary,
      key_points: (authored($kp) // ($summary | sentences)),
      misconceptions: (authored($mc) // (if $mis then [$summary] else [] end))
    }' >> "$sources_file"
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
NORMALIZE_FILE=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --normalize)                 NORMALIZE_FILE="$2";  shift 2 ;;
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

if [[ -n "$NORMALIZE_FILE" ]]; then
  if [[ ! -f "$NORMALIZE_FILE" ]] || ! jq -e '(.sources | type) == "array"' "$NORMALIZE_FILE" >/dev/null 2>&1; then
    log_error "--normalize needs a readable corpus JSON with a sources array: ${NORMALIZE_FILE}"
    echo "{\"status\":\"error\",\"message\":\"--normalize: not a corpus file: ${NORMALIZE_FILE}\"}"
    exit 1
  fi
  # Identity comes from the file unless the caller overrides it.
  [[ -n "$KB_ID" ]]   || KB_ID="$(jq -r '.kb_source // empty' "$NORMALIZE_FILE")"
  [[ -n "$KB_ID" ]]   || KB_ID="local:${NORMALIZE_FILE}"
  [[ -n "$SUBJECT" ]] || SUBJECT="$(jq -r '.subject // empty' "$NORMALIZE_FILE")"
  [[ -n "$SUBJECT" ]] || SUBJECT="$(basename "$NORMALIZE_FILE" .json)"
  LEVEL="$(jq -r --arg d "$LEVEL" '.target_level // $d' "$NORMALIZE_FILE")"
fi

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

# Parse --kb prefix (normalize mode reads a file, not a KB, so no prefix check)
KB_TYPE="${KB_ID%%:*}"
KB_VALUE="${KB_ID#*:}"
[[ -z "$NORMALIZE_FILE" ]] || KB_TYPE="normalize"

if [[ "$KB_TYPE" != "normalize" ]] && [[ "$KB_TYPE" == "$KB_VALUE" ]]; then
  # No colon found — treat as unknown
  log_error "--kb must be prefixed with dify:, palace:, or local: (got: ${KB_ID})"
  echo "{\"status\":\"error\",\"message\":\"--kb must be prefixed with dify:, palace:, or local:\"}"
  exit 1
fi

case "$KB_TYPE" in
  dify|palace|local|normalize) ;;
  *)
    log_error "Unknown --kb type '${KB_TYPE}' — must be one of: dify, palace, local"
    echo "{\"status\":\"error\",\"message\":\"Unknown --kb type: ${KB_TYPE}\"}"
    exit 1
    ;;
esac

SUBJECT_SLUG="$(subject_to_slug "$SUBJECT")"
KB_SLUG="$(subject_to_slug "$KB_ID")"
CORPUS_ID="${KB_SLUG}-${SUBJECT_SLUG}"
if [[ -n "$NORMALIZE_FILE" ]]; then
  CORPUS_ID="$(jq -r --arg d "$CORPUS_ID" '.corpus_id // $d' "$NORMALIZE_FILE")"
fi
BUILD_AT="$(iso_now)"

# Temp accumulator
SOURCES_TMP="$(mktemp /tmp/content-grounding-kb-sources-XXXXXX.ndjson)"
NORMALIZE_EXTRAS_FILE=""
trap 'rm -f "$SOURCES_TMP" ${NORMALIZE_EXTRAS_FILE:+"$NORMALIZE_EXTRAS_FILE"}' EXIT

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
        local has_sources
        has_sources="$(jq -r 'if (.sources | type) == "array" then "yes" else "no" end' \
                       "$filepath" 2>/dev/null || echo "no")"

        if [[ "$has_sources" == "yes" ]]; then
          log_info "  ~ ${filename}: grounding-corpus schema detected — extracting inner sources"
          while IFS= read -r inner_item; do
            budget_reached && break
            local iref itype iconf imisco isum ikp imc
            iref="$(echo "$inner_item"   | jq -r '.source_ref  // "local-source"')"
            itype="$(echo "$inner_item"  | jq -r '.source_type // "mcp_filesystem"')"
            iconf="$(echo "$inner_item"  | jq -r '.confidence  // 0.75')"
            imisco="$(echo "$inner_item" | jq -r '.is_misconception // false')"
            isum="$(echo "$inner_item"   | jq -r '.content_summary // "" | .[0:500]')"
            # Authored key_points / misconceptions travel through unchanged.
            ikp="$(echo "$inner_item"    | jq -c '.key_points // null')"
            imc="$(echo "$inner_item"    | jq -c '.misconceptions // null')"
            [[ -z "$isum" ]] && isum="Corpus entry from ${filename}"
            append_source "$SOURCES_TMP" "${iref}" "${itype}" "${iconf}" "${imisco}" "${isum}" "${ikp}" "${imc}"
            log_info "  + corpus-entry from ${filename}: ${iref}"
          done < <(jq -c '.sources[]? // empty' "$filepath" 2>/dev/null)
          file_count=$((file_count + 1))
          continue
        else
          summary="$(jq -r '.content_summary // .content // .summary // "" | .[0:500]' \
                      "$filepath" 2>/dev/null || echo "")"
          [[ -z "$summary" ]] && summary="$(head -c 500 "$filepath" 2>/dev/null || echo "")"
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
# Normalize: re-emit every source of an existing corpus through append_source
# (no budget: a corpus is normalized whole, never truncated)
# ---------------------------------------------------------------------------
run_normalize() {
  local file="$1"
  log_info "Normalize: re-emitting sources of '${file}' at schema 1.1.0 ..."
  local extras_tmp; extras_tmp="$(mktemp "${TMPDIR:-/tmp}/content-grounding-kb-extras-XXXXXX.ndjson")"
  while IFS= read -r inner_item; do
    local iref itype iconf imisco isum ikp imc
    iref="$(echo "$inner_item"   | jq -r '.source_ref  // "local-source"')"
    itype="$(echo "$inner_item"  | jq -r '.source_type // "mcp_filesystem"')"
    iconf="$(echo "$inner_item"  | jq -r '.confidence  // 0.75')"
    imisco="$(echo "$inner_item" | jq -r '.is_misconception // false')"
    isum="$(echo "$inner_item"   | jq -r '.content_summary // ""')"
    ikp="$(echo "$inner_item"    | jq -c '.key_points // null')"
    imc="$(echo "$inner_item"    | jq -c '.misconceptions // null')"
    [[ -z "$isum" ]] && isum="Corpus entry ${iref}"
    append_source "$SOURCES_TMP" "${iref}" "${itype}" "${iconf}" "${imisco}" "${isum}" "${ikp}" "${imc}"
    # Per-source fields this builder does not own (a KB's own concept_id, tags,
    # ...) survive normalization instead of being dropped silently. The rebuilt
    # fields win on conflict, so key_points/misconceptions stay authoritative.
    printf '%s\n' "$inner_item" \
      | jq -c 'del(.source_ref, .source_type, .confidence, .is_misconception,
                   .content_summary, .key_points, .misconceptions)' >> "$extras_tmp"
  done < <(jq -c '.sources[]? // empty' "$file")
  NORMALIZE_EXTRAS_FILE="$extras_tmp"
}

# ---------------------------------------------------------------------------
# Dispatch to adapter
# ---------------------------------------------------------------------------
case "$KB_TYPE" in
  normalize)
    run_normalize "$NORMALIZE_FILE"
    ;;
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
[[ -z "$NORMALIZE_FILE" ]] || BUDGET_SOURCES="$FINAL_COUNT"
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

# Normalize mode: fold each source's preserved extra fields back under the
# rebuilt object, position by position (the two files were written in lockstep).
if [[ -n "$NORMALIZE_EXTRAS_FILE" ]] && [[ -s "$NORMALIZE_EXTRAS_FILE" ]]; then
  SOURCES_JSON="$(jq -c --argjson rebuilt "$SOURCES_JSON" -n \
    '[inputs] as $extras
     | [ $rebuilt | to_entries[] | ($extras[.key] // {}) + .value ]' \
    "$NORMALIZE_EXTRAS_FILE")"
fi

# In normalize mode the input's other top-level fields (concept_id, maintainer,
# ...) are kept; the fields below override them.
EXTRA_JSON='{}'
if [[ -n "$NORMALIZE_FILE" ]]; then
  EXTRA_JSON="$(jq -c 'del(.sources)' "$NORMALIZE_FILE")"
fi

jq -n \
  --argjson extra "$EXTRA_JSON" \
  --arg corpus_id "$CORPUS_ID" \
  --arg subject "$SUBJECT" \
  --arg level "$LEVEL" \
  --arg built_at "$BUILD_AT" \
  --arg kb_source "$KB_ID" \
  --argjson sources "$SOURCES_JSON" '$extra + {
    corpus_id: $corpus_id,
    subject: $subject,
    target_level: $level,
    schema_version: "1.1.0",
    built_at: $built_at,
    kb_source: $kb_source,
    privacy_mode: true,
    sources: $sources
  }' > "$OUTPUT"

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
