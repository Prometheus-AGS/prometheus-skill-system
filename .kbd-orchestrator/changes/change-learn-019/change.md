---
id: change-learn-019
title: surface-bridge Axum MCP App server (Tier 2 substrate)
type: rust service
status: DONE
phase: phase-learn-feynman
depends_on:
  - change-learn-002
  - change-learn-004b
---

# change-learn-019 — surface-bridge Axum MCP App server

## Summary

Add a `substrate/surface-bridge/` Rust crate that implements an Axum-based MCP
App server providing Tier 2 UI surface capability. Exposes three MCP tools:
`detect_surface_tier` (returns the active tier string), `render_ui_intent`
(serves an A2UI HTML shell for structured operator input), and
`collect_response` (polls for operator input submitted through the HTML shell).
A launchd plist handles macOS service installation.

## Motivation

Tier 2 (A2UI HTML shell) requires a local HTTP server that harness-agnostic
skills can reach via MCP. Without this substrate the learn domain is limited to
Tier 0 (text only) and Tier 1 (AskUserQuestion) on all harnesses.

## Scope

- New Rust crate: `substrate/surface-bridge/`
- Three MCP tools exposed via Axum
- macOS launchd plist for service management
- No changes to existing substrate crates

## Tasks

- [x] Write `substrate/surface-bridge/` Rust crate scaffold: `Cargo.toml` (axum, tokio, serde, serde_json, mcp-sdk or rmcp dependencies), `src/main.rs` with Axum router setup and graceful shutdown
- [x] Implement `detect_surface_tier` MCP tool: probe the environment for Tier 2 capability (check if HTML rendering is available), return a tier string (`"tier0"` | `"tier1"` | `"tier2"`) as JSON
- [x] Implement `render_ui_intent` MCP tool: accept an intent descriptor (JSON), render an A2UI HTML shell to a temp file or embedded HTTP route, return the URL the operator should open
- [x] Implement `collect_response` MCP tool: accept a session ID, poll an in-memory store for the operator's submitted response (written by the HTML shell via a POST endpoint), return the response JSON when available or a timeout sentinel
- [x] Write `substrate/surface-bridge/launchd/com.prometheus.surface-bridge.plist` for macOS service installation, referencing the release binary path `~/.prometheus/bin/surface-bridge`
