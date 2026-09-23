---
paths: ['**/src-tauri/**', '**/tauri.conf.json']
---

# Tauri 2

Finish the frontend, Rust command, and IPC wiring for a coherent change before validation. At the change
boundary, exercise the real application-to-command integration; reserve bundle builds for the final phase
or release boundary because they cross-compile and invalidate incremental caches. Components never call `invoke()`; only `services/`
do, and stores call services. The feature-folder, kebab-case and 500-line rules apply unchanged.
