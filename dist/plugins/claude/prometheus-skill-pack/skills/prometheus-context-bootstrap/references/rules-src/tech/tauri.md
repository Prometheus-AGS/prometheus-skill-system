---
paths: ['**/src-tauri/**', '**/tauri.conf.json']
---

# Tauri 2

Frontend tiers (TypeScript rules) plus Rust tiers during implementation. **Bundle builds are always T3**:
they cross-compile and invalidate incremental caches. Components never call `invoke()`; only `services/`
do, and stores call services. The feature-folder, kebab-case and 500-line rules apply unchanged.
