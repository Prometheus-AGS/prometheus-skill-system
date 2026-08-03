#!/usr/bin/env bash
# Certify the durable v2 memory operation contract against a running local server.
# This intentionally writes one deterministic scoped memory and is not called by doctor.
set -euo pipefail

BASE_URL="${SURREAL_MEMORY_URL:-http://127.0.0.1:23001}"
OPERATION_ID="prometheus-release-1.6.1-memory-certification"
LONG_MEMORY=false

while [ "$#" -gt 0 ]; do
  case "$1" in
    --url) BASE_URL="${2:?missing value for --url}"; shift 2 ;;
    --operation-id) OPERATION_ID="${2:?missing value for --operation-id}"; shift 2 ;;
    --long-memory) LONG_MEMORY=true; shift ;;
    --help|-h)
      sed -n '2,3p' "$0" | sed 's/^# *//'
      echo "Usage: $0 [--url URL] [--operation-id ID] [--long-memory]"
      exit 0
      ;;
    *) echo "Unknown argument: $1" >&2; exit 2 ;;
  esac
done

for command in curl jq shasum; do
  command -v "$command" >/dev/null 2>&1 || {
    echo "Required command is missing: $command" >&2
    exit 1
  }
done

content="Prometheus 1.6.1 deterministic memory receipt certification."
if $LONG_MEMORY; then
  content="$(awk 'BEGIN { for (i = 1; i <= 6000; i++) printf "certification-token-%04d%s", i, (i == 6000 ? "" : " ") }')"
fi

payload="$(jq -cnS --arg content "$content" '{categories:["certification","release-1.6.1"],content:$content,user_id:"prometheus-skill-pack"}')"
payload_hash="$(printf '%s' "$payload" | shasum -a 256 | awk '{print $1}')"
request="$(jq -cnS \
  --arg operation_id "$OPERATION_ID" \
  --arg payload_hash "$payload_hash" \
  --argjson payload "$payload" \
  '{dependencies:[],kind:"add_memory",operation_id:$operation_id,payload:$payload,payload_hash:$payload_hash,schema_version:2}')"

health="$(curl -fsS "$BASE_URL/health")"
ready="$(curl -fsS "$BASE_URL/ready")"
jq -e '.status == "ok"' >/dev/null <<<"$health"
jq -e '.status == "ready" and .ingestion_ready == true and .capabilities.ledger == true and .capabilities.storage == true and .capabilities.coordinator == true' >/dev/null <<<"$ready"

# Simulate response loss: persist the request, discard the response, then reconcile by ID.
initial_status="$(curl -sS -o /dev/null -w '%{http_code}' \
  -H 'content-type: application/json' -d "$request" "$BASE_URL/api/v2/operations")"
case "$initial_status" in 200|202) ;; *) echo "initial submission returned HTTP $initial_status" >&2; exit 1 ;; esac

receipt=""
for _ in $(seq 1 120); do
  receipt="$(curl -fsS "$BASE_URL/api/v2/operations/$OPERATION_ID")"
  state="$(jq -r '.state' <<<"$receipt")"
  case "$state" in
    committed) break ;;
    rejected) jq . <<<"$receipt" >&2; exit 1 ;;
  esac
  sleep 0.25
done
jq -e --arg id "$OPERATION_ID" --arg hash "$payload_hash" \
  '.operation_id == $id and .payload_hash == $hash and .state == "committed" and (.progress_seq > 0)' \
  >/dev/null <<<"$receipt"

# A same-ID/same-hash replay must return the byte-equivalent terminal receipt.
replay_file="$(mktemp)"
events_file="$(mktemp)"
resumed_file="$(mktemp)"
conflict_file="$(mktemp)"
trap 'rm -f "$replay_file" "$events_file" "$resumed_file" "$conflict_file"' EXIT
replay_status="$(curl -sS -o "$replay_file" -w '%{http_code}' \
  -H 'content-type: application/json' -d "$request" "$BASE_URL/api/v2/operations")"
[ "$replay_status" = 200 ] || { echo "same-hash replay returned HTTP $replay_status" >&2; exit 1; }
jq -S . "$replay_file" | diff -u <(jq -S . <<<"$receipt") - >/dev/null

# The same ID with a different canonical payload hash must conflict.
conflict_payload="$(jq -cnS '{content:"intentional conflict",user_id:"prometheus-skill-pack"}')"
conflict_hash="$(printf '%s' "$conflict_payload" | shasum -a 256 | awk '{print $1}')"
conflict_request="$(jq -cnS \
  --arg operation_id "$OPERATION_ID" --arg payload_hash "$conflict_hash" \
  --argjson payload "$conflict_payload" \
  '{dependencies:[],kind:"add_memory",operation_id:$operation_id,payload:$payload,payload_hash:$payload_hash,schema_version:2}')"
conflict_status="$(curl -sS -o "$conflict_file" -w '%{http_code}' \
  -H 'content-type: application/json' -d "$conflict_request" "$BASE_URL/api/v2/operations")"
[ "$conflict_status" = 409 ] || { echo "different-hash replay returned HTTP $conflict_status" >&2; exit 1; }

# Historical event replay plus `after` proves resumable SSE ordering.
curl -sS --max-time 2 "$BASE_URL/api/v2/operations/$OPERATION_ID/events?after=0" >"$events_file" || [ "$?" = 28 ]
first_sequence="$(awk '/^id:/{gsub(/\r/, "", $2); print $2; exit}' "$events_file")"
[ -n "$first_sequence" ] || { echo "SSE history contained no event IDs" >&2; exit 1; }
curl -sS --max-time 2 "$BASE_URL/api/v2/operations/$OPERATION_ID/events?after=$first_sequence" >"$resumed_file" || [ "$?" = 28 ]
resumed_sequence="$(awk '/^id:/{gsub(/\r/, "", $2); print $2; exit}' "$resumed_file")"
[ -n "$resumed_sequence" ] && [ "$resumed_sequence" -gt "$first_sequence" ] || {
  echo "SSE resume did not advance after sequence $first_sequence" >&2
  exit 1
}

# Certification evidence must stay compact and safe to archive. Preserve
# dimensions and byte counts while omitting the full embedding and memory body.
redacted_receipt="$(jq '
  if .result == null then .
  else .result |= (
    . + {
      content_bytes: ((.content // "") | length),
      embedding_dimensions: ((.embedding // []) | length)
    }
    | del(.content, .embedding)
  )
  end
' <<<"$receipt")"

jq -n \
  --arg operation_id "$OPERATION_ID" \
  --arg payload_hash "$payload_hash" \
  --argjson receipt "$redacted_receipt" \
  --argjson readiness "$ready" \
  --arg first_sequence "$first_sequence" \
  --arg resumed_sequence "$resumed_sequence" \
  '{schema_version:1,operation_id:$operation_id,payload_hash:$payload_hash,health:"ok",readiness:$readiness,response_loss_reconciled:true,exact_replay:true,hash_conflict_http:409,sse:{first_sequence:($first_sequence|tonumber),resumed_sequence:($resumed_sequence|tonumber)},terminal_receipt:$receipt}'
