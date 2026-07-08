# Tasks — change-prui-004-ci-workflow

- [x] task-001: Read `.github/workflows/sovereign-sync.yml` — extract matrix structure, toolchain action, cache action
- [x] task-002: Write `.github/workflows/prometheus-research.yml` with fmt/clippy/test matrix, path triggers, cache key
- [x] task-003: Validate YAML syntax (`python3 -c "import yaml; yaml.safe_load(open('.github/workflows/prometheus-research.yml'))"`)
- [x] task-004: Commit with message `ci: add GitHub Actions workflow for prometheus-research crate`
