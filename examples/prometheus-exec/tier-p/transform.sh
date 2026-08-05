#!/bin/bash
set -euo pipefail

total=0
while IFS= read -r value; do
  [[ "$value" =~ ^[0-9]+$ ]] || {
    echo "invalid integer input" >&2
    exit 2
  }
  total=$((total + value))
done < "${PROMETHEUS_INPUT_DIR}/numbers"

printf '%s\n' "$total" | tee "${PROMETHEUS_OUTPUT_DIR}/total.txt"
