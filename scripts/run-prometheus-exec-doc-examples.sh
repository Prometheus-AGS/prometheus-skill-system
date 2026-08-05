#!/bin/bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROMETHEUS_DOC_EXEC_BIN="${PROMETHEUS_EXEC_BIN:-$(command -v prometheus-exec || true)}"
PROMETHEUS_DOC_PLUGIN_ROOT="${PROMETHEUS_EXEC_PLUGIN_ROOT:-${HOME}/.prometheus/plugins/prometheus-skill-pack}"
PROMETHEUS_DOC_COMPONENT="${REPO_ROOT}/skills/react/prometheus-entity-skills/entity-graph-optimize/skill.wasm"
PROMETHEUS_DOC_OUTPUT="$(mktemp -d /tmp/prometheus-exec-docs.XXXXXX)"

cleanup() {
  case "${PROMETHEUS_DOC_OUTPUT}" in
    /tmp/prometheus-exec-docs.*) rm -rf -- "${PROMETHEUS_DOC_OUTPUT}" ;;
    *) echo "refusing to remove unexpected documentation output path" >&2 ;;
  esac
}
trap cleanup EXIT

if [[ -z "${PROMETHEUS_DOC_EXEC_BIN}" || ! -x "${PROMETHEUS_DOC_EXEC_BIN}" ]]; then
  echo "prometheus-exec is not installed; set PROMETHEUS_EXEC_BIN to the release binary" >&2
  exit 2
fi
if [[ ! -e "${PROMETHEUS_DOC_PLUGIN_ROOT}/current" ]]; then
  echo "active signed plugin generation is unavailable at ${PROMETHEUS_DOC_PLUGIN_ROOT}" >&2
  exit 2
fi

node "${REPO_ROOT}/scripts/check-prometheus-exec-doc-examples.mjs"
python3 "${REPO_ROOT}/scripts/certify-prometheus-exec-use-cases.py" \
  --binary "${PROMETHEUS_DOC_EXEC_BIN}" \
  --plugin-root "${PROMETHEUS_DOC_PLUGIN_ROOT}" \
  --component "${PROMETHEUS_DOC_COMPONENT}" \
  --output "${PROMETHEUS_DOC_OUTPUT}" \
  --source-commit "$(git -C "${REPO_ROOT}" rev-parse HEAD)"

echo "Prometheus Exec documentation examples completed with disposable state."
