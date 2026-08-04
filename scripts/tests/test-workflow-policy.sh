#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$REPO_ROOT/scripts/check-workflow-policy.mjs"
FIXTURE_ROOT="$(mktemp -d)"
trap 'rm -rf "$FIXTURE_ROOT"' EXIT

make_repo() {
  local name="$1"
  mkdir -p "$FIXTURE_ROOT/$name/.github/workflows"
  printf '%s\n' "$FIXTURE_ROOT/$name"
}

expect_pass() {
  local root="$1"
  node "$CHECKER" --root "$root" >/dev/null
}

expect_fail() {
  local root="$1"
  if node "$CHECKER" --root "$root" >/dev/null 2>&1; then
    echo "workflow policy unexpectedly accepted $root" >&2
    exit 1
  fi
}

allowed_pages="$(make_repo allowed-pages)"
cat >"$allowed_pages/.github/workflows/docs-pages.yml" <<'YAML'
name: Pages deployment
on:
  push:
    branches: [main]
  workflow_dispatch:
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - run: npm ci
      - run: npm run build
YAML
expect_pass "$allowed_pages"

allowed_sync="$(make_repo allowed-sync)"
cat >"$allowed_sync/.github/workflows/docs-sync.yml" <<'YAML'
name: Deterministic documentation synchronization
on:
  push:
    branches: [main]
  workflow_dispatch:
jobs:
  sync:
    runs-on: ubuntu-latest
    steps:
      - run: npm ci
      - run: npm run docs:sync
      - run: gh pr merge --auto --squash "$PR_URL"
YAML
expect_pass "$allowed_sync"

hosted_test="$(make_repo hosted-test)"
cat >"$hosted_test/.github/workflows/validate.yml" <<'YAML'
name: Hosted validation
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - run: npm test
YAML
expect_fail "$hosted_test"

pr_pages="$(make_repo pr-pages)"
cat >"$pr_pages/.github/workflows/docs-pages.yml" <<'YAML'
name: Pages deployment
on:
  pull_request:
  push:
    branches: [main]
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - run: npm run build
YAML
expect_fail "$pr_pages"

sync_validation="$(make_repo sync-validation)"
cat >"$sync_validation/.github/workflows/docs-sync.yml" <<'YAML'
name: Deterministic documentation synchronization
on:
  push:
    branches: [main]
jobs:
  sync:
    runs-on: ubuntu-latest
    steps:
      - run: npm run docs:check
YAML
expect_fail "$sync_validation"

echo "workflow policy fixtures passed"
