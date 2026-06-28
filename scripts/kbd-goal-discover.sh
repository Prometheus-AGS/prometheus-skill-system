#!/usr/bin/env bash
# kbd-goal-discover.sh — Recommend skills and MCP servers for a goal description.
#
# Usage:
#   kbd-goal-discover.sh "<goal description>"
#   kbd-goal-discover.sh "build a weekly standup generator CLI in Go"
#
# Output: JSON block with recommended_skills, recommended_mcps, and rationale.
# This is advisory — non-blocking.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

GOAL_DESC="${1:-}"
if [[ -z "$GOAL_DESC" ]]; then
  echo '{"recommended_skills":[],"recommended_mcps":[],"rationale":"No goal description provided."}'
  exit 0
fi

GOAL_LOWER=$(echo "$GOAL_DESC" | tr '[:upper:]' '[:lower:]')

# ── Keyword → skill/mcp lookup ────────────────────────────────────────────────
declare -a SKILLS_REC=()
declare -a MCPS_REC=()
declare -a REASONS=()

add_skill() {
  local s="$1"
  local reason="$2"
  # Deduplicate
  for existing in "${SKILLS_REC[@]:-}"; do
    [[ "$existing" == "$s" ]] && return
  done
  SKILLS_REC+=("$s")
  REASONS+=("$reason")
}

add_mcp() {
  local m="$1"
  for existing in "${MCPS_REC[@]:-}"; do
    [[ "$existing" == "$m" ]] && return
  done
  MCPS_REC+=("$m")
}

match() {
  local pattern="$1"
  echo "$GOAL_LOWER" | grep -qE "$pattern"
}

# Go
if match '\bgo\b|golang|gopher|\bgor\b'; then
  add_skill "golang-patterns" "Goal mentions Go; golang-patterns covers idioms"
  add_skill "golang-testing"  "Goal mentions Go; golang-testing for test patterns"
  add_mcp   "context7"
fi

# Rust
if match '\brust\b|cargo|\bcrate\b'; then
  add_skill "rust-reviewer" "Goal mentions Rust; rust-reviewer for idioms and safety"
  add_mcp   "context7"
fi

# Python
if match '\bpython\b|flask|django|fastapi|asyncio|\bpip\b'; then
  add_skill "python-reviewer" "Goal mentions Python"
  add_mcp   "context7"
fi

# TypeScript/JS/React
if match '\btypescript\b|\bts\b|\breact\b|next\.?js|nextjs|\bvue\b|svelte|angular'; then
  add_skill "typescript-reviewer" "Goal mentions TypeScript/React framework"
  add_mcp   "context7"
fi
if match '\bjavascript\b|\bjs\b|\bnode\b|\bbun\b|\bdeno\b'; then
  add_skill "typescript-reviewer" "Goal mentions JavaScript runtime"
  add_mcp   "context7"
fi

# Database
if match 'database|\bsql\b|postgres|postgresql|sqlite|mysql|\borm\b'; then
  add_skill "database-reviewer" "Goal mentions database/SQL"
fi

# Auth / Security
if match '\bauth\b|authentication|oauth|jwt|session|password|login|signup'; then
  add_skill "security-reviewer" "Goal mentions authentication — security review recommended"
fi
if match 'security|vulnerability|\bxss\b|injection|\bcsrf\b|sanitize'; then
  add_skill "security-reviewer" "Goal mentions security concerns"
fi

# DevOps
if match '\bdeploy\b|deployment|docker|kubernetes|\bk8s\b|\bci\b|\bcd\b|pipeline|helm'; then
  add_skill "devops-engineer" "Goal mentions deployment/infrastructure"
fi

# Testing
if match '\btest\b|testing|\btdd\b|\bbdd\b|\be2e\b|playwright|jest|vitest|pytest|cargo test'; then
  add_skill "tdd-guide" "Goal mentions testing — use TDD approach"
fi

# Performance
if match 'performance|\bperf\b|optim|bottleneck|\bslow\b|latency|throughput'; then
  add_skill "performance-optimizer" "Goal mentions performance optimization"
fi

# UI/UX
if match '\bui\b|\bux\b|\bdesign\b|\bcss\b|tailwind|figma|storybook|component'; then
  add_skill "ui-ux-designer" "Goal mentions UI/UX design"
  add_mcp   "shadcn"
fi

# AI/LLM
if match '\bllm\b|\bai\b|\brag\b|vector|embedding|\bprompt\b|\bagent\b'; then
  add_skill "ai-engineer"      "Goal mentions AI/LLM"
  add_skill "prompt-engineer"  "Goal mentions prompts or agents"
  add_mcp   "surreal-memory"
fi

# Docs
if match 'documentation|\bdocs\b|readme|changelog|\bwiki\b'; then
  add_skill "doc-updater" "Goal mentions documentation"
fi

# Refactor
if match 'refactor|cleanup|dead code|unused|\bdry\b'; then
  add_skill "refactor-cleaner" "Goal mentions refactoring or cleanup"
fi

# Code review
if match 'review|audit|quality|best practice'; then
  add_skill "code-reviewer" "Goal mentions code review or audit"
fi

# GitHub
if match '\bgit\b|\bgithub\b|\bpr\b|\bbranch\b|\bmerge\b|\bcommit\b'; then
  add_mcp "mcp__github"
fi

# Always useful for context
if [[ ${#MCPS_REC[@]} -gt 0 ]]; then
  add_mcp "surreal-memory"
fi

# ── Build JSON output ─────────────────────────────────────────────────────────
skills_json=$(printf '"%s",' "${SKILLS_REC[@]:-}" | sed 's/,$//')
mcps_json=$(printf '"%s",' "${MCPS_REC[@]:-}" | sed 's/,$//')
rationale=$(printf '%s; ' "${REASONS[@]:-No specific domain detected — general KBD loop will run.}" | sed 's/; $//')

echo "{\"recommended_skills\":[$skills_json],\"recommended_mcps\":[$mcps_json],\"rationale\":\"$rationale\"}"
